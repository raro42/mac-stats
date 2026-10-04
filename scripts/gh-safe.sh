#!/usr/bin/env bash
# Safe GitHub CLI wrapper for the public mac-stats repo.
# Forces issue/PR bodies through scripts/redact_public_text.py before posting.
#
#   ./scripts/gh-safe.sh issue comment 13 --body "Agent 001: planned."
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REDACT="${ROOT}/scripts/redact_public_text.py"
GH_REPO="${MAC_STATS_GH_REPO:-raro42/mac-stats}"

if [[ ! -x "$(command -v python3)" ]]; then
  echo "gh-safe: python3 required" >&2
  exit 2
fi
if [[ ! -f "$REDACT" ]]; then
  echo "gh-safe: missing $REDACT" >&2
  exit 2
fi

args=("$@")
body=""
body_file=""
new_args=()
i=0
while [[ $i -lt ${#args[@]} ]]; do
  a="${args[$i]}"
  if [[ "$a" == "--body" ]]; then
    i=$((i + 1))
    body="${args[$i]:-}"
  elif [[ "$a" == --body=* ]]; then
    body="${a#--body=}"
  elif [[ "$a" == "--body-file" ]]; then
    i=$((i + 1))
    body_file="${args[$i]:-}"
  elif [[ "$a" == --body-file=* ]]; then
    body_file="${a#--body-file=}"
  else
    new_args+=("$a")
  fi
  i=$((i + 1))
done

if [[ -n "$body_file" ]]; then
  if [[ "$body_file" == "-" ]]; then
    body="$(cat)"
  else
    body="$(cat "$body_file")"
  fi
fi

tmp=""
cleanup() {
  [[ -n "$tmp" && -f "$tmp" ]] && rm -f "$tmp"
}
trap cleanup EXIT

if [[ -n "$body" ]]; then
  tmp="$(mktemp)"
  redact_err="${tmp}.err"
  if [[ "${MAC_STATS_GH_SOFT_REDACT:-0}" == "1" ]]; then
    redact_ok=0
    if printf '%s' "$body" | python3 "$REDACT" --redact --soft-redact >"$tmp" 2>"$redact_err"; then
      redact_ok=1
    fi
  else
    redact_ok=0
    if printf '%s' "$body" | python3 "$REDACT" --redact >"$tmp" 2>"$redact_err"; then
      redact_ok=1
    fi
  fi
  if [[ "$redact_ok" -ne 1 ]]; then
    cat "$redact_err" >&2 || true
    echo "gh-safe: refusing to post. Sensitive patterns detected." >&2
    exit 1
  fi
  if [[ -s "$redact_err" ]]; then
    cat "$redact_err" >&2 || true
  fi
  has_repo=0
  for a in "${new_args[@]}"; do
    [[ "$a" == "--repo" || "$a" == --repo=* ]] && has_repo=1
  done
  if [[ $has_repo -eq 0 ]]; then
    new_args+=(--repo "$GH_REPO")
  fi
  exec gh "${new_args[@]}" --body-file "$tmp"
fi

has_repo=0
for a in "${new_args[@]}"; do
  [[ "$a" == "--repo" || "$a" == --repo=* ]] && has_repo=1
done
if [[ $has_repo -eq 0 ]]; then
  case "${new_args[0]:-}" in
    issue|pr) new_args+=(--repo "$GH_REPO") ;;
  esac
fi
exec gh "${new_args[@]}"
