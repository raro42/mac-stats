# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
