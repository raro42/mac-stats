# Test results log

Results of every test run on iOS Stats, newest first, so anyone (people or agents) who clones the repo knows what was verified, where, and what failed.

**Rules for new entries**
- Add an entry after each run: date, commit, environment, what ran, results, failures and likely cause.
- No personal data: no device names, UDIDs or CoreDevice ids, hostnames, usernames or home paths, Apple IDs, emails, serials, IPs, or text from personal chats. Device model and iOS version are fine.
- English only.

How to run each check is in [README.md](../README.md) (Tests, Debug tools).

---

## 2026-10-08: saved history and background samples (simulator)

**Environment:** iOS Simulator (iPhone 17 Pro, iOS 26.5 runtime), debug build.

- `cargo test`: 57 passed. New tests cover saved minutes (partial then final record of a minute, day files, 30-day retention, compaction of past days), bucket downsampling (weighted means, worst thermal state, background flag) and loading the last hour at startup.
- `pnpm build` and `check_i18n.py`: pass.
- **Early task registration works:** the log shows `background refresh registered: yes` (registered from `main.mm` before the app starts).
- **Scheduling fails in the simulator** with `BGTaskSchedulerErrorDomain` code 1 (unavailable). This is expected: the simulator does not run background tasks. To be verified on an iPhone.
- **Minutes are saved** to `history/YYYY-MM-DD.jsonl`, for example `{"ts":…,"cpu":27.1,"ram":80.9,"thermal":"nominal","bg":false,"n":19}`. After a relaunch, the 1 h view still shows the earlier minutes and the 24 h view shows them.
- Some minutes held fewer than 60 samples and there were long gaps: macOS seems to slow the simulator down (App Nap) when its window is not in front. This is not an app issue, but simulator numbers should not be used to judge sample counts.
- Found and fixed: with five ranges, the history switcher wrapped onto two lines in Spanish. It now has its own full-width row under the title.

**Pending (iPhone disconnected during this session):**
- the multilingual benchmark and self-test with the prompt fixes;
- real Background App Refresh wake-ups (needs a night with the app in the background).

---

## 2026-10-08: Monitor CPU after the ring optimization (real iPhone)

**Change measured:** the ring gauges now move in whole-percent steps, only animate jumps of 3 points or more, use a 0.3 s transition (was 0.6 s), and skip DOM writes when a value did not change (`ring-gauge.ts`, `monitor.ts`).

**Environment and method:** iPhone 12 Pro, iOS 26.6.1, wired. Same probe as the earlier run (`IOS_STATS_CPU_PROBE=30`), Monitor tab, two passes in opposite order. The iPhone cooled for 8 minutes after the benchmark first.

| Theme | Before (lower pass) | After pass 1 | After pass 2 | After (lower pass) |
|---|---|---|---|---|
| System | 9.4 | 17.8 ¹ | 3.8 | **3.8** |
| Glass (`apple`) | 8.9 | 16.5 ¹ | 4.3 | **4.3** |
| Light | 12.3 | 4.9 | 6.0 | **4.9** |
| Neon | 10.3 | 4.9 | 4.2 | **4.2** |
| Futuristic | 8.7 | 3.9 | 4.1 | **3.9** |
| Data Poster | 2.9 | 3.2 | 3.5 | **3.2** |
| Dark (TUI) | 9.9 | 6.5 | 4.5 | **4.5** |
| Material | 8.6 | 4.3 | 4.4 | **4.3** |
| Architect | 9.2 | 4.4 | 3.4 | **3.4** |
| Swiss Minimalistic | 7.7 | 5.6 | 4.0 | **4.0** |

¹ The first two runs after the benchmark; the iPhone was still busy in the background (pass 2 measured 3.8 and 4.3).

**Result:** the Monitor's iPhone-wide CPU dropped by about half on every theme with rings (System 9.4 → 3.8 %), and all themes now sit within 3–5 %. Data Poster, which has no rings, did not change, which confirms the ring animation was the main cost. Light is still the highest theme, but only by about 1 point.

---

## 2026-10-08: multilingual model benchmark on a real iPhone

**Environment:** iPhone 12 Pro (A14, 6 GB), iOS 26.6.1, wired and charging (one 30 s unplug during the third model), Metal, debug build of commit `9334f40c`. Command: `IOS_STATS_BENCH=all IOS_STATS_BENCH_LANGS=es,en,de,fr,pt-BR,zh-Hans IOS_STATS_SOAK_SECS=60`. Each model answered the same 10 questions in 6 languages (60 answers, temperature 0). Raw data: [llm-bench-multilang-iphone12pro.json](llm-bench-multilang-iphone12pro.json). Run time: 24 min.

