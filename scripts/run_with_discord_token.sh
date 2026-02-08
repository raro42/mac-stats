#!/usr/bin/env bash
# Run mac-stats with Discord bot token from .config.env (so token is stored at startup and gateway connects).
# Usage: ./scripts/run_with_discord_token.sh [--cpu] [--dev]
# Reads src-tauri/.config.env for DISCORD-USER1-TOKEN or DISCORD-USER2-TOKEN and passes it as DISCORD_BOT_TOKEN.

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_ENV="${PROJECT_ROOT}/src-tauri/.config.env"

if [[ ! -f "$CONFIG_ENV" ]]; then
  echo "Missing $CONFIG_ENV (add DISCORD-USER1-TOKEN=your_token)" >&2
  exit 1
fi

# Export first Discord token found (USER1 or USER2)
while IFS= read -r line; do
  if [[ "$line" =~ ^DISCORD-USER[12]-TOKEN=(.+)$ ]]; then
    export DISCORD_BOT_TOKEN="${BASH_REMATCH[1]}"
    break
  fi
done < "$CONFIG_ENV"

if [[ -z "${DISCORD_BOT_TOKEN:-}" ]]; then
  echo "No DISCORD-USER1-TOKEN or DISCORD-USER2-TOKEN in $CONFIG_ENV" >&2
  exit 1
fi

echo "Using Discord token from .config.env (length ${#DISCORD_BOT_TOKEN})"
echo "Run with -v to see logs: Discord: Stored token from env, Keychain: stored credential, etc."
echo ""

cd "$PROJECT_ROOT"
if [[ "${1:-}" == "--dev" ]]; then
  shift
  ./run dev --cpu -v "$@"
else
  ./run --cpu -v "$@"
fi
