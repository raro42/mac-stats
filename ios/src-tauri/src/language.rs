//! App language and the assistant's reply language.
//!
//! - The UI language is the in-app setting, or ("Automatic") the first iPhone preferred
//!   language the app supports. Unsupported languages fall back to English.
//! - The reply language is decided on every chat turn: a clear message in another
//!   language switches it, short or ambiguous messages keep the conversation's current
//!   language, and without either signal the app language is used.

use serde::Serialize;
use tauri::State;
use tauri_plugin_llm::DetectedLanguage;

use crate::chat::ChatState;
use crate::error::AppError;

/// UI languages, same list as `src/i18n/languages.ts`.
pub const SUPPORTED: [&str; 6] = ["es", "en", "de", "fr", "pt-BR", "zh-Hans"];

/// Debug-only pseudo-locale for the UI (`src/i18n`); the assistant answers in English.
pub const PSEUDO: &str = "qps";

pub const FALLBACK: &str = "en";

/// Minimum confidence of the on-device detector to switch the reply language.
pub const MIN_CONFIDENCE: f64 = 0.8;

/// Minimum amount of text to trust a detection; CJK characters count three times.
pub const MIN_LETTERS: usize = 12;

/// The iPhone's preferred languages, most preferred first (includes the per-app choice
/// from iOS Settings).
pub fn preferred_languages() -> Vec<String> {
    objc2_foundation::NSLocale::preferredLanguages().iter().map(|s| s.to_string()).collect()
}

/// Supported UI language for a BCP-47 tag such as `es-MX`, `pt-PT` or `zh-Hans-CN`.
pub fn match_tag(tag: &str) -> Option<&'static str> {
    let lower = tag.to_ascii_lowercase().replace('_', "-");
    let mut parts = lower.split('-');
    match parts.next()? {
        "es" => Some("es"),
        "en" => Some("en"),
        "de" => Some("de"),
        "fr" => Some("fr"),
        "pt" => Some("pt-BR"),
        "zh" => {
            let rest: Vec<&str> = parts.collect();
            let traditional = rest.contains(&"hant") || rest.iter().any(|p| matches!(*p, "tw" | "hk" | "mo"));
            (rest.contains(&"hans") || !traditional).then_some("zh-Hans")
        }
        _ => None,
    }
}

/// Language for "Automatic" and the preferred tag it came from (for regional number
/// formats such as `es-MX`).
pub fn resolve(preferred: &[String]) -> (&'static str, Option<String>) {
    preferred
        .iter()
        .find_map(|tag| match_tag(tag).map(|lang| (lang, Some(tag.replace('_', "-")))))
        .unwrap_or((FALLBACK, None))
}

/// The user's preferred tag for `language`, if any (a manual choice keeps the region).
fn region_tag(language: &str, preferred: &[String]) -> Option<String> {
    preferred.iter().find(|tag| match_tag(tag) == Some(language)).map(|tag| tag.replace('_', "-"))
}

