//! `DOMSnapshot.captureSnapshot` for layout bounds (CSS px, DPR-scaled), whitelisted computed
//! styles, and paint-order occlusion filtering layered on the JS interactable collector.
//!
//! Chrome sometimes emits **`-1`** string-table indices in snapshot arrays; generated CDP structs
//! use `u32`, so we deserialize the capture result as **JSON** and coerce sentinels locally.

use std::collections::HashMap;

use headless_chrome::protocol::cdp::DOMSnapshot;
use headless_chrome::protocol::cdp::DOM;
use serde_json::Value as Json;

use super::{backend_id_for_object_id, InteractableRow};
use crate::{mac_stats_debug, mac_stats_info};

const STYLE_ORDER: [&str; 5] = [
    "pointer-events",
    "cursor",
    "visibility",
    "opacity",
    "display",
];

pub(crate) struct PaintContext {
    /// `(paint_order, x, y, w, h)` in CSS pixels for every layout box (occlusion geometry).
    layers: Vec<(u32, f64, f64, f64, f64)>,
    by_backend: HashMap<DOM::BackendNodeId, NodeSnapInfo>,
    paint_orders_present: bool,
}

struct NodeSnapInfo {
    paint_order: u32,
    bounds: (f64, f64, f64, f64),
    pointer_events_none: bool,
    not_visible_hit: bool,
}

pub(crate) fn tab_device_pixel_ratio(tab: &headless_chrome::Tab) -> f64 {
    tab.evaluate("window.devicePixelRatio", false)
        .ok()
        .and_then(|r| r.value)
        .and_then(|v| v.as_f64())
        .filter(|x| *x > 0.01 && x.is_finite())
        .unwrap_or(1.0)
}

fn parse_bounds_css(raw: &[f64], dpr: f64) -> Option<(f64, f64, f64, f64)> {
    if raw.len() < 4 {
        return None;
    }
    let x = raw[0] / dpr;
    let y = raw[1] / dpr;
    let w = raw[2] / dpr;
    let h = raw[3] / dpr;
    if !(x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite()) {
        return None;
    }
    if w < 1.0 || h < 1.0 {
        return None;
    }
    Some((x, y, w, h))
}

fn rect_contains_outer(outer: (f64, f64, f64, f64), inner: (f64, f64, f64, f64)) -> bool {
    const EPS: f64 = 0.5;
    let (ox, oy, ow, oh) = outer;
    let (ix, iy, iw, ih) = inner;
    ix + EPS >= ox && iy + EPS >= oy && ix + iw <= ox + ow + EPS && iy + ih <= oy + oh + EPS
}

