# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Full Ollama API coverage**: List models with details, get version, list running models, pull/update/delete models, generate embeddings, load/unload models from memory.
  - Tauri commands: `list_ollama_models_full`, `get_ollama_version`, `list_ollama_running_models`, `pull_ollama_model`, `delete_ollama_model`, `ollama_embeddings`, `unload_ollama_model`, `load_ollama_model`. All use the configured Ollama endpoint (same as chat/Discord/scheduler).
  - Backend: `ollama/mod.rs` types and `OllamaClient` methods for GET /api/tags (full), GET /api/version, GET /api/ps, POST /api/pull, DELETE /api/delete, POST /api/embed, and load/unload via keep_alive on generate/chat.
  - Documentation: `docs/015_ollama_api.md`.
- **User info (user-info.json)**: Per-user details from `~/.mac-stats/user-info.json` (keyed by Discord user id) are merged into the agent context (display_name, notes, timezone, extra). See `docs/007_discord_agent.md`.

### Changed
- **Agent status messages**: When the agent uses a skill or the Ollama API, the status line now shows details: "Using skill &lt;number&gt;-&lt;topic&gt;…" (e.g. "Using skill 3-create-rule…") and "Ollama API: &lt;action&gt; [args]…" (e.g. "Ollama API: list_models…", "Ollama API: pull llama3.2…").

## [0.1.9] - 2026-02-09

### Added
- **Discord API agent**: When a request comes from Discord, Ollama can call the Discord HTTP API via the DISCORD_API tool (e.g. list guilds, channels, members, get user). Endpoint list is documented in `docs/007_discord_agent.md` and injected into the agent context. Only GET and POST to `/channels/{id}/messages` are allowed.
- **Discord user names**: The bot records the message author's display name and passes it to Ollama so it can address the user by name; names are cached for reuse in the session.
- **MCP Agent (Model Context Protocol)**: Ollama can use tools from any MCP server
  - Configure via `MCP_SERVER_URL` (HTTP/SSE) or `MCP_SERVER_STDIO` (e.g. `npx|-y|@openbnb/mcp-server-airbnb`) in env or `~/.mac-stats/.config.env`
  - When configured, the app fetches the tool list and adds it to the agent descriptions; Ollama invokes tools by replying `MCP: <tool_name> <arguments>`
  - Supported in Discord bot, scheduler, and CPU window chat (same tool loop)
  - Documentation: `docs/010_mcp_agent.md`
- **Task agent**: Task files under `~/.mac-stats/task/` with TASK_APPEND, TASK_STATUS, TASK_CREATE. Scheduler supports `TASK: <path or id>` / `TASK_RUN: <path or id>` to run a task loop until status is `finished`; optional `reply_to_channel_id` sends start and result to Discord. Documentation: `docs/013_task_agent.md`.
- **PYTHON_SCRIPT agent**: Ollama can create and run Python scripts; scripts are written to `~/.mac-stats/scripts/` and executed with `python3`. Disable with `ALLOW_PYTHON_SCRIPT=0`. Documentation: `docs/014_python_agent.md`.

## [0.1.8] - 2026-02-08

### Added
- **Ollama context window and model/params**: Per-model context size via `POST /api/show`, cached; Discord can override model (`model: llama3.2`), temperature and num_ctx (`temperature: 0.7`, `num_ctx: 8192` or `params: ...`). Config supports optional default temperature/num_ctx. See `docs/012_ollama_context_skills.md`.
- **Context-aware FETCH_URL**: When fetched page content would exceed the model context, the app summarizes it via one Ollama call or truncates with a note. Uses heuristic token estimate (chars/4) and reserved space for the reply.
- **Skills**: Markdown files in `~/.mac-stats/skills/skill-<number>-<topic>.md` can be selected in Discord with `skill: 2` or `skill: code`; content is prepended to the system prompt so different “agents” respond differently.
- **Ollama agent at startup**: The app configures and checks the default Ollama endpoint at startup so the agent is available for Discord, scheduler, and CPU window without opening the CPU window first.