**Reply language:** all three models answered in the language they were asked in, in all six languages. The detector flagged 2 answers per language, both expected:
- Q5 ("answer only with the number"): "391" has no detectable language.
- Q7 asks for a translation into another language, so the answer is in that language.
- Two extra flags were numeric one-liners misread as Portuguese ("45 GB / 1,5 GB = 30 Videos.").

| Model | Load cold / warm | Generation (bench, cool) | Generation during the 60 questions | Peak app memory | Lowest headroom |
|---|---|---|---|---|---|
| Qwen3.5-2B | 1.9 s / 0.5 s | 18.5 tok/s | 11.5 tok/s first, then ~7.3 tok/s | 0.22 GB | 4.08 GB |
| LFM2.5-1.2B | 1.3 s / 0.1 s | 17.3 tok/s | ~8.3 tok/s | 0.21 GB | 4.09 GB |
| Qwen2.5-1.5B | 1.5 s / 0.3 s | 11.1 tok/s | ~7.8 tok/s | 0.26 GB | 4.04 GB |

- The iPhone reached the **serious** thermal state early in the first model and stayed there. In that state the engine pauses 100 ms per token on purpose (`LlamaEngine.swift`), which caps generation at about 8–9 tok/s. The quality-phase speeds above measure that cap, not the models.
- No errors, no reply cut by the length limit (every answer ended with `eos`). This is the first measurement of Qwen2.5-1.5B.

**Answer quality** (1–5 per answer, scored by an LLM reviewer reading all 180 answers; same scale as [llm-spike.md](llm-spike.md); a human native-speaker check is still pending):

| Model | es | en | de | fr | pt-BR | zh-Hans | Overall |
|---|---|---|---|---|---|---|---|
| Qwen3.5-2B | 3.9 | 4.1 | 3.7 | 3.6 | 3.6 | 3.5 | **3.7** |
| LFM2.5-1.2B | 2.9 | 4.2 | 2.8 | 3.0 | 2.8 | 2.0 | **3.0** |
| Qwen2.5-1.5B | 3.7 | 4.2 | 2.7 | 2.8 | 3.0 | 3.1 | **3.3** |

