//! Chat with the local model: downloads, loading into memory, conversations and sending
//! with a streamed reply.

pub mod catalog;
pub mod prompt;
pub mod store;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_llm::{DownloadRequest, EngineStatus, GenerateRequest, LlmExt, LoadInfo, LoadRequest};

use crate::error::{AppError, ErrorCode};
use crate::language;
use crate::metrics::{apple::Thermal, MetricsState};
use catalog::CatalogEntry;
use store::{ChatStore, Conversation, ConversationSummary, ReplyStats, StoredMessage};

/// Recommended model (outcome of the phase A test: ios/docs/llm-spike.md).
pub const DEFAULT_MODEL: &str = "qwen3.5-2b";

const MAX_TOKENS: u32 = 512;
const TEMPERATURE: f32 = 0.7;

/// Extra memory Swift requires on top of the model size before loading it
/// (`LlamaEngine.load`); used to explain a `not_enough_memory` error.
const LOAD_MEMORY_MARGIN: u64 = 512 * 1_048_576;

/// Extra free space Swift requires on top of the model size before downloading it
/// (`ModelStore.start`); used to explain a `not_enough_storage` error.
const DOWNLOAD_SPACE_MARGIN: u64 = 1_073_741_824;

pub(crate) fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

pub(crate) fn new_id() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    format!("{:x}{:04x}", now_ms(), COUNTER.fetch_add(1, Ordering::Relaxed) & 0xffff)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Settings {
    pub model_id: Option<String>,
    /// UI language chosen in Settings; `None` = Automatic (follow iOS).
    pub language: Option<String>,
    /// UI theme id (same ids as the desktop app); `None` = System.
    pub theme: Option<String>,
    /// Background samples for the saved history; `None` = on (the default).
    pub background_history: Option<bool>,
}

pub struct ChatState {
    pub(crate) store: ChatStore,
    settings_path: PathBuf,
    /// Model currently loaded in llama.cpp. It is an async mutex so two loads do not
    /// step on each other.
    loaded: tokio::sync::Mutex<Option<String>>,
}

impl ChatState {
    pub fn new(app: &AppHandle) -> Self {
        let dir = app.path().app_data_dir().unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Application Support/ios-stats")
        });
        ChatState {
            store: ChatStore::new(dir.join("chats")),
            settings_path: dir.join("chat-settings.json"),
            loaded: tokio::sync::Mutex::new(None),
        }
    }

    pub(crate) fn settings(&self) -> Settings {
        std::fs::read(&self.settings_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    /// Read-modify-write, so changing one setting keeps the others.
    pub(crate) fn update_settings(&self, change: impl FnOnce(&mut Settings)) -> Result<(), AppError> {
        let mut settings = self.settings();
        change(&mut settings);
        if let Some(dir) = self.settings_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&self.settings_path, serde_json::to_vec_pretty(&settings)?)?;
        Ok(())
    }

    pub(crate) fn selected(&self) -> CatalogEntry {
        self.settings()
            .model_id
            .and_then(|id| catalog::find(&id))
            .or_else(|| catalog::find(DEFAULT_MODEL))
            .expect("the default model is in the catalog")
    }
}

fn find_model(id: &str) -> Result<CatalogEntry, AppError> {
    catalog::find(id).ok_or_else(|| AppError::new(ErrorCode::ModelUnknown).with("id", id))
}

pub(crate) async fn ensure_loaded(app: &AppHandle, state: &ChatState, entry: &CatalogEntry) -> Result<Option<LoadInfo>, AppError> {
    let mut loaded = state.loaded.lock().await;
    // Swift may have released the model on its own (memory warning), so check the
    // engine's real state and not just what Rust remembers.
    let path = entry.path().to_string_lossy().into_owned();
    let engine = app.llm().status().await?;
    if loaded.as_deref() == Some(entry.id.as_str()) && engine.loaded && engine.path.as_deref() == Some(path.as_str()) {
        return Ok(None);
    }
    if !entry.installed() {
        return Err(AppError::new(ErrorCode::ModelNotInstalled).with("name", entry.name.as_str()));
    }
    let request = LoadRequest { path, n_ctx: 4096, n_batch: 512, n_threads: 2, gpu: true };
    let info = match app.llm().load(request).await.map_err(AppError::from) {
        Err(e) if e.is(ErrorCode::NotEnoughMemory) => {
            let available = app.llm().status().await.map(|s| s.available_memory).unwrap_or(0);
            return Err(e.with("needed", entry.size + LOAD_MEMORY_MARGIN).with("available", available));
        }
        other => other?,
    };
    *loaded = Some(entry.id.clone());
    Ok(Some(info))
}

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatModel {
    #[serde(flatten)]
    pub entry: CatalogEntry,
    pub installed: bool,
    pub selected: bool,
    pub recommended: bool,
}

