# agents.md — Project instructions (Cursor, Claude Code, and other assistants)

## Audience

This is the **single** project-instructions file for **Cursor**, **Claude Code** (claude.ai/code), and similar tools. Read it before making changes. Do **not** add `Co-authored-by:`, `Signed-off-by:`, or IDE/agent attribution to commit messages (see Coding Principles).

## Goal

Build a polished macOS (Apple Silicon) stats app (Rust + Tauri) that reads CPU/process metrics and presents them in a clean "macOS glass" UI. Favor correctness, safety, and maintainability over hacks.

## Project overview (at a glance)

**mac-stats** is a lightweight system monitor for macOS (Rust + Tauri). It shows real-time CPU, GPU, RAM, and disk usage in the menu bar with low overhead (on the order of ~0.5% idle, under 1% with the CPU window open). It includes real-time metrics, temperature (SMC), CPU frequency (IOReport), a process list, themes, idle optimizations, and Ollama chat with code execution, **FETCH_URL** (server-side fetch, no CORS), and **BROWSER_SCREENSHOT** (CDP: PNG under `~/.mac-stats/screenshots/`, attachable in Discord when requested).

## Build and run

```bash
# Development (hot reload where applicable)
./run dev
# or
cd src-tauri && cargo run

# Release build
./run
# or
cd src-tauri && cargo build --release

# CPU window (testing)
./target/release/mac_stats --cpu

# Verbosity: default is -vv. Use -vvv for maximum, -v for minimal.
./target/release/mac_stats
./target/release/mac_stats -vvv

# Detailed frequency / IOReport logging
./target/release/mac_stats --frequency
```

## Tech Stack
- Rust (core logic + FFI)
- Tauri (UI shell)
- Web UI (HTML/CSS/JS or framework as used in repo)
- macOS-specific metrics via:
  - User-mode: libproc / sysctl (safe keys only), NSProcessInfo thermal state
  - Root-mode (optional): powermetrics parsing OR IOReport (private) behind a helper

## Coding Principles
- Keep modules small and cohesive.
- When coding, create logs and read them afterwards.
- Execute the app yourself, read the logs, understand if changes are working from the logs - iterate until you are confident the change is good to be test by a human.
- **ALWAYS check for build errors before starting the app**: Run `cargo check` in `src-tauri/` directory and fix any compilation errors before attempting to start the app.
- **Commits**: Do **not** add `Co-authored-by:`, `Signed-off-by:`, or any Cursor/agent/IDE attribution to commit messages. Do not advertise the agent or tool in commits or in the repo. To enforce this locally, run `./scripts/install-git-hooks.sh` once (installs a prepare-commit-msg hook that strips such lines).
- **After changes that affect runtime behavior** (Redmine, tasks, agent prompts, scheduler, Discord, Ollama tools): **restart mac-stats automatically** (e.g. `pkill -f mac_stats; cd src-tauri && cargo run --release`). Default verbosity is -vv so logs are visible. Then test (run a relevant task, trigger the feature, or check `~/.mac-stats/debug.log`). Do not assume it works without restart and verification. Do not skip the restart step.
- Never install software on the host. Always ask before installing.
- If you need to install software, use containers.  
- No "god files": split into `metrics/`, `ffi/`, `ui/`, `config/`, `logging/`.
- Prefer safe Rust. Minimize `unsafe` and isolate it in `ffi/` with strict audits.
- Every `unsafe` block must have a comment explaining invariants and safety assumptions.

## macOS Constraints (Apple Silicon)
- Do not claim that `sysctl hw.cpufrequency` exists on Apple Silicon — it doesn't.
- CPU frequency + many sensors require root and are unstable across macOS versions.
- Prefer Apple-supported indicators:
  - CPU usage (host stats/libproc)
  - Thermal state (`NSProcessInfo.thermalState`)
  - Thermal pressure (where available)
- If implementing frequency/temperature:
  - Use `powermetrics --format json` when running as root OR
  - Use IOReport (private) only behind a root helper and treat output as unstable.

## FFI Safety Rules (Critical)
- Never allow Obj-C/C++ exceptions to cross into Rust.
  - If calling Obj-C/C++: use a shim that catches exceptions and returns error codes.
