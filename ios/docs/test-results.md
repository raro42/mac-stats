# Test results log

Results of every test run on iOS Stats, newest first, so anyone (people or agents) who clones the repo knows what was verified, where, and what failed.

**Rules for new entries**
- Add an entry after each run: date, commit, environment, what ran, results, failures and likely cause.
- No personal data: no device names, UDIDs or CoreDevice ids, hostnames, usernames or home paths, Apple IDs, emails, serials, IPs, or text from personal chats. Device model and iOS version are fine.
- English only.

How to run each check is in [README.md](../README.md) (Tests, Debug tools).

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
- **Data Poster is far cheaper** than every other theme, and it is the only one without SVG rings. That suggests the main cost of the Monitor is the four ring arcs animating (`stroke-dasharray` transition of 0.6 s on every 1 s update), not the themes. Worth optimizing for every theme; not changed yet.
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
