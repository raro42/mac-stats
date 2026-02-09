# Router and agents: who can call the API, and how they work together

## Short answers

1. **Can agent-000 (or any other agent) explicitly call the router to execute things on the API?**  
   **Only the entry-point model** can. When a request starts (e.g. from Discord with `agent: 000`), that agent **is** the one in the router loop: it doesn’t “call” the router as a separate service; it **is** the model the router is talking to. So agent-000 **can** “call the router” in the sense that it outputs tool lines (`FETCH_URL: ...`, `TASK_CREATE: ...`, `AGENT: 002 ...`) and the **router** (the Rust code in `answer_with_ollama_and_fetch`) executes them.  
   **Sub-agents** (e.g. 001, 002) **cannot**. When the orchestrator outputs `AGENT: 002 <task>`, the app runs that agent in a **single** Ollama request (no tool list, no router). So 002 cannot output `FETCH_URL:` or `TASK_APPEND:` and have it executed.

2. **Is the router involved in the discussion between agents?**  
   Yes. The **router is the only thing that talks to the entry-point model** and runs tools. When the entry-point outputs `AGENT: 002 <task>`, the router runs 002 (one turn), gets the reply, and **injects it back into the router’s conversation** with the entry-point. So the “discussion” is: **router ↔ entry-point (orchestrator)**. Sub-agents do **not** talk to the router and do **not** talk to each other; they only receive a task and return a single reply.

3. **How do agents and the API router work together?**  
   - **One** “main” conversation: router (Rust) + **entry-point model** (e.g. agent-000 when overridden, or default Ollama model).  
   - The entry-point gets the **full tool list** (FETCH_URL, BRAVE_SEARCH, AGENT, TASK_*, SCHEDULE, RUN_CMD, OLLAMA_API, etc.) and runs in a **tool loop**: reply → parse tool line → execute → inject result → call model again.  
   - **AGENT** is one of those tools: when the entry-point replies `AGENT: <id or slug> <task>`, the router calls `run_agent_ollama_session(agent, task)` — a **single** Ollama request with that agent’s prompt and model, **no** tool list. The sub-agent’s reply is then injected back to the entry-point.  
   - So: **router** = executor of all tools; **entry-point agent** = the only “mind” that can ask the router to run tools (including running other agents). **Sub-agents** = tools that return text; they have no API.

## Flow diagram (conceptual)

```
User (e.g. Discord)
    │
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  ROUTER (answer_with_ollama_and_fetch)                          │
│  - Builds tool list (FETCH_URL, AGENT, TASK_*, SCHEDULE, …)     │
│  - One “main” conversation with the ENTRY-POINT model           │
│  - Tool loop: get reply → if tool line → execute → inject result │
└─────────────────────────────────────────────────────────────────┘
    │
    │  entry-point = agent_override (e.g. agent-000) or default
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  ENTRY-POINT MODEL (e.g. orchestrator / agent-000)              │
│  - Sees full API; can output: FETCH_URL:, TASK_CREATE:,         │
│    AGENT: 002 <task>, etc.                                       │
│  - When it outputs AGENT: 002 …, router runs sub-agent 002      │
└─────────────────────────────────────────────────────────────────┘
    │
    │  when entry-point outputs "AGENT: 002 <task>"
    ▼
┌─────────────────────────────────────────────────────────────────┐
│  run_agent_ollama_session(agent_002, task)                       │
│  - Single Ollama request: system=002’s prompt, user=task        │
│  - No tool list; 002 cannot call FETCH_URL, TASK_*, etc.         │
│  - Returns one reply → router injects it back to entry-point     │
└─────────────────────────────────────────────────────────────────┘
```

## Implications

- **Orchestrator (e.g. agent-000)** should have the Router API Commands in its `skill.md` so it knows how to use FETCH_URL, TASK_*, AGENT, etc. It is the only agent that gets to use those commands in that request.
- **Specialist agents (001, 002, 003)** only need to do their job in one turn; they cannot schedule, create tasks, or call other agents via the router. If a specialist “needs” to do that, the orchestrator must do it (e.g. orchestrator calls 002, gets result, then orchestrator does TASK_APPEND or SCHEDULE).
- There is no “agent calling the router as an API” in a separate request: the router is the loop that drives the entry-point. So “explicitly call the router” = the entry-point outputting tool lines in that same conversation.

## References

- **Tool loop**: `commands/ollama.rs` → `answer_with_ollama_and_fetch`, `parse_tool_from_response`, and the `while tool_count < max_tool_iterations` loop.
- **Sub-agent run (no tools)**: `run_agent_ollama_session` in `commands/ollama.rs` (single request, no tool list).
- **Router API snippet for orchestrator**: `docs/agent_000_router_commands_snippet.md`, `docs/017_llm_agents.md`.
