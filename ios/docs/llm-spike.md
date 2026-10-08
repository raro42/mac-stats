# Model test on the iPhone (phase A)

**Summary:** Qwen3.5-2B stays as the default model. LFM2.5-1.2B stays as the lightweight, fast option.

Measured on October 1, 2026 under the following conditions:

- **Device:** iPhone 12 Pro (A14, 6 GB, iOS 26.6.1).
- **Engine:** llama.cpp b11321 with Metal, 4096-token context and 2 threads.
- **Memory:** with the `increased-memory-limit` entitlement.
- **Connection:** the iPhone was connected by cable and charging for the whole test.

The models were measured one after the other: first Qwen3.5-2B and then LFM2.5-1.2B, which started with the iPhone already warm. Qwen2.5-1.5B was not measured because it was not on the iPhone.

The full data is in [llm-bench-iphone12pro.json](llm-bench-iphone12pro.json). "GB" means 2³⁰ bytes, as in the app's monitor.

## Results

| | Qwen3.5-2B Q4_K_M | LFM2.5-1.2B Q4_K_M |
|---|---|---|
| File | 1.19 GB | 0.68 GB |
| Parameters | 1.9 B | 1.2 B |
| Cold / warm load | 2.0 s / 0.48 s | 1.1 s / 0.15 s |
| First word in a new conversation | 1.3–2.0 s | 0.8–1.4 s |
| Prompt processing (pp512) | 162 tok/s | 214 tok/s ¹ |
| Generation (tg128) | 18.5 tok/s | 32.3 tok/s ¹ |
| Generating non-stop for 3 min | 13 tok/s at the start, 8.6 tok/s from minute 1 on ² | 9.1 tok/s ² |
| App memory (peak) | 0.20 GB | 0.17 GB |
| Minimum memory headroom | 3.80 GB | 3.83 GB |
| 3 load/unload cycles | no failures | no failures |
| Quality (10 questions, scored 1 to 5) | 3.7 | 3.3 |

¹ Measured with the iPhone already warm; cold, it should be somewhat higher.
² Capped by the app's thermal pause (see "Heat").

### Memory

The weights are read from the file with mmap and do not count toward the app's memory. With either model the app never went above 0.2 GB and the headroom stayed above 3.8 GB. Without the entitlement, the headroom was about 3 GB.

### Heat

Generating non-stop with the cable connected, the iPhone goes to "serious" in under a minute. In that state the engine pauses for 100 ms per token. Since the GPU computes the next token during the pause, the speed settles at about 9 tok/s with either model. The sustained-test figures measure that pause, not the chip's limit. In 3 minutes it never reached "critical".

### Quality

One answer per question, at temperature 0.7, scored by hand. The questions were asked in Spanish (es-MX); the exact prompts are in the JSON file. Quoted prompts and model output are kept in the original Spanish, with an English gloss.

| # | Question | Qwen3.5-2B | LFM2.5-1.2B |
|---|---|---|---|
| 1 | ¿Cómo van mi batería y mi memoria? ("How are my battery and memory doing?") | 5 | 2: mixes up figures («3.9 GB disponibles en un total de 0.1 GB», "3.9 GB available out of a total of 0.1 GB") |
| 2 | RAM in two sentences | 4: only one sentence | 5: uses the iPhone's data |
| 3 | 3 tips to keep it from overheating | 4 | 3: «mantén la batería al 50 %» ("keep the battery at 50 %") |
| 4 | Summarize a sentence | 4: adds something that was not there | 5 |
| 5 | 17 × 23 | 1: 351 | 1: 486 |
| 6 | Haiku | 4 | 4 |
| 7 | Translate into English | 5 | 3: "Your phone" instead of "My phone" |
| 8 | 45 GB ÷ 1.5 GB | 5 | 5 |
| 9 | 3 steps to free up space | 2: makes up menus and says that restarting clears the cache | 4: generic but correct |
| 10 | ¿Qué es el estado térmico «serio»? ("What is the 'serious' thermal state?") | 3: good at first, then exaggerates and says the app can see other apps' usage | 1: says the phone is cooling down to perform better |

Both get the multiplication wrong: their arithmetic should not be trusted.

## Decision

**Qwen3.5-2B stays as the default model.** It meets the three phase A requirements:

- **Speed:** 8.6 tok/s sustained (minimum: 8).
- **Memory:** 0.20 GB peak (maximum: 2 GB).
- **Stability:** 3 load/unload cycles with no failures.

It also answers questions about the iPhone itself better than LFM2.5 (questions 1 and 10) and writes better Spanish.

**LFM2.5-1.2B stays as the lightweight option.** It is half the size, loads in half the time and, cold, generates 1.75 times faster. On the other hand, it makes more mistakes with the iPhone's data.

**Qwen2.5-1.5B** stays in the catalog so it can be downloaded from the app.

## Test limitations

- The sustained test lasted 3 minutes instead of 10.
- Battery drain could not be measured: the iPhone was plugged in and stayed at 100 %.
- LFM2.5 was not measured with a cold iPhone.
- There is a single answer per question and the scoring is manual.

## Changes that came out of the test

- **iPhone data in a separate system message.** When it was included in the question, the models copied it into the answer.
- **Prompt processed in 64-token batches.** With 512-token batches, the Stop button («Detener») took seconds to react.
- **Each turn continues the previous turn's prompt exactly.** Qwen3.5 and LFM2.5 are hybrid models: they only reuse what has already been processed if the new prompt starts exactly like the previous one. To achieve that:
  - the history includes the iPhone data that was sent with each question;
  - replies are stored untrimmed;
  - the empty `<think>` block is repeated in previous replies;
  - the history is trimmed in jumps of half the budget, not on every turn.

## How to repeat it

With the iPhone unlocked and the app in the foreground (the app keeps the screen on):

```bash
xcrun devicectl device process launch --device <id> --terminate-existing \
  --environment-variables '{"IOS_STATS_BENCH":"all","IOS_STATS_SOAK_SECS":"180"}' com.gilberto.iosstats
```

- **Progress:** shown in an on-screen banner and in `Documents/llm-bench-progress.jsonl`.
- **Finish:** when it is done, `Documents/llm-bench.done` is created.

To copy the results to the Mac:

```bash
xcrun devicectl device copy from --device <id> --domain-type appDataContainer \
  --domain-identifier com.gilberto.iosstats --source Documents/llm-bench.json --destination llm-bench.json
```