- No panicking across FFI boundaries. All extern functions must be `noexcept` style.
- Always validate CoreFoundation types with `CFGetTypeID()` before casting.
- Always check for null pointers from CF APIs.
- Follow CF ownership rules (Create/Copy = you release; Get = do not release).
- Avoid `transmute` for IOReport payloads; validate sizes and use `read_unaligned` if needed.

## Root Helper Design (if needed)
- UI must run unprivileged.
- Root-only functionality must be isolated:
  - separate binary/daemon, launched via launchd or explicit sudo in dev mode
  - communicate over a Unix domain socket or localhost with auth token
- Make root metrics optional; app must work gracefully without them.
- When doing "frontend" changes, you can pkill running instance and restart the app.

## Logging & Diagnostics
- Use `tracing` for structured logs.
- Provide `--verbose` / `--json` flags for CLI diagnostics.
- Always execute the code after changes and read the logs.
- When debugging crashes:
  - include an LLDB recipe in `docs/debugging.md`
  - include env flags: `MallocScribble`, `MallocGuardEdges`
  - document how to breakpoint `objc_exception_throw`, `__cxa_throw`, `abort`

## UI Guidelines
- Use macOS-like glass design:
  - backdrop blur + subtle borders + soft shadows
  - consistent spacing, typography, and units
- Always display units (°C, GHz, W, %).
- Prefer ring gauges (SVG) over heavy pie charts.
- Animate value changes subtly (200–350ms), never flashy.

## Output & Formatting
- Keep responses and code changes minimal and targeted.
- When asked to refactor, propose a file/module plan first, then implement.
- Add or update tests for parsing and metrics extraction where possible.

## What to Avoid
- Hard-coded absolute paths (especially root-only locations).
- Global mutable state unless justified; prefer explicit state passing.
- Copying private Apple keys/constants without isolating them and documenting risk.
- Over-promising sensor availability on Apple Silicon.

## Repo Conventions
- Use `rustfmt` and `clippy` clean builds.
- Prefer `anyhow` + `thiserror` for error handling.
- Use `serde` for structured data exchanged with UI.

## Updating ~/.mac-stats from defaults
- When asked to update MD files or agents inside `.mac-stats`, or to sync repo defaults into `~/.mac-stats`: **merge** default content into existing files; **do not overwrite**.
- Add missing sections, new agents, or new bullets; preserve the user’s existing content. See `docs/024_mac_stats_merge_defaults.md`.

## When Uncertain
- Ask for the exact macOS version and target (Dev vs notarized distribution).
- Default to the safest, Apple-supported approach.

---

## Performance Measurement & Optimization

### Measuring CPU, GPU, and RAM Usage

Use the provided measurement script to track performance metrics:

```bash
# Measure with CPU window open (30 seconds)
./scripts/measure_performance.sh 30 1 window

# Measure idle (menu bar only, 60 seconds)
./scripts/measure_performance.sh 60 1 idle

# Custom: 120 seconds, 2-second intervals
./scripts/measure_performance.sh 120 2 idle
```

**Script Features**:
- Measures CPU usage (%), Memory (%), RSS/VSZ (MB), thread count
- Outputs live measurements + statistics
- Saves results as text report + CSV for analysis
- Reusable for before/after comparison
- Located in `scripts/` directory (keep root clean)

**Output Files**:
- `performance_window_YYYYMMDD_HHMMSS.txt` - Detailed report
- `performance_window_YYYYMMDD_HHMMSS.csv` - Data for spreadsheets

### CPU Optimization Workflow

1. **Baseline Measurement** (before any changes):
   ```bash
   ./scripts/measure_performance.sh 30 1 window  # With window
   ./scripts/measure_performance.sh 30 1 idle    # Without window
   ```

2. **Implement Optimizations**:
   - See `docs/001_task_optimize_backend.md` (backend)
   - See `docs/002_task_optimize_frontend.md` (frontend)
   - See `docs/003_task_optimize_advanced_idle.md` (advanced)

3. **Measure After Each Phase**:
   ```bash
   # After Phase 1:
   ./scripts/measure_performance.sh 30 1 window

   # After Phase 2:
   ./scripts/measure_performance.sh 30 1 window

   # Compare: Phase 1 vs Phase 2 vs baseline
   ```

4. **Track Cumulative Improvement**:
   - Save results with phase names
   - Compare CSV files in spreadsheet
   - Document CPU reduction percentage

### Directory Structure: Keep Root Clean

