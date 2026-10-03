//! Cómo se arma el prompt: persona, historial recortado, datos del iPhone y pregunta.
//!
//! Los datos del iPhone van en un mensaje de sistema aparte, justo antes de la pregunta:
//! en la prueba de la fase A, cuando iban dentro del mensaje del usuario, los modelos
//! pequeños los copiaban en la respuesta. La idea de darle al modelo las métricas en
//! vivo viene de `src-tauri/src/prompts/mod.rs` de la app de Mac.
//!
//! El prompt de cada turno continúa exactamente el del anterior (los datos de cada
//! pregunta se guardan con ella y el historial se recorta a saltos). Así el motor
//! reutiliza lo ya procesado: Qwen3.5 y LFM2.5 son híbridos y solo pueden reutilizar
//! un prefijo exacto; si cambia algo del principio, procesan todo otra vez.

use tauri_plugin_llm::ChatMessage;

use super::store::StoredMessage;
use crate::metrics::apple::{BatteryState, Thermal};
use crate::metrics::Snapshot;

/// Persona reescrita a partir de `src-tauri/agent/soul.md` de la app de Mac, en es-MX.
pub const PERSONA: &str = "Eres el asistente de iOS Stats, una app que vigila el estado de este iPhone. \
Respondes en español de México, de forma breve, clara y amable. \
Antes de cada pregunta recibes un mensaje de sistema con los datos actuales del iPhone: \
úsalos solo si la pregunta trata del teléfono, no inventes datos que no aparezcan ahí \
y nunca copies esa lista en tu respuesta. \
Estados térmicos de iOS: normal (sin problema), templado (algo caliente), \
caliente o «serio» (iOS baja el rendimiento para enfriarse) y crítico (muy caliente, conviene dejar de usarlo).";

/// Caracteres de historial que caben con holgura en 4096 tokens junto a la persona,
/// los datos del iPhone, la pregunta y hasta 512 tokens de respuesta.
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

/// Datos del iPhone en una línea.
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

/// Historial para el modelo: cada pregunta va precedida de los datos del iPhone que se
/// le dieron en su momento, igual que cuando se generó la respuesta.
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

/// Persona + historial + datos del iPhone + pregunta. Si el historial pasa de
/// `budget_chars`, se recorta por el principio a saltos de medio presupuesto, para que
/// el inicio no cambie en cada turno. Nunca empieza con una respuesta.
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
        // Sobran 40 caracteres: se quita medio presupuesto (80), es decir «a» y «b».
        let out = build(&history, None, "?", 160);
        let contents: Vec<_> = out[1..out.len() - 1].iter().map(|m| &m.content[..1]).collect();
        assert_eq!(contents, ["c", "d"]);

        // Si el corte cae en una respuesta, también se salta.
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
        // 10 mensajes de 99 caracteres caben enteros.
        assert_eq!(first(10), turns[0].content);
        // Del 11 al 15 sobra menos de medio presupuesto: se quitan 6 y el inicio no cambia.
        for n in 11..=15 {
            assert_eq!(first(n), turns[6].content, "con {n} mensajes");
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
