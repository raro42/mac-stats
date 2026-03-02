# Expectation check at end of tool-chain (design)

## Goal

Add an optional check: at the **start** extract or record what “success” means for the user’s request, and at the **end** (before returning the final reply) quickly confirm whether we met those expectations. This can reduce cases where we stop early (e.g. model returns “BROWSER_EXTRACT” as text and we treat it as final answer without taking the screenshot) or where we never verify that deliverables (screenshot, attachment) were actually produced.

## How browser-use does it

The project lives at `../browser-use/` (or GitHub: browser-use/browser-use). Relevant pieces:

### 1. Explicit “done” action (agent self-reports)

- The agent has a **done** tool it must call to finish.
- Parameters include **`success: bool`** and a text summary.
- So the **model** decides when to stop and self-reports success. There is no separate “extract expectations” step at the beginning; the task string is the only upfront specification.

### 2. Optional validation before accepting “done” (code_use only)

In **code_use** (code-execution agent), when the model calls `done()` they do **not** trust it blindly:

- They call **`validate_task_completion(task, output, llm)`**:
  - Inputs: **original task**, **agent’s output** (what the model put in `done`).
  - One LLM call with a prompt like: “Determine if the agent has successfully completed the user’s task. Consider: delivered what was requested? data extraction has actual data? truly impossible? could the agent continue?”
  - Response format: **Reasoning** + **Verdict: YES/NO**.
- If **NO**: they **clear** the “task done” flag and the agent **continues** (no extra tool, just “you’re not done yet”).
- If YES or validation errors, they accept done.

So browser-use does **not** extract structured “expectations” upfront; they do a **single verification step at the end** (only in code_use) that can force the agent to keep going.

### 3. Post-hoc judge (telemetry / eval only)

- After the run, a **judge** evaluates the full trace (task + final result + steps + screenshots, optional ground truth).
- Output: verdict (true/false), failure_reason, impossible_task, reached_captcha.
- This is used for **logging and evaluation**; it does **not** change the reply or loop back. So it’s “did we meet expectations?” for analytics, not for control flow.

---

## How OpenClaw does it

The project lives at `../openclaw/` (GitHub: openclaw/openclaw). It’s a personal AI assistant (Gateway + channels + skills); the main agent is turn-based (Pi/OpenAI-style tool loop).

### 1. Main agent (chat + tools): no explicit expectation or verification

- The run **ends** when the model returns with **no tool calls** (e.g. `finish_reason: "stop"`). There is **no** “done” tool; the model simply stops emitting tool calls and returns a final message.
- **No** upfront “extract expectations” step.
- **No** end-of-run LLM verification in the main agent path. The assistant’s last message is sent as-is.

So for normal chat/tool use, OpenClaw does **not** check “did we meet the user’s request?” in code; it relies on the model to decide when to stop and what to say.

### 2. Open-prose extension: post-run inspection (workflow runs only)

The **open-prose** extension (`.prose` workflow programs: session, parallel, loop) has a **post-run inspector** and **calibrator**:

- **Inspector** (`extensions/open-prose/skills/prose/lib/inspector.prose`): After a .prose run completes, you can run “inspect” with depth light/deep and target vm/task/all. An evaluator agent scores:
  - **vm**: completion, binding_integrity, vm_verdict (pass/partial/fail).
  - **task**: output_substance, **goal_alignment** (1–10, “does output fit program purpose?”), **task_verdict** (pass/partial/fail).
- **Calibrator** (`calibrator.prose`): Compares light vs deep inspections for reliability.

This is **evaluation of completed runs** (like browser-use’s judge), not control flow during the run. It’s also specific to **prose workflow runs**, not the main channel chat/tool loop.

### 3. Subagents: completion is push-based

- When a subagent finishes, it **announces** completion; the main session gets a message like “Subagent X completed / finished” and is instructed to turn that into a user-facing update. So “done” is signaled by the subagent lifecycle, not by a verification step.

**Summary:** OpenClaw’s main path does **not** do expectation extraction or end-of-run verification. Only the open-prose extension does post-run task/goal evaluation, and only for .prose runs.

---

## Options for mac-stats

### A. End-only verification (browser-use code_use style)

