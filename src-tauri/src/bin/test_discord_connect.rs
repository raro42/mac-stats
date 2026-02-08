//! Test binary: connect to Discord using token from .config.env or DISCORD_BOT_TOKEN.
//! Run from project root: ./scripts/run_with_discord_token.sh then in another terminal
//!   cd src-tauri && cargo run --bin test_discord_connect
//! Or with token from file: cargo run --bin test_discord_connect -- [path/to/.config.env]
//!
//! Logs "Discord: Connecting...", "Gateway client built...", "Discord: Bot connected as X"
//! on success; runs ~15s then exits.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or(".config.env");

    let token = if let Ok(t) = std::env::var("DISCORD_BOT_TOKEN") {
        t.trim().to_string()
    } else if let Ok(content) = std::fs::read_to_string(path) {
        content
            .lines()
            .find(|l| l.starts_with("DISCORD-USER1-TOKEN=") || l.starts_with("DISCORD-USER2-TOKEN="))
            .and_then(|l| l.split_once('='))
            .map(|(_, v)| v.trim().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    if token.is_empty() {
        eprintln!("No token: set DISCORD_BOT_TOKEN or put DISCORD-USER1-TOKEN=... in {}", path);
        std::process::exit(1);
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("mac_stats=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    rt.block_on(async {
        let token = token.clone();
        let task = tokio::spawn(async move { mac_stats::discord::run_discord_client(token).await });
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        task.abort();
        let _ = task.await;
    });
}
