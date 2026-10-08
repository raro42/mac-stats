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
use crate::metrics::apple::{BatteryState, Thermal};
use crate::metrics::Snapshot;

/// Persona rewritten from `src-tauri/agent/soul.md` in the Mac app.
pub const PERSONA: &str = "You are the assistant of iOS Stats, an app that monitors the state of this iPhone. \
Answer briefly, clearly and kindly. \
Before each question you receive a system message with the iPhone's current data and the language to answer in. \
Use the data only if the question is about the phone, never invent data that is not there, \
and never copy that list into your answer. Always answer in the language that message asks for. \
iOS thermal states: nominal (normal), fair (slightly elevated), \
serious (high: iOS lowers performance to cool down) and critical (very high: better to stop using it).";

/// History bytes (UTF-8) that fit comfortably in 4096 tokens alongside the persona,
/// the per-turn note, the question and up to 512 reply tokens. CJK text uses about three
/// bytes per character and roughly one token per character, so it stays within budget.
pub const HISTORY_BUDGET_BYTES: usize = 8_000;

fn gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
}

/// Same words as the UI in English and the desktop app (`src/i18n/en.ts`, `thermal.*`).
fn thermal_name(t: Thermal) -> &'static str {
    match t {
        Thermal::Nominal => "nominal",
        Thermal::Fair => "fair",
        Thermal::Serious => "serious",
        Thermal::Critical => "critical",
        Thermal::Unknown => "unknown",
    }
}

/// iPhone data on one line.
pub fn device_note(s: &Snapshot) -> String {
    let mut parts = Vec::new();
    if let Some(cpu) = s.cpu {
        parts.push(format!("CPU {cpu:.0}%"));
    }
    if let Some(used) = s.ram_used {
        parts.push(format!("RAM {} of {}", gb(used), gb(s.ram_total)));
    }
    if let Some(app) = s.app_footprint {
        match s.app_available {
            Some(headroom) => parts.push(format!("app memory {} (headroom {})", gb(app), gb(headroom))),
            None => parts.push(format!("app memory {}", gb(app))),
        }
    }
    if let Some(b) = s.battery {
        let state = match b.state {
            BatteryState::Charging => " (charging)",
            BatteryState::Full => " (full)",
            BatteryState::Unplugged => " (not charging)",
            BatteryState::Unknown => "",
        };
        parts.push(format!("battery {:.0}%{state}", b.level * 100.0));
    }
    parts.push(format!("thermal state {}", thermal_name(s.thermal)));
    if s.low_power {
        parts.push("Low Power Mode on".into());
    }
    if let Some(st) = s.storage {
        parts.push(format!("{} free", gb(st.available)));
    }
    format!("Current data for this iPhone: {}.", parts.join(", "))
}

/// System message sent before a question: iPhone data (if any) and the reply language.
pub fn turn_note(device: Option<&Snapshot>, reply_language: &str) -> String {
    let language = format!("Reply in {reply_language}.");
    match device {
        Some(s) => format!("{}\n{language}", device_note(s)),
        None => language,
    }
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
    fn device_note_is_compact() {
        assert_eq!(
            device_note(&snapshot()),
            "Current data for this iPhone: CPU 23%, RAM 3.0 GB of 6.0 GB, app memory 1.5 GB \
(headroom 1.0 GB), battery 78% (charging), thermal state fair, 41.0 GB free."
        );
    }

    #[test]
    fn turn_note_ends_with_the_reply_language() {
        let note = turn_note(Some(&snapshot()), "German");
        assert!(note.starts_with("Current data for this iPhone: "));
        assert!(note.ends_with("\nReply in German."));
        assert_eq!(turn_note(None, "Simplified Chinese"), "Reply in Simplified Chinese.");
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