/// Setting value accepted from the web layer.
pub fn valid_setting(value: &str) -> bool {
    SUPPORTED.contains(&value) || (cfg!(debug_assertions) && value == PSEUDO)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppLanguage {
    /// `None` = Automatic.
    pub setting: Option<String>,
    /// Language the UI uses now.
    pub resolved: String,
    /// What Automatic resolves to on this iPhone (for the "Automatic (…)" label).
    pub automatic: String,
    /// Full tag for number formats, e.g. `es-MX`.
    pub locale: Option<String>,
}

pub fn app_language_from(setting: Option<String>, preferred: &[String]) -> AppLanguage {
    let (automatic, auto_tag) = resolve(preferred);
    let setting = setting.filter(|s| valid_setting(s));
    let (resolved, locale) = match setting.as_deref() {
        Some(lang) => (lang.to_string(), region_tag(lang, preferred)),
        None => (automatic.to_string(), auto_tag),
    };
    AppLanguage { setting, resolved, automatic: automatic.into(), locale }
}

/// Language the UI uses now (also the assistant's default reply language).
pub fn current(state: &ChatState) -> String {
    app_language_from(state.settings().language, &preferred_languages()).resolved
}

#[tauri::command]
pub fn app_language(state: State<'_, ChatState>) -> AppLanguage {
    app_language_from(state.settings().language, &preferred_languages())
}

/// `None` = Automatic. The web layer reloads after this.
#[tauri::command]
pub fn set_app_language(state: State<'_, ChatState>, language: Option<String>) -> Result<AppLanguage, AppError> {
    let language = language.filter(|l| valid_setting(l));
    state.update_settings(|s| s.language = language.clone())?;
    Ok(app_language_from(language, &preferred_languages()))
}

// ---------------------------------------------------------------------------
// Reply language
// ---------------------------------------------------------------------------

/// English name of a language code, used in the "Reply in <Language>." instruction.
pub fn english_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "es" => "Spanish",
        "en" | PSEUDO => "English",
        "de" => "German",
        "fr" => "French",
        "pt-BR" => "Brazilian Portuguese",
        "pt" => "Portuguese",
        "zh-Hans" => "Simplified Chinese",
        "zh-Hant" => "Traditional Chinese",
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyLanguage {
    pub code: String,
    pub english_name: String,
}

impl ReplyLanguage {
    fn of(code: &str) -> Self {
        let code = if code == PSEUDO { FALLBACK } else { code };
        ReplyLanguage { code: code.into(), english_name: english_name(code).unwrap_or("English").into() }
    }
}

/// Amount of text for language detection: letters, with CJK characters counting three
/// times because one character carries about a word.
fn letters(text: &str) -> usize {
    text.chars()
        .filter(|c| c.is_alphabetic())
        .map(|c| if matches!(c, '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{ac00}'..='\u{d7af}') { 3 } else { 1 })
        .sum()
}

/// A detection worth acting on: confident and with enough text.
pub fn is_clear(detected: &DetectedLanguage, text: &str) -> bool {
    detected.confidence >= MIN_CONFIDENCE && letters(text) >= MIN_LETTERS
}

/// Decides the reply language for one turn.
///
/// 1. A clear detection of the user's message (see [`is_clear`]) wins: the user changed
///    language. If it is the app language in another variant (`pt` vs `pt-BR`), the app
///    variant is kept.
/// 2. Otherwise the conversation's sticky language (last clear detection) continues.
/// 3. Otherwise the app language.
pub fn reply_language(
    detected: Option<&DetectedLanguage>,
    text: &str,
    sticky: Option<&DetectedLanguage>,
    app: &str,
) -> ReplyLanguage {
    let base = |code: &str| code.split('-').next().unwrap_or(code).to_string();
    let from_detection = |d: &DetectedLanguage| {
        if base(&d.code) == base(app) && app != PSEUDO {
            ReplyLanguage::of(app)
        } else {
            ReplyLanguage {
                code: d.code.clone(),
                english_name: english_name(&d.code).map(str::to_string).unwrap_or_else(|| d.english_name.clone()),
            }
        }
    };
    match (detected.filter(|d| is_clear(d, text)), sticky) {
        (Some(d), _) => from_detection(d),
        (None, Some(s)) => from_detection(s),
        (None, None) => ReplyLanguage::of(app),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn detected(code: &str, name: &str, confidence: f64) -> DetectedLanguage {
        DetectedLanguage { code: code.into(), english_name: name.into(), confidence }
    }

    #[test]
    fn tags_match_supported_languages() {
        assert_eq!(match_tag("es-MX"), Some("es"));
        assert_eq!(match_tag("de-AT"), Some("de"));
        assert_eq!(match_tag("pt-PT"), Some("pt-BR"));
        assert_eq!(match_tag("zh-Hans-CN"), Some("zh-Hans"));
        assert_eq!(match_tag("zh-CN"), Some("zh-Hans"));
        assert_eq!(match_tag("zh-Hant-TW"), None);
        assert_eq!(match_tag("zh-TW"), None);
        assert_eq!(match_tag("it"), None);
        assert_eq!(match_tag("en_GB"), Some("en"));
    }

    #[test]
    fn automatic_uses_the_first_supported_preference() {
        assert_eq!(resolve(&tags(&["it-IT", "fr-CA", "en-US"])), ("fr", Some("fr-CA".into())));
        assert_eq!(resolve(&tags(&["it"])), ("en", None));
        assert_eq!(resolve(&[]), ("en", None));
    }

    #[test]
    fn manual_setting_keeps_the_region_when_the_iphone_has_that_language() {
        let preferred = tags(&["en-US", "es-MX"]);
        let lang = app_language_from(Some("es".into()), &preferred);
        assert_eq!(lang.resolved, "es");
        assert_eq!(lang.locale.as_deref(), Some("es-MX"));
        assert_eq!(lang.automatic, "en");
        assert_eq!(app_language_from(Some("de".into()), &preferred).locale, None);
        // Unknown values are ignored and mean Automatic.
        assert_eq!(app_language_from(Some("xx".into()), &preferred).resolved, "en");
    }

    #[test]
    fn clear_message_in_another_language_switches_the_reply_language() {
        let d = detected("en", "English", 0.97);
        let r = reply_language(Some(&d), "How is my battery doing today?", None, "es");
        assert_eq!(r, ReplyLanguage { code: "en".into(), english_name: "English".into() });
    }

    #[test]
    fn short_or_unsure_messages_keep_the_sticky_language() {
        let sticky = detected("en", "English", 0.95);
        let ok = detected("en", "English", 0.6);
        assert_eq!(reply_language(Some(&ok), "ok", Some(&sticky), "es").code, "en");
        let gracias = detected("es", "Spanish", 0.99);
        assert_eq!(reply_language(Some(&gracias), "gracias", Some(&sticky), "es").code, "en");
    }

    #[test]
    fn without_any_signal_the_app_language_is_used() {
        let unsure = detected("it", "Italian", 0.4);
        assert_eq!(reply_language(Some(&unsure), "ciao ciao ciao ciao", None, "de").code, "de");
        assert_eq!(reply_language(None, "👍", None, "fr").english_name, "French");
        assert_eq!(reply_language(None, "hi", None, PSEUDO).code, "en");
    }

    #[test]
    fn same_language_keeps_the_app_variant_and_other_languages_use_the_detected_name() {
        let pt = detected("pt", "Portuguese", 0.9);
        assert_eq!(reply_language(Some(&pt), "Como está a minha bateria hoje?", None, "pt-BR").code, "pt-BR");
        let it = detected("it", "Italian", 0.93);
        let r = reply_language(Some(&it), "Come sta la batteria del mio telefono?", None, "es");
        assert_eq!((r.code.as_str(), r.english_name.as_str()), ("it", "Italian"));
    }

    #[test]
    fn cjk_text_counts_as_enough_with_few_characters() {
        let zh = detected("zh-Hans", "Chinese, Simplified", 0.99);
        let r = reply_language(Some(&zh), "我的电池怎么样", None, "en");
        assert_eq!((r.code.as_str(), r.english_name.as_str()), ("zh-Hans", "Simplified Chinese"));
    }
}
