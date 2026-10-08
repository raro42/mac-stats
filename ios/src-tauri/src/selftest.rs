//! Chat self-test (phase B), debug builds only.
//!
//! With `IOS_STATS_SELFTEST=1` the app checks the phase B criteria on the iPhone itself
//! and saves the result in `Documents/selftest.json` (and `selftest.done` when it
//! finishes):
//! - loading the selected model and a multi-turn chat saved to disk, where each
//!   turn reuses what the engine already processed in the previous one;
//! - the Stop button stops the reply in under 300 ms;
//! - going to the background cancels generation;
//! - after a memory warning the model is released and reloaded on the next question;
//! - with `IOS_STATS_FAKE_THERMAL=critical`, the chat refuses to generate (in that mode it
//!   is the only test, because nothing else can generate);
//! - the reply language follows the user's language changes (es, en, "ok", de);
//! - leaves a conversation with malicious HTML to check by eye that it does not run;
//! - a download with a wrong SHA-256 is rejected and leaves no file behind;
//! - with `IOS_STATS_SELFTEST_DOWNLOAD=<id>`, really downloads that model (with progress).
//!
//! Keeps the screen on while it runs and at the end deletes the test conversations,
//! except the malicious HTML one.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri::{ipc::Channel, AppHandle, Manager};
use tauri_plugin_llm::{DownloadRequest, LlmExt};

use crate::chat::{
    self,
    catalog,
    store::{Conversation, StoredMessage},
    ChatState, SendResult,
};
use crate::error::{AppError, ErrorCode};
use crate::metrics::MetricsState;

const LONG_PROMPT: &str = "Escribe un cuento muy largo, con muchos capítulos, sobre un robot que aprende a cocinar.";

fn quiet() -> Channel<Value> {
    Channel::new(|_| Ok(()))
}

fn log(value: &Value) {
    println!("SELFTEST {value}");
}

/// Starts a long reply in the background and interrupts it with `interrupt` after 3 s.
/// Returns the result and how long it took to stop after the interruption.
async fn interrupted(app: &AppHandle, interrupt: &str) -> (Result<SendResult, AppError>, Duration) {
    let handle = app.clone();
    let task = tauri::async_runtime::spawn(async move {
        let state = handle.state::<ChatState>();
        let metrics = handle.state::<MetricsState>();
        chat::send(&handle, &state, &metrics, None, LONG_PROMPT, quiet()).await
    });
    tokio::time::sleep(Duration::from_secs(3)).await;
    let start = Instant::now();
    let _ = match interrupt {
        "cancel" => app.llm().cancel().await,
        event => app.llm().debug_simulate(event).await,
    };
    let result = match task.await {
        Ok(r) => r,
        Err(_) => Err(ErrorCode::Internal.into()),
    };
    (result, start.elapsed())
}

async fn finish(app: &AppHandle, results: Vec<Value>) {
    let _ = app.llm().keep_awake(false).await;
    let passed = results.iter().filter(|r| r["ok"] == true).count();
    log(&json!({ "event": "done", "passed": passed, "total": results.len() }));
    let documents = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Documents");
    let _ = std::fs::write(documents.join("selftest.json"), serde_json::to_string_pretty(&results).unwrap_or_default());
    let _ = std::fs::write(documents.join("selftest.done"), "ok");
}

