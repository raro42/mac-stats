# Redmine ticket review: why it hallucinated and how to fix it

## What happened

A user asked to **review a Redmine ticket** and received a **hallucinated / wrong answer** (no real ticket data, made-up summary).

## Root cause

### 1. Two different code paths

| Entry point | Command / flow | Redmine support |
|-------------|----------------|----------------|
| **Discord, CLI `--discord run-ollama`, scheduler, task runner** | `answer_with_ollama_and_fetch` → full agent router | ✅ REDMINE_API in tools; **pre-route** when question contains ticket/issue/redmine + numeric ID → we force `REDMINE_API: GET /issues/{id}.json?include=journals,attachments` and run it before the model answers. |
| **In-app chat (CPU window)** | `ollama_chat_with_execution` | ❌ No REDMINE_API. Only FETCH_URL (+ code execution). System prompt is soul + `NON_AGENT_TOOL_INSTRUCTIONS` (no Redmine tool list). |

So:

- If the user asked from the **CPU window**, the model **never had a way to call the Redmine API**. It was told to answer about “reviewing a ticket” but had no tool and no injected data → it **hallucinated** from general knowledge or prior chat.
- If the user asked from **Discord** but **without a ticket ID** (e.g. “review the redmine ticket” or “analyze the last ticket”), `extract_ticket_id()` returns `None`, so we **don’t pre-route**. The planner might still output `REDMINE_API: GET /issues/...` if it infers an ID from context, but if it doesn’t, the model again has no ticket data → **hallucination**.

### 2. What the logs show

- When pre-route **does** run (e.g. “review redmine 7332” / “review redmine ticket 7239”), we see:
  - `Agent router: pre-routed to REDMINE_API for ticket #7332`
  - `REDMINE_API GET /issues/7332.json?include=journals,attachments`
  - `tool REDMINE_API completed, sending result back to Ollama (3239 chars)`
- So when the **agent router** is used **and** a ticket ID is present, real data is fetched and the model can summarize it. The bug is either (a) **in-app path** (no Redmine at all) or (b) **no ticket ID** in the question (no pre-route, model may not output the tool).

### 3. Comment in code

In `ollama.rs` (agent router):

```rust
// Redmine reviews must not be polluted by prior turns — the model hallucinates.
let fresh_session_tools = ["REDMINE_API"];
```

So we already clear conversation history when the plan contains REDMINE_API to avoid the model mixing old context with the new API result. That only helps when we’re on the **agent path** and the tool is actually run.

---

## What we need to get right

1. **Retrieve the ticket data from Redmine**  
   For any “review this ticket” request, we must **always** call the Redmine API (e.g. `GET /issues/{id}.json?include=journals,attachments`) and give that JSON to the model. No answer should be based on “I think this ticket is about…” without that call.

2. **Summarize and analyze the ticket**  
   Once we have the JSON, the model should produce:
   - **Summary** (subject, description, what it’s about)
   - **Status & completion** (status, assignee, done_ratio, dates)
   - **Missing parts** (e.g. documentation, unclear description, no acceptance criteria)
   - **Final thoughts** (blockers, next steps, recommendations)

3. **No hallucination**  
   The answer must be grounded **only** in the API response. If the API fails, we should say “Redmine API failed: …” and not invent a ticket.

---

## Solution options

### A. Route “review ticket” from in-app chat through the agent router (recommended)

- **Idea:** When the user sends a message from the CPU window, the backend detects “review/redmine/ticket/issue” + optional ticket ID. If Redmine is configured, **call `answer_with_ollama_and_fetch`** instead of the simple `ollama_chat_with_execution` path for that turn.
- **Pros:** One place (agent router) handles Redmine; pre-route and REDMINE_API already work; conversation history is cleared for REDMINE so no pollution.
- **Cons:** That one request gets the full router (planning, tools, verification). Need to ensure CPU-window UX (e.g. status, streaming if any) still works.

**Implementation sketch:**

- In the Tauri command that the frontend calls for “send message”:
  - If the message looks like a Redmine review (e.g. `extract_ticket_id` + “ticket”/“issue”/“redmine” in the question, or planner-like “review … ticket”),
  - and Redmine is configured,
  - then call `answer_with_ollama_and_fetch` with that question (and existing history if desired) instead of `ollama_chat_with_execution`.