### Changed
- **Discord agent**: Reply uses full Ollama + tools pipeline (planning + execution). Message prefixes for model, temperature, num_ctx, and skill documented in `docs/007_discord_agent.md` and `docs/012_ollama_context_skills.md`.

## [0.1.7] - 2026-02-06

### Added
- **Discord Agent (Gateway)**: Optional Discord bot that connects via the Gateway and responds to DMs and @mentions
  - Bot token stored in macOS Keychain (account `discord_bot_token`); never logged or exposed
  - Listens for direct messages and guild messages that mention the bot; ignores own messages
  - Requires Message Content Intent enabled in Discord Developer Portal
  - Tauri commands: `configure_discord(token?)` to set/clear token, `is_discord_configured()` to check
  - Reply is currently a stub; Ollama/browser pipeline to be wired in a follow-up
  - Documentation: `docs/007_discord_agent.md`

## [0.1.6] - 2026-01-22

### Fixed
- **DMG Asset Bundling**: Fixed missing assets (Ollama icon, JavaScript/Tauri icons) in DMG builds
  - Added explicit `resources` configuration in `tauri.conf.json` to bundle `dist/assets/` files
  - Assets are now properly included in production DMG builds
- **Ollama Icon Path**: Fixed Ollama icon not displaying in DMG builds
  - Changed icon paths from relative (`../../assets/ollama.svg`) to absolute (`/assets/ollama.svg`) in all theme HTML files
  - Icons now resolve correctly in Tauri production builds
- **History Chart Initialization**: Fixed history charts not drawing in DMG builds
  - Moved canvas element lookup and context initialization to `initializeCanvases()` function
  - Added defensive initialization in `updateCharts()` to handle delayed DOM loading
  - Charts now properly initialize in production builds

### Added
- **DMG Testing Script**: Added `scripts/test-dmg.sh` for automated DMG verification before release
  - Mounts DMG and verifies app structure
  - Checks for required assets and theme files
  - Provides installation and testing instructions
  - Validates bundle correctness before distribution

### Changed
- **Test Script Path Detection**: Updated test script to check correct asset path (`dist/assets/` instead of `assets/`)

## [0.1.5] - 2026-01-22

### Changed
- **Release**: Version bump for release build

## [0.1.4] - 2026-01-22

### Added
- **Welcome Message**: Added a friendly welcome message that displays when the application starts and the menu bar is ready
  - Always visible in console (not dependent on verbosity flags)
  - Includes app version, warm greetings, and encouragement to share on GitHub and Mastodon
  - Encourages community contributions and feedback

## [0.1.3] - 2026-01-19

### Added
- **CLI Parameter Support**: Added support for passing CLI arguments through the `run` script
  - `./run --help` or `./run -h` to show help
  - `./run --openwindow` flag to optionally open CPU window at startup
  - All CLI flags (`-v`, `-vv`, `-vvv`, `--cpu`, `--frequency`, `--power-usage`, `--changelog`) now work through the `run` script
  - Development mode (`./run dev`) also passes arguments to `cargo run`

### Fixed
- **Window Opening at Startup**: Fixed issue where CPU window was automatically opening at startup
  - Removed manual `sendAction` test code that was triggering the click handler during setup
  - All windows are now properly hidden at startup (menu bar only mode)
  - Window only opens when explicitly requested via `--cpu` or `--openwindow` flags or when menu bar is clicked
- **Compilation Warnings**: Suppressed dead code warnings for utility methods
  - Added `#[allow(dead_code)]` to `total_points()`, `estimate_memory_bytes()`, `save_to_disk()`, and `load_from_disk()` methods
  - These methods are reserved for future use or used in tests