pub async fn run(app: AppHandle) {
    let state = app.state::<ChatState>();
    let metrics = app.state::<MetricsState>().inner().clone();
    for _ in 0..20 {
        if metrics.latest().is_some_and(|s| s.cpu.is_some()) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let _ = app.llm().keep_awake(true).await;

    let mut results = Vec::new();
    let mut record = |name: &str, ok: bool, detail: Value| {
        let item = json!({ "test": name, "ok": ok, "detail": detail });
        log(&item);
        results.push(item);
    };

    // Forced critical thermal state: the chat must refuse to generate.
    if std::env::var("IOS_STATS_FAKE_THERMAL").as_deref() == Ok("critical") {
        let refused = chat::send(&app, &state, &metrics, None, "Hola", quiet()).await;
        record(
            "thermal_critical",
            refused.as_ref().is_err_and(|e| e.is(ErrorCode::TooHot)),
            json!({ "result": refused.as_ref().map(|r| &r.stop_reason).map_err(|e| e) }),
        );
        finish(&app, results).await;
        return;
    }

    // Conversations deleted at the end.
    let mut created = Vec::new();

    // 1. Load the selected model.
    let entry = state.selected();
    let start = Instant::now();
    let loaded = chat::ensure_loaded(&app, &state, &entry).await;
    record("load", loaded.is_ok(), json!({ "model": entry.id, "ms": start.elapsed().as_millis(), "error": loaded.err() }));

    // 2. Multi-turn chat saved to disk.
    let questions = ["Hola, ¿qué puedes hacer?", "¿Y cómo va mi batería?", "Resume en una frase lo que me dijiste."];
    let mut conversation_id = None;
    let mut answers = Vec::new();
    let mut errors = Vec::new();
    let mut reused = Vec::new();
    for q in questions {
        match chat::send(&app, &state, &metrics, conversation_id.clone(), q, quiet()).await {
            Ok(r) => {
                reused.push(r.n_cached);
                answers.push(json!({
                    "question": q,
                    "tokensPrompt": r.n_prompt,
                    "reused": r.n_cached,
                    "promptMs": r.prompt_ms.round(),
                    "tgTps": r.tg_tps,
                    "stop": r.stop_reason,
                }));
                conversation_id = Some(r.conversation_id);
            }
            Err(e) => errors.push(e.to_string()),
        }
    }
    created.extend(conversation_id.clone());
    let stored = conversation_id.as_deref().and_then(|id| state.store.get(id));
    let messages = stored.as_ref().map_or(0, |c| c.messages.len());
    let replies: Vec<_> = stored
        .iter()
        .flat_map(|c| c.messages.iter().filter(|m| m.role == "assistant").map(|m| m.content.clone()))
        .collect();
    record(
        "multi_turn_chat",
        errors.is_empty() && messages == 6 && replies.iter().all(|r| !r.trim().is_empty()),
        json!({ "savedMessages": messages, "replies": replies, "turns": answers, "errors": errors }),
    );
    // Turns 2 and 3 continue the previous prompt: the engine must not reprocess all of it.
    record(
        "reuses_cache",
        reused.len() == 3 && reused[1..].iter().all(|&n| n > 0),
        json!({ "reusedPerTurn": reused }),
    );

    // 2b. The reply language follows the user: a clear message in another language
    // switches it, a short "ok" keeps it. Each reply's language is checked with the same
    // on-device detector.
    let turns = [
        ("Hola, ¿me explicas en dos frases qué es la memoria RAM?", "es"),
        ("Thanks! Now explain in two sentences what the CPU does.", "en"),
        ("ok", "en"),
        ("Danke! Erkläre mir bitte in zwei Sätzen, was der Akku macht.", "de"),
    ];
    let mut switch_id = None;
    let mut switch_ok = true;
    let mut switch_turns = Vec::new();
    for (question, expected) in turns {
        let result = chat::send(&app, &state, &metrics, switch_id.clone(), question, quiet()).await;
        let (asked, detected) = match &result {
            Ok(r) => {
                switch_id = Some(r.conversation_id.clone());
                let reply = state
                    .store
                    .get(&r.conversation_id)
                    .and_then(|c| c.messages.last().filter(|m| m.role == "assistant").map(|m| m.content.clone()))
                    .unwrap_or_default();
                let detected = app.llm().detect_language(&reply).await.ok().flatten().map(|d| d.code);
                (Some(r.reply_language.clone()), detected)
            }
            Err(_) => (None, None),
        };
        let base = |code: &str| code.split('-').next().unwrap_or(code).to_string();
        let ok = asked.as_deref().map(base).as_deref() == Some(expected)
            && detected.as_deref().map(base).as_deref() == Some(expected);
        switch_ok &= ok;
        switch_turns.push(json!({
            "question": question,
            "expected": expected,
            "asked": asked,
            "replyDetected": detected,
            "ok": ok,
            "error": result.err().map(|e| e.to_string()),
        }));
    }
    created.extend(switch_id);
    record("language_switch", switch_ok, json!({ "turns": switch_turns }));

    // 3. Stop button.
    let (reply, latency) = interrupted(&app, "cancel").await;
    created.extend(reply.as_ref().ok().map(|r| r.conversation_id.clone()));
    let stop = reply.map(|r| r.stop_reason);
    record(
        "stop",
        stop.as_deref() == Ok("cancelled") && latency < Duration::from_millis(300),
        json!({ "stop": stop, "ms": latency.as_millis() }),
    );

    // 4. Going to the background.
    let (reply, latency) = interrupted(&app, "resignActive").await;
    created.extend(reply.as_ref().ok().map(|r| r.conversation_id.clone()));
    let stop = reply.map(|r| r.stop_reason);
    record("background", stop.as_deref() == Ok("cancelled"), json!({ "stop": stop, "ms": latency.as_millis() }));

    // 5. Memory warning: the model is released and the next question reloads it.
    let _ = app.llm().debug_simulate("memoryWarning").await;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let released = app.llm().status().await.map(|s| !s.loaded).unwrap_or(false);
    let again = chat::send(&app, &state, &metrics, None, "¿Sigues ahí?", quiet()).await;
    created.extend(again.as_ref().ok().map(|r| r.conversation_id.clone()));
    record(
        "memory_warning",
        released && again.is_ok(),
        json!({ "modelReleased": released, "repliesAfter": again.as_ref().map(|r| &r.stop_reason).map_err(|e| e) }),
    );

    // 6. Download with a wrong SHA-256: rejected and no file is left behind.
    let bad_file = "selftest-bad-sha.bin";
    let bad = app
        .llm()
        .download(DownloadRequest {
            url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/91cad51170dc346986eccefdc2dd33a9da36ead9/README.md".into(),
            sha256: "0".repeat(64),
            size: 10_000,
            file: bad_file.into(),
            allow_cellular: false,
            on_event: quiet(),
        })
        .await;
    let leftover = catalog::models_dir().join(bad_file).exists();
    record(
        "download_wrong_sha",
        bad.as_ref().is_err_and(|e| e.code() == Some("checksum_mismatch")) && !leftover,
        json!({ "result": bad.as_ref().map_err(|e| e.to_string()), "fileLeftBehind": leftover }),
    );

    // 7. Real download of a catalog model (optional).
    if let Ok(id) = std::env::var("IOS_STATS_SELFTEST_DOWNLOAD") {
        if let Some(entry) = catalog::find(&id) {
            let events = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
            let counter = events.clone();
            let progress = Channel::<Value>::new(move |_| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            });
            let start = Instant::now();
            let result = app
                .llm()
                .download(DownloadRequest {
                    url: entry.url(),
                    sha256: entry.sha256.clone(),
                    size: entry.size,
                    file: entry.file.clone(),
                    allow_cellular: false,
                    on_event: progress,
                })
                .await;
            let secs = start.elapsed().as_secs_f64();
            record(
                "real_download",
                result.is_ok() && entry.installed(),
                json!({
                    "model": id,
                    "seconds": secs.round(),
                    "MBps": (entry.size as f64 / 1_048_576.0 / secs * 10.0).round() / 10.0,
                    "progressEvents": events.load(std::sync::atomic::Ordering::Relaxed),
                    "error": result.err().map(|e| e.to_string()),
                }),
            );
        }
    }

    // 8. Conversation with malicious HTML, to check by eye that it is shown as text.
    let now = chat::now_ms();
    let mut xss = Conversation::new(chat::new_id(), now);
    xss.title = "Malicious HTML test".into();
    xss.messages.push(StoredMessage {
        role: "user".into(),
        content: "Show this HTML".into(),
        ts: now,
        turn_note: None,
        language: None,
        stats: None,
    });
    xss.messages.push(StoredMessage {
        role: "assistant".into(),
        content: "**Normal** text. <img src=x onerror=\"document.body.style.background='red'\"> \
<script>document.body.innerHTML='HACKED'</script> <a href=\"javascript:alert(1)\">link</a> \
<iframe src=\"https://example.com\"></iframe> <b style=\"color:red\">end</b>"
            .into(),
        ts: now + 1,
        turn_note: None,
        language: None,
        stats: None,
    });
    let saved = state.store.save(&xss);
    record("xss_saved", saved.is_ok(), json!({ "id": xss.id }));

    for id in &created {
        let _ = state.store.delete(id);
    }
    finish(&app, results).await;
}