fn rects_intersect(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

fn occlusion_flags(
    layers: &[(u32, f64, f64, f64, f64)],
    po: u32,
    rect: (f64, f64, f64, f64),
) -> (bool, bool) {
    let mut fully = false;
    let mut partial = false;
    for &(lpo, x, y, w, h) in layers {
        if lpo <= po {
            continue;
        }
        let r = (x, y, w, h);
        if rect_contains_outer(r, rect) {
            fully = true;
            break;
        }
        if rects_intersect(r, rect) {
            partial = true;
        }
    }
    (fully, partial && !fully)
}

fn json_number_to_i64(v: &Json) -> Option<i64> {
    v.as_i64()
        .or_else(|| v.as_u64().map(|u| u as i64))
        .or_else(|| v.as_f64().map(|f| f as i64))
}

fn strings_get(strings: &[String], idx: i64) -> String {
    if idx < 0 {
        return String::new();
    }
    strings.get(idx as usize).cloned().unwrap_or_default()
}

fn style_hit_flags(strings: &[String], indices: &[i64]) -> (bool, bool) {
    let mut pointer_events_none = false;
    let mut not_visible_hit = false;
    for (i, &si) in indices.iter().enumerate() {
        if i >= STYLE_ORDER.len() {
            break;
        }
        let v = strings_get(strings, si);
        let low = v.to_lowercase();
        match STYLE_ORDER[i] {
            "pointer-events" => {
                if low.contains("none") {
                    pointer_events_none = true;
                }
            }
            "visibility" => {
                if low.contains("hidden") || low.contains("collapse") {
                    not_visible_hit = true;
                }
            }
            "opacity" => {
                if let Ok(f) = low.parse::<f64>() {
                    if f <= 0.001 {
                        not_visible_hit = true;
                    }
                } else if low == "0" {
                    not_visible_hit = true;
                }
            }
            "display" => {
                if low == "none" {
                    not_visible_hit = true;
                }
            }
            _ => {}
        }
    }
    (pointer_events_none, not_visible_hit)
}

fn paint_order_at(paint_orders: Option<&Vec<i64>>, li: usize) -> u32 {
    let Some(po) = paint_orders else {
        return 0;
    };
    let v = po.get(li).copied().unwrap_or(0);
    v.max(0) as u32
}

fn bounds_row_from_json(v: &Json) -> Option<Vec<f64>> {
    let arr = v.as_array()?;
    let mut out = Vec::with_capacity(arr.len());
    for x in arr {
        let n = x
            .as_f64()
            .or_else(|| json_number_to_i64(x).map(|i| i as f64))?;
        out.push(n);
    }
    Some(out)
}

fn backend_id_from_json(v: &Json) -> Option<DOM::BackendNodeId> {
    let u = v
        .as_u64()
        .or_else(|| json_number_to_i64(v).map(|i| i as u64))?;
    u.try_into().ok()
}

/// Parses the first document of a `DOMSnapshot.captureSnapshot` **JSON** result.
fn build_paint_context_from_json_doc(
    doc: &Json,
    strings: &[String],
    dpr: f64,
) -> Option<PaintContext> {
    let layout = doc.get("layout")?;
    let node_index = layout.get("nodeIndex")?.as_array()?;
    let bounds_arr = layout.get("bounds")?.as_array()?;
    let styles_arr = layout.get("styles")?.as_array()?;
    let nlayout = node_index.len();
    if nlayout == 0 {
        return None;
    }

    let paint_orders: Option<Vec<i64>> = layout
        .get("paintOrders")
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().filter_map(json_number_to_i64).collect());

    let paint_orders_present = paint_orders
        .as_ref()
        .map(|p| !p.is_empty())
        .unwrap_or(false);

    let nodes = doc.get("nodes")?;
    let node_count = nodes
        .get("backendNodeId")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .or_else(|| {
            nodes
                .get("parentIndex")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
        })
        .unwrap_or(0);

    let backend_ids: Option<&Vec<Json>> = nodes.get("backendNodeId").and_then(|v| v.as_array());

    let mut layers: Vec<(u32, f64, f64, f64, f64)> = Vec::with_capacity(nlayout);
    let mut by_backend: HashMap<DOM::BackendNodeId, NodeSnapInfo> = HashMap::new();

    for li in 0..nlayout {
        let dom_idx_i = json_number_to_i64(node_index.get(li)?)?;
        if dom_idx_i < 0 {
            continue;
        }
        let dom_idx = dom_idx_i as usize;
        if dom_idx >= node_count {
            continue;
        }
        let raw_bounds = bounds_row_from_json(bounds_arr.get(li)?)?;
        let Some(bounds) = parse_bounds_css(
            &raw_bounds.iter().map(|x| *x as f64).collect::<Vec<_>>(),
            dpr,
        ) else {
            continue;
        };
        let po = paint_order_at(paint_orders.as_ref(), li);
        let style_indices: Vec<i64> = styles_arr
            .get(li)
            .and_then(|row| row.as_array())
            .map(|row| row.iter().filter_map(json_number_to_i64).collect())
            .unwrap_or_default();
        let (pointer_events_none, not_visible_hit) = style_hit_flags(strings, &style_indices);

        layers.push((po, bounds.0, bounds.1, bounds.2, bounds.3));

        let Some(backend_arr) = backend_ids else {
            continue;
        };
        let Some(bid) = backend_arr.get(dom_idx).and_then(backend_id_from_json) else {
            continue;
        };
        let area = bounds.2 * bounds.3;
        let entry = NodeSnapInfo {
            paint_order: po,
            bounds,
            pointer_events_none,
            not_visible_hit,
        };
        let replace = match by_backend.get(&bid) {
            None => true,
            Some(e) => area > e.bounds.2 * e.bounds.3,
        };
        if replace {
            by_backend.insert(bid, entry);
        }
    }

    Some(PaintContext {
        layers,
        by_backend,
        paint_orders_present,
    })
}

