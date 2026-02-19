# Task system fix (Feb 2026): duplicate tasks and agent chat never running

## What happened

From `~/.mac-stats/debug.log` (2026-02-19, task `task-casual-Chat-20260219-165058-open.md` created via Discord):

1. **User asked for a casual chat between all agents** (via Discord).
2. **Plan produced**: `TASK_CREATE: Casual Chat Task 174 "Have a casual chat..." | AGENT: orchestrator`
3. **Router executes one tool per response.** The first line was `TASK_CREATE`, so the app created the task file and never ran `AGENT: orchestrator`. The chat between agents never happened.
4. **Model then replied with natural language** (“Great! I've appended the summary…”) instead of outputting `AGENT: orchestrator ...`, so the conversation was never started.
5. **In later turns the model repeatedly output `TASK_CREATE`** with the same intent (e.g. `TASK_CREATE: Casual Chat between Agents ~/mac-stats/task/casual-chat-agents.md`). Each call created a **new** task file (new timestamp in filename). With up to 15 tool iterations, this produced many duplicate tasks (e.g. multiple `task-casual-Chat-20260219-*-open.md`).

## Root causes

- **Agent chat**: Planning said both “create task” and “run AGENT: orchestrator”, but execution only runs one tool per Ollama response. The first tool was TASK_CREATE, so AGENT was never invoked.
- **Duplicate tasks**: No check for “task with same topic+id already exists”; every TASK_CREATE created a new file.

## Fixes applied

1. **TASK_CREATE deduplication** (`src-tauri/src/task/mod.rs`):
   - Before creating a task, we check for an existing file with the same `topic_slug` and `id` (filename prefix `task-{topic_slug}-{id}-`).
   - If one exists, `create_task` returns an error: *"A task with this topic and id already exists: ... Use TASK_APPEND or TASK_STATUS to update it."*
   - This stops the loop of creating many identical tasks.

2. **Prompt guidance** (`src-tauri/src/commands/ollama.rs`):
   - **Planning**: Added to the planning prompt: *"If the user wants agents to have a conversation or chat together, your plan must start with AGENT: orchestrator (or the appropriate agent) so the conversation actually runs; do not only create a task file (TASK_CREATE)."*
   - **TASK description**: Added that when the user wants agents to chat, invoke `AGENT: orchestrator` so the conversation runs, and that if a task with that topic+id already exists, use TASK_APPEND or TASK_STATUS instead of TASK_CREATE.

With this, a request like “have a casual chat between all agents” should produce a plan that starts with `AGENT: orchestrator ...`, so the first tool executed is AGENT and the chat runs; and repeated TASK_CREATE with the same topic+id will be rejected.

## References

- Task agent: `docs/013_task_agent.md`
- LLM agents / AGENT tool: `docs/017_llm_agents.md`
- Review loop (scheduler/default): `src-tauri/src/task/review.rs`
