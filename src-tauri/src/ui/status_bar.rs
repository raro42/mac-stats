//! Status bar UI implementation
//! 
//! Handles the macOS menu bar status item, click handlers, and window management.

use std::ffi::CStr;
use std::sync::OnceLock;
use objc2::declare::ClassBuilder;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, NSObject, Sel};
use objc2::{msg_send, ClassType, MainThreadMarker, sel};
use objc2_app_kit::{
    NSAboutPanelOptionApplicationName, NSAboutPanelOptionApplicationVersion,
    NSAboutPanelOptionCredits, NSAboutPanelOptionVersion, NSApplication, NSColor, NSFont,
    NSFontWeightRegular, NSFontWeightSemibold, NSBaselineOffsetAttributeName,
    NSFontAttributeName, NSForegroundColorAttributeName, NSParagraphStyleAttributeName,
    NSMutableParagraphStyle, NSStatusBar,
    NSVariableStatusItemLength, NSTextAlignment, NSTextTab, NSTextTabOptionKey, NSEvent,
};
use objc2_foundation::{
    NSArray, NSDictionary, NSMutableAttributedString, NSMutableDictionary, NSNumber,
    NSAttributedString, NSRange, NSString,
};
use tauri::{Manager, WindowBuilder, WindowUrl};

use crate::state::*;
use crate::logging::write_structured_log;
use crate::config::Config;
use crate::metrics::SystemMetrics;

// Import debug macros
#[allow(unused_imports)]
use crate::{debug1, debug2, debug3};

/// Helper function to convert any object to AnyObject reference
fn as_any<T: objc2::Message>(obj: &T) -> &AnyObject {
    unsafe { &*(obj as *const T as *const AnyObject) }
}

/// Build status text from metrics
pub fn build_status_text(metrics: &SystemMetrics) -> String {
    let label_line = "CPU\tGPU\tRAM\tSSD".to_string();
    let value_line = format!(
        "{:.0}%\t{:.0}%\t{:.0}%\t{:.0}%",
        metrics.cpu.round() as i32,
        metrics.gpu.round() as i32,
        metrics.ram.round() as i32,
        metrics.disk.round() as i32
    );
    format!("{label_line}\n{value_line}")
}

/// Process menu bar update (must be called from main thread)
pub fn process_menu_bar_update() {
    // This function must be called from the main thread
    if let Some(mtm) = MainThreadMarker::new() {
        let update_text = {
            if let Ok(mut pending) = MENU_BAR_TEXT.try_lock() {
                pending.take()
            } else {
                write_structured_log("ui/status_bar.rs", "MENU_BAR_TEXT lock failed", &serde_json::json!({}), "G");
                return;
            }
        };
        
        if let Some(text) = update_text {
            debug3!("Processing menu bar update: '{}'", text);
            let attributed = make_attributed_title(&text);
            STATUS_ITEM.with(|cell| {
                if let Some(item) = cell.borrow().as_ref() {
                    if let Some(button) = item.button(mtm) {
                        button.setAttributedTitle(&attributed);
                        debug3!("Menu bar text updated successfully");
                    } else {
                        write_structured_log("ui/status_bar.rs", "Button not found", &serde_json::json!({}), "G");
                    }
                }
            });
        }
    } else {
        write_structured_log("ui/status_bar.rs", "MainThreadMarker::new() FAILED", &serde_json::json!({}), "G");
    }
}

