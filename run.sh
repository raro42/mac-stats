#!/bin/zsh
set -euo pipefail

root_dir="$(cd "$(dirname "$0")" && pwd)"
exec "$root_dir/scripts/run.sh" "${1:-}"