- **Qwen3.5-2B stays the default for every language.** It has the best grammar outside English and the most reliable arithmetic (Q8 right in all six languages), and it usually explains the "serious" state correctly. In English the three models are within noise.
- **LFM2.5-1.2B:** the multiplication is wrong in 5 of 6 languages. It pulls the device data into unrelated answers (it used the free-storage number instead of the question's 45 GB) and gives weak Chinese.
- **Qwen2.5-1.5B:** switches language mid-answer (German answer ending in Japanese, French "Mon phone is very slow"). It also has factual slips ("RAM = Read-Only Memory") and once suggested erasing all data.
- **All models** invent iOS Settings paths when asked how to free space.

**Prompt problems found (app side, not model side), not fixed yet:**
1. The device note says "55.8 GB free" without saying it is storage, so models read it as free memory. Label it "storage 55.8 GB free".
2. "app memory X (headroom Y)" confuses every model: they report the app's 0.2 GB or the 3.8 GB headroom as the phone's RAM. Drop it or rephrase it as "RAM X of Y used (Z free)".
3. The thermal state goes to the model in English only ("serious"), which leaks into other languages ("seriös", "serieux"). Send the state name and meaning in the reply language; the UI already has them (`thermal.*` in `src/i18n`).
4. The note goes with every question, and small models leak it into unrelated answers.
5. The language check in the benchmark should skip Q5 (only a number) and expect the target language for Q7 (translation). It also misses answers that mix languages.
6. The whole run was at thermal state serious; a run at nominal would show the normal case.

---

## 2026-10-07: themes, thermal card, i18n on a real iPhone

**Environment:** iPhone 12 Pro (A14, 6 GB), iOS 26.6.1, wired. Debug build (`pnpm tauri ios build --debug --export-method debugging`). Commit `589f5dea` plus the debug-only CPU probe and keep-awake options added in the same session.

### Device self-test (`IOS_STATS_SELFTEST=1`): 10/10

Model Qwen3.5-2B Q4_K_M on Metal.

| Test | Result | Detail |
|---|---|---|
| load | pass | 2,209 ms |
| multi_turn_chat | pass | 3 turns saved, generation 14.4–16.5 tok/s, prompt 424–462 ms on turns 2–3 |
| reuses_cache | pass | tokens reused per turn: 0, 355, 563 |
| language_switch | pass | es → en → "ok" → de: the reply language asked for and the language detected in each reply matched (es, en, en, de) |
| stop | pass | stopped in 51 ms (limit 300 ms) |
| background | pass | cancelled in 32 ms when the app left the foreground |
| memory_warning | pass | model released, next question reloaded it and answered |
| download_wrong_sha | pass | rejected with `checksum_mismatch`, no file left |
| xss_saved | pass | conversation with malicious HTML saved for a visual check |

Note: the first read of `Documents/selftest.json` returned the file of an older run (a stale `selftest.done` was still on the device). Wait for a file that contains the expected tests, not just for `selftest.done`.

### Theme CPU cost

Instruments could not record from the device: `xctrace record` (Xcode's xctrace 16.0) stops at "Timed out waiting for device to boot" although the iPhone is booted, unlocked and wired. The app's debug probe (`IOS_STATS_CPU_PROBE=30`) was used instead: 15 s to settle, then the mean of the iPhone's total CPU usage (all processes, including WebKit) over 30 s, Monitor tab, screen kept on, two passes (forward and reverse order).

iPhone total CPU (% of all cores) while the Monitor tab is open, mean over 30 s. Pass 1 ran System → Swiss; pass 2 ran in reverse order. The lower of the two passes is the best estimate of the theme's own cost: background system activity only adds CPU.

| Theme | Pass 1 | Pass 2 | Lower pass | vs System |
|---|---|---|---|---|
| Data Poster | 2.9 | 6.7 | 2.9 | −6.5 |
| Swiss Minimalistic | 7.7 | 31.8 ¹ | 7.7 | −1.7 |
| Material | 8.6 | 14.3 | 8.6 | −0.8 |
| Futuristic | 8.7 | 32.7 ¹ | 8.7 | −0.7 |
| Glass (`apple`) | 10.7 | 8.9 | 8.9 | −0.5 |
| Architect | 9.2 | 11.4 | 9.2 | −0.2 |
| System | 9.4 | 9.8 | 9.4 | 0 |
| Dark (TUI) | 9.9 | 10.5 | 9.9 | +0.5 |
| Neon | 10.3 | 15.0 | 10.3 | +0.9 |
| Light | 12.3 | 19.9 | 12.3 | +2.9 |

¹ Sustained ~32 % for the whole 30 s (the median was just as high), while the same theme measured 7.7–8.7 % in the other pass. That points to iOS background work, not to the theme.

Findings:
- Blur and glow themes cost about the same as System, within the noise. The exception is **Light**, consistently higher (+3 to +10 points), probably because of its drop-shadow filter on the rings plus a 30 px blur on every card.
- **Data Poster is far cheaper** than every other theme, and it is the only one without SVG rings. That suggests the main cost of the Monitor is the four ring arcs animating (`stroke-dasharray` transition of 0.6 s on every 1 s update), not the themes. Optimized on 2026-10-08 (see that entry).
- Limits of the method: total-device CPU includes background iOS work, and 30 s samples are short. Instruments would separate WebKit from the rest; it could not connect to the device (see above).

---

## 2026-10-07: themes and thermal card (simulator)

**Environment:** iOS Simulator (iPhone 17 Pro, iOS 26.2) on an Apple Silicon Mac. Commit `589f5dea`.

- `cargo test` (src-tauri): 47 passed. Covers the worst thermal state per minute in history, the theme list matching Rust and web, languages and error codes.
- `pnpm build`: passes (every key present in every dictionary). `python3 scripts/check_i18n.py`: passes.
- `cargo check --target aarch64-apple-ios-sim`: passes (UIKit window style code).
- Screenshots of all nine themes and "System", in English and Spanish, with `IOS_STATS_FAKE_THERMAL=serious|fair`. The status bar is readable on dark and light themes. Settings → Appearance lists System plus the nine desktop names (Glass for `apple`). The thermal strip shows the forced state at the end of the history.
- Found and fixed: in Data Poster the big number overlapped the mini bar chart, because the desktop's 80 px chart was too wide for the phone tile. The chart was scaled to the tile and moved beside the label.
- Not covered by the simulator: taps (no UI automation), VoiceOver, real thermal states, GPU (Metal) performance.

---

## 2026-10-07: localization and chat language switching (simulator)

**Environment:** iOS Simulator (iPhone 17, iOS 26.2). Commit `b72aab9f`.

- `cargo test`: 44 passed. `pnpm build` and `check_i18n.py`: pass.
- Screenshots in German, Simplified Chinese, French and the pseudo-locale: every visible string translated (pseudo-locale brackets everywhere), errors translated with their parameters, numbers and units per locale (for example `1,2 Go` in French, `462,1 GB frei` in German).
- Self-test with Qwen3.5-2B on the simulator CPU: 9/10.
  - `language_switch` passed (es, en, en, de).
  - `stop` failed: 6.4 s against a 300 ms limit. Cause: the simulator runs the model on the CPU, and cancellation only happens between decode chunks. On the iPhone (Metal) the same test passed in 51 ms (see above).
- Found and fixed: long placeholders (German, French) wrapped in the one-line chat input; they now end with an ellipsis.