- Alternatively, the frontend could call a new command like `ollama_chat_agent_style` that always uses `answer_with_ollama_and_fetch` for the CPU window (simplifies backend detection).

### B. Inject ticket data in the simple chat path (no new tool)

- **Idea:** Keep using `ollama_chat_with_execution`, but **before** sending the first request to Ollama:
  - If the question looks like “review redmine ticket N” (or “review ticket N” with Redmine configured), call `redmine::redmine_api_request("GET", format!("/issues/{}.json?include=journals,attachments", id), None)`.
  - Prepend a system or user message with the raw (or lightly trimmed) JSON and an instruction: “Use only this Redmine issue data to summarize and analyze the ticket: status, completion, missing parts (e.g. documentation), and final thoughts.”
- **Pros:** Small change; no new tool; same code path as today; model always sees real data for “review ticket N”.
- **Cons:** Redmine logic is duplicated (agent path already does this); “review the ticket” without ID still can’t be satisfied without more heuristics (e.g. “last ticket” from memory).

### C. Add REDMINE_API to the simple chat tool loop

- **Idea:** In `ollama_chat_with_execution`, add parsing for `REDMINE_API: ...` in the model reply and call `redmine::redmine_api_request` like the agent router does; inject the result and continue the loop. Also add the Redmine tool description to `default_non_agent_system_prompt` (or a variant used for that window).
- **Pros:** CPU window gains the same Redmine capability as the agent; user can say “review redmine 7209” and the model can output the tool call.
- **Cons:** Larger change; duplicates tool-handling logic; we still need to **force** a fetch when the user says “review ticket N” so the model doesn’t skip the tool (e.g. pre-inject REDMINE_API as first “model” turn when we detect intent).

---

## Recommended direction

1. **Short term (minimal change):**  
   **Option B** — In `ollama_chat_with_execution`, before the first Ollama call:
   - Normalize the question (e.g. lowercase), run `extract_ticket_id` and check for “ticket”/“issue”/“redmine”/“review”.
   - If we get an ID and Redmine is configured, **synchronously** call the Redmine API for that issue (with `include=journals,attachments`), then prepend to the **user** (or system) message a block like:  
     `Redmine issue data (use only this to answer):\n\n<json>`  
     and append an instruction:  
     “Summarize this ticket: subject, description, status, assignee, completion; list what’s missing (e.g. documentation); give brief final thoughts.”
   - Then send the request as usual. No new tools, no router; the model just has the real data in context.

2. **Optional hardening:**  
   - In the **agent router**, when we **don’t** have a ticket ID but the user said “review the ticket” / “analyze the redmine ticket”, consider:
     - Searching recent conversation for a mentioned issue ID, or
     - One-shot prompt: “You don’t have a ticket number. Reply with: Please specify the ticket number (e.g. review redmine 7209).”  
   So we never answer “review the ticket” with a made-up summary.

3. **Structured review template (both paths):**  
   In the instruction we send **after** the Redmine JSON (agent path already has something similar), standardize the expected structure, e.g.:
   - **Summary:** …
   - **Status & completion:** …
   - **Missing (e.g. documentation):** …
   - **Final thoughts:** …

   That reduces the chance of the model drifting into invented content.

---

## Historical implementation checklist (Option B)

- This checklist reflects the original Option B proposal that motivated later router/Redmine changes.
- Do not treat it as the current canonical backlog without first comparing it to the implemented Redmine flow in `src-tauri/src/commands/ollama.rs` and `docs/025_redmine_api_skill.md`.

---

## Summary

| Why it went wrong | What to do |
|-------------------|------------|
| In-app chat has no REDMINE_API and no ticket data | Always fetch ticket when “review ticket/issue/redmine” + ID is detected; inject JSON into context (Option B) or route through agent (Option A). |
| User said “review the ticket” with no ID | Don’t answer from thin air; ask for the ticket number or use context (e.g. last mentioned ID) if we add that. |
| Model had no strict template | Add a short “Summary / Status / Missing / Final thoughts” instruction so the answer is grounded and structured. |

Retrieving the ticket from Redmine, then summarizing and analyzing it (status, completion, missing parts, final thoughts) is the required behavior; the fix is to **guarantee** we fetch and inject that data on every “review ticket” request, and to **never** let the model answer without it.