/// Create attributed title string for status bar
pub fn make_attributed_title(text: &str) -> Retained<NSMutableAttributedString> {
    let ns_text = NSString::from_str(text);
    let attributed = NSMutableAttributedString::from_nsstring(&ns_text);
    let length = ns_text.length();
    let full_range = NSRange { location: 0, length };

    let label_len = text
        .split('\n')
        .next()
        .unwrap_or("")
        .encode_utf16()
        .count();
    let value_len = text
        .split('\n')
        .nth(1)
        .unwrap_or("")
        .encode_utf16()
        .count();
    let label_range = NSRange {
        location: 0,
        length: label_len,
    };
    let value_range = NSRange {
        location: label_len + 1,
        length: value_len,
    };

    let label_font = NSFont::monospacedSystemFontOfSize_weight(8.5, unsafe {
        NSFontWeightRegular
    });
    let value_font = NSFont::monospacedSystemFontOfSize_weight(12.5, unsafe {
        NSFontWeightSemibold
    });
    // Use controlTextColor for menu bar - this works better than labelColor in status bar context
    // labelColor can sometimes turn black in menu bar, so use controlTextColor which adapts properly
    let color = NSColor::controlTextColor();
    let paragraph = NSMutableParagraphStyle::new();
    paragraph.setLineSpacing(-2.0);
    paragraph.setLineHeightMultiple(0.75);
    paragraph.setAlignment(NSTextAlignment::Left);
    paragraph.setDefaultTabInterval(38.0);

    let options: Retained<NSDictionary<NSTextTabOptionKey, AnyObject>> = NSDictionary::new();
    let tab1: Retained<NSTextTab> = unsafe {
        let tab: *mut NSTextTab = msg_send![NSTextTab::class(), alloc];
        let tab: *mut NSTextTab = msg_send![tab, initWithTextAlignment: NSTextAlignment::Left, location: 38.0f64, options: &*options];
        Retained::from_raw(tab).unwrap()
    };
    let tab2: Retained<NSTextTab> = unsafe {
        let tab: *mut NSTextTab = msg_send![NSTextTab::class(), alloc];
        let tab: *mut NSTextTab = msg_send![tab, initWithTextAlignment: NSTextAlignment::Left, location: 76.0f64, options: &*options];
        Retained::from_raw(tab).unwrap()
    };
    let tab3: Retained<NSTextTab> = unsafe {
        let tab: *mut NSTextTab = msg_send![NSTextTab::class(), alloc];
        let tab: *mut NSTextTab = msg_send![tab, initWithTextAlignment: NSTextAlignment::Left, location: 114.0f64, options: &*options];
        Retained::from_raw(tab).unwrap()
    };
    let tab4: Retained<NSTextTab> = unsafe {
        let tab: *mut NSTextTab = msg_send![NSTextTab::class(), alloc];
        let tab: *mut NSTextTab = msg_send![tab, initWithTextAlignment: NSTextAlignment::Left, location: 152.0f64, options: &*options];
        Retained::from_raw(tab).unwrap()
    };
    let tabs = NSArray::from_slice(&[&*tab1, &*tab2, &*tab3, &*tab4]);
    paragraph.setTabStops(Some(&tabs));
    let baseline_offset = NSNumber::new_f64(-4.8);

    unsafe {
        attributed.addAttribute_value_range(&NSFontAttributeName, as_any(&*label_font), label_range);
        attributed.addAttribute_value_range(&NSFontAttributeName, as_any(&*value_font), value_range);
        attributed.addAttribute_value_range(
            &NSForegroundColorAttributeName,
            as_any(&*color),
            full_range,
        );
        attributed.addAttribute_value_range(
            &NSParagraphStyleAttributeName,
            as_any(&*paragraph),
            full_range,
        );
        attributed.addAttribute_value_range(
            &NSBaselineOffsetAttributeName,
            as_any(&*baseline_offset),
            full_range,
        );
    }

    attributed
}

