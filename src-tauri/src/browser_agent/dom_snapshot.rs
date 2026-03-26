//! `DOMSnapshot.captureSnapshot` for layout bounds (CSS px, DPR-scaled), whitelisted computed
//! styles, and paint-order occlusion filtering layered on the JS interactable collector.

use std::collections::HashMap;

use headless_chrome::protocol::cdp::DOM;
use headless_chrome::protocol::cdp::DOMSnapshot;

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
    ix + EPS >= ox
        && iy + EPS >= oy
        && ix + iw <= ox + ow + EPS
        && iy + ih <= oy + oh + EPS
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

fn strings_get(strings: &[String], idx: u32) -> String {
    strings.get(idx as usize).cloned().unwrap_or_default()
}

fn style_hit_flags(strings: &[String], indices: &[u32]) -> (bool, bool) {
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

fn build_context_from_document(
    doc: &DOMSnapshot::DocumentSnapshot,
    strings: &[String],
    dpr: f64,
) -> Option<PaintContext> {
    let nlayout = doc.layout.node_index.len();
    if nlayout == 0 {
        return None;
    }
    let paint_orders_present = doc
        .layout
        .paint_orders
        .as_ref()
        .map(|p| !p.is_empty())
        .unwrap_or(false);
    let paint_orders = doc.layout.paint_orders.as_deref();
    let mut layers: Vec<(u32, f64, f64, f64, f64)> = Vec::with_capacity(nlayout);
    let mut by_backend: HashMap<DOM::BackendNodeId, NodeSnapInfo> = HashMap::new();

    let node_count = doc
        .nodes
        .parent_index
        .as_ref()
        .map(|v| v.len())
        .or_else(|| doc.nodes.backend_node_id.as_ref().map(|v| v.len()))
        .unwrap_or(0);

    for li in 0..nlayout {
        let dom_idx = *doc.layout.node_index.get(li)? as usize;
        if dom_idx >= node_count {
            continue;
        }
        let raw_bounds = doc.layout.bounds.get(li)?;
        let raw: Vec<f64> = raw_bounds.iter().map(|x| *x as f64).collect();
        let Some(bounds) = parse_bounds_css(&raw, dpr) else {
            continue;
        };
        let po = paint_orders
            .and_then(|p| p.get(li).copied())
            .unwrap_or(0);
        let (pointer_events_none, not_visible_hit) = doc
            .layout
            .styles
            .get(li)
            .map(|ix| style_hit_flags(strings, ix))
            .unwrap_or((false, false));

        layers.push((po, bounds.0, bounds.1, bounds.2, bounds.3));

        let Some(backend) = doc
            .nodes
            .backend_node_id
            .as_ref()
            .and_then(|v| v.get(dom_idx).copied())
        else {
            continue;
        };
        let area = bounds.2 * bounds.3;
        let entry = NodeSnapInfo {
            paint_order: po,
            bounds,
            pointer_events_none,
            not_visible_hit,
        };
        let replace = match by_backend.get(&backend) {
            None => true,
            Some(e) => area > e.bounds.2 * e.bounds.3,
        };
        if replace {
            by_backend.insert(backend, entry);
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
        .call_method(DOMSnapshot::CaptureSnapshot {
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
    let main = cap.documents.first()?;
    let ctx = build_context_from_document(main, &cap.strings, dpr)?;
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
