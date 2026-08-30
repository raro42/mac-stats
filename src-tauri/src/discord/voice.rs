//! Discord voice-message transcription via local Ollama (gemma4 audio).
//!
//! Discord voice notes arrive as Opus/OGG attachments with empty message text. We download,
//! convert to 16 kHz mono WAV (ffmpeg), then call Ollama `/v1/audio/transcriptions`.

use serenity::model::channel::{Attachment, Message, MessageFlags};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

const MAX_AUDIO_BYTES: u32 = 8 * 1024 * 1024; // 8 MiB

/// True when this Discord message is a native voice note or has an audio attachment we can try.
pub(super) fn message_has_voice_or_audio(msg: &Message) -> bool {
    let is_voice = msg
        .flags
        .is_some_and(|f| f.contains(MessageFlags::IS_VOICE_MESSAGE));
    is_voice || msg.attachments.iter().any(is_audio_attachment)
}

fn is_audio_attachment(att: &Attachment) -> bool {
    if att.duration_secs.is_some() || att.waveform.is_some() {
        return true;
    }
    let ct = att.content_type.as_deref().unwrap_or("").to_lowercase();
    if ct.starts_with("audio/") {
        return true;
    }
    let name = att.filename.to_lowercase();
    name.ends_with(".ogg")
        || name.ends_with(".opus")
        || name.ends_with(".mp3")
        || name.ends_with(".wav")
        || name.ends_with(".m4a")
        || name.ends_with(".webm")
        || name.ends_with(".flac")
}

fn voice_model_name() -> String {
    crate::config::Config::discord_voice_model()
}

fn ollama_endpoint_and_key() -> Option<(String, Option<String>)> {
    let guard = crate::commands::ollama_config::get_ollama_client().lock().ok()?;
    let client = guard.as_ref()?;
    let endpoint = client.config.endpoint.trim().trim_end_matches('/').to_string();
    if endpoint.is_empty() {
        return None;
    }
    let key = client.config.get_api_key().ok().flatten();
    Some((endpoint, key))
}

fn temp_voice_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("mac-stats-voice");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn unique_stem() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("voice-{nanos}-{}", std::process::id())
}

/// Convert any ffmpeg-readable audio to 16 kHz mono WAV for Gemma audio.
fn convert_to_wav16k(input: &Path, output: &Path) -> Result<(), String> {
    let ffmpeg = which_ffmpeg().ok_or_else(|| {
        "ffmpeg not found (needed to convert Discord voice Opus/OGG to WAV for Ollama)".to_string()
    })?;
    let status = std::process::Command::new(&ffmpeg)
        .args([
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-ar",
            "16000",
            "-ac",
            "1",
            "-f",
            "wav",
            &output.to_string_lossy(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()
        .map_err(|e| format!("ffmpeg spawn failed: {e}"))?;
    if !status.success() {
        return Err(format!("ffmpeg exited with {status}"));
    }
    if !output.is_file() {
        return Err("ffmpeg produced no WAV output".into());
    }
    Ok(())
}

/// True when ffmpeg is findable for Discord voice STT (path / `MAC_STATS_FFMPEG` only; no convert).
pub(super) fn ffmpeg_available() -> bool {
    which_ffmpeg().is_some()
}

fn which_ffmpeg() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("MAC_STATS_FFMPEG") {
        let pb = PathBuf::from(p.trim());
        if pb.is_file() {
            return Some(pb);
        }
    }
    for cand in [
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/usr/bin/ffmpeg",
    ] {
        let p = PathBuf::from(cand);
        if p.is_file() {
            return Some(p);
        }
    }
    // PATH lookup
    std::process::Command::new("which")
        .arg("ffmpeg")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(PathBuf::from(s))
            }
        })
}

async fn transcribe_wav_bytes(
    endpoint: &str,
    api_key: Option<&str>,
    model: &str,
    wav: &[u8],
    filename: &str,
) -> Result<String, String> {
    let url = format!(
        "{}/v1/audio/transcriptions",
        endpoint.trim_end_matches('/')
    );
    let part = reqwest::multipart::Part::bytes(wav.to_vec())
        .file_name(filename.to_string())
        .mime_str("audio/wav")
        .map_err(|e| format!("multipart wav: {e}"))?;
    let form = reqwest::multipart::Form::new()
        .text("model", model.to_string())
        .part("file", part);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let mut req = client.post(&url).multipart(form);
    if let Some(k) = api_key {
        req = req.bearer_auth(k);
    }
    let resp = req.send().await.map_err(|e| format!("transcription request: {e}"))?;
    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("transcription body: {e}"))?;
    if !status.is_success() {
        return Err(format!("transcription HTTP {status}: {}", crate::logging::ellipse(&body, 240)));
    }
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("transcription JSON: {e}"))?;
    let text = v
        .get("text")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(text)
}