/// Setup the status bar menu item
pub fn setup_status_item() {
    let mtm = MainThreadMarker::new().unwrap();
    let status_bar = NSStatusBar::systemStatusBar();
    let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

    let handler_class = click_handler_class();
    debug2!("Creating handler instance from class");
    write_structured_log("ui/status_bar.rs", "About to create handler instance", &serde_json::json!({"class": format!("{:?}", handler_class)}), "J");
    
    // Verify class responds to selector before creating instance
    let action_sel = sel!(onStatusItemClick:);
    let selector_name = action_sel.name().to_string_lossy();
    let class_responds = unsafe {
        let responds: bool = msg_send![handler_class, instancesRespondToSelector: action_sel];
        responds
    };
    debug1!("Handler class responds to selector '{}': {}", selector_name, class_responds);
    write_structured_log("ui/status_bar.rs", "Class selector check", &serde_json::json!({"selector": selector_name, "responds": class_responds}), "J");
    
    let handler: Retained<AnyObject> =
        unsafe { Retained::from_raw(msg_send![handler_class, new]) }
            .expect("click handler");
    debug3!("Handler instance created: {:?}", handler);
    write_structured_log("ui/status_bar.rs", "Handler instance created", &serde_json::json!({"handler": format!("{:p}", &*handler)}), "J");
    
    // Verify instance responds to selector
    let instance_responds = unsafe {
        let responds: bool = msg_send![&*handler, respondsToSelector: action_sel];
        responds
    };
    debug1!("Handler instance responds to selector: {}", instance_responds);
    write_structured_log("ui/status_bar.rs", "Instance selector check", &serde_json::json!({"responds": instance_responds}), "J");
    
    if !instance_responds {
        debug1!("ERROR: Handler instance does NOT respond to selector!");
        write_structured_log("ui/status_bar.rs", "ERROR: Instance does not respond to selector", &serde_json::json!({}), "J");
    }
    
    // CRITICAL: Store handler in thread-local FIRST to keep it alive
    // The button will also retain it when we set it as target, but we keep our own reference
    CLICK_HANDLER.with(|cell| {
        *cell.borrow_mut() = Some(handler.clone());
        debug3!("Handler stored in CLICK_HANDLER thread-local (retained)");
        write_structured_log("ui/status_bar.rs", "Handler stored in CLICK_HANDLER", &serde_json::json!({}), "J");
    });

    // CRITICAL: Do NOT set a menu on the status item if we want button action to work
    // Setting a menu disables the button's action/target behavior
    // Instead, use the button's action directly and handle events properly
    let action = sel!(onStatusItemClick:);
    
    if let Some(button) = status_item.button(mtm) {
        debug2!("Setting up button target and action (NO menu set)...");
        write_structured_log("ui/status_bar.rs", "Setting button target/action (no menu)", &serde_json::json!({"handler": format!("{:p}", &*handler), "action": action.name()}), "J");
        unsafe {
            // Set target and action on the button
            button.setTarget(Some(&*handler));
            button.setAction(Some(action));
            button.setEnabled(true);
            
            // CRITICAL: Use sendActionOn to specify which events trigger the action
            // This is required for NSStatusBarButton to work properly
            // sendActionOn returns the previous mask, we want left mouse up events
            // NSEventMask is a bitmask - use LeftMouseUpMask
            use objc2_app_kit::NSEventMask;
            let event_mask = NSEventMask::LeftMouseUp;
            let _previous_mask = button.sendActionOn(event_mask);
            
            write_structured_log("ui/status_bar.rs", "Button target/action and sendAction set", &serde_json::json!({}), "J");
            debug3!("Button target, action, and sendAction set");
            
            // Verify setup
            if let Some(target) = button.target() {
                debug3!("Button target verified: {:?}", target);
                write_structured_log("ui/status_bar.rs", "Button target verified", &serde_json::json!({"target": format!("{:p}", target)}), "J");
                
                // CRITICAL: Verify target responds to the action selector
                let target_responds = {
                    let responds: bool = msg_send![&*target, respondsToSelector: action];
                    responds
                };
                let selector_name = action.name().to_string_lossy();
                debug1!("Button target responds to action selector '{}': {}", selector_name, target_responds);
                write_structured_log("ui/status_bar.rs", "Target respondsToSelector check", &serde_json::json!({"responds": target_responds, "selector": selector_name}), "J");
                
                if !target_responds {
                    debug1!("ERROR: Button target does NOT respond to action selector!");
                    write_structured_log("ui/status_bar.rs", "ERROR: Target does not respond to selector", &serde_json::json!({}), "J");
                }
            }
            if let Some(set_action) = button.action() {
                debug3!("Button action verified: {:?}", set_action.name());
                write_structured_log("ui/status_bar.rs", "Button action verified", &serde_json::json!({"action": set_action.name()}), "J");
            }
            
            // CRITICAL: Check if button is enabled
            let is_enabled = button.isEnabled();
            debug1!("Button isEnabled: {}", is_enabled);
            write_structured_log("ui/status_bar.rs", "Button enabled check", &serde_json::json!({"enabled": is_enabled}), "J");
            
            // CRITICAL: Try manually sending the action to verify it works
            if let Some(target) = button.target() {
                debug1!("Attempting to manually send action to verify it works...");
                write_structured_log("ui/status_bar.rs", "Manual action send attempt", &serde_json::json!({}), "J");
                let action_sent = {
                    use objc2_app_kit::NSApplication;
                    let app = NSApplication::sharedApplication(mtm);
                    let sent: bool = msg_send![&*app, sendAction: action, to: &*target, from: &*button];
                    sent
                };
                debug1!("Manual sendAction result: {}", action_sent);
                write_structured_log("ui/status_bar.rs", "Manual sendAction result", &serde_json::json!({"sent": action_sent}), "J");
            }
        }
        debug2!("Button target and action set (no menu)");
    } else {
        debug1!("ERROR: Could not get button from status item!");
        write_structured_log("ui/status_bar.rs", "ERROR: Button not found", &serde_json::json!({}), "J");
    }
    
    // Handler is already stored in CLICK_HANDLER above, so it's retained
    // The button should also retain it via setTarget, so we have double retention
    debug3!("Handler retention: stored in CLICK_HANDLER and set as button target");

    STATUS_ITEM.with(|cell| {
        *cell.borrow_mut() = Some(status_item);
    });
    debug2!("Status item setup complete");
    
    // Start automatic menu bar updates by scheduling the first update
    // The handler will reschedule itself every 2 seconds
    if let Some(handler) = CLICK_HANDLER.with(|cell| cell.borrow().clone()) {
        let update_sel = sel!(processMenuBarUpdate:);
        unsafe {
            // Schedule first update after 2 seconds
            let _: () = msg_send![&*handler, performSelector: update_sel, withObject: std::ptr::null_mut::<AnyObject>(), afterDelay: 2.0];
            debug1!("Scheduled automatic menu bar updates (first update in 2 seconds)");
            write_structured_log("ui/status_bar.rs", "Automatic updates scheduled", &serde_json::json!({}), "M");
        }
    } else {
        debug1!("WARNING: Could not get handler for automatic updates");
    }
}

