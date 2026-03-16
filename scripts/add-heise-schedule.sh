#!/usr/bin/env bash
# Add the Heise.de summary schedule to ~/.mac-stats/schedules.json.
# Run from repo root: ./scripts/add-heise-schedule.sh

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEDULES="${HOME}/.mac-stats/schedules.json"
ENTRY_JSON="${SCRIPT_DIR}/heise-schedule-entry.json"

if [[ ! -f "$ENTRY_JSON" ]]; then
  echo "Missing ${ENTRY_JSON}" >&2
  exit 1
fi

mkdir -p "$(dirname "$SCHEDULES")"
if [[ ! -f "$SCHEDULES" ]]; then
  echo '{"schedules":[]}' > "$SCHEDULES"
fi

python3 - "$SCHEDULES" "$ENTRY_JSON" << 'PY'
import json, sys
path, entry_path = sys.argv[1], sys.argv[2]
with open(path) as f:
    data = json.load(f)
with open(entry_path) as f:
    entry = json.load(f)
data["schedules"] = [e for e in data["schedules"] if e.get("id") != entry.get("id")]
data["schedules"].append(entry)
with open(path, "w") as f:
    json.dump(data, f, indent=2)
print("Added heise-summary schedule (daily 08:00). Restart mac-stats or wait for next scheduler tick to pick up.")
PY
