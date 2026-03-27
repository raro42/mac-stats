//! Server-side screenshot overlays: dashed type-colored boxes and index labels for interactables.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, PxScale};
use image::DynamicImage;
use image::ImageFormat;
use image::Rgb;
use image::RgbImage;
use imageproc::drawing::{draw_line_segment_mut, draw_text_mut};

use super::Interactable;
use crate::{mac_stats_debug, mac_stats_info, mac_stats_warn};

static ANNOT_FONT: OnceLock<FontRef<'static>> = OnceLock::new();

fn annot_font() -> &'static FontRef<'static> {
    ANNOT_FONT.get_or_init(|| {
        FontRef::try_from_slice(include_bytes!("../../fonts/DejaVuSans.ttf"))
            .expect("embedded DejaVuSans.ttf must parse")
    })
}

fn color_for_tag(tag: &str) -> Rgb<u8> {
    match tag {
        "button" => Rgb([220, 55, 55]),
        "input" | "textarea" => Rgb([0, 165, 165]),
        "a" => Rgb([55, 175, 75]),
        "select" => Rgb([65, 110, 220]),
        _ => Rgb([140, 145, 155]),
    }
}

fn css_bounds_for_draw(i: &Interactable) -> Option<(f64, f64, f64, f64)> {
    if i.from_subframe {
        return None;
    }
    i.annot_bounds_css.or(i.bounds_css)
}

fn device_rect(
    css: (f64, f64, f64, f64),
    dpr: f64,
    img_w: u32,
    img_h: u32,
) -> Option<(f32, f32, f32, f32)> {
    if !(dpr.is_finite() && dpr > 0.01) {
        return None;
    }
    let (x, y, w, h) = css;
    if !(x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite()) {
        return None;
    }
    let mut xd = (x * dpr).round() as i64;
    let mut yd = (y * dpr).round() as i64;
    let mut wd = (w * dpr).round() as i64;
    let mut hd = (h * dpr).round() as i64;
    if wd < 1 {
        wd = 1;
    }
    if hd < 1 {
        hd = 1;
    }
    let iw = img_w as i64;
    let ih = img_h as i64;
    xd = xd.clamp(0, iw.saturating_sub(1));
    yd = yd.clamp(0, ih.saturating_sub(1));
    wd = wd.clamp(1, iw.saturating_sub(xd));
    hd = hd.clamp(1, ih.saturating_sub(yd));
    let left = xd as f32;
    let top = yd as f32;
    let right = (xd + wd) as f32;
    let bottom = (yd + hd) as f32;
    Some((left, top, right, bottom))
}

fn draw_dashed_segment(
    img: &mut RgbImage,
    (x0, y0): (f32, f32),
    (x1, y1): (f32, f32),
    color: Rgb<u8>,
    dash: f32,
    gap: f32,
) {
    let dash = dash.max(2.0);
    let gap = gap.max(1.0);
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let mut t = 0_f32;
    while t < len {
        let seg = (t + dash).min(len);
        draw_line_segment_mut(
            img,
            (x0 + ux * t, y0 + uy * t),
            (x0 + ux * seg, y0 + uy * seg),
            color,
        );
        t = seg + gap;
    }
}

fn draw_dashed_rect(
    img: &mut RgbImage,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    color: Rgb<u8>,
) {
    let w = right - left;
    let h = bottom - top;
    let dash = (4.0 + 0.08 * w.min(h)).clamp(4.0, 14.0);
    let gap = dash * 0.45;
    draw_dashed_segment(img, (left, top), (right, top), color, dash, gap);
    draw_dashed_segment(img, (left, bottom), (right, bottom), color, dash, gap);
    draw_dashed_segment(img, (left, top), (left, bottom), color, dash, gap);
    draw_dashed_segment(img, (right, top), (right, bottom), color, dash, gap);
}

fn visible_label_text(i: &Interactable) -> &str {
    i.text.as_str()
}

fn should_draw_index_label(i: &Interactable) -> bool {
    visible_label_text(i).chars().count() <= 3
}

fn draw_label_with_outline(
    img: &mut RgbImage,
    x: i32,
    y: i32,
    scale: PxScale,
    font: &impl Font,
    text: &str,
    fg: Rgb<u8>,
    outline: Rgb<u8>,
) {
    for (ox, oy) in [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
    ] {
        draw_text_mut(img, outline, x + ox, y + oy, scale, font, text);
    }
    draw_text_mut(img, fg, x, y, scale, font, text);
}