#[tauri::command]
pub fn chat_models(state: State<'_, ChatState>) -> Vec<ChatModel> {
    let selected = state.selected().id;
    catalog::catalog()
        .into_iter()
        .map(|entry| ChatModel {
            installed: entry.installed(),
            selected: entry.id == selected,
            recommended: entry.id == DEFAULT_MODEL,
            entry,
        })
        .collect()
}

#[tauri::command]
pub fn chat_select_model(state: State<'_, ChatState>, id: String) -> Result<(), AppError> {
    find_model(&id)?;
    state.update_settings(|s| s.model_id = Some(id))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatStatus {
    pub loaded_model: Option<String>,
    pub selected_model: String,
    pub engine: EngineStatus,
}

#[tauri::command]
pub async fn chat_status(app: AppHandle, state: State<'_, ChatState>) -> Result<ChatStatus, AppError> {
    Ok(ChatStatus {
        loaded_model: state.loaded.lock().await.clone(),
        selected_model: state.selected().id,
        engine: app.llm().status().await?,
    })
}

/// Verified download. Forwards `{type: "progress"|"verifying", …}` to the web layer.
#[tauri::command]
pub async fn chat_download(
    app: AppHandle,
    metrics: State<'_, MetricsState>,
    id: String,
    allow_cellular: bool,
    on_event: Channel<Value>,
) -> Result<(), AppError> {
    let entry = find_model(&id)?;
    let request = DownloadRequest {
        url: entry.url(),
        sha256: entry.sha256.clone(),
        size: entry.size,
        file: entry.file.clone(),
        allow_cellular,
        on_event,
    };
    match app.llm().download(request).await.map_err(AppError::from) {
        Ok(_) => Ok(()),
        Err(e) if e.is(ErrorCode::NotEnoughStorage) => {
            let available = metrics.latest().and_then(|s| s.storage).map(|s| s.available).unwrap_or(0);
            Err(e.with("needed", entry.size + DOWNLOAD_SPACE_MARGIN).with("available", available))
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn chat_cancel_download(app: AppHandle) -> Result<(), AppError> {
    Ok(app.llm().cancel_download().await?)
}

#[tauri::command]
pub async fn chat_delete_model(app: AppHandle, state: State<'_, ChatState>, id: String) -> Result<(), AppError> {
    let entry = find_model(&id)?;
    let mut loaded = state.loaded.lock().await;
    if loaded.as_deref() == Some(id.as_str()) {
        *loaded = None;
    }
    Ok(app.llm().delete_model(&entry.file).await?)
}

#[tauri::command]
pub async fn chat_load(app: AppHandle, state: State<'_, ChatState>) -> Result<Option<LoadInfo>, AppError> {
    let entry = state.selected();
    ensure_loaded(&app, &state, &entry).await
}

#[tauri::command]
pub async fn chat_unload(app: AppHandle, state: State<'_, ChatState>) -> Result<(), AppError> {
    *state.loaded.lock().await = None;
    Ok(app.llm().unload().await?)
}

// ---------------------------------------------------------------------------
// Conversations
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn chat_list(state: State<'_, ChatState>) -> Vec<ConversationSummary> {
    state.store.list()
}

#[tauri::command]
pub fn chat_get(state: State<'_, ChatState>, id: String) -> Option<Conversation> {
    state.store.get(&id)
}

#[tauri::command]
pub fn chat_delete(state: State<'_, ChatState>, id: String) -> Result<(), AppError> {
    Ok(state.store.delete(&id)?)
}

#[tauri::command]
pub async fn chat_cancel(app: AppHandle) -> Result<(), AppError> {
    Ok(app.llm().cancel().await?)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendResult {
    pub conversation_id: String,
    pub stop_reason: String,
    pub n_gen: u32,
    pub tg_tps: f64,
    /// Prompt tokens and how many were reused from the previous turn's memory.
    pub n_prompt: u32,
    pub n_cached: u32,
    pub prompt_ms: f64,
    /// Language the reply was asked for (BCP-47 code).
    pub reply_language: String,
}

/// Reply language for `question` in `conversation` (see `language::reply_language`).
/// Returns the language and the detection to store with the question, if it was clear.
pub(crate) async fn pick_reply_language(
    app: &AppHandle,
    state: &ChatState,
    conversation: &Conversation,
    question: &str,
) -> (language::ReplyLanguage, Option<tauri_plugin_llm::DetectedLanguage>) {
    let detected = app.llm().detect_language(question).await.unwrap_or_else(|e| {
        eprintln!("language detection failed: {e}");
        None
    });
    let sticky = conversation.messages.iter().rev().find_map(|m| m.language.as_ref());
    let reply = language::reply_language(detected.as_ref(), question, sticky, &language::current(state));
    let clear = detected.filter(|d| language::is_clear(d, question));
    (reply, clear)
}

/// Sends a question. The web layer receives `{type: "status", code: "loading_model"}`
/// while the model loads and `{type: "delta", text}` with each chunk of the reply. The
/// question is saved before generating and the reply when it finishes, even if it is
/// cancelled midway.
#[tauri::command]
pub async fn chat_send(
    app: AppHandle,
    state: State<'_, ChatState>,
    metrics: State<'_, MetricsState>,
    conversation_id: Option<String>,
    text: String,
    on_event: Channel<Value>,
) -> Result<SendResult, AppError> {
    send(&app, &state, &metrics, conversation_id, &text, on_event).await
}

pub(crate) async fn send(
    app: &AppHandle,
    state: &ChatState,
    metrics: &MetricsState,
    conversation_id: Option<String>,
    text: &str,
    on_event: Channel<Value>,
) -> Result<SendResult, AppError> {
    let question = text.trim().to_string();
    if question.is_empty() {
        return Err(ErrorCode::EmptyQuestion.into());
    }
    let snapshot = metrics.latest();
    if snapshot.as_ref().is_some_and(|s| s.thermal == Thermal::Critical) {
        return Err(ErrorCode::TooHot.into());
    }

    let entry = state.selected();
    if state.loaded.lock().await.as_deref() != Some(entry.id.as_str()) {
        let _ = on_event.send(json!({ "type": "status", "code": "loading_model" }));
    }
    ensure_loaded(app, state, &entry).await?;

    let now = now_ms();
    let mut conversation = conversation_id
        .and_then(|id| state.store.get(&id))
        .unwrap_or_else(|| Conversation::new(new_id(), now));
    if conversation.messages.is_empty() {
        conversation.title = store::title_from(&question);
    }

    let (reply_language, detected) = pick_reply_language(app, state, &conversation, &question).await;
    let note = prompt::turn_note(snapshot.as_ref(), &reply_language);
    let history = prompt::history(&conversation.messages);
    let messages = prompt::build(&history, Some(&note), &question, prompt::HISTORY_BUDGET_BYTES);

    conversation.messages.push(StoredMessage {
        role: "user".into(),
        content: question,
        ts: now,
        turn_note: Some(note),
        language: detected,
        stats: None,
    });
    conversation.updated_at = now;
    state.store.save(&conversation)?;

    // Own channel: accumulates the reply to save it and forwards each chunk to the web layer.
    let reply = Arc::new(Mutex::new(String::new()));
    let (acc, web) = (reply.clone(), on_event.clone());
    let channel = Channel::<Value>::new(move |body| {
        if let InvokeResponseBody::Json(raw) = body {
            if let Ok(event) = serde_json::from_str::<Value>(&raw) {
                if let Some(delta) = event.get("text").and_then(Value::as_str) {
                    acc.lock().unwrap_or_else(|e| e.into_inner()).push_str(delta);
                }
                let _ = web.send(event);
            }
        }
        Ok(())
    });

    let result = app
        .llm()
        .generate(GenerateRequest {
            messages,
            max_tokens: MAX_TOKENS,
            temperature: TEMPERATURE,
            seed: now as u32,
            think_prefill: entry.think_prefill,
            on_event: channel,
        })
        .await;

    // Saved as is, without trimming whitespace: the next turn sends it again and it has
    // to match what the model generated for the memory to be reused.
    let text = reply.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let (stop_reason, n_gen, tg_tps) = match &result {
        Ok(r) => (r.stop_reason.clone(), r.n_gen, r.tg_tps),
        Err(_) => ("error".to_string(), 0, 0.0),
    };
    if !text.trim().is_empty() {
        conversation.messages.push(StoredMessage {
            role: "assistant".into(),
            content: text,
            ts: now_ms(),
            turn_note: None,
            language: None,
            stats: Some(ReplyStats { model_id: entry.id.clone(), n_gen, tg_tps, stop_reason: stop_reason.clone() }),
        });
        conversation.updated_at = now_ms();
        state.store.save(&conversation)?;
    }

    let r = result?;
    Ok(SendResult {
        conversation_id: conversation.id,
        stop_reason,
        n_gen,
        tg_tps,
        n_prompt: r.n_prompt,
        n_cached: r.n_cached,
        prompt_ms: r.prompt_ms,
        reply_language: reply_language.code,
    })
}