- **Power Consumption Flickering**: Fixed power consumption values flickering to 0.0W when background thread updates cache
  - Added `LAST_SUCCESSFUL_POWER` fallback cache to prevent flickering when main lock is unavailable
  - Power values now persist across lock contention scenarios
  - Improved power cache update logic to always maintain last successful reading
- **Power Display Precision**: Fixed power values < 1W showing as "0 W" causing visual flicker
  - Changed from `Math.round()` to `.toFixed(1)` to show 1 decimal place (e.g., "0.3 W" instead of "0 W")
  - Applied to both CPU and GPU power displays
  - Total power calculation now uses cached values to prevent flickering
- **Ollama Logging Safety**: Enhanced JavaScript execution logging with comprehensive sanitization
  - Added `sanitizeForLogging()` function to prevent dangerous characters from breaking logs
  - Safe logging wrapper that never throws errors, ensuring logging failures don't break execution flow
  - Truncates long strings, removes control characters, and sanitizes quotes/backticks
  - Prevents log injection and system breakage from malformed execution results

### Changed
- **Startup Behavior**: App now starts in menu bar only mode by default
  - No windows are visible at startup
  - CPU window is created on-demand when menu bar is clicked
  - Improved startup logging to indicate menu bar only mode
- **History Chart Styling**: Improved visual design of history chart container
  - Enhanced glass effect with backdrop blur and subtle shadows
  - Removed border, added inset highlights for depth
  - Better visual consistency with macOS glass aesthetic
- **Power Capability Detection**: Improved `can_read_cpu_power()` and `can_read_gpu_power()` functions
  - Now checks power cache existence as fallback when capability flags aren't set yet
  - Handles edge cases where power reading works but flags haven't been initialized
- **Development Logging**: Added verbose logging (`-vvv`) to release build script for easier debugging

### Technical
- **State Management**: Added `LAST_SUCCESSFUL_POWER` static state for power reading fallback
- **Error Handling**: Enhanced error handling in power consumption reading with graceful fallbacks
- **Logging Infrastructure**: Improved Ollama JavaScript execution logging with sanitization and error isolation

## [0.1.2] - 2026-01-19

### Added
- **Universal Collapsible Sections**: Replicated Apple theme's USAGE card click behavior across all themes
  - Clicking the USAGE card toggles both Details and Processes sections
  - Clicking section headers individually hides respective sections
  - Sections are hidden by default (collapsed state)
  - State persists in localStorage across sessions
  - Added universal IDs (`cpu-usage-card`, `details-section`, `processes-section`, `details-header`, `processes-header`) to all themes
  - Added clickable cursor and hover effects for better UX

### Fixed
- **Ollama Icon Visibility**: Fixed Ollama icon not being visible/green in themes other than Apple
  - Added default gray filter and opacity to all themes for icon visibility
  - Fixed green status filter to properly override default styling using `!important`
  - Icon now correctly displays green when Ollama is connected, yellow/amber when unavailable
  - Applied fixes to all 9 themes (apple, dark, architect, data-poster, futuristic, light, material, neon, swiss-minimalistic)
- **Data-Poster Theme Layout**: Fixed battery/power strip layout alignment with Apple theme
  - Removed unwanted grey background box around "Power:" label
  - Fixed battery icon color for dark theme visibility
  - Added missing `--hairline` CSS variable
  - Aligned spacing, padding, and styling to match Apple theme exactly
  - Fixed charging indicator to display green when charging

## [0.1.1] - 2026-01-19

### Fixed
- **Monitor Stats Persistence**: Fixed issue where external monitor stats (last_check, last_status) were not persisting after host reboot
  - Monitor stats are now saved to disk after each check
  - Stats are restored when monitors are loaded on app startup
  - Added `get_monitor_status()` command to retrieve cached stats without performing a new check
  - Stats persist across reboots in the monitors configuration file

## [0.1.0] - 2026-01-19

