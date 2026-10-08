use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadRequest {
    pub path: String,
    pub n_ctx: u32,
    pub n_batch: u32,
    pub n_threads: u32,
    pub gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadInfo {
    pub load_ms: f64,
    pub description: String,
    pub n_params: u64,
    pub size_bytes: u64,
    pub n_ctx: u32,
    pub chat_template: Option<String>,
    pub gpu: bool,
    pub available_memory: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchRequest {
    pub pp: u32,
    pub tg: u32,
    pub reps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchResult {
    pub pp_tps: f64,
    pub tg_tps: f64,
    pub pp_runs: Vec<f64>,
    pub tg_runs: Vec<f64>,
    pub available_memory: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Generation request. Swift sends `{type: "delta", text}` through `on_event`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRequest {
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub seed: u32,
    pub think_prefill: bool,
    pub on_event: Channel<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GenerateResult {
    pub stop_reason: String,
    pub n_prompt: u32,
    pub n_cached: u32,
    pub n_gen: u32,
    pub prompt_ms: f64,
    pub gen_ms: f64,
    pub pp_tps: f64,
    pub tg_tps: f64,
    pub available_memory: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineStatus {
    pub loaded: bool,
    pub path: Option<String>,
    pub busy: bool,
    pub available_memory: u64,
    /// The app is in the foreground (otherwise Metal cannot use the GPU).
    pub active: bool,
}

/// Verified model download. Swift sends `{type: "progress", received, total}`
/// and `{type: "verifying"}` through `on_event`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub file: String,
    pub allow_cellular: bool,
    pub on_event: Channel<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectLanguageRequest {
    pub text: String,
}

/// Language of a text as detected on the device by Apple's NaturalLanguage framework.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DetectedLanguage {
    /// BCP-47 code, e.g. `es`, `en`, `pt`, `zh-Hans`.
    pub code: String,
    /// English name of the language, e.g. "Spanish".
    pub english_name: String,
    /// 0.0–1.0
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundRefreshRequest {
    pub enabled: bool,
}
