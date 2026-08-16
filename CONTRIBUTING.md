# Contributing to mac-stats

Thanks for helping. mac-stats is an **Apple Silicon** menu-bar monitor (Rust + Tauri) with an optional local AI agent. Keep changes small, safe, and easy to review.

## Before you start

1. Read [README.md](README.md) and [AGENTS.md](AGENTS.md) (project conventions for humans and coding agents).
2. Use a machine with **macOS on Apple Silicon** for build and UI checks.
3. Prefer an issue or short design note for large features before a big PR.

## Development setup

```bash
git clone https://github.com/raro42/mac-stats.git
cd mac-stats
# Needs Rust toolchain + Xcode Command Line Tools
./run          # release-ish local run
# or
./run dev      # when you need hot reload where applicable
```

Useful checks:

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test --lib
./scripts/sync-dist.sh   # after editing shared frontend under src/
```

Frontend source of truth for shared JS/CSS is `src/` → sync into `src-tauri/dist/` before release. Theme shells live under `src-tauri/dist/themes/*/`.

## What to contribute

**Welcome**

- Bug fixes with a clear repro
- UI polish that matches existing themes (especially Dark / TUI)
- Docs, Homebrew, notarization help
- Tests for parsing / config / deterministic logic
- Performance work that keeps idle and window-open CPU low

**Ask first**

- New cloud dependencies or telemetry
- Broad refactors without a user-visible win
- Features that need root helpers or private Apple APIs beyond current patterns

**Signing / Gatekeeper**

Release DMGs are not notarized yet. Help with an Apple Developer ID + CI secrets is welcome — see [docs/NOTARIZATION.md](docs/NOTARIZATION.md).

## Pull requests

1. Branch from `main`.
2. Keep the PR focused (one concern when possible).
3. Update [CHANGELOG.md](CHANGELOG.md) under `## [Unreleased]` for user-visible changes (Keep a Changelog style).
4. Do **not** add `Co-authored-by`, `Signed-off-by`, or IDE/agent attribution to commit messages.
5. Run `cargo check` (and relevant tests) before you push.
6. Describe **why** in the PR body; link issues when they exist.

## Code style

- Prefer small modules; avoid new “god files”.
- Isolate `unsafe` / FFI and comment invariants.
- Match existing naming and UI patterns (glass / TUI themes, save-button flash helpers).
- Do not commit secrets (`.env`, `.config.env`, API keys, private keys).

## Reporting bugs

Include:

- mac-stats version (`get_app_version` / About / `Cargo.toml`)
- macOS version and chip (e.g. M3 Max)
- Steps to reproduce
- Relevant lines from `~/.mac-stats/debug.log` (redact tokens)

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
