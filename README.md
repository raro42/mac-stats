# mac-stats

A lightweight system monitor for macOS built with Rust.

## Features

- Displays CPU, RAM, and Disk usage in the console
- Updates every 10 seconds
- Low CPU usage: ~0.1-0.5% idle

## Installation

1. Clone the repo.
2. `cd src-tauri`
3. `cargo build --release`
4. Run `./target/release/mac-stats`

## Auto-start on Login

Use launchd or add to Login Items manually.

## Development

### Prerequisites

- Rust

### Run

```bash
cd src-tauri
cargo run
```

## Notes

- Console output: "CPU: X%, RAM: Y%, Disk: Z%"
- Run in background with `nohup ./target/release/mac-stats &`
- Accurate against Activity Monitor.
- No GUI/menu bar due to framework issues; console version is efficient.
