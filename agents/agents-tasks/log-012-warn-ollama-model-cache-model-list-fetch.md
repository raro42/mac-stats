# log-012: warn-ollama-model-cache-model-list-fetch

- Created: 2026-10-05 20:42 UTC
- Count in window: 41
- Signature: `5f2f4a1542`
- Status: **fixed in v0.1.1388**

## Sample

```
2026-10-05T20:13:26.165100Z  WARN [ollama/model_cache] Model list fetch failed: Ollama is temporarily unavailable (circuit open, will retry in 29s); not updating cache
```

## Fix

`model_list_cache`: 30s fail cooldown per endpoint + at most one WARN / 5 minutes (further fails DEBUG). SWR skips background refresh during cooldown.
