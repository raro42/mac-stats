//! UI theme: "System" (follows iPhone light/dark) or one of the desktop mac-stats themes,
//! with the same ids as the Mac app (`src-tauri/dist/themes/<id>/` at the repo root).
//!
//! Desktop themes have a fixed appearance (Neon is always dark, Swiss always light). The
//! status bar is drawn by iOS, not by the web layer, so the app's windows get the theme's
//! light/dark override and the clock stays readable on the theme's background.

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::chat::ChatState;
use crate::error::AppError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
}

/// Desktop themes and their fixed appearance. Same list as `src/themes/index.ts`.
pub const THEMES: [(&str, Appearance); 9] = [
    ("apple", Appearance::Light),
    ("architect", Appearance::Light),
    ("dark", Appearance::Dark),
    ("data-poster", Appearance::Dark),
    ("futuristic", Appearance::Dark),
    ("light", Appearance::Light),
    ("material", Appearance::Light),
    ("neon", Appearance::Dark),
    ("swiss-minimalistic", Appearance::Light),
];

/// Fixed appearance of a theme; `None` for System (or an unknown id), which follows iOS.
pub fn appearance(theme: Option<&str>) -> Option<Appearance> {
    THEMES.iter().find(|(id, _)| Some(*id) == theme).map(|(_, a)| *a)
}

fn valid(theme: &str) -> bool {
    THEMES.iter().any(|(id, _)| *id == theme)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppTheme {
    /// `None` = System.
    pub theme: Option<String>,
}

/// Sets the light/dark override of the app's windows: the status bar and system controls
/// follow it. Runs on the main thread, as UIKit requires.
pub fn apply(app: &AppHandle, appearance: Option<Appearance>) {
    #[cfg(target_os = "ios")]
    {
        let _ = app.run_on_main_thread(move || set_window_style(appearance));
    }
    #[cfg(not(target_os = "ios"))]
    let _ = (app, appearance);
}

#[cfg(target_os = "ios")]
fn set_window_style(appearance: Option<Appearance>) {
    use objc2_ui_kit::{UIApplication, UIUserInterfaceStyle, UIWindowScene};

    let Some(mtm) = objc2::MainThreadMarker::new() else { return };
    let style = match appearance {
        Some(Appearance::Light) => UIUserInterfaceStyle::Light,
        Some(Appearance::Dark) => UIUserInterfaceStyle::Dark,
        None => UIUserInterfaceStyle::Unspecified,
    };
    for scene in UIApplication::sharedApplication(mtm).connectedScenes().iter() {
        if let Ok(scene) = scene.downcast::<UIWindowScene>() {
            for window in scene.windows().iter() {
                window.setOverrideUserInterfaceStyle(style);
            }
        }
    }
}

/// Current theme. Also applies its status bar style, because at app start the web layer
/// calls this once the window exists.
#[tauri::command]
pub fn app_theme(app: AppHandle, state: State<'_, ChatState>) -> AppTheme {
    let theme = state.settings().theme.filter(|t| valid(t));
    apply(&app, appearance(theme.as_deref()));
    AppTheme { theme }
}

/// `None` = System. The web layer reloads after this.
#[tauri::command]
pub fn set_app_theme(app: AppHandle, state: State<'_, ChatState>, theme: Option<String>) -> Result<AppTheme, AppError> {
    let theme = theme.filter(|t| valid(t));
    state.update_settings(|s| s.theme = theme.clone())?;
    apply(&app, appearance(theme.as_deref()));
    Ok(AppTheme { theme })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The web layer's list (`src/themes/index.ts`) must have the same ids and
    /// appearances, so the picker never offers a theme Rust would reject.
    #[test]
    fn themes_match_the_web_list() {
        let source = include_str!("../../src/themes/index.ts");
        let web: Vec<(String, String)> = source
            .lines()
            .filter_map(|line| {
                let id = line.split("id: \"").nth(1)?.split('"').next()?;
                let appearance = line.split("appearance: \"").nth(1)?.split('"').next()?;
                Some((id.to_string(), appearance.to_string()))
            })
            .collect();
        let rust: Vec<(String, String)> = THEMES
            .iter()
            .map(|(id, a)| (id.to_string(), if *a == Appearance::Dark { "dark" } else { "light" }.to_string()))
            .collect();
        assert_eq!(web, rust);
    }

    #[test]
    fn unknown_themes_mean_system() {
        assert_eq!(appearance(Some("neon")), Some(Appearance::Dark));
        assert_eq!(appearance(Some("swiss-minimalistic")), Some(Appearance::Light));
        assert_eq!(appearance(Some("nope")), None);
        assert_eq!(appearance(None), None);
        assert!(!valid("system"));
    }
}