/// Writes `<stem>-annotated.png` next to `raw_png_path`, returns that path.
pub(crate) fn write_annotated_screenshot_copy(
    raw_png_path: &Path,
    png_bytes: &[u8],
    interactables: &[Interactable],
    dpr: f64,
    viewport_width_css: f64,
) -> Result<PathBuf, String> {
    let img_dyn = image::load_from_memory(png_bytes)
        .map_err(|e| format!("decode screenshot PNG for annotation: {}", e))?;
    let mut rgb = img_dyn.to_rgb8();
    let (img_w, img_h) = rgb.dimensions();

    let vw = if viewport_width_css.is_finite() && viewport_width_css > 1.0 {
        viewport_width_css
    } else {
        (img_w as f64 / dpr).max(1.0)
    };
    let font_css = (vw * 0.01).clamp(8.0_f64, 20.0_f64);
    let font_px = (font_css * dpr) as f32;
    let scale = PxScale {
        x: font_px,
        y: font_px,
    };
    let font = annot_font();

    let mut drawn = 0_u32;
    for i in interactables {
        let Some(css) = css_bounds_for_draw(i) else {
            continue;
        };
        let Some((left, top, right, bottom)) = device_rect(css, dpr, img_w, img_h) else {
            continue;
        };
        let color = color_for_tag(i.tag.as_str());
        draw_dashed_rect(&mut rgb, left, top, right, bottom, color);

        if should_draw_index_label(i) {
            let label = i.index.to_string();
            let (tw, th) = imageproc::drawing::text_size(scale, font, &label);
            let css_h = css.3;
            let small = css_h < 30.0;
            let mut lx = left as i32 + 2;
            let mut ly = if small {
                top as i32 - th as i32 - 2
            } else {
                top as i32 + 2
            };
            if ly < 2 {
                ly = top as i32 + 2;
            }
            if lx + tw as i32 > img_w as i32 - 2 {
                lx = img_w as i32 - tw as i32 - 2;
            }
            if lx < 2 {
                lx = 2;
            }
            draw_label_with_outline(
                &mut rgb,
                lx,
                ly,
                scale,
                font,
                &label,
                Rgb([255, 255, 255]),
                Rgb([0, 0, 0]),
            );
        }
        drawn += 1;
    }

    let stem = raw_png_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "screenshot path has no file stem".to_string())?;
    let parent = raw_png_path
        .parent()
        .ok_or_else(|| "screenshot path has no parent directory".to_string())?;
    let out_name = format!("{}-annotated.png", stem);

    let dyn_img = DynamicImage::ImageRgb8(rgb);
    let mut encoded = Vec::new();
    dyn_img
        .write_to(&mut std::io::Cursor::new(&mut encoded), ImageFormat::Png)
        .map_err(|e| format!("encode annotated screenshot PNG: {}", e))?;
    super::artifact_limits::ensure_buffer_within_browser_artifact_cap(
        encoded.len(),
        "annotated PNG",
    )?;
    let out = super::artifact_atomic::write_bytes_atomic_same_dir(parent, &out_name, &encoded)
        .map_err(|e| format!("write annotated screenshot: {}", e))?;

    mac_stats_info!(
        "browser/cdp",
        "Browser agent [CDP]: screenshot annotation wrote {} ({} interactable overlays, dpr={:.3}, viewport_css_w={:.1})",
        out.display(),
        drawn,
        dpr,
        vw
    );
    mac_stats_debug!(
        "browser/cdp",
        "Browser agent [CDP]: annotated screenshot source raw={}",
        raw_png_path.display()
    );

    Ok(out)
}

/// On failure, logs and returns `Ok(raw_png_path)` so the tool still succeeds with an unannotated image.
pub(crate) fn try_annotate_screenshot(
    raw_png_path: &Path,
    png_bytes: &[u8],
    interactables: &[Interactable],
    dpr: f64,
    viewport_width_css: f64,
) -> PathBuf {
    match write_annotated_screenshot_copy(
        raw_png_path,
        png_bytes,
        interactables,
        dpr,
        viewport_width_css,
    ) {
        Ok(p) => p,
        Err(e) => {
            mac_stats_warn!(
                "browser/cdp",
                "Browser agent [CDP]: screenshot annotation failed (using raw only): {}",
                e
            );
            raw_png_path.to_path_buf()
        }
    }
}