### Added
- **Monitoring System**: Comprehensive website and social media monitoring
  - Website uptime monitoring with response time tracking
  - Social media platform monitoring (Twitter/X, Facebook, Instagram, LinkedIn, YouTube)
  - Monitor status indicators (up/down) with response time display
  - Configurable monitor intervals and timeout settings
  - Monitor health summary with up/down counts
- **Alert System**: Multi-channel alerting infrastructure
  - Alert rules engine for monitor status changes
  - Alert channel support (prepared for future integrations)
  - Alert history and management
- **Ollama AI Chat Integration**: AI-powered chat assistant
  - Integration with local Ollama instance
  - Chat interface for system metrics queries
  - Model selection and connection status indicators
  - System prompt customization
  - Code execution support for JavaScript
  - Markdown rendering with syntax highlighting
- **Status Icon Line**: Quick access icon bar with status indicators
  - Monitors icon with green status when all monitors are up
  - Ollama icon with green status when connected, yellow when unavailable
  - 15-icon layout with placeholders for future features
  - Click-to-toggle section visibility
- **Dashboard UI**: New dashboard view for monitoring overview
  - Centralized monitoring status display
  - Quick access to all monitoring features
- **Security Infrastructure**: Keychain integration for secure credential storage
  - API key storage in macOS Keychain
  - Secure credential management for monitors and services
- **Plugin System**: Extensible plugin architecture
  - Plugin loading and management infrastructure
  - Prepared for future plugin integrations

### Changed
- **UI Layout**: Added collapsible sections for Monitors and AI Chat
  - Sections can be toggled via header clicks or icon clicks
  - Smooth expand/collapse animations
  - State persistence across sessions
- **Icon Styling**: Enhanced icon display with status-based color coding
  - Green for healthy/connected status
  - Yellow/amber for warnings/unavailable status
  - CSS filters for external SVG icons
- **Connection Status**: Real-time connection status updates
  - Visual indicators for Ollama connection state
  - Automatic connection checking on section expansion

### Technical
- **Backend Commands**: New Tauri commands for monitoring and Ollama
  - `list_monitors`, `add_monitor`, `remove_monitor`, `check_monitor`
  - `check_ollama_connection`, `ollama_chat`, `configure_ollama`
  - `list_alerts`, `add_alert_rule`, `remove_alert_rule`
- **State Management**: Enhanced application state with monitoring and Ollama state
- **Error Handling**: Comprehensive error handling for network requests and API calls
- **Logging**: Structured logging for monitoring and Ollama operations
- **Cross-Theme Support**: All new features (monitoring, Ollama chat, status icons) are available across all 9 themes
- **CSS Architecture**: Universal CSS with cascading variable fallbacks for cross-theme compatibility

## [0.0.6] - 2026-01-18

### Added
- **Power Consumption Monitoring**: Real-time CPU and GPU power consumption monitoring via IOReport Energy Model API
  - CPU power consumption in watts (W)
  - GPU power consumption in watts (W)
  - Power readings only when CPU window is visible (optimized for low CPU usage)
  - `--power-usage` command-line flag for detailed power debugging logs
- **Battery Monitoring**: Battery level and charging status display
  - Battery percentage display
  - Charging status indicator
  - Battery information only read when CPU window is visible
- **Process Details Modal**: Click any process in the list to view comprehensive details including:
  - Process name, PID, and current CPU usage
  - Total CPU time, parent process information
  - Start time with relative time display
  - User and effective user information
  - Memory usage (physical and virtual)
  - Disk I/O statistics (read/written)
- **Force Quit Functionality**: Force quit processes directly from the process details modal
- **Process List Interactivity**: Process rows are now clickable and show cursor pointer
- **Auto-refresh Process Details**: Process details modal automatically refreshes every 2 seconds while open
- **Scrollable Sections**: Added scrollable containers for Details and Processes sections with custom scrollbar styling
- **Process PID Display**: Process list now includes PID information in data attributes
- **Embedded Changelog**: Changelog is now embedded in the binary for reliable access
- **Changelog CLI Flag**: Added `--changelog` flag to test changelog functionality

