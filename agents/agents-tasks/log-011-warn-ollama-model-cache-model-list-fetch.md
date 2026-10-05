# log-011: warn-ollama-model-cache-model-list-fetch

- Created: 2026-10-05 20:22 UTC
- Count in window: 6
- Signature: `2f63191ca9`

## Sample

```
2026-10-05T20:13:25.424782Z  WARN [ollama/model_cache] Model list fetch failed: Failed to request models: error sending request for url (http://localhost:11434/api/tags): error trying to connect: tcp connect error: Connection refused (os error 111); not updating cache
```

## Next

Triage in mac-stats; fix or mark benign in agents/log-monitor/README.md.
