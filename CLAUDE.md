# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚠️ Important: Read agents.md First

**Before making any changes or answering questions about this codebase, always read `agents.md` first.** 

The `agents.md` file contains:
- Project goals and coding principles
- Complete codebase structure and organization
- Module responsibilities and data flow
- Design patterns and conventions
- Where code lives and what it does

This ensures you understand the project context, architecture, and conventions before making changes.

## Project Overview

**mac-stats** is a lightweight system monitor for macOS built with Rust and Tauri. It displays real-time CPU, GPU, RAM, and disk usage in the menu bar with minimal CPU overhead (~0.5% idle, <1% with CPU window open). The app features:
- Real-time system metrics (CPU, RAM, Disk, GPU)
- Temperature readings (SMC integration)
- CPU frequency monitoring (IOReport native API)
- Process list with top CPU consumers
- Customizable UI themes
- Low CPU usage optimizations
- Ollama AI chat with code execution and **FETCH_URL** (navigate to URL, fetch page content server-side, no CORS)

## Build and Run Commands

```bash
# Development mode (hot reload)
./run dev
# or
cd src-tauri && cargo run

# Release build
./run
# or
cd src-tauri && cargo build --release

# Run with CPU window open (for testing)
./target/release/mac_stats --cpu

# Enable verbose logging (-v, -vv, -vvv for verbosity levels)
./target/release/mac_stats -vvv

# Enable detailed frequency logging
./target/release/mac_stats --frequency
```

## Architecture Overview

The codebase is organized into a Tauri + Rust backend with the following structure:

### Core Modules (src-tauri/src/)

1. **main.rs**: Entry point with CLI argument parsing (verbosity, `--cpu`, `--frequency` flags)

2. **lib.rs**: Main application logic (run(), run_with_cpu_window()) and setup:
   - Initializes Tauri builder with Tauri commands (get_cpu_details)
   - Sets up background threads for metrics collection and menu bar updates
   - Manages IOReport subscriptions and SMC connections (thread-local for efficiency)
   - **Key insight**: CPU optimization through selective resource usage - only reads expensive metrics (temperature, frequency) when CPU window is visible
   - Complex state management with careful attention to CoreFoundation memory management (CFRetain/CFRelease)

3. **metrics/mod.rs**: System metrics collection
   - `SystemMetrics`: CPU, GPU, RAM, disk percentages (fast cache-based metrics from sysinfo)
   - `CpuDetails`: Comprehensive data including temperature, frequency, power consumption, load averages, top processes
   - Functions: `get_metrics()`, `get_cpu_details()`, `get_chip_info()`, `get_nominal_frequency()`
   - All metrics are cached to reduce system load
   - Process list cached for 30 seconds

4. **state.rs**: Global application state using thread-safe primitives (Mutex, OnceLock)
   - System state: SYSTEM (sysinfo), DISKS
   - UI state: STATUS_ITEM (NSStatusItem), APP_HANDLE, MENU_BAR_TEXT
   - Caches: temperature, frequency (P-core/E-core), process list, chip info
   - IOReport state: subscription handles, channels dictionaries, last samples
   - Note: Global state design necessary due to Tauri architecture and main-thread-only UI requirements

