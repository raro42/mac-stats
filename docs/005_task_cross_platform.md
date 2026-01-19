# Cross-Platform Support Task

## Goal
Make the mac-stats application run on Linux and Windows in addition to macOS, while maintaining the same functionality and user experience across all platforms.

## Current State
The application is **macOS-only** and uses many platform-specific dependencies and APIs:

### macOS-Only Dependencies
- `objc2`, `objc2-foundation`, `objc2-app-kit` - Objective-C bindings (macOS/iOS only)
- `macsmc` - System Management Controller for temperature/power readings
- `core-foundation` - macOS Core Foundation framework
- `security-framework` - macOS Keychain integration
- IOReport FFI bindings - macOS private framework for CPU frequency

### Platform-Specific Code Locations

#### 1. Security Module (`src-tauri/src/security/mod.rs`)
- **Current**: Uses macOS Keychain via `security-framework`
- **Needs**: Platform-specific implementations:
  - **Linux**: `libsecret` or `secret-service` crate
  - **Windows**: Windows Credential Manager API
  - **macOS**: Keep existing Keychain implementation

#### 2. Metrics Module (`src-tauri/src/metrics/mod.rs`)
- **Current**: Uses `macsmc::Smc` for temperature, IOReport for frequency
- **Needs**: Platform-specific implementations:
  - **Linux**: 
    - Temperature: `/sys/class/thermal/thermal_zone*/temp`
    - CPU frequency: `/sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq`
    - Power: `/sys/class/powercap/` or `powertop` parsing
  - **Windows**: 
    - WMI (Windows Management Instrumentation) for metrics
    - Performance Counters API
    - `winapi` crate for system calls
  - **macOS**: Keep existing implementation

#### 3. FFI Module (`src-tauri/src/ffi/`)
- **Current**: IOReport bindings, Objective-C wrappers
- **Needs**: Platform-specific FFI or alternative approaches:
  - **Linux**: Direct sysfs/procfs reading (no FFI needed)
  - **Windows**: WMI COM interfaces or WinAPI calls
  - **macOS**: Keep existing FFI

#### 4. UI Module (`src-tauri/src/ui/status_bar.rs`)
- **Current**: Uses `objc2-app-kit` for macOS menu bar
- **Needs**: Platform-specific UI implementations:
  - **Linux**: `tray-item` or `systray` crate
  - **Windows**: Windows System Tray API
  - **macOS**: Keep existing AppKit implementation

#### 5. Commands Module (`src-tauri/src/commands/ollama.rs`)
- **Current**: Uses `crate::security::get_credential()` and `crate::metrics::*`
- **Needs**: Already uses abstractions, but depends on platform-specific modules
- **Action**: Ensure security and metrics modules are properly abstracted

## Implementation Strategy

### Phase 1: Platform Abstraction Layer

#### 1.1 Create Platform Traits
Create trait-based abstractions for platform-specific functionality:

```rust
// src-tauri/src/platform/mod.rs
pub trait CredentialStore {
    fn store_credential(account: &str, password: &str) -> Result<()>;
    fn get_credential(account: &str) -> Result<Option<String>>;
    fn delete_credential(account: &str) -> Result<()>;
}

pub trait SystemMetrics {
    fn get_cpu_details() -> CpuDetails;
    fn get_metrics() -> SystemMetrics;
    fn get_temperature() -> Result<f32>;
    fn get_cpu_frequency() -> Result<f32>;
    fn get_power_consumption() -> Result<(f32, f32)>; // (CPU, GPU)
}

pub trait SystemTray {
    fn create_status_item() -> Result<StatusItem>;
    fn update_menu(menu: Menu) -> Result<()>;
}
```

#### 1.2 Platform-Specific Implementations
Create platform modules:
- `src-tauri/src/platform/macos.rs`
- `src-tauri/src/platform/linux.rs`
- `src-tauri/src/platform/windows.rs`

Use conditional compilation:
```rust
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;
```

### Phase 2: Dependency Management

#### 2.1 Update Cargo.toml
Make platform-specific dependencies conditional:

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6.3"
objc2-foundation = { version = "0.3.2", features = [...] }
objc2-app-kit = { version = "0.3.2", features = [...] }
macsmc = "0.1"
security-framework = "2.9"

[target.'cfg(target_os = "linux")'.dependencies]
libsecret = "0.2"
systray = "0.3"