/// Download the first voice/audio attachment, convert, transcribe. Returns transcript or error.
pub(super) async fn transcribe_discord_voice_message(msg: &Message) -> Result<String, String> {
    let att = msg
        .attachments
        .iter()
        .find(|a| is_audio_attachment(a))
        .ok_or_else(|| "no audio attachment on voice message".to_string())?;

    if att.size > MAX_AUDIO_BYTES {
        return Err(format!(
            "voice attachment too large ({} bytes > {MAX_AUDIO_BYTES})",
            att.size
        ));
    }

    let (endpoint, api_key) = ollama_endpoint_and_key()
        .ok_or_else(|| "Ollama not configured (needed for voice transcription)".to_string())?;
    let model = voice_model_name();

    info!(
        "Discord: transcribing voice attachment {} ({} bytes, model={})",
        att.filename, att.size, model
    );

    let bytes = att
        .download()
        .await
        .map_err(|e| format!("download {}: {e}", att.filename))?;

    let stem = unique_stem();
    let dir = temp_voice_dir();
    let raw_path = dir.join(format!("{stem}-raw-{}", sanitize_filename(&att.filename)));
    let wav_path = dir.join(format!("{stem}.wav"));
    std::fs::write(&raw_path, &bytes).map_err(|e| format!("write temp audio: {e}"))?;

    let wav_bytes = match convert_to_wav16k(&raw_path, &wav_path) {
        Ok(()) => std::fs::read(&wav_path).map_err(|e| format!("read wav: {e}"))?,
        Err(e) if looks_like_wav(&bytes) => {
            // Last resort: some WAVs already work without resampling; prefer convert when possible.
            warn!("Discord: ffmpeg convert failed ({e}); falling back to raw WAV bytes");
            bytes
        }
        Err(e) => return Err(e),
    };

    let text = transcribe_wav_bytes(
        &endpoint,
        api_key.as_deref(),
        &model,
        &wav_bytes,
        "voice.wav",
    )
    .await;

    let _ = std::fs::remove_file(&raw_path);
    let _ = std::fs::remove_file(&wav_path);

    let text = text?;
    if text.is_empty() {
        return Err("transcription returned empty text".into());
    }
    info!(
        "Discord: voice transcript ({} chars): {}",
        text.chars().count(),
        crate::logging::ellipse(&text, 120)
    );
    Ok(text)
}

fn looks_like_wav(bytes: &[u8]) -> bool {
    bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE"
}

fn sanitize_filename(name: &str) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("audio.bin");
    base.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(64)
        .collect()
}

/// Strip Discord `<@…>` / `<#…>` / `<:emoji:…>` tokens so mention-only content counts as empty for voice.
fn strip_discord_markup(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            let mut closed = false;
            while j < chars.len() {
                if chars[j] == '>' {
                    closed = true;
                    break;
                }
                // Avoid swallowing free-form "< not a mention"
                if chars[j].is_whitespace() {
                    break;
                }
                j += 1;
            }
            if closed {
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Prefer voice transcript when text is empty (or prepend when both present).
pub(super) async fn maybe_augment_content_with_voice_transcript(
    msg: &Message,
    content: &str,
) -> Option<String> {
    if !message_has_voice_or_audio(msg) {
        return None;
    }
    // If user already typed a long question and attached unrelated audio, still transcribe
    // and append — but for empty/mention-only content, replace.
    match transcribe_discord_voice_message(msg).await {
        Ok(transcript) => {
            let trimmed = strip_discord_markup(content);
            if trimmed.is_empty() {
                Some(transcript)
            } else {
                Some(format!("{trimmed}\n\n[Voice message transcript]\n{transcript}"))
            }
        }
        Err(e) => {
            warn!("Discord: voice transcription failed: {e}");
            // Surface a short user-visible hint only when there was no text at all.
            if strip_discord_markup(content).is_empty() {
                Some(format!(
                    "(I received a voice message but could not transcribe it: {e}. Please retry as text, or ensure Ollama model `{}` and ffmpeg are available.)",
                    voice_model_name()
                ))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_strips_path() {
        assert_eq!(sanitize_filename("../../evil.ogg"), "evil.ogg");
        assert!(sanitize_filename("voice message.ogg").contains("voice"));
    }

    #[test]
    fn looks_like_wav_header() {
        let mut b = vec![0u8; 16];
        b[0..4].copy_from_slice(b"RIFF");
        b[8..12].copy_from_slice(b"WAVE");
        assert!(looks_like_wav(&b));
        assert!(!looks_like_wav(b"OggS........"));
    }

    #[test]
    fn strip_discord_markup_treats_mention_as_empty() {
        assert!(strip_discord_markup("<@1467208052443975763>").is_empty());
        assert!(strip_discord_markup("  <@!123>  ").is_empty());
        assert_eq!(
            strip_discord_markup("<@123> what's the weather"),
            "what's the weather"
        );
        assert_eq!(strip_discord_markup("plain text"), "plain text");
    }
}