5. **ui/status_bar.rs**: macOS menu bar integration
   - `setup_status_item()`: Creates NSStatusItem and click handler
   - `build_status_text()`: Formats metrics for display
   - `make_attributed_title()`: Creates formatted NSAttributedString for menu bar (handles font sizes, baselines, tabs)
   - `create_cpu_window()`: Builds CPU details window on demand (saves CPU when not visible)
   - Click handler processes pending menu bar updates (works around Tauri's main-thread callback limitations)

6. **ffi/**: Safe FFI wrappers for unsafe system calls
   - **ioreport.rs**: IOReport native API for real-time CPU frequency reading
   - **objc.rs**: Objective-C bridge (if needed)
   - All wrappers include null checks, error handling, and proper memory management

7. **logging/mod.rs**: Structured logging using tracing framework
   - Debug macros: `debug1!()`, `debug2!()`, `debug3!()` for verbosity levels
   - Logs written to `~/.mac-stats/debug.log`
   - Legacy logging compatibility layer

8. **config/mod.rs**: Centralized configuration
   - Log file paths: `$HOME/.mac-stats/debug.log`
   - Build information (version, date, authors)
   - Portable paths across systems

### Frontend

- **src-tauri/dist/**: Pre-built UI files (main.js, cpu.js, cpu-ui.js)
- **src-tauri/dist/themes/**: CSS theme files (Apple, Material, Architect, Data Poster, Swiss Minimalistic, Neon)
- Windows built on demand via WindowBuilder to minimize CPU usage

## Critical Performance Optimizations

The app achieves low CPU usage through several key strategies:

1. **Lazy Initialization**: CPU window created on-demand (not at startup)
2. **Selective Resource Usage**:
   - Temperature read every 15 seconds (only when window visible)
   - Frequency read every 20 seconds (via expensive IOReport API)
   - SMC and IOReport subscriptions cleared when window closes
3. **Efficient SMC Caching**:
   - M3 temperature keys cached after first discovery (avoids all_data() iteration)
   - Direct key reading used instead of iterating all SMC keys
4. **Menu Bar Updates**: Every 1-2 seconds via background thread, updates processed on click (works around Tauri main-thread limitation)
5. **Process List Caching**: 30-second cache to avoid expensive process enumeration

## Key Technical Decisions

### IOReport Subscription Lifecycle
- Created once when CPU window becomes visible (expensive operation)
- Kept alive and reused for frequency reads
- Cleared when window closes to save CPU
- Implementation follows exelban/stats approach with proper CFRetain/CFRelease

### M3/M4 Temperature Reading
- Standard `cpu_temperature()` attempted first (works for M1/M2)
- Falls back to M3-specific temperature keys (Tf04, Tf09, Tf0A, etc.) for compatibility
- First discovered key cached in M3_TEMP_KEY to avoid re-iteration

### Memory Management
- CoreFoundation objects properly retained/released (critical to prevent crashes)
- Global statics used for cross-thread access patterns
- Thread-local storage for main-thread-only UI elements

### Thread Safety
- Background thread for metrics collection runs independently
- Menu bar text updates stored in MENU_BAR_TEXT, processed on click
- SMC connection kept in thread-local to avoid Send requirement

## Coding Principles (from agents.md)

**OBEY THESE RULES AT ALL TIMES**

### Code Organization
- Keep modules small and cohesive
- No "god files": code is split into `metrics/`, `ffi/`, `ui/`, `config/`, `logging/`
- When making changes, create logs and read them afterwards
- Execute the app yourself after changes to verify they work

### FFI Safety (CRITICAL)
- Never allow Obj-C/C++ exceptions to cross into Rust
- No panicking across FFI boundaries
- Always validate CoreFoundation types before casting (use `CFGetTypeID()`)
- Always check for null pointers from CF APIs
- Follow CF ownership rules: "Create/Copy = you release"; "Get = do not release"
- Avoid `transmute` for IOReport payloads; use `read_unaligned` if needed
- Every `unsafe` block must have a comment explaining invariants and safety assumptions

### macOS Constraints (Apple Silicon)
- Do NOT claim `sysctl hw.cpufrequency` exists on Apple Silicon — it doesn't
- CPU frequency and sensors require careful API selection
- Prefer Apple-supported indicators: CPU usage (libproc), thermal state (NSProcessInfo)
- For frequency/temperature: use IOReport (private API) with proper error handling
- Treat IOReport output as unstable; handle failures gracefully

### Root/Privilege Handling
- UI must run unprivileged
- Privileged functionality must be isolated (separate binary, launchd, or explicit sudo)
- Communicate over Unix domain socket or localhost with auth
- Make privilege-required metrics optional; app must work without them

### Logging & Diagnostics
- Use `tracing` for structured logs
- Provide `--verbose` / `--frequency` flags for CLI diagnostics
- Always execute code after changes and read the logs
- Document crashes with LLDB recipes

### Error Handling
- Prefer `anyhow` + `thiserror` for error handling
- Use `serde` for structured data exchanged with UI
- Use `rustfmt` and `clippy` clean builds

## Development Notes

- **Vibe coding philosophy**: Features built iteratively based on what felt right; prioritize shipping over perfect design
- **Rust learner codebase**: May not follow all Rust best practices, but is functional and efficient
- **Tauri limitations**: run_on_main_thread callbacks unreliable; click handler workaround used instead
- **No test suite**: This is a simple monitoring app without formal tests
- **Git status**: Currently on main branch with changes to agents.md, Cargo files, CSS, and tauri.conf.json (feat/theming branch)

## Testing and Debugging

Enable detailed logging:
```bash
# Set verbosity (0-3)
./target/release/mac_stats -vvv  # Maximum verbosity
./target/release/mac_stats -v    # Minimal verbosity

# Enable frequency logging for debugging IOReport
./target/release/mac_stats --frequency

# Open CPU window directly
./target/release/mac_stats --cpu

# View debug logs
tail -f ~/.mac-stats/debug.log
```

When debugging:
- Check if CPU window is visible (affects expensive metric reads)
- Temperature reads may return 0.0 if SMC unavailable or if M3 key not discovered
- Frequency reads require IOReport subscription (only created when window visible)
- Menu bar updates via click handler, not automatic (background thread limitation)

## Version Management

**Single Source of Truth**: Version is stored in `src-tauri/Cargo.toml` only. No manual updates needed elsewhere.

The version number flows through the app as follows:
1. **Cargo.toml** defines `version = "0.0.3"`
2. **Rust backend** (`lib.rs::get_app_version()`) reads via `Config::version()` (which uses `env!("CARGO_PKG_VERSION")`)
3. **Tauri command** exposes `get_app_version()` to the frontend
4. **JavaScript** (`cpu-ui.js::injectAppVersion()`) calls the Tauri command at runtime
5. **All HTML footers** display the dynamic version (never hardcoded)

**Benefits**:
- Automatic version updates on every build
- No need to manually update 8+ HTML template files
- Consistent version across all themes
- Frontend always shows accurate version

**HTML templates** still contain placeholder version elements (e.g., `<span class="arch-version">Architect v0.0.3</span>`), but these are replaced by JavaScript at runtime with the actual version.