```
mac-stats/
├── scripts/                    ← All development/measurement scripts
│   ├── build-dmg.sh
│   ├── measure_performance.sh  ← Performance measurement
│   ├── take-screenshot.sh
│   └── trace_backend.sh
├── docs/                       ← Documentation and analysis
│   ├── 000_task_optimize_summary.md
│   ├── 001_task_optimize_backend.md
│   ├── 002_task_optimize_frontend.md
│   └── 003_task_optimize_advanced_idle.md
├── src-tauri/                  ← Source code
│   ├── src/                    ← Rust backend
│   ├── dist/                   ← Frontend assets
│   └── Cargo.toml
└── README.md                   ← Keep root clean, no scripts here!
```

**Rule**: All `.sh`, `.py`, and utility scripts go in `scripts/` directory, not root.

---

## Codebase Structure & Organization

### Overview

The codebase follows a clear separation between Rust backend (Tauri) and JavaScript frontend, with modular organization for maintainability.

### Directory Structure

```
mac-stats/
├── src/                          ← Frontend source (JavaScript/HTML/CSS)
│   ├── dashboard.html            ← Main dashboard UI
│   ├── dashboard.js              ← Dashboard logic (delegates to modules)
│   ├── dashboard.css             ← Dashboard styles
│   ├── ollama.js                 ← Ollama AI chat integration (unified module)
│   ├── tauri-logger.js           ← Console logging bridge to Rust
│   ├── main.js                   ← Main entry point
│   ├── index.html                ← App entry HTML
│   └── assets/                   ← Icons and images
├── src-tauri/                    ← Rust backend (Tauri application)
│   ├── src/
│   │   ├── main.rs               ← Entry point, CLI argument parsing
│   │   ├── lib.rs                ← Main app logic, Tauri setup, background threads
│   │   ├── state.rs              ← Application state management
│   │   │
│   │   ├── commands/             ← Tauri commands (exposed to frontend)
│   │   │   ├── mod.rs            ← Command module exports
│   │   │   ├── ollama.rs         ← Ollama AI chat commands (chat, code execution, FETCH_URL, BROWSER_SCREENSHOT)
│   │   │   ├── ollama_models.rs  ← Ollama model management (list, pull, delete, embed, load/unload)
│   │   │   ├── ollama_logging.rs ← Ollama JS execution logging bridge
│   │   │   ├── browser.rs        ← fetch_page for Ollama web tasks (no CORS)
│   │   │   ├── monitors.rs       ← Monitor management (add, remove, check)
│   │   │   ├── alerts.rs         ← Alert management
│   │   │   ├── security.rs       ← Keychain/credential management
│   │   │   ├── logging.rs        ← Generic JS console logging bridge
│   │   │   ├── plugins.rs        ← Plugin system commands
│   │   │   ├── pre_routing.rs    ← Deterministic pre-routing (screenshot, RUN_CMD, REDMINE)
│   │   │   ├── task_tool_handlers.rs ← TASK_* and SCHEDULE tool handlers
│   │   │   └── tool_parsing.rs   ← Tool invocation parsing from model responses
│   │   │
│   │   ├── metrics/              ← System metrics collection
│   │   │   └── mod.rs            ← CPU, RAM, GPU, disk, temperature, frequency
│   │   │
│   │   ├── ollama/               ← Ollama API client
│   │   │   └── mod.rs            ← HTTP client, chat API, model listing
│   │   │
│   │   ├── monitors/             ← Monitoring implementations
│   │   │   ├── mod.rs            ← Monitor trait and registry
│   │   │   ├── website.rs        ← HTTP/HTTPS website monitoring
│   │   │   └── social.rs         ← Social media monitoring (Mastodon, etc.)
│   │   │
│   │   ├── alerts/               ← Alert system
│   │   │   ├── mod.rs            ← Alert rules and engine
│   │   │   ├── rules.rs          ← Alert rule definitions
│   │   │   └── channels.rs       ← Alert delivery channels (Telegram, Slack, etc.)
│   │   │
│   │   ├── plugins/              ← Plugin system
│   │   │   └── mod.rs            ← Script-based plugin execution
│   │   │
│   │   ├── security/             ← Security utilities
│   │   │   └── mod.rs            ← macOS Keychain integration
│   │   │
│   │   ├── config/               ← Configuration management
│   │   │   └── mod.rs            ← Paths, build info, app config, screenshots_dir()
│   │   │
│   │   ├── browser_agent/        ← CDP browser (screenshots)
│   │   │   └── mod.rs            ← connect_cdp, navigate, take_screenshot (PNG to ~/.mac-stats/screenshots/)
│   │   │
│   │   ├── ffi/                  ← Foreign Function Interface
│   │   │   ├── mod.rs            ← FFI module exports
│   │   │   ├── ioreport.rs       ← IOReport API wrappers (CPU frequency)
│   │   │   └── objc.rs           ← Objective-C wrappers (thermal state)
│   │   │
│   │   ├── ui/                   ← UI components
│   │   │   ├── mod.rs            ← UI module exports
│   │   │   └── status_bar.rs     ← Menu bar status item
│   │   │
│   │   └── logging/              ← Logging infrastructure
│   │       ├── mod.rs            ← Tracing setup and configuration
│   │       └── legacy.rs         ← Legacy logging (if any)
│   │
│   ├── dist/                     ← Frontend assets (copied from src/)
│   │   ├── cpu.js                ← CPU window logic (UI-specific, delegates to ollama.js)
│   │   ├── ollama.js             ← Synced from src/ollama.js
│   │   ├── tauri-logger.js       ← Synced from src/tauri-logger.js
│   │   └── themes/               ← UI themes
│   │       └── apple/
│   │           ├── cpu.html      ← CPU window HTML template
│   │           └── cpu.css       ← CPU window styles
│   │
│   └── Cargo.toml                ← Rust dependencies
│
├── scripts/                      ← Development and utility scripts
│   ├── sync-dist.sh              ← Sync frontend files to dist/
│   ├── measure_performance.sh    ← Performance measurement tool
│   └── ...
└── docs/                         ← Documentation
    └── ...
```

