//! One-off: heise.de → cookie OK → first article → screenshot (for agent/browser testing).
//! Run: `cd src-tauri && cargo run --example heise_flow`

use mac_stats::browser_agent::{self, navigate_and_get_state};
use regex::Regex;

fn parse_element_line(line: &str) -> Option<(u32, &'static str, String)> {
    let line = line.trim();
    let rest = line.strip_prefix('[')?;
    let (idx_s, after) = rest.split_once("] ")?;
    let idx: u32 = idx_s.parse().ok()?;
    let (kind_raw, title_rest) = after.split_once(" '")?;
    let kind = if kind_raw.starts_with("link") {
        "link"
    } else if kind_raw.starts_with("button") {
        "button"
    } else if kind_raw.starts_with("input") {
        "input"
    } else {
        return None;
    };
    let title = title_rest.strip_suffix('\'')?.to_string();
    Some((idx, kind, title))
}

fn find_ok_index(snapshot: &str) -> Option<u32> {
    let ok = Regex::new(r"(?i)^\s*\[(\d+)\]\s+button\s+'OK'\s*$").ok()?;
    for line in snapshot.lines() {
        if let Some(c) = ok.captures(line) {
            return c.get(1)?.as_str().parse().ok();
        }
    }
    None
}

fn skip_link_label(s: &str) -> bool {
    let t = s.trim().to_lowercase();
    if t.len() < 18 {
        return true;
    }
    const BAD: &[&str] = &[
        "anmelden",
        "abonnieren",
        "abo ",
        "menü",
        "suche",
        "newsletter",
        "datenschutz",
        "impressum",
        "heise+",
        "heise plus",
        "zum inhalt",
        "zum shop",
        "jobs",
        "mediadaten",
    ];
    BAD.iter().any(|b| t.contains(b))
}

/// First main-teaser style link in DOM list order (heuristic filters skip chrome/footer).
fn first_article_link_index(snapshot: &str) -> Option<u32> {
    for line in snapshot.lines() {
        if let Some((idx, kind, title)) = parse_element_line(line) {
            if kind != "link" {
                continue;
            }
            if skip_link_label(&title) {
                continue;
            }
            return Some(idx);
        }
    }
    None
}

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_new("info,mac_stats=debug").unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("info")
            }),
        )
        .try_init();

    println!("Navigating to https://www.heise.de …");
    let snap = match navigate_and_get_state("https://www.heise.de") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("navigate failed: {e}");
            std::process::exit(1);
        }
    };

    if let Some(ok_i) = find_ok_index(&snap) {
        println!("Clicking cookie OK at index {ok_i} …");
        match browser_agent::click_by_index(ok_i) {
            Ok(s2) => {
                std::thread::sleep(std::time::Duration::from_millis(1200));
                let snap2 = s2;
                if let Some(a_i) = first_article_link_index(&snap2) {
                    println!("Opening first article link at index {a_i} …");
                    if let Err(e) = browser_agent::click_by_index(a_i) {
                        eprintln!("article click: {e}");
                    }
                } else {
                    eprintln!("Could not pick a headline link; trying snapshot from post-OK state.");
                    eprintln!("---\n{snap2}\n---");
                }
            }
            Err(e) => eprintln!("cookie OK click: {e} (banner may already be gone)"),
        }
    } else {
        println!("No [n] button 'OK' line found; assuming no consent dialog.");
        if let Some(a_i) = first_article_link_index(&snap) {
            println!("Opening first article link at index {a_i} …");
            let _ = browser_agent::click_by_index(a_i);
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(2));
    match browser_agent::take_screenshot_current_page() {
        Ok(p) => println!("{}", p.display()),
        Err(e) => {
            eprintln!("screenshot: {e}");
            std::process::exit(1);
        }
    }
}
