//! How the prompt is built: persona, trimmed history, per-turn note and question.
//!
//! The per-turn note is a separate system message right before the question. It holds
//! the iPhone data and the reply language ("Reply in Spanish."). In the phase A test,
//! when the data was inside the user message, the small models copied it into the reply.
//! The idea of giving the model live metrics comes from `src-tauri/src/prompts/mod.rs`
//! in the Mac app.
//!
//! The prompt is English for every language: small models follow English instructions
//! best, and the reply language is set per turn instead of in the persona.
//!
//! Each turn's prompt continues the previous one exactly (each question's note is saved
//! with it and the history is trimmed in jumps). That way the engine reuses what it
//! already processed: Qwen3.5 and LFM2.5 are hybrids and can only reuse an exact prefix;
//! if anything at the start changes, they process everything again. Because the persona
//! never changes, switching the reply language does not force that.

use tauri_plugin_llm::ChatMessage;

use super::store::StoredMessage;
use crate::language::ReplyLanguage;
use crate::metrics::apple::{BatteryState, Thermal};
use crate::metrics::Snapshot;

/// Persona rewritten from `src-tauri/agent/soul.md` in the Mac app. Kept free of facts:
/// in the multilingual benchmark, facts placed here (thermal levels, where to free space)
/// leaked into unrelated answers, so the per-turn note carries them only when relevant.
pub const PERSONA: &str = "You are the assistant of iOS Stats, an app that monitors the state of this iPhone. \
Answer briefly, clearly and kindly. \
Before each question you receive a system message with the iPhone's current data and the language to answer in. \
Use the data only when the question is about this iPhone; otherwise ignore it. \
Never invent data and never copy that list into your answer. Always answer in the language that message asks for.";

/// History bytes (UTF-8) that fit comfortably in 4096 tokens alongside the persona,
/// the per-turn note, the question and up to 512 reply tokens. CJK text uses about three
/// bytes per character and roughly one token per character, so it stays within budget.
pub const HISTORY_BUDGET_BYTES: usize = 8_000;

fn gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
}

/// A UI string in `language`, read from the app's dictionaries (`src/i18n/<lang>.ts`), so
/// the model gets the same words the app shows. English for languages the UI lacks.
fn ui_text(key: &str, language: &str) -> Option<String> {
    let dictionary = match language {
        "es" => include_str!("../../../src/i18n/es.ts"),
        "de" => include_str!("../../../src/i18n/de.ts"),
        "fr" => include_str!("../../../src/i18n/fr.ts"),
        "pt" | "pt-BR" => include_str!("../../../src/i18n/pt-BR.ts"),
        "zh-Hans" => include_str!("../../../src/i18n/zh-Hans.ts"),
        _ => include_str!("../../../src/i18n/en.ts"),
    };
    let (_, rest) = dictionary.split_once(&format!("\"{key}\": \""))?;
    let (text, _) = rest.split_once('"')?;
    Some(text.replace("\\u00a0", "\u{a0}"))
}

/// Thermal state as the app shows it, with what it means: "Serio (Alto. iOS baja el
/// rendimiento para enfriarse.)". A bare number ("3 of 4") was read as "moderate".
fn thermal_text(t: Thermal, language: &str) -> Option<String> {
    let key = match t {
        Thermal::Nominal => "nominal",
        Thermal::Fair => "fair",
        Thermal::Serious => "serious",
        Thermal::Critical => "critical",
        Thermal::Unknown => return None,
    };
    let word = ui_text(&format!("thermal.{key}"), language)?;
    Some(match ui_text(&format!("thermal.meaning.{key}"), language) {
        Some(meaning) => format!("{word} ({meaning})"),
        None => word,
    })
}

/// Apple's path to free up space, in the reply language (from the app's dictionaries).
fn storage_path(language: &str) -> String {
    ui_text("prompt.storagePath", language).unwrap_or_else(|| "Settings > General > iPhone Storage".into())
}

/// Whether a question is about storage space, in any of the app's languages. Bare German
/// "Speicher" is left out: it also means memory.
fn asks_about_space(question: &str) -> bool {
    const WORDS: [&str; 14] = [
        "space", "storage", "espacio", "almacenamiento", "speicherplatz", "platz", "iphone-speicher",
        "espace", "stockage", "espaço", "armazenamento", "空间", "存储", "储存",
    ];
    let q = question.to_lowercase();
    WORDS.iter().any(|w| q.contains(w))
}