### Changed
- **Process List Refresh**: Increased refresh interval from 5 seconds to 15 seconds for better CPU efficiency
- **Process Cache**: Improved process cache handling with immediate refresh on window open
- **UI Layout**: Improved flex layout with proper min-height and overflow handling for scrollable sections
- **Process Data Structure**: Added PID field to ProcessUsage struct for better process identification
- **Changelog Reading**: Improved changelog reading with multiple fallback strategies (current directory, executable directory, embedded)

### Performance
- **Smart Process Refresh**: Process details only refresh when CPU window is visible (saves CPU when window is hidden)
- **Conditional Process Updates**: Process list updates immediately on initial load and when window becomes visible
- **Efficient Modal Updates**: Process details modal only refreshes when actually visible
- **Power Reading Optimization**: Power consumption and battery readings only occur when CPU window is visible, maintaining <0.1% CPU usage when window is closed
- **IOReport Power Subscription**: Power subscription is created on-demand and cleaned up when window closes

### Technical
- **IOReport Power Integration**: Implemented IOReport Energy Model API integration for power monitoring
- **Array Channel Support**: Added support for IOReportChannels as arrays (Energy Model format)
- **Memory Management**: Proper CoreFoundation memory management for power channel dictionaries
- **Error Handling**: Graceful handling when power channels are not available on certain Mac models

## [0.0.5] - 2026-01-18

### Performance Improvements
- **Access Flags Optimization**: Replaced `Mutex<Option<_>>` with `OnceLock<bool>` for capability flags (temperature, frequency, power reading) - eliminates locking overhead on every access
- **Process Cache TTL**: Increased process list cache from 5 seconds to 10 seconds to reduce CPU overhead
- **Temperature Update Interval**: Increased from 15 seconds to 20 seconds for better efficiency
- **Frequency Read Interval**: Increased from 20 seconds to 30 seconds to reduce IOReport overhead
- **DOM Update Optimization**: Changed from `innerHTML` rebuilds to direct text node updates for metric values (reduces WebKit rendering overhead)
- **Ring Gauge Thresholds**: Increased update thresholds from 2% to 5% (visual) and 15% to 20% (animation) to reduce unnecessary animations
- **Window Cleanup**: Added cleanup handlers on window unload to clear animation state and pending updates

### Fixed
- **GitHub Actions Workflow**: Fixed workflow to properly handle missing code signing secrets (builds successfully without secrets)
- **Code Signing**: Made code signing optional - workflow builds unsigned DMG when secrets are not available
- **Legacy Code**: Removed outdated ACCESS_CACHE comment references

### Changed
- **Theme Gallery**: Updated README with comprehensive theme gallery showing all 9 themes
- **Screenshot Organization**: Removed old screenshot folders (screen_actual, screen-what-i-see), consolidated to screens/ folder

## [0.0.4] - 2026-01-18

### Added
- **App Version Display**: Added version number display in footer of all HTML templates
- **Version API**: Added `get_app_version` Tauri command to fetch version at runtime
- **Window Decorations Toggle**: Added window frame toggle in settings (affects new windows)
- **Config File Support**: Added persistent configuration file (`~/.mac-stats/config.json`) for window decorations preference
- **Toggle Switch Component**: Added modern toggle switch styling to all themes
- **GitHub Actions Workflow**: Automated DMG build and release on GitHub
- **Build Script**: Added `scripts/build-dmg.sh` for local DMG creation
- **DMG Download Section**: Added download instructions to README with Gatekeeper bypass steps

### Changed
- **Theme Improvements**: Massively improved all themes with better styling and visual consistency
- **Data Poster Theme**: Improved Details section styling to match Processes section (flex layout, consistent font sizes and weights)
- **Metric Unit Styling**: Improved metric unit display (%, GHz) with better font sizing and positioning
- **CPU Usage Display**: Fixed CPU usage value updates to properly maintain HTML structure with unit spans
- **Frequency Display**: Enhanced frequency display to include unit (GHz) with proper formatting
- **HTTPS Support**: Changed git clone URLs from SSH to HTTPS for better accessibility
- **Window Creation**: CPU window now respects window decorations preference from config file

