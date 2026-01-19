# Implementation Notes - Monitoring App v0.1.0

## Implementation Status

### Completed Backend Modules

1. **Security Module** (`src/security/mod.rs`)
   - Keychain integration for secure credential storage
   - Store, retrieve, delete credentials
   - List credentials (partial - needs proper Keychain API for account extraction)
   - Uses `security-framework` crate with proper error handling

2. **Monitors Module** (`src/monitors/`)
   - Website monitoring (HTTP/HTTPS checks, response times, SSL verification)
   - Social monitoring (Mastodon mentions, Twitter placeholder)
   - Monitor status tracking and history

3. **Alerts Module** (`src/alerts/`)
   - Rule-based alert system
   - Alert rules: SiteDown, NewMentions, BatteryLow, TemperatureHigh, CpuHigh, Custom
   - Alert channels: Telegram, Slack, Mastodon, Signal (placeholder)
   - Cooldown mechanism to prevent spam

4. **Plugins Module** (`src/plugins/mod.rs`)
   - Script-based plugin system (bash/python)
   - JSON output contract
   - Scheduling and execution
   - Plugin validation

5. **Ollama Module** (`src/ollama/mod.rs`)
   - Local LLM integration
   - Chat interface
   - Model listing
   - Connection checking

6. **Tauri Commands** (`src/commands/`)
   - All backend functionality exposed as Tauri commands
   - Proper error handling and serialization

### Known Issues / TODOs

1. **Security Module**
   - `list_credentials()` function needs proper Keychain API implementation to extract account names
   - Currently returns empty vector - needs implementation using Keychain Services API

2. **Plugin System**
   - Timeout handling not fully implemented (std::process::Command doesn't have timeout)
   - Should use tokio or crossbeam for proper timeout handling
   - Plugin script execution could be improved with better error messages

3. **Monitor System**
   - Monitor state persistence not implemented (in-memory only)
   - Should persist to disk (JSON/TOML config file)
   - Monitor scheduling/background checking not implemented yet

4. **Alert System**
   - Alert channel registration not exposed via Tauri commands
   - Need to add commands for registering Telegram/Slack/Mastodon channels
   - Alert evaluation needs to be called periodically (background task)

5. **Ollama Integration**
   - Stream support not implemented (currently only non-streaming chat)
   - Could add streaming for better UX

6. **UI Implementation**
   - Frontend UI not yet updated to show new dashboard
   - Need to create:
     - 3 core gauges (Temperature, CPU Usage, CPU Frequency)
     - Battery & Power status strip
     - Collapsible External/Monitors section
     - Settings UI for adding monitors, configuring alerts
     - Ollama chat interface

### Architecture Decisions

1. **State Management**: Using `OnceLock` for global state (monitors, alerts, plugins, ollama)
   - In production, should use proper state management (e.g., Tauri state, database)
   - Current implementation is in-memory only

2. **Error Handling**: Using `anyhow` for error handling throughout
   - Tauri commands return `Result<T, String>` for serialization
   - Internal functions use `anyhow::Result`

3. **Security**: All credentials stored in macOS Keychain
   - Never exposed in logs, UI, or files
   - Proper error handling for missing credentials

4. **Plugin Contract**: JSON-based output
   - Simple and language-agnostic
   - Easy to validate and parse
   - Low barrier for developers

### Build Process Note

**Important**: Tauri serves files from the `dist/` directory, but source files are in `src/`. 

To sync files for development, run:
```bash
./scripts/sync-dist.sh
```

Or manually copy files:
```bash
mkdir -p dist
cp src/*.html src/*.css src/*.js dist/
cp -r src/assets dist/
```

The `dist/` directory is gitignored as it's a build output.

### Next Steps

1. **UI Implementation** (Priority) ✅ COMPLETE
   - ✅ Update frontend HTML/CSS/JS to show new dashboard
   - ✅ Implement 3 core gauges with SVG
   - ✅ Add battery/power status strip
   - ✅ Create collapsible sections for external monitors
   - ⏳ Add settings UI (basic placeholder implemented)

2. **Background Tasks**
   - Implement monitor checking scheduler
   - Implement alert evaluation scheduler
   - Implement plugin execution scheduler

3. **Persistence**
   - Save monitor configurations to disk
   - Save alert configurations to disk
   - Save plugin configurations to disk
   - Load on startup

4. **Testing**
   - Test all Tauri commands
   - Test Keychain operations
   - Test monitor checking
   - Test alert evaluation
   - Test plugin execution

### Questions / Findings

1. **Keychain API**: The `list_credentials()` function needs more research on proper Keychain Services API usage to extract account names from items.

2. **Plugin Timeout**: Need to decide on approach for plugin timeout - tokio spawn with timeout, or use a different approach.

3. **Monitor Scheduling**: Should monitors be checked in a single background thread, or separate threads per monitor? Current design allows for both.

4. **Alert Channel Registration**: Should channels be registered via Tauri commands, or configured in a config file? Currently designed for programmatic registration.

5. **UI Framework**: Current UI is vanilla HTML/CSS/JS. Should we consider a framework (React, Vue, Svelte) for better state management?

### Dependencies Added

- `reqwest` - HTTP client for website monitoring and API calls
- `tokio` - Async runtime (for future async operations)
- `security-framework` / `security-framework-sys` - macOS Keychain integration
- `chrono` - Date/time handling
- `url` - URL parsing and validation
- `anyhow` - Error handling
- `toml` - TOML parsing (for plugin configs)

### Version

- Updated to 0.1.0 (major milestone)
- Branch: `feature/monitoring-app-0.1.0`
