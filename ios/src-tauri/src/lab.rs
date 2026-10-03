//! Laboratorio de modelos (fase A): cargar, medir y probar modelos locales en el iPhone.
//!
//! Además de los comandos para la tarjeta «Laboratorio», incluye un modo automático
//! para builds de depuración: si la app arranca con `IOS_STATS_BENCH=<ids>|all`, mide
//! cada modelo (carga, velocidad, memoria, temperatura y calidad en es-MX), imprime una
//! línea `BENCH {json}` por evento y guarda el resultado en `Documents/`.

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{
    ipc::{Channel, InvokeResponseBody},
    AppHandle, Manager, State,
};
use tauri_plugin_llm::{
    BenchRequest, BenchResult, ChatMessage, GenerateRequest, GenerateResult, LlmExt, LoadInfo,
    LoadRequest,
};

use crate::chat::catalog::{self, CatalogEntry};
use crate::chat::prompt::{self, PERSONA};
use crate::metrics::apple::Thermal;
use crate::metrics::MetricsState;

/// Persona, datos del iPhone y pregunta: el mismo prompt que usa el chat.
fn build_messages(metrics: &MetricsState, question: &str) -> Vec<ChatMessage> {
    let device = metrics.latest().map(|s| prompt::device_note(&s));
    prompt::build(&[], device.as_deref(), question, prompt::HISTORY_BUDGET_CHARS)
}

fn entry(id: &str) -> Result<CatalogEntry, String> {
    catalog::find(id).ok_or_else(|| format!("No existe el modelo «{id}» en el catálogo"))
}

fn load_request(entry: &CatalogEntry, threads: u32) -> LoadRequest {
    LoadRequest {
        path: entry.path().to_string_lossy().into_owned(),
        n_ctx: 4096,
        n_batch: 512,
        n_threads: threads,
        gpu: true,
    }
}

// ---------------------------------------------------------------------------
// Comandos de la tarjeta «Laboratorio»
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabModel {
    #[serde(flatten)]
    pub entry: CatalogEntry,
    pub installed: bool,
}

#[tauri::command]
pub fn lab_models() -> Vec<LabModel> {
    catalog::catalog()
        .into_iter()
        .map(|entry| LabModel { installed: entry.installed(), entry })
        .collect()
}