### Fixed
- **Build Configuration**: Fixed Tauri build configuration (custom-protocol feature, bundle settings)
- **Binary Naming**: Fixed binary name from `mac-stats-backend` to `mac_stats` to match package name
- **DMG Detection**: Fixed build-dmg.sh script to properly detect DMG files using zsh array expansion
- **Release Workflow**: Fixed GitHub Actions workflow to properly upload DMG files to releases
- **Version Fetching**: Fixed duplicate command definition by moving `get_app_version` to metrics module

### Documentation
- **README Updates**: Added comprehensive DMG download instructions with Gatekeeper bypass methods
- **Known Limitations**: Added note about window frame toggle behavior (affects new windows only)
- **Installation Guide**: Improved installation section with multiple options and troubleshooting

## [0.0.3] - 2026-01-18

### Added
- **DMG Build Support**: Added DMG bundle creation for macOS distribution
- **GitHub Actions**: Added automated release workflow for building and publishing DMG files

### Changed
- **Version**: Bumped to 0.0.3

## [0.0.2] - 2026-01-18

### Fixed
- **CPU Frequency Reading**: Fixed frequency reading from IOReport to use delta samples instead of absolute counters, providing accurate recent frequency values instead of long-term averages
- **Memory Leaks**: Fixed CoreFoundation object leaks by properly retaining and releasing CF objects (channels_dict, subscription_dict, samples)
- **Crash Safety**: Added validation for IOReport channel dictionaries before calling IOReport functions to prevent crashes from invalid data
- **Channel Filtering**: Made `is_performance_channel()` more restrictive to only match actual CPU performance channels (ECPU*, PCPU*), reducing unnecessary processing

### Changed
- **Delta Sampling**: Frequency calculation now uses `IOReportCreateSamplesDelta()` to compute recent frequency over the sampling interval (20s) instead of since boot
- **Channel Classification**: Improved channel classification to correctly identify E-core (ECPU*) and P-core (PCPU*) channels
- **Frequency Extraction**: Enhanced frequency extraction to handle VxPy voltage/performance state format (e.g., V0P5, V19P0)
- **Command Execution**: Replaced fragile `sh -c` commands with direct binary calls using full paths (`/usr/sbin/sysctl`, `/usr/sbin/system_profiler`)
- **Code Organization**: Removed large redundant comment blocks from refactoring

### Refactored
- **Frequency Reading Logic**: Extracted complex nested frequency reading code from `lib.rs` into modular functions in `ffi/ioreport.rs`, reducing nesting from 5+ levels to max 2-3 levels
- **Array Processing**: Added support for IOReportChannels as an array (type_id=19) in addition to dictionary format
- **Logging**: Refactored `debug1/2/3` macros to use `write_structured_log` with timestamps for consistent logging format

### Added
- **Frequency Logging**: Added `--frequency` command-line flag for detailed frequency debugging logs
- **Validation**: Added validation checks for IOReport channel dictionaries (channel name, state count) before processing
- **Memory Management**: Added proper CFRetain/CFRelease cycles for all stored CoreFoundation objects
- **Cleanup**: Added cleanup path to release all CF objects when CPU window closes

### Security
- **FFI Safety**: Improved FFI safety by validating CoreFoundation types and null pointers before use
- **Memory Safety**: Fixed use-after-free risks by properly managing CF object lifetimes with guards

## [0.1.0] - Initial Release

### Added
- Basic system monitoring (CPU, RAM, Disk, GPU)
- Temperature monitoring via SMC
- CPU frequency monitoring via IOReport
- Process list with top CPU consumers
- Menu bar integration
- Multiple UI themes
- Low CPU usage optimizations
