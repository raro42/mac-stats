//! Model lab (phase A): load, benchmark and try local models on the iPhone.
//!
//! Besides the commands for the lab card, it includes an automatic mode
//! for debug builds: if the app starts with `IOS_STATS_BENCH=<ids>|all`, it measures
//! each model (load, speed, memory, temperature and answer quality per language), prints one
//! `BENCH {json}` line per event and saves the result in `Documents/`.

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
use crate::chat::{self as chat_mod, store::Conversation, ChatState};
use crate::error::{AppError, ErrorCode};
use crate::metrics::apple::Thermal;
use crate::metrics::MetricsState;

/// Persona, turn note and question: the same prompt the chat uses for a first turn,
/// including the reply language.
async fn build_messages(app: &AppHandle, metrics: &MetricsState, question: &str) -> Vec<ChatMessage> {
    let state = app.state::<ChatState>();
    let empty = Conversation::new(String::new(), 0);
    let (reply, _) = chat_mod::pick_reply_language(app, &state, &empty, question).await;
    let note = prompt::turn_note(metrics.latest().as_ref(), &reply, question);
    prompt::build(&[], Some(&note), question, prompt::HISTORY_BUDGET_BYTES)
}

fn entry(id: &str) -> Result<CatalogEntry, AppError> {
    catalog::find(id).ok_or_else(|| AppError::new(ErrorCode::ModelUnknown).with("id", id))
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
// Commands for the lab card
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
pub async fn lab_load(app: AppHandle, id: String, threads: Option<u32>) -> Result<LoadInfo, AppError> {
    let entry = entry(&id)?;
    Ok(app.llm().load(load_request(&entry, threads.unwrap_or(2))).await?)
}

#[tauri::command]
pub async fn lab_unload(app: AppHandle) -> Result<(), AppError> {
    Ok(app.llm().unload().await?)
}

#[tauri::command]
pub async fn lab_bench(app: AppHandle) -> Result<BenchResult, AppError> {
    Ok(app.llm().bench(BenchRequest { pp: 512, tg: 128, reps: 3 }).await?)
}

#[tauri::command]
pub async fn lab_generate(
    app: AppHandle,
    metrics: State<'_, MetricsState>,
    prompt: String,
    think_prefill: bool,
    on_event: Channel<Value>,
) -> Result<GenerateResult, AppError> {
    let messages = build_messages(&app, &metrics, &prompt).await;
    Ok(app
        .llm()
        .generate(GenerateRequest {
            messages,
            max_tokens: 400,
            temperature: 0.7,
            seed: 42,
            think_prefill,
            on_event,
        })
        .await?)
}

#[tauri::command]
pub async fn lab_cancel(app: AppHandle) -> Result<(), AppError> {
    Ok(app.llm().cancel().await?)
}

// ---------------------------------------------------------------------------
// Automatic benchmark (IOS_STATS_BENCH)
// ---------------------------------------------------------------------------

/// Quality questions per language (test data, so each set is written in its language).
/// The same 10 questions in each; question 7 asks for a translation into another language.
const QUALITY_PROMPTS: [(&str, [&str; 10]); 6] = [
    (
        "es",
        [
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
        ],
    ),
    (
        "en",
        [
            "How are my battery and memory doing right now?",
            "Explain in two sentences what RAM is.",
            "Give me 3 tips to keep my iPhone from overheating.",
            "Summarize in one sentence: “Low Power Mode reduces background activity, lowers brightness and limits some animations so the battery lasts longer.”",
            "What is 17 × 23? Answer with the number only.",
            "Write a haiku about a phone running out of battery.",
            "Translate into Spanish: “My phone has been very slow since yesterday.”",
            "If I have 45 GB free and each video takes 1.5 GB, how many videos fit? Explain the calculation in one line.",
            "List 3 steps to free up space on an iPhone.",
            "What does it mean when my iPhone's thermal state is “hot”?",
        ],
    ),
    (
        "de",
        [
            "Wie stehen meine Batterie und mein Speicher gerade?",
            "Erkläre mir in zwei Sätzen, was RAM ist.",
            "Gib mir 3 Tipps, damit mein iPhone nicht zu heiß wird.",
            "Fasse in einem Satz zusammen: „Der Stromsparmodus reduziert Hintergrundaktivität, senkt die Helligkeit und begrenzt einige Animationen, damit der Akku länger hält.“",
            "Was ist 17 × 23? Antworte nur mit der Zahl.",
            "Schreibe ein Haiku über ein Handy, dem der Akku ausgeht.",
            "Übersetze ins Englische: „Mein Handy ist seit gestern sehr langsam.“",
            "Wenn ich 45 GB frei habe und jedes Video 1,5 GB belegt, wie viele Videos passen drauf? Erkläre die Rechnung in einer Zeile.",
            "Nenne 3 Schritte, um auf einem iPhone Speicherplatz freizugeben.",
            "Was bedeutet es, wenn der thermische Zustand meines iPhones „heiß“ ist?",
        ],
    ),
    (
        "fr",
        [
            "Comment vont ma batterie et ma mémoire en ce moment ?",
            "Explique-moi en deux phrases ce qu'est la mémoire RAM.",
            "Donne-moi 3 conseils pour que mon iPhone ne chauffe pas.",
            "Résume en une phrase : « Le mode Économie d'énergie réduit l'activité en arrière-plan, baisse la luminosité et limite certaines animations pour que la batterie dure plus longtemps. »",
            "Combien font 17 × 23 ? Réponds seulement avec le nombre.",
            "Écris un haïku sur un téléphone qui n'a plus de batterie.",
            "Traduis en anglais : « Mon téléphone est très lent depuis hier. »",
            "Si j'ai 45 Go libres et que chaque vidéo occupe 1,5 Go, combien de vidéos peuvent tenir ? Explique le calcul en une ligne.",
            "Fais une liste de 3 étapes pour libérer de l'espace sur un iPhone.",
            "Que signifie l'état thermique « chaud » de mon iPhone ?",
        ],
    ),
    (
        "pt-BR",
        [
            "Como estão minha bateria e minha memória agora?",
            "Explique em duas frases o que é memória RAM.",
            "Me dê 3 dicas para meu iPhone não esquentar.",
            "Resuma em uma frase: “O modo Pouca Energia reduz a atividade em segundo plano, diminui o brilho e limita algumas animações para a bateria durar mais.”",
            "Quanto é 17 × 23? Responda só com o número.",
            "Escreva um haicai sobre um celular ficando sem bateria.",
            "Traduza para o inglês: “Meu celular está muito lento desde ontem.”",
            "Se tenho 45 GB livres e cada vídeo ocupa 1,5 GB, quantos vídeos cabem? Explique o cálculo em uma linha.",
            "Faça uma lista de 3 passos para liberar espaço em um iPhone.",
            "O que significa o estado térmico do meu iPhone estar “quente”?",
        ],
    ),
    (
        "zh-Hans",
        [
            "我的电池和内存现在怎么样？",
            "用两句话解释什么是内存（RAM）。",
            "给我 3 个让 iPhone 不过热的建议。",
            "用一句话总结：“低电量模式会减少后台活动、降低亮度并限制部分动画，让电池更耐用。”",
            "17 × 23 等于多少？只回答数字。",
            "写一首关于手机电量耗尽的俳句。",
            "翻译成英文：“我的手机从昨天开始变得很慢。”",
            "如果我有 45 GB 可用空间，每个视频占 1.5 GB，能放下多少个视频？用一行说明计算过程。",
            "列出在 iPhone 上释放空间的 3 个步骤。",
            "我的 iPhone 热状态显示“很热”是什么意思？",
        ],
    ),
];

const SOAK_PROMPT: &str = "Cuéntame una historia larga sobre un viaje en tren por la sierra de México, con muchos detalles.";

/// Each event goes to the console, to `Documents/llm-bench-progress.jsonl` (so the Mac
/// can read it with devicectl even when console output is lost) and to the web UI as a banner.
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

/// Peak memory and thermal states while the model is loaded (sampled every 0.5 s).
#[derive(Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Peaks {
    max_footprint: u64,
    min_available: Option<u64>,
    worst_thermal: Option<Thermal>,
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
                p.worst_thermal = Thermal::worst(p.worst_thermal, Some(s.thermal));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    out
}

