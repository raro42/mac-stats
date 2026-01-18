# agents.md — Project Instructions for Cursor

## Goal
Build a polished macOS (Apple Silicon) stats app (Rust + Tauri) that reads CPU/process metrics and presents them in a clean "macOS glass" UI. Favor correctness, safety, and maintainability over hacks.

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
- Never install software on the host. Always ask before installing.
- If you need to install software, use containers.  
- No "god files": split into `metrics/`, `ffi/`, `ui/bridge/`, `cli/`, `logging/`.
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

## When Uncertain
- Ask for the exact macOS version and target (Dev vs notarized distribution).
- Default to the safest, Apple-supported approach.
