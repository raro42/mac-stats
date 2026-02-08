//! Test binary: read Discord token from .config.env, store in Keychain, read back.
//! Run from src-tauri: cargo run --bin test_discord_keychain [path/to/.config.env]
//!
//! If it hangs after "Token length: N chars", Keychain is blocking (e.g. locked keychain
//! or permission prompt). Unlock Keychain Access and ensure the app is allowed to add items.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or(".config.env");

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path, e);
            if let Ok(t) = std::env::var("DISCORD_BOT_TOKEN") {
                if !t.trim().is_empty() {
                    run_store_and_read(&t.trim());
                    return;
                }
            }
            std::process::exit(1);
        }
    };

    let token = content
        .lines()
        .find(|l| l.starts_with("DISCORD-USER1-TOKEN=") || l.starts_with("DISCORD-USER2-TOKEN="))
        .and_then(|l| l.split_once('='))
        .map(|(_, v)| v.trim().to_string());

    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => {
            eprintln!("No DISCORD-USER1-TOKEN or DISCORD-USER2-TOKEN in {}", path);
            if let Ok(t) = std::env::var("DISCORD_BOT_TOKEN") {
                if !t.trim().is_empty() {
                    run_store_and_read(&t.trim());
                    return;
                }
            }
            std::process::exit(1);
        }
    };

    run_store_and_read(&token);
}

fn run_store_and_read(token: &str) {
    const ACCOUNT: &str = "discord_bot_token";
    println!("Keychain test: storing credential for account '{}' (service 'com.raro42.mac-stats')", ACCOUNT);
    println!("Token length: {} chars", token.len());
    println!("(If this hangs, Keychain may be blocking - unlock Keychain Access and try again.)");

    match mac_stats::security::store_credential(ACCOUNT, token) {
        Ok(()) => println!("store_credential: OK"),
        Err(e) => {
            eprintln!("store_credential: FAILED: {}", e);
            std::process::exit(1);
        }
    }

    match mac_stats::security::get_credential(ACCOUNT) {
        Ok(Some(read_back)) => {
            println!("get_credential: OK (read back {} chars)", read_back.len());
            if read_back == token {
                println!("Verify: token matches.");
            } else {
                eprintln!("Verify: token MISMATCH (stored {} vs read {})", token.len(), read_back.len());
            }
        }
        Ok(None) => eprintln!("get_credential: not found (keychain read returned None)"),
        Err(e) => {
            eprintln!("get_credential: FAILED: {}", e);
            std::process::exit(1);
        }
    }
}