### Key Modules & Responsibilities

#### Frontend (JavaScript)

**`src/ollama.js`** - **Ollama AI Chat Integration** (Single Source of Truth)
- **Purpose**: Unified module for all Ollama-related functionality
- **Features**:
  - Connection management (auto-configuration, connection checking)
  - Chat message handling with conversation history
  - Code execution flow (ping-pong with Ollama)
  - Model management
  - Exposed via `window.Ollama` global object
- **Conversation History**: Maintains in-memory conversation history (last 20 messages) for context-aware responses
- **Code Execution**: Handles `ROLE=code-assistant` responses, executes JavaScript, sends results back to Ollama
- **Location**: Primary source in `src/ollama.js`, synced to `src-tauri/dist/ollama.js`

**`src/dashboard.js`**
- Main dashboard logic
- Delegates Ollama functionality to `ollama.js` module
- UI event handling

**`src/tauri-logger.js`**
- Intercepts `console.log/warn/error` calls
- Forwards logs to Rust via `log_from_js` Tauri command
- Provides source file detection from stack traces

**`src-tauri/dist/cpu.js`**
- CPU window-specific UI logic
- Delegates Ollama functionality to `window.Ollama.*`
- Monitor history, collapsible sections, icon status management

#### Backend (Rust)

**`src-tauri/src/commands/ollama.rs`** - **Ollama Tauri Commands**
- **`ollama_chat_with_execution`**: Unified chat command that:
  - Gets system metrics for context
  - Sends question to Ollama with conversation history
  - Handles **FETCH_URL** tool: if the model replies with `FETCH_URL: <url>`, the app fetches the page and sends the content back to Ollama (no CORS; server-side fetch)
  - Handles **BROWSER_SCREENSHOT**: opens URL via CDP (browser_agent), saves PNG to ~/.mac-stats/screenshots/, returns path; from Discord the screenshot is attached to the channel
  - URL parsing for FETCH_URL and BROWSER_SCREENSHOT: first token only, trailing punctuation (`.`, `,`, `;`, `:`) stripped so model text like "...usecases. The..." does not produce a trailing dot on the URL
  - Detects JavaScript code in responses (`ROLE=code-assistant` or pattern matching)
  - Extracts code (handles `console.log()` wrappers)
  - Returns structured response indicating if code execution is needed
- **`answer_with_ollama_and_fetch`**: Used by Discord and scheduler; returns **OllamaReply** `{ text, attachment_paths }` so callers can attach screenshot paths (e.g. Discord sends the PNG).
- **`ollama_chat_continue_with_result`**: Follow-up command for code execution flow:
  - Takes JavaScript execution result
  - Sends follow-up message to Ollama with full conversation history
  - Handles ping-pong (recursive code execution up to 5 iterations)
  - Returns final answer or next code block
