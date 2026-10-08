# iOS Stats

iPhone app derived from [mac-stats](../README.md), with two parts:

- **Monitor:** CPU, RAM, app memory, storage, battery, thermal state, Low Power Mode, network, and 5 min / 1 h history.
- **Local AI chat:** a small language model that runs **on the iPhone itself** with [llama.cpp](https://github.com/ggml-org/llama.cpp), with no server and no network connection. Ollama does not run on iOS; llama.cpp is the engine Ollama is built on, and it uses the same GGUF models.

It is a standalone Tauri 2 project inside the repo: it does not touch the Mac app.

## Layout

| Path | Contents |
|---|---|
| `src/` | UI (Vite + TypeScript, no framework): monitor, chat and lab |
| `src-tauri/src/metrics/` | iPhone readings in Rust (Mach, Foundation, UIKit, `getifaddrs`) |
| `src-tauri/src/chat/` | Model catalog, prompt, conversations and chat commands |
| `src-tauri/src/lab.rs`, `selftest.rs` | Lab, automatic benchmark and self-test (debug) |
| `plugins/tauri-plugin-llm/` | Tauri plugin: `LlamaEngine.swift` (llama.cpp), `ModelStore.swift` (downloads) |
| `models/catalog.json` | Available models, pinned to a Hugging Face commit with their SHA-256 |
| `scripts/` | `build-llama.sh` (builds llama.cpp) and `fetch-models.sh` (downloads models on the Mac) |
| `vendor/` | Built `llama.xcframework` and its privacy manifest (the framework is not committed to git) |

## Requirements

- macOS with Xcode 26 and a developer account signed in to Xcode (*Settings → Accounts*).
- Rust with the iOS targets: `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`.
- `brew install xcodegen cmake` and pnpm.

## Setup

```bash
pnpm install
./scripts/build-llama.sh          # llama.cpp b11321 for iPhone and simulator (~15 min the first time)
pnpm tauri ios init               # only if src-tauri/gen/apple is missing
```

`tauri ios init` rewrites the entitlements file; the plugin's `build.rs` adds `increased-memory-limit` back on every build.

## Run

```bash
# Simulator (the model runs on the CPU)
pnpm tauri ios build --debug --target aarch64-sim
xcrun simctl install booted "src-tauri/gen/apple/build/arm64-sim/iOS Stats.app"
xcrun simctl launch booted com.gilberto.iosstats

# iPhone connected by cable (GPU with Metal)
pnpm tauri ios build --debug --export-method debugging
xcrun devicectl device install app --device <id> "src-tauri/gen/apple/build/arm64/iOS Stats.ipa"
xcrun devicectl device process launch --device <id> --terminate-existing com.gilberto.iosstats
```

Always build from Terminal: builds started from the Xcode UI cannot find Node or pnpm.

## Models

The normal way is to download them from the app (Chat tab): Wi‑Fi only by default, with resume and SHA-256 verification. For testing you can also download them on the Mac (`./scripts/fetch-models.sh`) and copy them to the iPhone, although `devicectl` copies at ~1 MB/s:

```bash
xcrun devicectl device copy to --device <id> --domain-type appDataContainer \
  --domain-identifier com.gilberto.iosstats --source models/<file>.gguf \
  --destination "Library/Application Support/models/<file>.gguf"
```

The model test results are in [docs/llm-spike.md](docs/llm-spike.md).

## Debug tools

These only exist in debug builds. They are passed as environment variables when launching the app: on the iPhone with `devicectl ... process launch --environment-variables '{"VAR":"value"}'` and in the simulator with the `SIMCTL_CHILD_` prefix.

| Variable | What it does |
|---|---|
| `IOS_STATS_BENCH=all` or `<id>,<id>` | Benchmarks the models (load, speed, memory, temperature, 10 quality questions per language) and saves `Documents/llm-bench.json`; progress is shown on screen and in `Documents/llm-bench-progress.jsonl` |
| `IOS_STATS_BENCH_LANGS=es,en,de,fr,pt-BR,zh-Hans` | Languages of the quality questions (default `es`); each answer records the language it was written in |
| `IOS_STATS_SOAK_SECS`, `IOS_STATS_THREADS` | Duration of the sustained test (600 s) and number of threads (2) for the benchmark |
| `IOS_STATS_SELFTEST=1` | Chat self-test, including a language switch (es, en, "ok", de); saves `Documents/selftest.json`. With `IOS_STATS_SELFTEST_DOWNLOAD=<id>` it also downloads that model |
| `IOS_STATS_FAKE_THERMAL=fair\|serious\|critical` | Simulates the thermal state in the monitor (`fair` only there) and, for serious/critical, in the engine |
| `IOS_STATS_DEMO_PROMPT="…"` | Opens the chat and sends that question on launch |
| `IOS_STATS_CPU_PROBE=<seconds>` | After 15 s, averages the iPhone's total CPU (including WebKit) and writes `Documents/cpu-probe.json`; used to compare themes |
| `IOS_STATS_KEEP_AWAKE=1` | Keeps the screen on, for measurements with the iPhone untouched |
| `IOS_STATS_DEMO_VIEW=monitor\|chat\|settings[-bottom][-<range>]` | Opens that tab on launch; `-bottom` scrolls to the end and a range (`1h`, `24h`, `7d`, `30d`) opens that history view (for screenshots) |

The Chat tab also includes a "Lab" card for loading, benchmarking and trying models by hand. Debug builds also offer a "Pseudo" language in Settings: every string shows as `[!! … ~~~ !!]`, about 40% longer, so hard-coded text and clipped layouts stand out.

## Themes and thermal indicator

- **Themes:** the nine desktop mac-stats themes with the same names and look (the desktop "Apple" theme is called "Glass" here, without the logo, because of App Store rules), plus the default "System" theme that follows iPhone light/dark. Pick one in Settings → Appearance. Details and the desktop-to-iPhone mapping: [docs/themes.md](docs/themes.md).
- **Thermal indicator:** iOS exposes only the thermal state (Nominal, Fair, Serious, Critical), not °C. The Monitor shows it as a card with the four levels and what the current one means, plus a strip under the history charts (in the 1 h view, the worst state of each minute). Labels and colors match the desktop's thermal card.

## Languages

The app ships in Spanish, English, German, French, Brazilian Portuguese and Simplified Chinese. It follows the iPhone's language (including the per-app language in iOS Settings) and can be changed in the app's Settings tab. Unsupported languages fall back to English.

- **UI text** lives in `src/i18n/`: one dictionary per language, all typed against `messages.ts`, so a missing key fails `pnpm build`. Static HTML uses `data-i18n*` attributes; code uses `t()` / `tp()` (plurals). Numbers and units use `Intl` with the iPhone's region (for example `es-MX` keeps the decimal point).
- **Errors** never carry text: Rust and Swift return a code from `src/i18n/error-codes.ts` plus parameters, and the web layer translates it.
- **Rust decides the language** (`src-tauri/src/language.rs`), so the UI and the chat always agree.
- **The chat** prompt is English. Each turn ends with "Reply in <Language>.": a clear message in another language (detected on the device with Apple's NaturalLanguage) switches the reply language, short messages keep the current one, and otherwise the app language is used.
- To add a language: add it to `src/i18n/languages.ts`, create its dictionary, add it to `SUPPORTED` and `match_tag` in `language.rs`, and to `CFBundleLocalizations` plus a `<lang>.lproj/InfoPlist.strings` in `src-tauri/gen/apple/` (then run `xcodegen generate` there).

## Tests

Every run is logged in [docs/test-results.md](docs/test-results.md) (no personal data). Add an entry after each run.

```bash
(cd src-tauri && cargo test)   # metrics, history, network, catalog, prompt, conversations, languages, error codes
pnpm build                     # TypeScript type check (every key in every language) and bundling
python3 scripts/check_i18n.py  # no hard-coded UI text, Spanish leftovers or unknown error codes outside src/i18n
```

## iOS limits

- No temperatures in °C, GPU usage, frequency or process list: iOS only exposes the thermal state (4 levels).
- The app is suspended in the background: the monitor leaves a gap in the history and generation is cancelled.
- The app only loads a model if its memory headroom exceeds the model size plus 512 MB. On an iPhone 12 Pro the headroom is ~3.8 GB with the `increased-memory-limit` entitlement and ~3 GB without it. The weights are read with mmap and barely count toward the app's memory.
