//! Chat con el modelo local: descargas, carga en memoria, conversaciones y envío con
//! respuesta en streaming.

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

use crate::metrics::{apple::Thermal, MetricsState};
use catalog::CatalogEntry;
use store::{ChatStore, Conversation, ConversationSummary, ReplyStats, StoredMessage};

/// Modelo recomendado (resultado de la prueba de la fase A: ios/docs/llm-spike.md).
pub const DEFAULT_MODEL: &str = "qwen3.5-2b";

const MAX_TOKENS: u32 = 512;
const TEMPERATURE: f32 = 0.7;

pub(crate) fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

pub(crate) fn new_id() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    format!("{:x}{:04x}", now_ms(), COUNTER.fetch_add(1, Ordering::Relaxed) & 0xffff)
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Settings {
    model_id: Option<String>,
}

pub struct ChatState {
    pub(crate) store: ChatStore,
    settings_path: PathBuf,
    /// Modelo cargado ahora en llama.cpp. Es un mutex asíncrono para que dos cargas no
    /// se pisen.
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

    fn settings(&self) -> Settings {
        std::fs::read(&self.settings_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub(crate) fn selected(&self) -> CatalogEntry {
        self.settings()
            .model_id
            .and_then(|id| catalog::find(&id))
            .or_else(|| catalog::find(DEFAULT_MODEL))
            .expect("el modelo por defecto está en el catálogo")
    }
}

pub(crate) async fn ensure_loaded(app: &AppHandle, state: &ChatState, entry: &CatalogEntry) -> Result<Option<LoadInfo>, String> {
    let mut loaded = state.loaded.lock().await;
    // Swift puede haber soltado el modelo por su cuenta (aviso de memoria), así que se
    // comprueba el estado real del motor y no solo lo que recuerda Rust.
    let path = entry.path().to_string_lossy().into_owned();
    let engine = app.llm().status().await.map_err(|e| e.to_string())?;
    if loaded.as_deref() == Some(entry.id.as_str()) && engine.loaded && engine.path.as_deref() == Some(path.as_str()) {
        return Ok(None);
    }
    if !entry.installed() {
        return Err(format!("Primero descarga «{}».", entry.name));
    }
    let info = app
        .llm()
        .load(LoadRequest {
            path,
            n_ctx: 4096,
            n_batch: 512,
            n_threads: 2,
            gpu: true,
        })
        .await
        .map_err(|e| e.to_string())?;
    *loaded = Some(entry.id.clone());
    Ok(Some(info))
}

// ---------------------------------------------------------------------------
// Modelos
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
pub fn chat_select_model(state: State<'_, ChatState>, id: String) -> Result<(), String> {
    catalog::find(&id).ok_or_else(|| format!("No existe el modelo «{id}»"))?;
    let bytes = serde_json::to_vec_pretty(&Settings { model_id: Some(id) }).map_err(|e| e.to_string())?;
    if let Some(dir) = state.settings_path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    std::fs::write(&state.settings_path, bytes).map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatStatus {
    pub loaded_model: Option<String>,
    pub selected_model: String,
    pub engine: EngineStatus,
}

#[tauri::command]
pub async fn chat_status(app: AppHandle, state: State<'_, ChatState>) -> Result<ChatStatus, String> {
    Ok(ChatStatus {
        loaded_model: state.loaded.lock().await.clone(),
        selected_model: state.selected().id,
        engine: app.llm().status().await.map_err(|e| e.to_string())?,
    })
}

/// Descarga verificada. Reenvía a la web `{type: "progress"|"verifying", …}`.
#[tauri::command]
pub async fn chat_download(app: AppHandle, id: String, allow_cellular: bool, on_event: Channel<Value>) -> Result<(), String> {
    let entry = catalog::find(&id).ok_or_else(|| format!("No existe el modelo «{id}»"))?;
    app.llm()
        .download(DownloadRequest {
            url: entry.url(),
            sha256: entry.sha256.clone(),
            size: entry.size,
            file: entry.file.clone(),
            allow_cellular,
            on_event,
        })
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_cancel_download(app: AppHandle) -> Result<(), String> {
    app.llm().cancel_download().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_delete_model(app: AppHandle, state: State<'_, ChatState>, id: String) -> Result<(), String> {
    let entry = catalog::find(&id).ok_or_else(|| format!("No existe el modelo «{id}»"))?;
    let mut loaded = state.loaded.lock().await;
    if loaded.as_deref() == Some(id.as_str()) {
        *loaded = None;
    }
    app.llm().delete_model(&entry.file).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_load(app: AppHandle, state: State<'_, ChatState>) -> Result<Option<LoadInfo>, String> {
    let entry = state.selected();
    ensure_loaded(&app, &state, &entry).await
}

#[tauri::command]
pub async fn chat_unload(app: AppHandle, state: State<'_, ChatState>) -> Result<(), String> {
    *state.loaded.lock().await = None;
    app.llm().unload().await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Conversaciones
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
pub fn chat_delete(state: State<'_, ChatState>, id: String) -> Result<(), String> {
    state.store.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_cancel(app: AppHandle) -> Result<(), String> {
    app.llm().cancel().await.map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendResult {
    pub conversation_id: String,
    pub stop_reason: String,
    pub n_gen: u32,
    pub tg_tps: f64,
    /// Tokens del prompt y cuántos se reutilizaron de la memoria del turno anterior.
    pub n_prompt: u32,
    pub n_cached: u32,
    pub prompt_ms: f64,
}

/// Envía una pregunta. La web recibe `{type: "status", text}` mientras se carga el
/// modelo y `{type: "delta", text}` con cada fragmento de la respuesta. La pregunta se
/// guarda antes de generar y la respuesta al terminar, aunque se cancele a mitad.
#[tauri::command]
pub async fn chat_send(
    app: AppHandle,
    state: State<'_, ChatState>,
    metrics: State<'_, MetricsState>,
    conversation_id: Option<String>,
    text: String,
    on_event: Channel<Value>,
) -> Result<SendResult, String> {
    send(&app, &state, &metrics, conversation_id, &text, on_event).await
}

pub(crate) async fn send(
    app: &AppHandle,
    state: &ChatState,
    metrics: &MetricsState,
    conversation_id: Option<String>,
    text: &str,
    on_event: Channel<Value>,
) -> Result<SendResult, String> {
    let question = text.trim().to_string();
    if question.is_empty() {
        return Err("Escribe una pregunta.".into());
    }
    let snapshot = metrics.latest();
    if snapshot.as_ref().is_some_and(|s| s.thermal == Thermal::Critical) {
        return Err("El iPhone está muy caliente. Deja que se enfríe antes de seguir.".into());
    }

    let entry = state.selected();
    if state.loaded.lock().await.as_deref() != Some(entry.id.as_str()) {
        let _ = on_event.send(json!({ "type": "status", "text": "Cargando el modelo…" }));
    }
    ensure_loaded(app, state, &entry).await?;

    let now = now_ms();
    let mut conversation = conversation_id
        .and_then(|id| state.store.get(&id))
        .unwrap_or_else(|| Conversation::new(new_id(), now));
    if conversation.messages.is_empty() {
        conversation.title = store::title_from(&question);
    }

    let device = snapshot.as_ref().map(prompt::device_note);
    let history = prompt::history(&conversation.messages);
    let messages = prompt::build(&history, device.as_deref(), &question, prompt::HISTORY_BUDGET_CHARS);

    conversation.messages.push(StoredMessage {
        role: "user".into(),
        content: question,
        ts: now,
        device_note: device,
        stats: None,
    });
    conversation.updated_at = now;
    state.store.save(&conversation).map_err(|e| e.to_string())?;

    // Canal propio: acumula la respuesta para guardarla y reenvía cada fragmento a la web.
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

    // Se guarda tal cual, sin recortar espacios: el siguiente turno la vuelve a enviar y
    // tiene que coincidir con lo que el modelo generó para reutilizar la memoria.
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
            device_note: None,
            stats: Some(ReplyStats { model_id: entry.id.clone(), n_gen, tg_tps, stop_reason: stop_reason.clone() }),
        });
        conversation.updated_at = now_ms();
        state.store.save(&conversation).map_err(|e| e.to_string())?;
    }

    let r = result.map_err(|e| e.to_string())?;
    Ok(SendResult {
        conversation_id: conversation.id,
        stop_reason,
        n_gen,
        tg_tps,
        n_prompt: r.n_prompt,
        n_cached: r.n_cached,
        prompt_ms: r.prompt_ms,
    })
}
