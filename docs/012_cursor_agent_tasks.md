# Using cursor-agent CLI for Tasks

This doc describes what is already in place and what you may need to do so that the mac-stats agent can use the **cursor-agent** CLI for tasks (e.g. scheduled or manual task runs that involve coding).

## Current state

### Already wired

1. **CURSOR_AGENT tool**  
   When `cursor-agent` is on PATH, the orchestrator (and any agent using the tool loop) sees:
   - **CURSOR_AGENT:** `<prompt>` — runs cursor-agent in the configured workspace with `--print --trust --output-format text`, returns stdout to the conversation.

2. **RUN_CMD allowlist**  
   `cursor-agent` is in the default allowlist (and in `~/.mac-stats/agents/agent-000/skill.md`). The agent can also run e.g. `RUN_CMD: cursor-agent --help` or `RUN_CMD: cursor-agent <args>` (path rules don’t apply to cursor-agent).

3. **Task runner uses full tool loop**  
   When the scheduler runs `TASK: <path_or_id>` (or `TASK_RUN: <path_or_id>`), or when you run a task until finished, the code uses `answer_with_ollama_and_fetch`. That includes **all** tools (TASK_*, RUN_CMD, CURSOR_AGENT, etc.). So the model **can** already:
   - Read the task file content (it’s in the prompt).
   - Reply with `CURSOR_AGENT: <implement this: …>`.
   - Then `TASK_APPEND` with the result and `TASK_STATUS: <id> finished`.

4. **Config**  
   - **CURSOR_AGENT_WORKSPACE**: env or `.config.env` (cwd, `src-tauri/`, `~/.mac-stats/.config.env`). Default: `$HOME/projects/mac-stats` if that dir exists, else `.`.
   - **CURSOR_AGENT_MODEL**: optional; passed as `--model` to cursor-agent.

So **no code change is strictly required** for the agent to use cursor-agent for tasks: the tool is available in the same loop that runs tasks.

## What you need to do

### 1. Install cursor-agent and put it on PATH

- The app only offers **CURSOR_AGENT** when `which cursor-agent` succeeds.
- Install the Cursor Agent CLI so that `cursor-agent` is available in the environment used to run mac-stats (e.g. the same terminal or launch context).
- Verify: `cursor-agent --help` (and ideally `cursor-agent --print --trust --output-format text "echo hello"` in a test dir).

### 2. (Optional) Set workspace and model

- In `~/.mac-stats/.config.env` (or env):
  - `CURSOR_AGENT_WORKSPACE=/path/to/your/code` — directory cursor-agent will run in.
  - `CURSOR_AGENT_MODEL=...` — if you want a specific model.
- If unset, workspace defaults to `$HOME/projects/mac-stats` when that directory exists.

### 3. (Optional) Guide the model to use CURSOR_AGENT for coding tasks

The task runner prompt says: *"Decide the next step. Use TASK_APPEND to add feedback and TASK_STATUS to set wip or finished when done."* It does **not** mention CURSOR_AGENT. So the model might not choose it for “implement this” style tasks.

To make that more likely:

- **Option A – Task runner prompt**  
  In `src-tauri/src/task/runner.rs`, extend the `question` to mention that for coding tasks (implement, refactor, fix, add feature) the agent may use **CURSOR_AGENT:** `<prompt>` to delegate to the Cursor Agent CLI, then TASK_APPEND the result and TASK_STATUS finished.

- **Option B – Orchestrator skill**  
  In `~/.mac-stats/agents/agent-000/skill.md` (or your orchestrator’s skill), add a short rule: for task files that describe code changes or implementation work, use CURSOR_AGENT with the task description, then update the task with the result and set status to finished.

- **Option C – Task file convention**  
  In task content, include a line like “Implement using Cursor Agent” or “Use CURSOR_AGENT for this.” The model will then see the hint in the task text.

### 4. (Optional) Workspace per task

Right now cursor-agent always runs in **one** workspace (from config). There is no syntax like `CURSOR_AGENT: workspace:/other/proj <prompt>`. If you need different workspaces per task:

- Either put the desired project path in the task text and add a convention in the orchestrator skill (e.g. “If the task says ‘project: /path/to/X’, run cursor-agent in that directory”) — then you’d need a **code change** to pass `--workspace` from the task (e.g. parse a prefix or a special line and call `run_cursor_agent` with a different workspace).
- Or use different `CURSOR_AGENT_WORKSPACE` values per environment (e.g. per user or per schedule) and keep one workspace per run.

## Summary

| Requirement | Status |
|------------|--------|
| cursor-agent on PATH | **You must ensure this.** |
| CURSOR_AGENT tool in tool loop | Done. |
| Task runner uses tool loop (so CURSOR_AGENT is available) | Done. |
| RUN_CMD allows cursor-agent | Done (default + skill.md). |
| Prompt/skill hints to use CURSOR_AGENT for coding tasks | Optional; improves likelihood the model delegates. |
| Per-task or per-invocation workspace | Not implemented; optional enhancement. |

So: **for the agent to use cursor-agent for tasks, you mainly need cursor-agent installed and on PATH.** Optionally, add a short hint in the task runner prompt or in the orchestrator skill so the model is more likely to use CURSOR_AGENT for implementation tasks.