[target.'cfg(target_os = "windows")'.dependencies]
winapi = { version = "0.3", features = [...] }
windows = { version = "0.52", features = [...] }
```

#### 2.2 Cross-Platform Dependencies
Keep these for all platforms:
- `sysinfo` - Already cross-platform
- `battery` - Already cross-platform
- `reqwest`, `tokio`, `serde` - Already cross-platform
- `tauri` - Already cross-platform

### Phase 3: Platform-Specific Implementations

#### 3.1 Linux Implementation

**Credentials (Linux)**:
- Use `libsecret` or `secret-service` crate
- Store in GNOME Keyring or KDE Wallet
- Fallback to encrypted file if keyring unavailable

**Metrics (Linux)**:
- Temperature: Read from `/sys/class/thermal/thermal_zone*/temp`
- CPU frequency: Read from `/sys/devices/system/cpu/cpu*/cpufreq/scaling_cur_freq`
- Power: Parse `powertop` output or read from `/sys/class/powercap/`
- Use `sysinfo` for CPU/RAM/Disk (already cross-platform)

**System Tray (Linux)**:
- Use `tray-item` or `systray` crate
- Support both X11 and Wayland (via DBus)

#### 3.2 Windows Implementation

**Credentials (Windows)**:
- Use Windows Credential Manager API
- Use `winapi` crate with `wincred` features
- Store in Windows Credential Store

**Metrics (Windows)**:
- Use WMI (Windows Management Instrumentation) for temperature
- Use Performance Counters for CPU frequency
- Use `winapi` for power consumption (if available)
- Use `sysinfo` for CPU/RAM/Disk (already cross-platform)

**System Tray (Windows)**:
- Use Windows System Tray API
- Use `winapi` crate with `shellapi` features

### Phase 4: Testing Strategy

#### 4.1 Platform-Specific Tests
- Unit tests for each platform implementation
- Integration tests for credential storage
- Metrics collection tests on each platform

#### 4.2 CI/CD Updates
Update `.github/workflows/release.yml`:
- Add Linux build job
- Add Windows build job
- Test on all three platforms

### Phase 5: UI/UX Considerations

#### 5.1 Platform-Specific UI
- **macOS**: Keep existing "macOS glass" aesthetic
- **Linux**: Adapt to GTK/KDE themes
- **Windows**: Adapt to Windows 11 Fluent Design

#### 5.2 Feature Parity
Ensure all features work on all platforms:
- ✅ CPU/RAM/Disk monitoring (via `sysinfo`)
- ✅ Process list and details (via `sysinfo`)
- ⚠️ Temperature (platform-specific)
- ⚠️ CPU frequency (platform-specific)
- ⚠️ Power consumption (platform-specific)
- ⚠️ Battery monitoring (via `battery` crate - already cross-platform)
- ⚠️ System tray (platform-specific)

## Migration Checklist

### Backend
- [ ] Create platform abstraction traits
- [ ] Implement macOS platform module (refactor existing code)
- [ ] Implement Linux platform module
- [ ] Implement Windows platform module
- [ ] Update security module to use platform abstraction
- [ ] Update metrics module to use platform abstraction
- [ ] Update UI module to use platform abstraction
- [ ] Update Cargo.toml with conditional dependencies
- [ ] Remove direct platform-specific imports from command modules

### Frontend
- [ ] Test UI on all platforms
- [ ] Update documentation for cross-platform support
- [ ] Adapt UI themes for each platform (optional)

### Testing
- [ ] Add platform-specific unit tests
- [ ] Add integration tests for each platform
- [ ] Test credential storage on all platforms
- [ ] Test metrics collection on all platforms
- [ ] Test system tray on all platforms

### CI/CD
- [ ] Add Linux build to GitHub Actions
- [ ] Add Windows build to GitHub Actions
- [ ] Test builds on all platforms

### Documentation
- [ ] Update README with platform support
- [ ] Document platform-specific features/limitations
- [ ] Add build instructions for each platform

## Known Limitations

### Platform-Specific Features
Some features may not be available on all platforms:

1. **Temperature Monitoring**:
   - macOS: ✅ Full support via SMC
   - Linux: ⚠️ Depends on hardware sensors (may not be available on all systems)
   - Windows: ⚠️ Requires WMI support (may not be available on all systems)

2. **CPU Frequency**:
   - macOS: ✅ Full support via IOReport
   - Linux: ✅ Full support via sysfs
   - Windows: ⚠️ Limited support (may require admin privileges)

3. **Power Consumption**:
   - macOS: ✅ Full support via SMC
   - Linux: ⚠️ Limited support (requires specific hardware)
   - Windows: ⚠️ Limited support (may not be available)

4. **System Tray**:
   - macOS: ✅ Full support via AppKit
   - Linux: ⚠️ Depends on desktop environment (X11/Wayland)
   - Windows: ✅ Full support via System Tray API

## Priority Order

1. **High Priority**: Core metrics (CPU, RAM, Disk) - already cross-platform via `sysinfo`
2. **Medium Priority**: Credential storage - needed for Ollama integration
3. **Medium Priority**: System tray - core UI feature
4. **Low Priority**: Temperature, frequency, power - nice-to-have features

## Notes

- The `ollama.rs` commands module should work cross-platform once the security and metrics modules are abstracted
- Most of the Tauri frontend code is already cross-platform (HTML/CSS/JS)
- The main work is in the Rust backend platform abstractions
- Consider using existing cross-platform crates where possible (e.g., `keyring` crate for credentials)