fn try_capture_paint_context(tab: &headless_chrome::Tab) -> Option<PaintContext> {
    let _ = tab.call_method(DOMSnapshot::Enable(None));
    let dpr = tab_device_pixel_ratio(tab);
    let cap = tab
        .call_method_json(DOMSnapshot::CaptureSnapshot {
            computed_styles: STYLE_ORDER.iter().map(|s| s.to_string()).collect(),
            include_paint_order: Some(true),
            include_dom_rects: None,
            include_blended_background_colors: None,
            include_text_color_opacities: None,
        })
        .map_err(|e| {
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: DOMSnapshot.captureSnapshot failed: {} — using JS-only interactables",
                e
            );
            e
        })
        .ok()?;

    let strings: Vec<String> = cap
        .get("strings")
        .and_then(|s| serde_json::from_value(s.clone()).ok())
        .unwrap_or_default();

    let doc = cap
        .get("documents")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())?;
    let ctx = build_paint_context_from_json_doc(doc, &strings, dpr)?;
    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: DOMSnapshot paint context — layout_boxes={} backend_map={} dpr={:.3} paint_order={}",
        ctx.layers.len(),
        ctx.by_backend.len(),
        dpr,
        ctx.paint_orders_present
    );
    Some(ctx)
}

/// Removes elements the snapshot marks non-hittable or fully covered by a higher paint layer;
/// sets `bounds_css` / `covered` when snapshot data exists for the backend node.
pub(crate) fn apply_snapshot_paint_filter(
    tab: &headless_chrome::Tab,
    rows: &mut Vec<InteractableRow>,
    object_ids: &mut Vec<String>,
) {
    if rows.len() != object_ids.len() {
        mac_stats_debug!(
            "browser/cdp",
            "Browser agent [CDP]: DOMSnapshot filter skipped (rows/object_ids length mismatch)"
        );
        return;
    }
    let Some(ctx) = try_capture_paint_context(tab) else {
        return;
    };

    let mut out_rows: Vec<InteractableRow> = Vec::with_capacity(rows.len());
    let mut out_oids: Vec<String> = Vec::with_capacity(object_ids.len());
    let mut excluded_style = 0u32;
    let mut excluded_paint = 0u32;

    let drained: Vec<_> = rows.drain(..).zip(object_ids.drain(..)).collect();
    for (mut row, oid) in drained {
        let Ok(bid) = backend_id_for_object_id(tab, &oid) else {
            out_rows.push(row);
            out_oids.push(oid);
            continue;
        };
        row.cached_backend_node_id = Some(bid);
        let Some(info) = ctx.by_backend.get(&bid) else {
            out_rows.push(row);
            out_oids.push(oid);
            continue;
        };
        if info.pointer_events_none || info.not_visible_hit {
            excluded_style += 1;
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: interactable excluded by snapshot styles (backendNodeId={})",
                bid
            );
            continue;
        }
        row.bounds_css = Some(info.bounds);
        let (full, partial) = if ctx.paint_orders_present {
            occlusion_flags(&ctx.layers, info.paint_order, info.bounds)
        } else {
            (false, false)
        };
        if full {
            excluded_paint += 1;
            mac_stats_debug!(
                "browser/cdp",
                "Browser agent [CDP]: interactable excluded by paint-order occlusion (backendNodeId={})",
                bid
            );
            continue;
        }
        row.covered = partial;
        out_rows.push(row);
        out_oids.push(oid);
    }

    if excluded_style > 0 || excluded_paint > 0 {
        mac_stats_info!(
            "browser/cdp",
            "Browser agent [CDP]: DOMSnapshot filter — excluded style/hidden={} paint_occluded={}",
            excluded_style,
            excluded_paint
        );
    }

    *rows = out_rows;
    *object_ids = out_oids;
}
