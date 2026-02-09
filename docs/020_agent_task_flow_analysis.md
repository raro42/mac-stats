# Why agents stop: tool loop limit and sequential execution

## What the logs show

1. **Max tool iterations**  
   The agent router runs a **tool loop** with a per-agent cap. The limit is set in each agent’s **agent.json** as `max_tool_iterations` (default **15** when missing). When a request uses an agent override (e.g. Discord `agent: 001`), that agent’s limit applies; otherwise the default 15 is used. Each turn, the model can output one tool line (e.g. `AGENT: orchestrator`, `TASK_CREATE: ...`, `TASK_APPEND: ...`). After that many tool invocations, the loop stops and the last model response is returned.  
   Log line: `Agent router: max tool iterations reached (N), using last response as final`

2. **Execution is sequential, not parallel**  
   There is no parallel execution of agents. In each turn the model outputs **one** tool line; we run that tool (e.g. one AGENT call), inject the result into the conversation, and call the model again. So "work on it in parallel" is not supported: agents are invoked one after another. To have multiple agents "each fulfilling its tasks", the orchestrator would need to call agent A, then agent B, then maybe TASK_APPEND, then agent A again — which consumes the per-agent iteration budget (see `max_tool_iterations` in agent.json).

3. **Task files vs. “working on” a task**  
   `TASK_CREATE` creates a task file under `~/.mac-stats/task/`. The actual **task loop** (repeatedly read task → Ollama → TASK_APPEND/TASK_STATUS until status is `finished`) is a **separate** flow:
   - Triggered by the **scheduler** (e.g. a schedule entry like `TASK_RUN: <id>` in `schedules.json`), or  
   - By the **task review loop** (every 1 minute, picks open tasks assigned to scheduler/default and runs `run_task_until_finished`).  
   So in one Discord request we can create a task and maybe append once or twice, but after 5 tools we stop. The task then sits there; the review loop or a scheduled TASK_RUN would pick it up later.

## Changes made

- **Per-agent limit**: `max_tool_iterations` in **agent.json** (default 15). Each agent can have its own limit; when no agent override is used, the default is 15.
- This doc explains why agents stop and that execution is sequential.

## If you want “agents working in parallel”

That would require a different design (e.g. model outputs multiple tool lines per turn, or we spawn several agent runs and wait for all). The current design is one-tool-per-turn, which is why they don’t “keep working” beyond the iteration limit and don’t run in parallel.