- **No** upfront extraction.
- **After** the tool loop, before returning `OllamaReply`:
  - One short Ollama call: “Original request: … . What we did (summary): … . Did we fully satisfy the request (e.g. screenshot taken and attached if asked)? Reply: YES or NO. If NO, one sentence what’s missing.”
  - If **NO**: either
    - **Option A1**: Append to the reply: “Note: we may not have fully met your request: &lt;reason&gt;.”
    - **Option A2**: Retry once (e.g. re-enter with “You said we didn’t fully complete: … . Do one more tool if needed then reply.”) — more complex and cost.

Recommendation: **A1** first (append disclaimer when verification says NO). No loop change, minimal cost (one small request per run when enabled).

### B. Extract expectations at start (optional)

- Before or alongside the first RECOMMEND step, one short Ollama call: “From this user request, list 1–5 concrete success criteria (e.g. ‘screenshot of page containing X’, ‘screenshot attached to Discord’).”
- Store as a short list of strings.
- At the end, verification prompt includes: “Criteria: … . Did we meet each? YES/NO and brief reason.”

Pros: clearer audit trail and more targeted verification. Cons: extra latency and token cost at start; criteria can be redundant with the question.

### C. Heuristic “deliverable” check (no LLM)

- Before returning, check simple conditions: e.g. if the user message contained “screenshot” (or “BROWSER_SCREENSHOT” in plan), require `attachment_paths` non-empty; if “find X” then require that the last tool output or final text mentions X.
- If check fails: append “Note: a screenshot was requested but none was attached.” or similar.
- No extra Ollama call; can miss nuanced cases.

### D. Combine

- **C** as a cheap guard (e.g. “screenshot asked but no attachment” → append short note).
- **A** (end-only verification) — with local Ollama we can run it **by default**; no need to opt-in for cost reasons.

## Token cost: we’re 100% local

mac-stats runs **Ollama locally**. We don’t pay per token, so we can afford:

- A **mandatory “done” tool** (browser-use style) if we want the model to explicitly commit to success/fail every time.
- **Always-on verification** (one short “did we meet the request?” call at the end) instead of opt-in.
- **Upfront criteria extraction** (1–3 success criteria at start) and feeding them into the end check.
- **Retry on verification NO** (A2): if verification says we didn’t complete, re-enter the loop once with “Verification said we didn’t complete. Do the remaining steps now.”

So we should **not** avoid these on token grounds. Design for correctness and user satisfaction; use heuristics where they’re sufficient and LLM steps where they help.

## Best of both: how mac-stats can stand out

- **Consider a “done” tool.** With local LLM, the extra prompt/tokens for a mandatory `done(success=…)` are fine. It forces the model to commit and gives us a clear hook: if `success=false` or verification says NO, we don’t accept — we retry or append a disclaimer. Optional: we can keep “no tool call = final answer” as fallback but prefer done when we have it.
- **Always-on verification.** One short Ollama call at the end: “Did we fully satisfy the request?” If NO → retry once (A2) with “Verification said we didn’t complete. Complete it now.” or append disclaimer (A1). No need to make it opt-in.
- **Cheap guards.** Heuristic (C): e.g. “screenshot requested but no attachment” → append note or trigger retry. Catches obvious misses without waiting for verification.
- **Optional: criteria at start.** “List 1–3 concrete success criteria.” Feed into verification. Makes the check more targeted.

**Stand-out line:** *Local agent that verifies completion before replying and retries or warns when we didn’t get it done.*

---

## User escalation: “think harder”, “get it done”, anger

When the user is **not satisfied** — they say “think harder”, “get it done”, “you didn’t do it”, “try again”, or they’re clearly frustrated — we should treat that as a strong signal: **do not just reply with text; actually complete the task.**

### Detection

- **Explicit phrases** (case-insensitive): e.g. “think harder”, “get it done”, “do it”, “you didn’t”, “try again”, “not done”, “wrong”, “didn’t work”, “doesn’t work”, “finish it”, “complete it”, “I said …” (when restating the same request).
- **Session context:** If the previous assistant reply was short or had a disclaimer (“we may not have fully met…”), and the user sends a short follow-up (e.g. “no”, “nope”, “get it done”), treat as escalation.
- **Optional:** Simple sentiment or length — very short, all-caps, or exclamation-heavy follow-ups after a failed or partial reply.

