# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