/// Get or create the Objective-C click handler class
pub fn click_handler_class() -> &'static AnyClass {
    static REGISTER: OnceLock<&'static AnyClass> = OnceLock::new();
    REGISTER.get_or_init(|| {
        let name = unsafe { CStr::from_bytes_with_nul_unchecked(b"MacStatsStatusHandler\0") };
        debug2!("Creating Objective-C class: {:?}", name);
        let mut builder = ClassBuilder::new(name, NSObject::class()).expect("class already exists");
        
        // Add method to process menu bar updates (called automatically every 2 seconds)
        extern "C-unwind" fn process_menu_bar_update_timer(
            this: &AnyObject,
            _cmd: Sel,
            _sender: *mut AnyObject,
        ) {
            // This is called from Objective-C runtime, we're on the main thread
            process_menu_bar_update();
            
            // Schedule next update in 2 seconds
            let sel = sel!(processMenuBarUpdate:);
            unsafe {
                let _: () = msg_send![this, performSelector: sel, withObject: std::ptr::null_mut::<AnyObject>(), afterDelay: 2.0];
            }
        }
        
        extern "C-unwind" fn on_status_item_click(
            this: &AnyObject,
            _cmd: Sel,
            sender: *mut AnyObject,
        ) {
            // This is called from Objective-C runtime, we're on the main thread
            // CRITICAL: Log immediately to verify this function is called
            write_structured_log("ui/status_bar.rs", "Click handler FUNCTION CALLED", &serde_json::json!({"this": format!("{:p}", this), "sender": format!("{:p}", sender)}), "J");
            debug1!("Click handler called! cmd={:?}, sender={:p}, this={:p}", _cmd, sender, this);
            
            // Note: The menu will show briefly, but that's okay - the action fires
            // We could hide it immediately, but for now let's just let it work
            
            // Process any pending menu bar updates while we're on the main thread
            process_menu_bar_update();
            
            // Get event info immediately while we're on the main thread
            let mtm = match MainThreadMarker::new() {
                Some(mtm) => mtm,
                None => {
                    debug1!("ERROR: Could not get MainThreadMarker!");
                    return;
                }
            };
            
            let app = NSApplication::sharedApplication(mtm);
            let is_right_click = app
                .currentEvent()
                .map(|event: Retained<NSEvent>| {
                    let button_number = event.buttonNumber();
                    debug3!("Event button number: {}", button_number);
                    button_number == 1
                })
                .unwrap_or(false);
            debug2!("Is right click: {}", is_right_click);
            
            if is_right_click {
                debug1!("Showing about panel");
                show_about_panel();
            } else {
                debug1!("Left click - toggling CPU window");
                write_structured_log("ui/status_bar.rs", "Click handler: about to toggle window", &serde_json::json!({}), "I");
                // We're already on main thread, so we can access the window directly
                if let Some(app_handle) = APP_HANDLE.get() {
                    write_structured_log("ui/status_bar.rs", "APP_HANDLE found", &serde_json::json!({}), "I");
                    
                    // Check if window exists and is visible
                    if let Some(window) = app_handle.get_window("cpu") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        write_structured_log("ui/status_bar.rs", "CPU window found", &serde_json::json!({"is_visible": is_visible}), "I");
                        
                        if is_visible {
                            // Window is visible - close it completely to save CPU
                            debug1!("CPU window is visible, closing it completely...");
                            let _ = window.close();
                            write_structured_log("ui/status_bar.rs", "Window closed completely", &serde_json::json!({}), "I");
                        } else {
                            // Window exists but is hidden - close and recreate to ensure decorations are up-to-date
                            // This ensures that if the user changed the decorations setting, it will be applied
                            debug1!("CPU window exists but is hidden, closing and recreating to apply decorations...");
                            let _ = window.close();
                            write_structured_log("ui/status_bar.rs", "Window closed for recreation", &serde_json::json!({}), "I");
                            // Fall through to create new window
                        }
                    } else {
                        // Window doesn't exist - create and show it
                        debug1!("CPU window doesn't exist, creating it...");
                        write_structured_log("ui/status_bar.rs", "Creating new CPU window", &serde_json::json!({}), "I");
                        create_cpu_window(app_handle);
                    }
                    
                    // If we closed a window above, create a new one now
                    if app_handle.get_window("cpu").is_none() {
                        debug1!("Creating CPU window after close...");
                        create_cpu_window(app_handle);
                    }
                } else {
                    write_structured_log("ui/status_bar.rs", "APP_HANDLE not available", &serde_json::json!({}), "I");
                    debug1!("APP_HANDLE not available!");
                }
            }
        }
        unsafe {
            let action_sel = sel!(onStatusItemClick:);
            debug2!("Adding method: {:?}", action_sel.name());
            builder.add_method(
                action_sel,
                on_status_item_click as extern "C-unwind" fn(_, _, _),
            );
            
            let update_sel = sel!(processMenuBarUpdate:);
            debug2!("Adding method: {:?}", update_sel.name());
            builder.add_method(
                update_sel,
                process_menu_bar_update_timer as extern "C-unwind" fn(_, _, _),
            );
        }
        let registered_class = builder.register();
        debug2!("Objective-C class registered: {:?}", registered_class);
        
        // CRITICAL: Verify the class responds to the selector
        let action_sel = sel!(onStatusItemClick:);
        let selector_name = action_sel.name().to_string_lossy();
        let responds = unsafe {
            let responds: bool = msg_send![registered_class, instancesRespondToSelector: action_sel];
            responds
        };
        debug1!("Class responds to selector '{}': {}", selector_name, responds);
        write_structured_log("ui/status_bar.rs", "Class selector verification", &serde_json::json!({"selector": selector_name, "responds": responds}), "J");
        
        if !responds {
            debug1!("ERROR: Class does NOT respond to selector! Method registration may have failed!");
            write_structured_log("ui/status_bar.rs", "ERROR: Class does not respond to selector", &serde_json::json!({}), "J");
        }
        
        registered_class
    })
}