#[tauri::command]
pub async fn lab_load(app: AppHandle, id: String, threads: Option<u32>) -> Result<LoadInfo, String> {
    let entry = entry(&id)?;
    app.llm()
        .load(load_request(&entry, threads.unwrap_or(2)))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lab_unload(app: AppHandle) -> Result<(), String> {
    app.llm().unload().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lab_bench(app: AppHandle) -> Result<BenchResult, String> {
    app.llm()
        .bench(BenchRequest { pp: 512, tg: 128, reps: 3 })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lab_generate(
    app: AppHandle,
    metrics: State<'_, MetricsState>,
    prompt: String,
    think_prefill: bool,
    on_event: Channel<Value>,
) -> Result<GenerateResult, String> {
    let messages = build_messages(&metrics, &prompt);
    app.llm()
        .generate(GenerateRequest {
            messages,
            max_tokens: 400,
            temperature: 0.7,
            seed: 42,
            think_prefill,
            on_event,
        })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn lab_cancel(app: AppHandle) -> Result<(), String> {
    app.llm().cancel().await.map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Medición automática (IOS_STATS_BENCH)
// ---------------------------------------------------------------------------

const QUALITY_PROMPTS: [&str; 10] = [
    "¿Cómo van mi batería y mi memoria ahora mismo?",
    "Explícame en dos frases qué es la memoria RAM.",
    "Dame 3 consejos para que mi iPhone no se caliente.",
    "Resume en una frase: «El modo de bajo consumo reduce la actividad en segundo plano, baja el brillo y limita algunas animaciones para que la batería dure más».",
    "¿Cuánto es 17 × 23? Responde solo con el número.",
    "Escribe un haiku sobre un teléfono que se queda sin batería.",
    "Traduce al inglés: «Mi teléfono está muy lento desde ayer».",
    "Si tengo 45 GB libres y cada video ocupa 1.5 GB, ¿cuántos videos caben? Explica el cálculo en una línea.",
    "Haz una lista de 3 pasos para liberar espacio en un iPhone.",
    "¿Qué significa que el estado térmico de mi iPhone sea «serio»?",
];

const SOAK_PROMPT: &str = "Cuéntame una historia larga sobre un viaje en tren por la sierra de México, con muchos detalles.";

/// Cada evento va a la consola, a `Documents/llm-bench-progress.jsonl` (para leerlo
/// desde el Mac con devicectl aunque la consola no llegue) y a la web como aviso.
fn bench_log(value: Value) {
    println!("BENCH {value}");
    let path = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Documents/llm-bench-progress.jsonl");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let line = json!({ "ts": crate::chat::now_ms(), "event": value });
        let _ = writeln!(file, "{line}");
    }
    if let Some(app) = BENCH_APP.get() {
        use tauri::Emitter;
        let _ = app.emit("bench-progress", &value);
    }
}

static BENCH_APP: std::sync::OnceLock<AppHandle> = std::sync::OnceLock::new();

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// Pico de memoria y estados térmicos mientras el modelo está cargado (lectura cada 0.5 s).
#[derive(Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Peaks {
    max_footprint: u64,
    min_available: Option<u64>,
    worst_thermal: Option<Thermal>,
}

fn thermal_rank(t: Thermal) -> u8 {
    match t {
        Thermal::Nominal => 0,
        Thermal::Fair => 1,
        Thermal::Serious => 2,
        Thermal::Critical => 3,
        Thermal::Unknown => 0,
    }
}

fn watch_peaks(metrics: MetricsState, stop: Arc<AtomicBool>) -> Arc<Mutex<Peaks>> {
    let peaks = Arc::new(Mutex::new(Peaks::default()));
    let out = peaks.clone();
    tauri::async_runtime::spawn(async move {
        while !stop.load(Ordering::Relaxed) {
            if let Some(s) = metrics.latest() {
                let mut p = peaks.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(f) = s.app_footprint {
                    p.max_footprint = p.max_footprint.max(f);
                }
                if let Some(a) = s.app_available {
                    p.min_available = Some(p.min_available.map_or(a, |m| m.min(a)));
                }
                let worse = p.worst_thermal.is_none_or(|w| thermal_rank(s.thermal) > thermal_rank(w));
                if worse {
                    p.worst_thermal = Some(s.thermal);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    out
}

/// Genera y devuelve el texto completo, el tiempo hasta el primer fragmento y el resultado.
async fn generate_collect(
    app: &AppHandle,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    seed: u32,
    think_prefill: bool,
) -> Result<(String, Option<f64>, GenerateResult), String> {
    let text = Arc::new(Mutex::new(String::new()));
    let first: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let (t, f) = (text.clone(), first.clone());
    let channel = Channel::<Value>::new(move |body| {
        if let InvokeResponseBody::Json(raw) = body {
            if let Ok(event) = serde_json::from_str::<Value>(&raw) {
                if let Some(delta) = event.get("text").and_then(Value::as_str) {
                    f.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Instant::now);
                    t.lock().unwrap_or_else(|e| e.into_inner()).push_str(delta);
                }
            }
        }
        Ok(())
    });
    let start = Instant::now();
    let result = app
        .llm()
        .generate(GenerateRequest {
            messages,
            max_tokens,
            temperature,
            seed,
            think_prefill,
            on_event: channel,
        })
        .await
        .map_err(|e| e.to_string())?;
    let ttft = first
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .map(|i| i.duration_since(start).as_secs_f64() * 1000.0);
    let text = text.lock().unwrap_or_else(|e| e.into_inner()).clone();
    Ok((text, ttft, result))
}

async fn bench_model(app: &AppHandle, metrics: &MetricsState, entry: &CatalogEntry, soak_secs: u64, threads: u32) -> Value {
    let start = metrics.latest();
    let request = load_request(entry, threads);
    let llm = app.llm();
    let mut report = json!({
        "id": entry.id,
        "file": entry.file,
        "threads": threads,
        "availableBefore": start.as_ref().and_then(|s| s.app_available),
        "batteryBefore": start.as_ref().and_then(|s| s.battery.map(|b| b.level)),
    });

    let stop = Arc::new(AtomicBool::new(false));
    let peaks = watch_peaks(metrics.clone(), stop.clone());

    let result = async {
        let cold = llm.load(request.clone()).await.map_err(|e| e.to_string())?;
        report["coldLoadMs"] = json!(cold.load_ms);
        report["model"] = json!({ "description": cold.description, "params": cold.n_params, "bytes": cold.size_bytes, "gpu": cold.gpu });
        bench_log(json!({ "event": "loaded", "id": entry.id, "info": cold }));

        // Primera generación: incluye compilar los shaders de Metal.
        let warmup = vec![
            ChatMessage { role: "system".into(), content: PERSONA.into() },
            ChatMessage { role: "user".into(), content: "Hola".into() },
        ];
        let (_, ttft, first) = generate_collect(app, warmup, 8, 0.0, 1, entry.think_prefill).await?;
        report["firstReplyTtftMs"] = json!(ttft);
        report["firstReply"] = json!(first);

        llm.unload().await.map_err(|e| e.to_string())?;
        let warm = llm.load(request.clone()).await.map_err(|e| e.to_string())?;
        report["warmLoadMs"] = json!(warm.load_ms);

        let bench = llm.bench(BenchRequest { pp: 512, tg: 128, reps: 3 }).await.map_err(|e| e.to_string())?;
        bench_log(json!({ "event": "bench", "id": entry.id, "bench": bench }));
        report["bench"] = json!(bench);

        // Calidad: respuestas deterministas (temperatura 0) a 10 preguntas en es-MX.
        let mut quality = Vec::new();
        for (i, prompt) in QUALITY_PROMPTS.iter().enumerate() {
            let messages = build_messages(metrics, prompt);
            let (text, ttft, r) = generate_collect(app, messages, 220, 0.0, 7, entry.think_prefill).await?;
            let item = json!({ "n": i + 1, "prompt": prompt, "answer": text, "ttftMs": ttft, "tgTps": r.tg_tps, "nGen": r.n_gen, "stop": r.stop_reason });
            bench_log(json!({ "event": "quality", "id": entry.id, "item": item }));
            quality.push(item);
        }
        report["quality"] = json!(quality);

        // Prueba sostenida: generar sin parar durante `soak_secs`.
        let soak_start = Instant::now();
        let mut soak = Vec::new();
        let mut seed = 100;
        while soak_start.elapsed() < Duration::from_secs(soak_secs) {
            let messages = vec![
                ChatMessage { role: "system".into(), content: PERSONA.into() },
                ChatMessage { role: "user".into(), content: SOAK_PROMPT.into() },
            ];
            let (_, _, r) = generate_collect(app, messages, 256, 0.7, seed, entry.think_prefill).await?;
            seed += 1;
            let s = metrics.latest();
            let sample = json!({
                "t": soak_start.elapsed().as_secs(),
                "tgTps": r.tg_tps,
                "stop": r.stop_reason,
                "thermal": s.as_ref().map(|s| s.thermal),
                "battery": s.as_ref().and_then(|s| s.battery.map(|b| b.level)),
                "footprint": s.as_ref().and_then(|s| s.app_footprint),
            });
            bench_log(json!({ "event": "soak", "id": entry.id, "sample": sample }));
            soak.push(sample);
            if r.stop_reason == "thermal" || r.stop_reason == "cancelled" {
                break;
            }
        }
        report["soak"] = json!(soak);
        llm.unload().await.map_err(|e| e.to_string())?;

        // Estabilidad: tres ciclos de carga y descarga.
        let mut cycles = 0;
        for _ in 0..3 {
            llm.load(request.clone()).await.map_err(|e| e.to_string())?;
            llm.unload().await.map_err(|e| e.to_string())?;
            cycles += 1;
        }
        report["loadUnloadCycles"] = json!(cycles);
        Ok::<(), String>(())
    }
    .await;

    stop.store(true, Ordering::Relaxed);
    let _ = llm.unload().await;
    report["peaks"] = json!(*peaks.lock().unwrap_or_else(|e| e.into_inner()));
    report["batteryAfter"] = json!(metrics.latest().and_then(|s| s.battery.map(|b| b.level)));
    if let Err(e) = result {
        report["error"] = json!(e);
    }
    report
}

pub async fn auto_bench(app: AppHandle, spec: String) {
    let _ = BENCH_APP.set(app.clone());
    let soak_secs = env_u64("IOS_STATS_SOAK_SECS", 600);
    let threads = env_u64("IOS_STATS_THREADS", 2) as u32;
    let models: Vec<CatalogEntry> = catalog::catalog()
        .into_iter()
        .filter(|m| spec == "all" || spec.split(',').any(|id| id.trim() == m.id))
        .collect();
    let metrics = app.state::<MetricsState>().inner().clone();

    // Espera a que el monitor tenga lecturas (la CPU necesita dos ticks).
    for _ in 0..20 {
        if metrics.latest().is_some_and(|s| s.cpu.is_some()) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    bench_log(json!({ "event": "start", "models": models.iter().map(|m| &m.id).collect::<Vec<_>>(), "soakSecs": soak_secs, "threads": threads }));
    let _ = app.llm().keep_awake(true).await;

    // Se guarda después de cada modelo para no perder resultados si la prueba se corta.
    let path = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Documents/llm-bench.json");
    let mut reports = Vec::new();
    for entry in &models {
        let report = bench_model(&app, &metrics, entry, soak_secs, threads).await;
        bench_log(json!({ "event": "model_done", "report": report }));
        reports.push(report);
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&reports).unwrap_or_default());
    }

    let _ = app.llm().keep_awake(false).await;
    let done = path.with_file_name("llm-bench.done");
    let _ = std::fs::write(&done, "ok");
    bench_log(json!({ "event": "done", "file": path }));
}