/// iPhone data, one labeled value per line: in the benchmark, models read an unlabeled
/// "55.8 GB free" as free memory, the app's own memory as the phone's RAM, and subtracted
/// storage figures wrongly. `language` picks the words for the thermal state.
pub fn device_note(s: &Snapshot, language: &str) -> String {
    let mut lines = vec!["This iPhone now (use only if the question is about it):".to_string()];
    if let Some(cpu) = s.cpu {
        lines.push(format!("- CPU: {cpu:.0}%"));
    }
    if let Some(used) = s.ram_used {
        let free = s.ram_total.saturating_sub(used);
        lines.push(format!("- RAM: {} used, {} free of {}", gb(used), gb(free), gb(s.ram_total)));
    }
    if let Some(b) = s.battery {
        let state = match b.state {
            BatteryState::Charging => " (charging)",
            BatteryState::Full => " (full)",
            BatteryState::Unplugged => " (not charging)",
            BatteryState::Unknown => "",
        };
        lines.push(format!("- Battery charge: {:.0}%{state}", b.level * 100.0));
    }
    if let Some(thermal) = thermal_text(s.thermal, language) {
        lines.push(format!("- Thermal state: {thermal}"));
    }
    if s.low_power {
        lines.push("- Low Power Mode: on".into());
    }
    if let Some(st) = s.storage {
        let used = st.total.saturating_sub(st.available);
        lines.push(format!("- Storage: {} used, {} free of {}", gb(used), gb(st.available), gb(st.total)));
    }
    lines.join("\n")
}

/// System message sent before a question: iPhone data (if any), a fact the question needs
/// (where to free space), then the reply language last, where small models follow it best.
pub fn turn_note(device: Option<&Snapshot>, reply: &ReplyLanguage, question: &str) -> String {
    let mut parts = Vec::new();
    if let Some(s) = device {
        parts.push(device_note(s, &reply.code));
    }
    if asks_about_space(question) {
        let path = storage_path(&reply.code);
        parts.push(format!("To free up space on an iPhone: {path} (offload or delete apps there)."));
    }
    parts.push(format!("Reply in {}.", reply.english_name));
    parts.join("\n")
}

/// History for the model: each question is preceded by the note it was given at the
/// time, just as when the reply was generated.
pub fn history(messages: &[StoredMessage]) -> Vec<ChatMessage> {
    let mut out = Vec::new();
    for m in messages {
        if let Some(note) = &m.turn_note {
            out.push(ChatMessage { role: "system".into(), content: note.clone() });
        }
        out.push(ChatMessage { role: m.role.clone(), content: m.content.clone() });
    }
    out
}

