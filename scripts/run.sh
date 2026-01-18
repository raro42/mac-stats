#!/bin/zsh
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root_dir/src-tauri"

if [[ "${1:-release}" == "dev" ]]; then
  cargo run
else
  cargo build --release
  exec ./target/release/mac-stats-backend
fi
