//! How the prompt is built: persona, trimmed history, iPhone data and question.
//!
//! The iPhone data goes in a separate system message right before the question:
//! in the phase A test, when it was inside the user message, the small models
//! copied it into the reply. The idea of giving the model live metrics comes
//! from `src-tauri/src/prompts/mod.rs` in the Mac app.
//!
//! Each turn's prompt continues the previous one exactly (each question's data is
//! saved with it and the history is trimmed in jumps). That way the engine reuses
//! what it already processed: Qwen3.5 and LFM2.5 are hybrids and can only reuse
//! an exact prefix; if anything at the start changes, they process everything again.

use tauri_plugin_llm::ChatMessage;

use super::store::StoredMessage;
use crate::metrics::apple::{BatteryState, Thermal};
use crate::metrics::Snapshot;

/// Persona rewritten from `src-tauri/agent/soul.md` in the Mac app, in es-MX.
pub const PERSONA: &str = "Eres el asistente de iOS Stats, una app que vigila el estado de este iPhone. \
Respondes en español de México, de forma breve, clara y amable. \
Antes de cada pregunta recibes un mensaje de sistema con los datos actuales del iPhone: \
úsalos solo si la pregunta trata del teléfono, no inventes datos que no aparezcan ahí \
y nunca copies esa lista en tu respuesta. \
Estados térmicos de iOS: normal (sin problema), templado (algo caliente), \
caliente o «serio» (iOS baja el rendimiento para enfriarse) y crítico (muy caliente, conviene dejar de usarlo).";

/// History characters that fit comfortably in 4096 tokens alongside the persona,
/// the iPhone data, the question and up to 512 reply tokens.
pub const HISTORY_BUDGET_CHARS: usize = 8_000;

fn gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
}

fn thermal_es(t: Thermal) -> &'static str {
    match t {
        Thermal::Nominal => "normal",
        Thermal::Fair => "templado",
        Thermal::Serious => "caliente",
        Thermal::Critical => "crítico",
        Thermal::Unknown => "desconocido",
    }
}

/// iPhone data on one line.
pub fn device_note(s: &Snapshot) -> String {
    let mut parts = Vec::new();
    if let Some(cpu) = s.cpu {
        parts.push(format!("CPU {cpu:.0} %"));
    }
    if let Some(used) = s.ram_used {
        parts.push(format!("RAM {} de {}", gb(used), gb(s.ram_total)));
    }
    if let Some(app) = s.app_footprint {
        match s.app_available {
            Some(margin) => parts.push(format!("memoria de la app {} (margen {})", gb(app), gb(margin))),
            None => parts.push(format!("memoria de la app {}", gb(app))),
        }
    }
    if let Some(b) = s.battery {
        let state = match b.state {
            BatteryState::Charging => ", cargando",
            BatteryState::Full => ", llena",
            BatteryState::Unplugged => ", sin cargador",
            BatteryState::Unknown => "",
        };
        parts.push(format!("batería {:.0} %{state}", b.level * 100.0));
    }
    parts.push(format!("estado térmico {}", thermal_es(s.thermal)));
    if s.low_power {
        parts.push("modo de bajo consumo activado".into());
    }
    if let Some(st) = s.storage {
        parts.push(format!("{} libres", gb(st.available)));
    }
    format!("Datos actuales de este iPhone: {}.", parts.join(", "))
}

/// History for the model: each question is preceded by the iPhone data it was given
/// at the time, just as when the reply was generated.
pub fn history(messages: &[StoredMessage]) -> Vec<ChatMessage> {
    let mut out = Vec::new();
    for m in messages {
        if let Some(note) = &m.device_note {
            out.push(ChatMessage { role: "system".into(), content: note.clone() });
        }
        out.push(ChatMessage { role: m.role.clone(), content: m.content.clone() });
    }
    out
}

/// Persona + history + iPhone data + question. If the history exceeds
/// `budget_chars`, it is trimmed from the start in jumps of half the budget, so that
/// the start does not change on every turn. It never starts with a reply.
pub fn build(history: &[ChatMessage], device: Option<&str>, question: &str, budget_chars: usize) -> Vec<ChatMessage> {
    let total: usize = history.iter().map(|m| m.content.len()).sum();
    let step = (budget_chars / 2).max(1);
    let drop = total.saturating_sub(budget_chars).div_ceil(step) * step;
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
    if let Some(note) = device {
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

    #[test]
    fn device_note_is_compact_and_in_spanish() {
        let s = Snapshot {
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
        };
        assert_eq!(
            device_note(&s),
            "Datos actuales de este iPhone: CPU 23 %, RAM 3.0 GB de 6.0 GB, memoria de la app 1.5 GB \
(margen 1.0 GB), batería 78 %, cargando, estado térmico templado, 41.0 GB libres."
        );
    }

    #[test]
    fn order_is_persona_history_device_question() {
        let history = vec![msg("user", "hola"), msg("assistant", "¡hola!")];
        let out = build(&history, Some("Datos"), "¿y la batería?", 1_000);
        let roles: Vec<_> = out.iter().map(|m| m.role.as_str()).collect();
        assert_eq!(roles, ["system", "user", "assistant", "system", "user"]);
        assert_eq!(out[0].content, PERSONA);
        assert_eq!(out[3].content, "Datos");
        assert_eq!(out[4].content, "¿y la batería?");
    }

    #[test]
    fn history_is_trimmed_from_the_start_and_never_starts_with_a_reply() {
        let history = vec![
            msg("user", &"a".repeat(50)),
            msg("assistant", &"b".repeat(50)),
            msg("user", &"c".repeat(50)),
            msg("assistant", &"d".repeat(50)),
        ];
        // 40 characters too many: half the budget (80) is dropped, i.e. `a` and `b`.
        let out = build(&history, None, "?", 160);
        let contents: Vec<_> = out[1..out.len() - 1].iter().map(|m| &m.content[..1]).collect();
        assert_eq!(contents, ["c", "d"]);

        // If the cut lands on a reply, that reply is skipped too.
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
        // 10 messages of 99 characters fit entirely.
        assert_eq!(first(10), turns[0].content);
        // From 11 to 15 the overflow is under half the budget: 6 are dropped and the start stays the same.
        for n in 11..=15 {
            assert_eq!(first(n), turns[6].content, "with {n} messages");
        }
        assert_eq!(first(16), turns[12].content);
    }

    #[test]
    fn history_repeats_each_question_with_its_device_note() {
        let stored = |role: &str, content: &str, note: Option<&str>| StoredMessage {
            role: role.into(),
            content: content.into(),
            ts: 0,
            device_note: note.map(Into::into),
            stats: None,
        };
        let out = history(&[stored("user", "hola", Some("Datos 1")), stored("assistant", "¡hola!", None)]);
        let pairs: Vec<_> = out.iter().map(|m| (m.role.as_str(), m.content.as_str())).collect();
        assert_eq!(pairs, [("system", "Datos 1"), ("user", "hola"), ("assistant", "¡hola!")]);
    }

    #[test]
    fn without_device_note_there_is_no_extra_system_message() {
        let out = build(&[], None, "hola", 100);
        assert_eq!(out.len(), 2);
    }
}
