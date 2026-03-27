//! Optional resize of images before sending them to a vision LLM (browser-use `llm_screenshot_size` pattern).
//! Files on disk and Discord attachments stay full size; only the copy embedded in the Ollama request is resized.

use std::path::{Path, PathBuf};

use base64::Engine;

use crate::config::Config;

fn image_path_filter(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp"))
        .unwrap_or(false)
}

/// Encode image bytes for a vision request. When `browserLlmScreenshotWidth/Height` are set, resizes with
/// Lanczos3 to that exact size and returns `Some((rw, rh))` for coordinate scaling; otherwise returns
/// PNG bytes at original resolution (re-encoded) and `None` for dims.
pub fn encode_image_bytes_for_llm_vision(
    bytes: &[u8],
) -> Result<(String, Option<(u32, u32)>), String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let (orig_w, orig_h) = (img.width(), img.height());

    let Some((tw, th)) = Config::browser_llm_screenshot_size() else {
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        return Ok((base64::engine::general_purpose::STANDARD.encode(&buf), None));
    };

    let resized = img.resize_exact(tw, th, image::imageops::FilterType::Lanczos3);
    let mut buf = Vec::new();
    resized
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    tracing::info!(
        "browser/llm_screenshot: resizing screenshot for LLM from {}x{} to {}x{} (LANCZOS)",
        orig_w,
        orig_h,
        tw,
        th
    );

    Ok((
        base64::engine::general_purpose::STANDARD.encode(&buf),
        Some((tw, th)),
    ))
}

/// First image attachment as base64 for vision verification. Uses optional LLM resize when configured.
/// Returns `(None, _)` when no suitable path exists.
pub fn prepare_first_attachment_image_for_vision(
    paths: &[PathBuf],
) -> (Option<String>, Option<(u32, u32)>) {
    for path in paths.iter().filter(|p| image_path_filter(p)) {
        if crate::browser_agent::artifact_limits::stat_path_within_browser_artifact_cap(
            path.as_path(),
            "vision prep image",
        )
        .is_err()
        {
            continue;
        }
        let Ok(bytes) = std::fs::read(path) else {
            tracing::warn!(
                "browser/llm_screenshot: could not read image {:?} for vision prep",
                path
            );
            continue;
        };

        return match encode_image_bytes_for_llm_vision(&bytes) {
            Ok((b64, dims)) => (Some(b64), dims),
            Err(e) => {
                tracing::warn!(
                    "browser/llm_screenshot: vision prep failed ({}), falling back to raw file base64",
                    e
                );
                (
                    Some(base64::engine::general_purpose::STANDARD.encode(&bytes)),
                    None,
                )
            }
        };
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_produces_valid_base64_roundtrip_smoke() {
        let mut png_one_pixel = Vec::new();
        let img = image::RgbImage::from_raw(1, 1, vec![255u8, 0, 0]).expect("1x1 rgb");
        image::DynamicImage::ImageRgb8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut png_one_pixel),
                image::ImageFormat::Png,
            )
            .unwrap();
        let (b64, _) = encode_image_bytes_for_llm_vision(&png_one_pixel).unwrap();
        assert!(!b64.is_empty());
    }
}