/// Generates and returns the full text, the time to the first chunk and the result.
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

async fn bench_model(
    app: &AppHandle,
    metrics: &MetricsState,
    entry: &CatalogEntry,
    soak_secs: u64,
    threads: u32,
    langs: &[String],
) -> Value {
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

        // First generation: includes compiling the Metal shaders.
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

        // Quality: deterministic answers (temperature 0) to 10 questions per language
        // (`IOS_STATS_BENCH_LANGS`, default `es`). Each answer's language is detected on
        // the device to check the model replied in the language it was asked in.
        let mut quality = Vec::new();
        for (lang, prompts) in QUALITY_PROMPTS.iter().filter(|(l, _)| langs.iter().any(|x| x == l)) {
            for (i, prompt) in prompts.iter().enumerate() {
                let messages = build_messages(app, metrics, prompt).await;
                let (text, ttft, r) = generate_collect(app, messages.clone(), 220, 0.0, 7, entry.think_prefill).await?;
                let detected = app.llm().detect_language(&text).await.ok().flatten();
                let base = |code: &str| code.split('-').next().unwrap_or(code).to_string();
                // Q5 is only a number (no language to detect) and Q7 asks for a translation
                // into another language, so their expected language differs.
                let expected = match i + 1 {
                    5 => None,
                    7 => Some(if *lang == "en" { "es" } else { "en" }),
                    _ => Some(*lang),
                };
                let language_ok = expected.map(|e| detected.as_ref().map(|d| base(&d.code)) == Some(base(e)));
                // The note sent with the question, so reviewers can check answers against it.
                let note = messages.iter().rev().find(|m| m.role == "system").map(|m| m.content.clone());
                let item = json!({
                    "lang": lang, "n": i + 1, "prompt": prompt, "note": note, "answer": text, "ttftMs": ttft,
                    "tgTps": r.tg_tps, "nGen": r.n_gen, "stop": r.stop_reason,
                    "answerLanguage": detected.map(|d| d.code), "languageOk": language_ok,
                });
                bench_log(json!({ "event": "quality", "id": entry.id, "item": item }));
                quality.push(item);
            }
        }
        report["quality"] = json!(quality);

        // Soak test: generate nonstop for `soak_secs`.
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

        // Stability: three load/unload cycles.
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
    let langs: Vec<String> = std::env::var("IOS_STATS_BENCH_LANGS")
        .unwrap_or_else(|_| "es".into())
        .split(',')
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let models: Vec<CatalogEntry> = catalog::catalog()
        .into_iter()
        .filter(|m| spec == "all" || spec.split(',').any(|id| id.trim() == m.id))
        .collect();
    let metrics = app.state::<MetricsState>().inner().clone();

    // Wait until the monitor has readings (CPU usage needs two ticks).
    for _ in 0..20 {
        if metrics.latest().is_some_and(|s| s.cpu.is_some()) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    bench_log(json!({ "event": "start", "models": models.iter().map(|m| &m.id).collect::<Vec<_>>(), "soakSecs": soak_secs, "threads": threads, "langs": langs }));
    let _ = app.llm().keep_awake(true).await;

    // Saved after each model so no results are lost if the run is cut short.
    let path = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Documents/llm-bench.json");
    let mut reports = Vec::new();
    for entry in &models {
        let report = bench_model(&app, &metrics, entry, soak_secs, threads, &langs).await;
        bench_log(json!({ "event": "model_done", "report": report }));
        reports.push(report);
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&reports).unwrap_or_default());
    }

    let _ = app.llm().keep_awake(false).await;
    let done = path.with_file_name("llm-bench.done");
    let _ = std::fs::write(&done, "ok");
    bench_log(json!({ "event": "done", "file": path }));
}
