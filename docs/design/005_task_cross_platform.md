# mac-stats Cross-Platform Support

## Overview
Enable macOS, Linux, and Windows support while maintaining core functionality (CPU/RAM monitoring, Ollama integration, system tray).

## Current Limitations
- macOS-only dependencies: `objc2`, `macsmc`, `security-framework`, IOReport
- Platform-specific code in:
  - Security module (Keychain)
  - Metrics module (SMC, IOReport)
  - UI module (AppKit)
  - FFI module (Objective-C)

## Implementation Strategy

### Phase 1: Platform Abstraction
```rust
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
    fn get_power_consumption() -> Result<(f32, f32)>;
}

pub trait SystemTray {
    fn create_status_item() -> Result<StatusItem>;
    fn update_menu(menu: Menu) -> Result<()>;
}
```

### Phase 2: Conditional Dependencies
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

### Phase 3: Platform Implementations

#### Linux
- **Credentials**: `libsecret`/`secret-service` (GNOME Keyring/KDE Wallet)
- **Metrics**: 
  - Temperature: `/sys/class/thermal/`
  - CPU freq: `/sys/devices/system/cpu/`
  - Power: `/sys/class/powercap/` or `powertop`
- **Tray**: `tray-item`/`systray` (X11/Wayland)

#### Windows
- **Credentials**: Windows Credential Manager API
- **Metrics**: WMI + Performance Counters
- **Tray**: Windows System Tray API

#### macOS
- **Credentials**: Keychain
- **Metrics**: SMC + IOReport
- **Tray**: AppKit

## Migration Checklist

### Backend
- [ ] Create platform traits
- [ ] Implement macOS module (refactor existing code)
- [ ] Implement Linux module
- [ ] Implement Windows module
- [ ] Update security/metrics modules
- [ ] Update UI module
- [ ] Update Cargo.toml dependencies

### Frontend
- [ ] Test UI on all platforms
- [ ] Update documentation for cross-platform support

### Testing
- [ ] Add platform-specific unit tests
- [ ] Add integration tests
- [ ] Test credential storage
- [ ] Test metrics collection
- [ ] Test system tray

### CI/CD
- [ ] Add Linux/Windows build jobs

## Known Limitations

| Feature               | macOS | Linux | Windows |
|----------------------|-------|-------|---------|
| Temperature          | ✅    | ⚠️    | ⚠️      |
| CPU Frequency        | ✅    | ✅    | ⚠️      |
| Power Consumption    | ✅    | ⚠️    | ⚠️      |
| System Tray          | ✅    | ⚠️    | ✅       |

## Open tasks:
- [ ] Verify Linux temperature sensor availability
- [ ] Implement Windows WMI metrics
- [ ] Add Wayland support for Linux tray
- [ ] Document platform-specific credential storage
- [ ] Test power metrics on all platforms
- [ ] Update README with cross-platform build instructions
- [ ] Create platform-specific UI themes