### What to do when escalation is detected

1. **Inject a stronger system or user framing** so the next run is not “another chat turn” but “the user demands completion”:
   - Prepend to the user message or add to system: *“The user is not satisfied. They want the task actually completed, not just discussed. You MUST use tools to fulfill the request (e.g. take the screenshot, fetch the page, run the command). Do not reply with only text. If you need to use a tool, use it; then reply briefly confirming what you did.”*
2. **Raise the bar for “done”.**
   - If we have verification: run it at the end; if it says NO, **auto-retry once** with the same escalation framing (so we get two real attempts, not one).
   - If we have a “done” tool: in escalation mode, **do not accept** a reply that doesn’t include a successful `done(success=true)` or equivalent (e.g. we require at least one tool call in the run, or we require verification YES).
3. **Optionally increase tool budget** for that turn (e.g. allow more tool iterations so the model can “think harder” by doing more steps).
4. **Log escalation** so we can see when users hit this path and tune prompts or heuristics.

### Implementation sketch

- **Discord/chat handler:** Before calling `answer_with_ollama_and_fetch`, check if the latest user message (and optionally prior turn) matches escalation patterns. If yes, set a flag or append the escalation framing to the request.
- **Agent router:** Accept an `escalation: bool` (or `user_insistent: bool`). When true:
  - Add the “user is not satisfied, you MUST complete the task” instruction to the system or first user message.
  - After the tool loop, if we have verification and it returns NO → retry once with the same escalation framing and a short “Verification said we didn’t complete. Complete it now.”
  - Optionally: bump `max_tool_iterations` for that run.

Result: when the user says “get it done” or “think harder”, we don’t just answer again — we **force a completion-oriented run** and, if verification exists, we retry once when we didn’t actually meet the request.

---

## Should we review Claude Code or Cursor?

Both are **proprietary**; their completion/verification logic isn’t public. You can’t “read the code” unless you work there or have special access. You can still **review behavior** to see how they signal “task done” and whether they ever double-check.

- **Cursor**  
  - mac-stats already integrates **cursor-agent** (CURSOR_AGENT tool, see `docs/012_cursor_agent_tasks.md`).  
  - **Worth reviewing:** When you use Composer / Agent for a multi-step task, does it ever say “I’ve verified the change” or “Checking the result…”? Does it stop after N steps, on a “done” gesture, or when it thinks the goal is met?  
  - **How:** Use Composer on a few tasks (e.g. “add a test for X”, “find and screenshot Y on this site”). Note: (1) how it decides to stop, (2) whether it runs any explicit “verify” step (tests, reload, grep), (3) whether it ever backs off (“I couldn’t complete…”). That informs how we might mirror or complement Cursor when we delegate to it.

- **Claude Code**  
  - Agentic coding in terminal/IDE; similar multi-step, tool-use style.  
  - **Worth reviewing:** Same questions as Cursor: when does it stop? Any explicit verification (run tests, re-read file, confirm with user)?  
  - **Relevance:** Slightly less direct than Cursor because mac-stats doesn’t delegate to Claude Code today; still useful for “how do the big agents signal and verify completion?”

**Recommendation:** **Review Cursor first** (you already use cursor-agent; Composer’s behavior is the closest to our tool loop). Then, if you want a second data point, run a few Claude Code tasks and note how they end and whether there’s any visible “check” step. Capture what you see in a short note (e.g. in this doc or `docs/030_*`) so we can align our verification design with real UX.

## References

- **browser-use**  
  - code_use validation: `browser_use/code_use/namespace.py` → `validate_task_completion()`.  
  - done action: `browser_use/tools/service.py` → `DoneAction`, `_register_done_action`, `success` field.  
  - judge: `browser_use/agent/judge.py` → `construct_judge_messages`, verdict/failure_reason (post-hoc only).
- **OpenClaw**  
  - Main agent: run ends on `finish_reason: "stop"` (no done tool, no verification).  
  - open-prose inspector: `extensions/open-prose/skills/prose/lib/inspector.prose` (goal_alignment, task_verdict; post-run only).  
  - Subagent completion: `src/agents/subagent-announce.ts` (push-based “completed” / “finished”).
