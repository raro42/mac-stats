# Werner harness vs OpenClaw / Hermes (why it felt “stupid”)

## Root cause (not the soul prompt)

Werner felt worse because the **harness fought the model**:

| | Classic Werner | OpenClaw / Hermes | Direct Werner (default now) |
|--|----------------|-------------------|-----------------------------|
| Tools | Free-text `TOOL: arg` only | Native `tools` / `tool_calls` | Native schemas + text fallback |
| Loop | criteria → topic → plan → execute → verify | model → tools → model until done | Same as OpenClaw/Hermes |
| Prompt | Huge duplicated tool essays | Short catalog; schemas carry detail | Compact catalog when native+direct |
| Meta-LLMs | 3–5 extra calls before work | None | Skipped by default |

## What we changed (v0.1.88+)

1. **`agentHarnessMode: "direct"` (default)** — skip success-criteria, new-topic, planning `RECOMMEND`, and verify LLM calls. One execute tool-loop. Set `"classic"` to restore the old pipeline.
2. **`agentNativeTools: true` (default)** — send Ollama/OpenAI `tools` schemas on execute + follow-ups; synthesize `TOOL: arg` lines for existing dispatch. Meta-calls still send `tools: []`.
3. **Compact tool prompt** in direct+native mode — short registry lines instead of the giant `AGENT_DESCRIPTIONS_BASE` essay.

Env overrides: `MAC_STATS_AGENT_HARNESS_MODE`, `MAC_STATS_AGENT_NATIVE_TOOLS`.

## What still differs

- Shell is still an allowlist (`RUN_CMD`), not a full terminal with approvals.
- Skills are not yet progressive (`skill_view` on demand) like Hermes.
- Strong cloud tool-calling models still outperform small local ones; harness parity does not replace model quality.

## Rollback

In `~/.mac-stats/config.json`:

```json
{
  "agentHarnessMode": "classic",
  "agentNativeTools": false
}
```