/// Show the about panel
pub fn show_about_panel() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    
    // Use a nicer application name
    let name = NSString::from_str("mac-stats");
    let version = NSString::from_str(&Config::version());
    let build = NSString::from_str(&Config::build_date());
    
    // Create a nicely formatted credits text with better styling
    let credits_text = format!(
        "A lightweight system monitor for macOS\n\n\
        Built with Rust and Tauri\n\
        Inspired by Stats by exelban\n\n\
        Version {}\n\
        Build: {}\n\n\
        © 2026",
        Config::version(),
        Config::build_date()
    );
    let credits = NSAttributedString::from_nsstring(&NSString::from_str(&credits_text));

    let keys = unsafe {
        [
            &*NSAboutPanelOptionApplicationName,
            &*NSAboutPanelOptionApplicationVersion,
            &*NSAboutPanelOptionVersion,
            &*NSAboutPanelOptionCredits,
        ]
    };
    let values: [&AnyObject; 4] = [
        as_any(&*name),
        as_any(&*version),
        as_any(&*build),
        as_any(&*credits),
    ];
    let options = NSMutableDictionary::from_slices(&keys, &values);

    unsafe {
        app.orderFrontStandardAboutPanelWithOptions(options.as_ref());
    }
}

/// Create the CPU details window
pub fn create_cpu_window(app_handle: &tauri::AppHandle) {
    debug1!("Creating CPU window...");
    write_structured_log("ui/status_bar.rs", "create_cpu_window ENTRY", &serde_json::json!({}), "I");
    
    // Read window decorations preference from config file
    // This allows the preference to be changed without recompiling
    use crate::config::Config;
    let decorations = Config::get_window_decorations();
    debug1!("Window decorations preference: {} (from config file)", decorations);
    
    let cpu_window = WindowBuilder::new(
        app_handle,
        "cpu",
        WindowUrl::App("cpu.html".into()),
    )
    .title("CPU")
    .visible(true)  // Show immediately when created
    .inner_size(644.0, 995.0)
    .resizable(true)
    .always_on_top(true)
    .decorations(decorations)
    .build();
    
    match cpu_window {
        Ok(window) => {
            debug1!("CPU window created successfully");
            write_structured_log("ui/status_bar.rs", "CPU window created successfully", &serde_json::json!({}), "I");
            
            // Don't clear process cache - keep existing data for instant display
            // The cache will refresh naturally when it expires (20 seconds)
            // This prevents expensive refresh_processes() blocking the first call
            debug2!("Window opened - keeping existing process cache for instant display");
            
            // Clear rate limiter so first call always goes through (instant data on window open)
            use crate::state::LAST_CPU_DETAILS_CALL;
            if let Ok(mut last_call) = LAST_CPU_DETAILS_CALL.try_lock() {
                *last_call = None;
                debug2!("Rate limiter cleared - first get_cpu_details() call will execute immediately");
            }
            
            // Enable devtools for right-click inspect
            // In debug builds, devtools should be available by default
            // We can also try to enable it via the webview if needed
            #[cfg(debug_assertions)]
            {
                // Devtools are typically enabled by default in debug builds
                // Right-click inspect should work
            }
            
            let _ = window.set_always_on_top(true);
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.unminimize();
            debug1!("CPU window shown and focused");
            write_structured_log("ui/status_bar.rs", "CPU window shown and focused", &serde_json::json!({}), "I");
        },
        Err(e) => {
            debug1!("ERROR: Failed to create CPU window: {:?}", e);
            write_structured_log("ui/status_bar.rs", "ERROR: Failed to create CPU window", &serde_json::json!({"error": format!("{:?}", e)}), "I");
        }
    }
}
