#!/usr/bin/env zsh
# Security review loop: run cursor-agent to review the project (security in mind),
# create tasks in the agents-tasks/task directory, and mark as done tasks that
# no longer need work.
#
# Usage:
#   ./scripts/security_review_agent_loop.sh          # run once
#   ./scripts/security_review_agent_loop.sh once     # run once
#   ./scripts/security_review_agent_loop.sh 5       # run 5 times, 1h apart
#   ./scripts/security_review_agent_loop.sh loop    # run forever, 1h apart
#
# Requires: cursor-agent on PATH. Task dir: ~/.mac-stats/task/ (or TASK_DIR).

set -e
TASK_DIR="${TASK_DIR:-$HOME/.mac-stats/task}"
WORKSPACE="${CURSOR_AGENT_WORKSPACE:-$(cd -P "$(dirname "$0")/.." && pwd)}"
SLEEP_SECS="${SLEEP_SECS:-3600}"

if ! command -v cursor-agent &>/dev/null; then
  echo "cursor-agent not found on PATH. Install it and ensure it is in PATH." >&2
  exit 1
fi

# Build prompt for cursor-agent: security review + create tasks + mark done
build_prompt() {
  cat <<'PROMPT'
You are a code reviewer with security in mind. Do the following in order:

1) **Review the project** (this codebase) for security issues: dependencies, FFI/safe code, input validation, paths, privilege use, logging of secrets, and any macOS-specific risks. Focus on the Rust backend and any user-controllable inputs.

2) **Create tasks** for things that should be done: create task files in TASK_DIR (the agents-task/task directory path is given below). Task file format:
   - Path: TASK_DIR/task-YYYYMMDD-HHMMSS-open.md (use current date/time for new tasks).
   - Inside each file put at least: ## Topic: <short-topic> and ## Id: <id> and a short description of the work. One task file per actionable item.
   - Create TASK_DIR if it does not exist.
   - Do not create duplicate tasks for the same topic/id.

3) **Review existing tasks** in the task directory: list all task-*-open.md and task-*-wip.md files. For each task that we do not need to work on anymore (obsolete, already addressed, or out of scope), rename the file to mark it done: change the filename from task-*-open.md or task-*-wip.md to task-*-finished.md (replace the last segment before .md with "finished"). Leave other tasks unchanged.

TASK_DIR for this run:
PROMPT
  echo "\"$TASK_DIR\""
  echo ""
  echo "Current task directory listing (if any):"
  if [[ -d "$TASK_DIR" ]]; then
    ls -la "$TASK_DIR" 2>/dev/null || true
  else
    echo "(directory does not exist yet)"
  fi
}

run_review() {
  local prompt
  prompt="$(build_prompt)"
  echo "=== Security review run at $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
  cursor-agent --print --trust --output-format text --workspace "$WORKSPACE" "$prompt"
}

case "${1:-once}" in
  once)
    run_review
    ;;
  loop)
    while true; do
      run_review
      echo "Sleeping ${SLEEP_SECS}s until next run..."
      sleep "$SLEEP_SECS"
    done
    ;;
  [0-9]*)
    integer count=$1
    for (( i = 1; i <= count; i++ )); do
      run_review
      if (( i < count )); then
        echo "Sleeping ${SLEEP_SECS}s until next run ($i/$count)..."
        sleep "$SLEEP_SECS"
      fi
    done
    ;;
  *)
    echo "Usage: $0 [once|loop|N]" >&2
    echo "  once  - run once (default)" >&2
    echo "  loop  - run forever, sleep ${SLEEP_SECS}s between runs" >&2
    echo "  N     - run N times, sleep ${SLEEP_SECS}s between runs" >&2
    exit 1
    ;;
esac
