# Expectation check at end of tool-chain (design)

## Goal

At **start**: extract 1–3 concrete success criteria from the user’s request. At **end**: before returning, verify we met them (one Ollama call). If not, append a disclaimer or retry. Reduces early stops (e.g. model returns text instead of taking a screenshot) and unverified deliverables.

## How others do it (short)

| System | Loop end | Upfront criteria? | End verification? | Heuristic guard? | Escalation? |
|--------|----------|-------------------|--------------------|------------------|-------------|
| **browser-use** | Model calls `done` | No | code_use only: `validate_task_completion` → can force continue | No | No |
| **OpenClaw** | No tool calls = stop | No | No (open-prose post-run only for .prose) | No | No |
| **Hermes** | No tool calls = done | No | No (completed = structural) | No | No |
| **mac-stats** | No tool calls = final answer | **Yes: 1–3 criteria at start** | **Yes: always-on** (if NO → disclaimer) | **Yes** (screenshot asked, no attachment) | **Yes** (stronger run + higher tool budget) |

Refs: [browser-use](https://github.com/browser-use/browser-use), [OpenClaw](https://github.com/openclaw/openclaw), [Hermes](https://github.com/NousResearch/hermes-agent).

## Design choices (mac-stats)

- **Criteria at start.** One short Ollama call: “From this request, list 1–3 concrete success criteria (e.g. ‘screenshot of X attached’).” Feed them into end verification. Not optional.
- **Always-on verification.** One Ollama call at end: “Did we fully satisfy the request?” (+ criteria if present). If NO → append disclaimer (A1); optional retry (A2) later.
- **Heuristic guard.** If question/plan mentions screenshot but `attachment_paths` is empty → append note.
- **Escalation.** User says “think harder” / “get it done” etc. → prepend “user is not satisfied, you MUST complete the task”, bump `max_tool_iterations` by 10.

**Stand-out:** *Local agent that verifies completion before replying and retries or warns when we didn’t get it done.*

**Broader lens:** Shen et al. (“Task Completion Agents are Not Ideal Collaborators”): verification answers “did we meet it?”; collaborative agents also sustain engagement and scaffold the user across turns. Akshathala et al. (“Beyond Task Completion”): assess along LLM, Memory, Tools, Environment; our verification is one pillar.

---

## User escalation

**Detection:** Phrases like “think harder”, “get it done”, “try again”, “no”, “nope” after a short or disclaimed reply.  
**Action:** Stronger system framing (“you MUST use tools to fulfill the request”), optional retry once if verification says NO, increase tool budget (+10 iterations).  
**Implementation:** Patterns from **~/.mac-stats/escalation_patterns.md** (user-editable, one phrase per line). `Config::load_escalation_patterns()`; `is_escalation_message()` in Discord handler; `escalation: bool` on `answer_with_ollama_and_fetch`; when true, prepend framing and add 10 to `max_tool_iterations`.

### How to steer the behaviour (without being rude to the code)

When the bot gives you a shrug or a half-baked answer, you don’t have to stick to “think harder”. Edit **~/.mac-stats/escalation_patterns.md**: one phrase per line, anything that means “I’m not satisfied, do better.” The next time your message *contains* one of those phrases (case doesn’t matter), mac-stats flips into *completion mode*: it tells the model the user is not happy and bumps the tool budget so it can actually finish the job.

**Examples you can add:**  
`I don't like your answer` · `You are stupid` · `That was useless` · `Try again properly` · `Nope, do it for real` · `I said do it` · or your favourite polite variant. No restart required — we re-read the file on every message. So go ahead: make your displeasure *actionable*.

**Auto-add:** When we detect escalation (the message matched an existing pattern), we append the user’s phrase to the file if it isn’t already there. So the list grows with the way *you* complain — next time the same wording will trigger escalation without you having to edit the file.

---

## Implementation plan

| Item | Status | Location |
|------|--------|----------|
| Criteria at start (extract 1–3, feed into verification) | **Done** | `extract_success_criteria()` in `ollama.rs`; passed to `verify_completion()` |
| Always-on end verification (one Ollama call, disclaimer on NO) | **Done** | `verify_completion()` in `ollama.rs`; called before `Ok(OllamaReply)` |
| Heuristic: screenshot requested but no attachment | **Done** | `ollama.rs` before `verify_completion`: append note if (screenshot in question/plan && `attachment_paths.is_empty()`) |
| Escalation detection and framing | **Done** | `discord/mod.rs`: `is_escalation_message()`, `default_verbose_for_dm`; `ollama.rs`: `escalation` param, prepend text, `max_tool_iterations += 10` |
| Retry once on verification NO (A2) | **Done** | When verification says NO we re-run once with “complete the remaining steps”; retry run uses `retry_on_verification_no: false`. Discord, scheduler, task runner pass `true`. |
| Optional: “done” tool (browser-use style) | Deferred | Would require planner/execution to emit and honour `done(success=…)`; not required for best-of-breed. |

---

## Deferred vs best-of-breed

We promised **“retries or warns”** in the stand-out line. To satisfy that and the user:

- **A2 (retry once on verification NO)** is now **implemented**: when the verifier says we didn’t complete, we run one more pass with a “complete the remaining steps” prompt and return that result (or append the disclaimer if the retry still doesn’t satisfy). So we truly **retry or warn**, not only warn.
- **“Done” tool** remains deferred: it would add a clear commit step (model calls `done(success=…)`) but would need planner/execution changes. We already have criteria at start, always-on verification, heuristic guard, escalation, and one retry — which is enough to be best-of-breed. The done tool would be a UX/structural improvement, not a requirement for “did we satisfy the user?”

---

## Related work (papers)

- **Plan Verification for LLM-Based Embodied Task Completion Agents** — [arXiv:2509.02761](https://arxiv.org/abs/2509.02761). Judge + Planner iterative verification.
- **Auto-Eval Judge** — [arXiv:2508.05508](https://arxiv.org/abs/2508.05508). Decompose tasks, validate steps; Judge aligns with human task success.
- **LLM evaluation of constraint-satisfaction in agent responses** — [arXiv:2409.14371](https://arxiv.org/abs/2409.14371). LLM as verifier for constraints.
- **Beyond Task Completion** (Akshathala et al.) — [arXiv:2512.12791](https://arxiv.org/abs/2512.12791). Four pillars: LLM, Memory, Tools, Environment; beyond binary completion.
- **Task Completion Agents are Not Ideal Collaborators** (Shen et al.) — [OpenReview](https://openreview.net/forum?id=JMblCtvaDH). Collaborative agents; effort scaling.

---

## References (code pointers)

- **browser-use:** `code_use/namespace.py` → `validate_task_completion()`; `tools/service.py` → `DoneAction`; `agent/judge.py` (post-hoc).
- **OpenClaw:** Main agent ends on `finish_reason: "stop"`; open-prose inspector for .prose runs only.
- **Hermes:** `run_agent.py` (completed = final_response + under max_iterations); `agent_loop.py` (“No tool calls — model is done”).
