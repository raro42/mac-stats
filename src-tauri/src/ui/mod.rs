//! UI module for AppKit/StatusBar interface
//! 
//! This module handles all UI-related code including:
//! - Status bar menu item
//! - CPU window creation
//! - About panel
//! - Objective-C class registration for click handlers

pub mod status_bar;

// Re-export public functions (used in lib.rs)
pub use status_bar::{
    setup_status_item,
    process_menu_bar_update,
    make_attributed_title,
    create_cpu_window,
};
