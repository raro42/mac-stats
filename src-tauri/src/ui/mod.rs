//! UI module
//!
//! Contains UI-related functionality including status bar and window management.

#[cfg(target_os = "macos")]
pub mod status_bar;
#[cfg(not(target_os = "macos"))]
#[path = "status_bar_linux.rs"]
pub mod status_bar;
