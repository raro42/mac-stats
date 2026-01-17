//! UI module for AppKit/StatusBar interface
//! 
//! This module handles all UI-related code including:
//! - Status bar menu item
//! - CPU window creation
//! - About panel
//! - Objective-C class registration for click handlers

pub mod status_bar;

// Note: lib.rs imports directly from status_bar, so re-exports are not needed.
// If other modules want to import from ui:: instead of ui::status_bar::,
// uncomment the re-exports below.
// pub use status_bar::{
//     setup_status_item,
//     make_attributed_title,
//     create_cpu_window,
//     build_status_text,
// };