- **`check_ollama_connection`**: Connection status checking
- **`list_ollama_models`**: Model listing
- **`configure_ollama`**: Endpoint/model configuration

**`src-tauri/src/ollama/mod.rs`** - **Ollama API Client**
- HTTP client for Ollama API
- Chat request/response handling
- Model listing
- Connection checking
- API key management (Keychain integration)

**`src-tauri/src/commands/monitors.rs`** - **Monitor Management**
- Add/remove monitors
- Check monitor status
- Monitor persistence (disk storage)
- Monitor stats caching (last check, last status)

**`src-tauri/src/monitors/`** - **Monitor Implementations**
- **`website.rs`**: HTTP/HTTPS website monitoring (response time, status codes, SSL)
- **`social.rs`**: Social media monitoring (Mastodon mentions, etc.)
- **`mod.rs`**: Monitor trait definition and registry

**`src-tauri/src/alerts/`** - **Alert System**
- Rule-based alerting (SiteDown, BatteryLow, TemperatureHigh, etc.)
- Alert channels (Telegram, Slack, Mastodon, Signal)
- Cooldown mechanism to prevent spam

**`src-tauri/src/metrics/mod.rs`** - **System Metrics**
- CPU usage, temperature, frequency
- RAM, disk, GPU usage
- Battery status
- Process list
- Optimized with caching to minimize CPU overhead

**`src-tauri/src/ffi/`** - **Foreign Function Interface**
- **`ioreport.rs`**: IOReport API wrappers for CPU frequency (Apple Silicon)
- **`objc.rs`**: Objective-C wrappers for thermal state
- All `unsafe` code isolated here with safety comments

**`src-tauri/src/security/mod.rs`** - **Security**
- macOS Keychain integration
- Credential storage/retrieval
- API key management

**`src-tauri/src/lib.rs`** - **Main Application Logic**
- Tauri application setup
- Background threads for metrics collection
- Menu bar updates
- Window management
- Resource lifecycle management (IOReport, SMC)

### Data Flow Examples

#### Ollama Chat with Code Execution

1. **User sends message** → `src/ollama.js::sendChatMessage()`
2. **Add to history** → `addToHistory('user', message)`
3. **Invoke Rust** → `ollama_chat_with_execution` command
4. **Rust command** (`commands/ollama.rs`):
   - Gets system metrics
   - Builds messages array with conversation history
   - Calls Ollama API via `ollama/mod.rs`
   - Detects code in response
   - Returns structured response
5. **JavaScript execution** → `executeJavaScriptCode()` in `ollama.js`
6. **Follow-up** → `ollama_chat_continue_with_result` with execution result
7. **Final answer** → Displayed in UI, added to history

#### Monitor Check Flow

1. **Frontend** → `check_monitor(monitor_id)` Tauri command
2. **Rust** → `commands/monitors.rs::check_monitor()`
3. **Monitor lookup** → `monitors/mod.rs` registry
4. **Execute check** → `monitors/website.rs` or `monitors/social.rs`
5. **Update stats** → In-memory cache (last_check, last_status)
6. **Return result** → Frontend updates UI

### File Sync & Build Process

- **Frontend files** in `src/` are synced to `src-tauri/dist/` via `scripts/sync-dist.sh`
- **Single source of truth**: `src/ollama.js` is the primary file, synced to `dist/ollama.js`
- **Always sync after changes**: Run sync script or manually copy when modifying frontend files
- **Tauri serves from**: `src-tauri/dist/` directory

### Key Design Patterns

1. **Module Delegation**: Frontend modules delegate to specialized modules (e.g., `dashboard.js` → `ollama.js`)
2. **Single Source of Truth**: One primary file per feature, synced to distribution locations
3. **Conversation History**: Maintained in JavaScript, passed to Rust for API calls
4. **Structured Responses**: Rust commands return structured data (needs_code_execution, final_answer, etc.)
5. **Error Handling**: Comprehensive error handling with logging at each layer
6. **State Management**: In-memory state in Rust (monitors, alerts), localStorage in JavaScript (Ollama config)

### Adding New Features

**Frontend Feature**:
1. Add to `src/` directory
2. Update `scripts/sync-dist.sh` if needed
3. Include in HTML templates (`dashboard.html`, `cpu.html`)