/// Persona + history + turn note + question. If the history exceeds `budget_bytes`, it
/// is trimmed from the start in jumps of half the budget, so that the start does not
/// change on every turn. It never starts with a reply.
pub fn build(history: &[ChatMessage], note: Option<&str>, question: &str, budget_bytes: usize) -> Vec<ChatMessage> {
    let total: usize = history.iter().map(|m| m.content.len()).sum();
    let step = (budget_bytes / 2).max(1);
    let drop = total.saturating_sub(budget_bytes).div_ceil(step) * step;
    let mut start = 0;
    let mut dropped = 0;
    while start < history.len() && dropped < drop {
        dropped += history[start].content.len();
        start += 1;
    }
    while start < history.len() && history[start].role == "assistant" {
        start += 1;
    }

    let mut messages = vec![ChatMessage { role: "system".into(), content: PERSONA.into() }];
    messages.extend_from_slice(&history[start..]);
    if let Some(note) = note {
        messages.push(ChatMessage { role: "system".into(), content: note.into() });
    }
    messages.push(ChatMessage { role: "user".into(), content: question.into() });
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::apple::{Battery, Storage};

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage { role: role.into(), content: content.into() }
    }

    fn snapshot() -> Snapshot {
        Snapshot {
            ts: 0,
            cpu: Some(23.4),
            ram_used: Some(3 * 1_073_741_824),
            ram_total: 6 * 1_073_741_824,
            app_footprint: Some(1_610_612_736),
            app_available: Some(1_073_741_824),
            net_down: None,
            net_up: None,
            battery: Some(Battery { level: 0.78, state: BatteryState::Charging }),
            storage: Some(Storage { total: 128 * 1_073_741_824, available: 41 * 1_073_741_824 }),
            thermal: Thermal::Fair,
            low_power: false,
        }
    }

    #[test]
    fn device_note_labels_every_value() {
        assert_eq!(
            device_note(&snapshot(), "en"),
            "This iPhone now (use only if the question is about it):\n\
- CPU: 23%\n\
- RAM: 3.0 GB used, 3.0 GB free of 6.0 GB\n\
- Battery charge: 78% (charging)\n\
- Thermal state: Fair (Slightly elevated. iOS may reduce background work.)\n\
- Storage: 87.0 GB used, 41.0 GB free of 128.0 GB"
        );
    }

    #[test]
    fn thermal_state_uses_the_app_words_in_the_reply_language() {
        assert!(device_note(&snapshot(), "es").contains("- Thermal state: Moderado (Algo elevado."));
        assert!(device_note(&snapshot(), "zh-Hans").contains("- Thermal state: 一般 ("));
        assert!(device_note(&snapshot(), "it").contains("- Thermal state: Fair ("));
        assert_eq!(
            thermal_text(Thermal::Serious, "fr").as_deref(),
            Some("Sérieux (Élevé. iOS réduit les performances pour refroidir.)")
        );
        assert_eq!(thermal_text(Thermal::Unknown, "en"), None);
    }

    #[test]
    fn storage_tip_only_for_questions_about_space() {
        let spanish = ReplyLanguage { code: "es".into(), english_name: "Spanish".into() };
        let note = turn_note(None, &spanish, "¿Cómo libero espacio en mi iPhone?");
        assert!(note.contains("Ajustes > General > Almacenamiento del iPhone"));
        assert!(note.ends_with("\nReply in Spanish."));
        assert!(!turn_note(None, &spanish, "¿Qué es la memoria RAM?").contains("Ajustes"));
        // German "Speicher" alone also means memory: no tip.
        let german = ReplyLanguage { code: "de".into(), english_name: "German".into() };
        assert!(!turn_note(None, &german, "Wie stehen Batterie und Speicher?").contains("Einstellungen"));
        assert!(turn_note(None, &german, "Wie bekomme ich mehr Speicherplatz?").contains("iPhone-Speicher"));
    }

    #[test]
    fn turn_note_ends_with_the_reply_language() {
        let german = ReplyLanguage { code: "de".into(), english_name: "German".into() };
        let note = turn_note(Some(&snapshot()), &german, "Hallo");
        assert!(note.starts_with("This iPhone now"));
        assert!(note.contains("Erhöht"));
        assert!(note.ends_with("\nReply in German."));
        let chinese = ReplyLanguage { code: "zh-Hans".into(), english_name: "Simplified Chinese".into() };
        assert_eq!(turn_note(None, &chinese, "你好"), "Reply in Simplified Chinese.");
    }

    #[test]
    fn order_is_persona_history_note_question() {
        let history = vec![msg("user", "hi"), msg("assistant", "hello!")];
        let out = build(&history, Some("Note"), "and the battery?", 1_000);
        let roles: Vec<_> = out.iter().map(|m| m.role.as_str()).collect();
        assert_eq!(roles, ["system", "user", "assistant", "system", "user"]);
        assert_eq!(out[0].content, PERSONA);
        assert_eq!(out[3].content, "Note");
        assert_eq!(out[4].content, "and the battery?");
    }

    #[test]
    fn history_is_trimmed_from_the_start_and_never_starts_with_a_reply() {
        let history = vec![
            msg("user", &"a".repeat(50)),
            msg("assistant", &"b".repeat(50)),
            msg("user", &"c".repeat(50)),
            msg("assistant", &"d".repeat(50)),
        ];
        // 40 bytes too many: half the budget (80) is removed, i.e. «a» and «b».
        let out = build(&history, None, "?", 160);
        let contents: Vec<_> = out[1..out.len() - 1].iter().map(|m| &m.content[..1]).collect();
        assert_eq!(contents, ["c", "d"]);

        // If the cut lands on a reply, that one is skipped too.
        let history = vec![
            msg("user", &"a".repeat(100)),
            msg("assistant", &"b".repeat(10)),
            msg("user", &"c".repeat(50)),
            msg("assistant", &"d".repeat(50)),
        ];
        let out = build(&history, None, "?", 200);
        let contents: Vec<_> = out[1..out.len() - 1].iter().map(|m| &m.content[..1]).collect();
        assert_eq!(contents, ["c", "d"]);
    }

    #[test]
    fn trimming_jumps_so_the_start_stays_the_same_for_several_turns() {
        let turns: Vec<_> = (0..16)
            .map(|i| msg(if i % 2 == 0 { "user" } else { "assistant" }, &format!("{i:03}").repeat(33)))
            .collect();
        let first = |n: usize| build(&turns[..n], None, "?", 1_000)[1].content.clone();
        // 10 messages of 99 bytes fit entirely.
        assert_eq!(first(10), turns[0].content);
        // From 11 to 15 the excess is under half the budget: 6 are removed and the start does not change.
        for n in 11..=15 {
            assert_eq!(first(n), turns[6].content, "with {n} messages");
        }
        assert_eq!(first(16), turns[12].content);
    }

    #[test]
    fn history_repeats_each_question_with_its_note() {
        let stored = |role: &str, content: &str, note: Option<&str>| StoredMessage {
            role: role.into(),
            content: content.into(),
            ts: 0,
            turn_note: note.map(Into::into),
            language: None,
            stats: None,
        };
        let out = history(&[stored("user", "hi", Some("Note 1")), stored("assistant", "hello!", None)]);
        let pairs: Vec<_> = out.iter().map(|m| (m.role.as_str(), m.content.as_str())).collect();
        assert_eq!(pairs, [("system", "Note 1"), ("user", "hi"), ("assistant", "hello!")]);
    }

    #[test]
    fn a_language_change_only_changes_the_end_of_the_prompt() {
        let history = vec![msg("system", "Reply in Spanish."), msg("user", "hola"), msg("assistant", "¡Hola!")];
        let spanish = build(&history, Some("Reply in Spanish."), "¿qué tal?", 1_000);
        let english = build(&history, Some("Reply in English."), "how are you?", 1_000);
        assert_eq!(spanish[..4], english[..4]);
    }

    #[test]
    fn without_note_there_is_no_extra_system_message() {
        let out = build(&[], None, "hi", 100);
        assert_eq!(out.len(), 2);
    }
}
