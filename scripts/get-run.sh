#!/bin/zsh
set -euo pipefail

repo_raw_base="https://raw.githubusercontent.com/raro42/mac-stats/refs/heads/main"
target="./run"
source_url="${repo_raw_base}/run"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$source_url" -o "$target"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$target" "$source_url"
else
  echo "Error: neither curl nor wget is available." >&2
  exit 1
fi

chmod +x "$target"
echo "Downloaded ./run"