**Backend Feature**:
1. Create module in `src-tauri/src/`
2. Add Tauri command in `src-tauri/src/commands/`
3. Register command in `src-tauri/src/lib.rs` → `tauri::generate_handler!`
4. Update `src-tauri/src/commands/mod.rs` exports

**Ollama Integration**:
- All Ollama code lives in `src/ollama.js` (frontend) and `src-tauri/src/commands/ollama.rs` + `src-tauri/src/ollama/mod.rs` (backend)
- Conversation history managed in JavaScript, passed to Rust
- Code execution handled in JavaScript, results sent back to Rust for follow-up

---

## Backend runtime and performance (summary)

High-level behavior that complements the directory tree above:

- **main.rs**: CLI entry (verbosity, `--cpu`, `--frequency`).
- **lib.rs**: Tauri setup, background metrics and menu bar; IOReport subscriptions and SMC. **Expensive metrics** (temperature, frequency) are emphasized when the **CPU window is visible** to limit idle cost. Careful CoreFoundation retain/release.
- **metrics/mod.rs**: Cached `SystemMetrics`, `CpuDetails`, process list (~30s cache), etc.
- **state.rs**: Shared state (`Mutex` / `OnceLock`): sysinfo, disks, status item, app handle, caches, IOReport handles.
- **ui/status_bar.rs**: Menu bar item; `build_status_text`; `create_cpu_window` on demand. **Click handler** applies pending menu bar updates (workaround for unreliable Tauri main-thread callbacks).
- **ffi/**: IOReport (frequency), Objective-C bridge; aligns with [FFI Safety Rules](#ffi-safety-rules-critical).
- **logging**: `tracing` with `debug1!` / `debug2!` / `debug3!` by verbosity; logs under `~/.mac-stats/debug.log`.

### Why CPU usage stays low

1. **Lazy** CPU window creation (not at startup).
2. **Selective reads**: temperature about every 20s when the window is visible; frequency about every 30s via IOReport; SMC/IOReport subscriptions cleared when the window closes.
3. **SMC**: M3 temperature keys cached after first discovery; prefer direct key reads over scanning all keys.
4. **Menu bar**: background thread prepares text; user click drains updates to the UI.
5. **Process list** cached ~30s.

### Key technical choices

- **IOReport lifecycle**: Subscribe when the CPU window opens (expensive); reuse for frequency reads; tear down on close (similar in spirit to exelban/stats, with correct CF retain/release).
- **M3/M4 temperature**: Try the standard path (M1/M2); fall back to M3-specific keys (e.g. Tf04, Tf09, Tf0A); cache the first working key.
- **Memory / threads**: Respect CF ownership; globals and thread-local patterns match Tauri and main-thread UI constraints; SMC may use thread-local where `Send` is an issue.

### Development notes

- Features grew iteratively; some code is pragmatic rather than textbook Rust.
- **Automated tests**: No large formal suite; still add parsing/unit tests where practical (see Output & Formatting).
- **Tauri**: Prefer the documented menu-bar click workaround over assuming all main-thread hooks behave the same everywhere.

### Testing and debugging (quick commands)

```bash
./target/release/mac_stats        # default -vv
./target/release/mac_stats -vvv
./target/release/mac_stats -v
./target/release/mac_stats --frequency
./target/release/mac_stats --cpu
tail -f ~/.mac-stats/debug.log
```

When debugging: **CPU window visibility** drives expensive reads; temperature may read 0 if SMC is unavailable or no M3 key was found; **frequency** needs an active IOReport subscription (window visible); menu bar text may update on **click**, not continuously.

### Version management

**Single source of truth**: `src-tauri/Cargo.toml` `version` only. Rust exposes `get_app_version()` via `env!("CARGO_PKG_VERSION")` / `Config`; the frontend loads it at runtime (e.g. CPU UI). HTML templates may contain placeholder version strings replaced by JavaScript—avoid hardcoding the version in many places.

---

## Reference — sibling repos (remember)

- **OpenClaw**: `../openclaw` (from repo root: multi-channel AI gateway; compaction, memory, session reset; see `docs/035_memory_and_topic_handling.md`).
- **Hermes**: `../hermes-agent` (from repo root: skills/workspace with SKILL.md files; not a chat gateway — the Hermes repo on disk is named **hermes-agent**, not `hermes`).
