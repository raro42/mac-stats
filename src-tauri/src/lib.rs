use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::process::Command;
use std::sync::OnceLock;
use std::sync::Mutex;
use sysinfo::{Disks, ProcessesToUpdate, System};
use macsmc::Smc;
// IOReport types kept for future use (extern block still references them)
use core_foundation::base::CFTypeRef;
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation::string::CFStringRef;

// IOReport FFI bindings (similar to macmon)
// Some functions are declared for future use
#[allow(dead_code)]
#[link(name = "IOReport", kind = "dylib")]
extern "C" {
    fn IOReportCopyAllChannels(a: u64, b: u64) -> CFDictionaryRef;
    fn IOReportCopyChannelsInGroup(
        group: CFStringRef,
        subgroup: CFStringRef,
        want_hierarchical: u64,
        want_sub_groups: u64,
        want_historical: u64,
    ) -> CFDictionaryRef;
    fn IOReportMergeChannels(
        dest: CFMutableDictionaryRef,
        src: CFDictionaryRef,
        nil: CFTypeRef,
    );
    fn IOReportCreateSubscription(
        allocator: CFTypeRef,
        channels: CFMutableDictionaryRef,
        subscription: *mut CFMutableDictionaryRef,
        channel_id: u64,
        options: CFTypeRef,
    ) -> *mut c_void;
    fn IOReportCreateSamples(
        subscription: *const c_void,
        channels: CFMutableDictionaryRef,
        options: CFTypeRef,
    ) -> CFDictionaryRef;
    fn IOReportCreateSamplesDelta(
        start: CFDictionaryRef,
        end: CFDictionaryRef,
        options: CFTypeRef,
    ) -> CFDictionaryRef;
    fn IOReportChannelGetGroup(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetSubGroup(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetChannelName(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportSimpleGetIntegerValue(item: CFDictionaryRef, index: i32) -> i64;
    fn IOReportChannelGetUnitLabel(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportStateGetCount(item: CFDictionaryRef) -> i32;
    fn IOReportStateGetNameForIndex(item: CFDictionaryRef, index: i32) -> CFStringRef;
    fn IOReportStateGetResidency(item: CFDictionaryRef, index: i32) -> i64;
}

// IOReport helper functions removed - IOReport operations were too expensive for real-time monitoring
// If needed in the future, these can be re-implemented with proper caching
use objc2::declare::ClassBuilder;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, NSObject, Sel};
use objc2::{msg_send, AnyThread, ClassType, MainThreadMarker, sel};
use objc2_app_kit::{
    NSAboutPanelOptionApplicationName, NSAboutPanelOptionApplicationVersion,
    NSAboutPanelOptionCredits, NSAboutPanelOptionVersion, NSApplication, NSColor, NSFont,
    NSFontWeightRegular, NSFontWeightSemibold, NSBaselineOffsetAttributeName,
    NSFontAttributeName, NSForegroundColorAttributeName, NSParagraphStyleAttributeName,
    NSMutableParagraphStyle, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength, NSTextAlignment, NSTextTab, NSTextTabOptionKey, NSEvent,
};
use objc2_foundation::{
    NSArray, NSDictionary, NSMutableAttributedString, NSMutableDictionary, NSNumber,
    NSAttributedString, NSProcessInfo, NSRange, NSString,
};
use tauri::{Manager, WindowBuilder, WindowUrl};

static SYSTEM: Mutex<Option<System>> = Mutex::new(None);
static DISKS: Mutex<Option<Disks>> = Mutex::new(None);
thread_local! {
    static STATUS_ITEM: RefCell<Option<Retained<NSStatusItem>>> = RefCell::new(None);
    static CLICK_HANDLER: RefCell<Option<Retained<AnyObject>>> = RefCell::new(None);
}
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
const BUILD_DATE: &str = env!("BUILD_DATE");

// Cache expensive resources to avoid recreating every second
// Note: IOReport subscriptions are very expensive to create/destroy - removed for performance
// SMC cannot be cached in static (not Sync) - will create on demand but cache access checks
static CHIP_INFO_CACHE: OnceLock<String> = OnceLock::new();
static ACCESS_CACHE: Mutex<Option<(bool, bool, bool, bool)>> = Mutex::new(None); // temp, freq, cpu_power, gpu_power

#[derive(serde::Serialize, Clone)]
struct SystemMetrics {
    cpu: f32,
    gpu: f32,
    ram: f32,
    disk: f32,
}

#[derive(serde::Serialize)]
struct ProcessUsage {
    name: String,
    cpu: f32,
}

#[derive(serde::Serialize)]
struct CpuDetails {
    usage: f32,
    temperature: f32,
    frequency: f32,
    cpu_power: f32,
    gpu_power: f32,
    load_1: f64,
    load_5: f64,
    load_15: f64,
    uptime_secs: u64,
    top_processes: Vec<ProcessUsage>,
    chip_info: String,
    // Access flags - true if we can read the value, false if access is denied
    can_read_temperature: bool,
    can_read_frequency: bool,
    can_read_cpu_power: bool,
    can_read_gpu_power: bool,
}

fn get_chip_info() -> String {
    // Cache chip info - only fetch once
    CHIP_INFO_CACHE.get_or_init(|| {
        // Get chip information from system_profiler (JSON format)
        let output = Command::new("sh")
            .arg("-c")
            .arg("system_profiler SPHardwareDataType -json 2>/dev/null")
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Try to parse JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(hardware) = json.get("SPHardwareDataType").and_then(|v| v.as_array()).and_then(|a| a.get(0)) {
                        let chip_type = hardware.get("chip_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let num_procs = hardware.get("number_processors")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        
                        if !chip_type.is_empty() {
                            // Format: "Apple M3 • 16 cores" or similar
                            let mut info = chip_type.to_string();
                            if !num_procs.is_empty() {
                                // number_processors format: "proc 1:8:8" (total:performance:efficiency)
                                let num_procs_clean = num_procs.strip_prefix("proc ").unwrap_or(num_procs);
                                let parts: Vec<&str> = num_procs_clean.split(':').collect();
                                if parts.len() >= 3 {
                                    let p_cores = parts.get(1).unwrap_or(&"0");
                                    let e_cores = parts.get(2).unwrap_or(&"0");
                                    let total: u32 = p_cores.parse().unwrap_or(0) + e_cores.parse().unwrap_or(0);
                                    if total > 0 {
                                        info.push_str(&format!(" • {} cores", total));
                                    }
                                } else if parts.len() == 1 {
                                    // Single number (total cores)
                                    if let Ok(total) = parts[0].parse::<u32>() {
                                        if total > 0 {
                                            info.push_str(&format!(" • {} cores", total));
                                        }
                                    }
                                }
                            }
                            return info;
                        }
                    }
                }
            }
        }
        
        // Fallback: try sysctl for Intel Macs
        let output = Command::new("sh")
            .arg("-c")
            .arg("sysctl -n machdep.cpu.brand_string 2>/dev/null | head -1")
            .output();
        
        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let trimmed = stdout.trim();
                if !trimmed.is_empty() && trimmed.len() < 50 {
                    return trimmed.to_string();
                }
            }
        }
        
        "—".to_string()
    }).clone()
}

fn get_gpu_usage() -> f32 {
    // GPU usage reading is expensive (ioreg commands) - return 0 for now to save CPU
    // TODO: Cache or optimize if GPU usage is needed
    0.0
}

fn can_read_temperature() -> bool {
    // Check cache first (only check once, then cache)
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((temp, _, _, _)) = cache.as_ref() {
        return *temp;
    }
    
    // Check if we can access temperature data (only once)
    let can_read = {
        // Try SMC first
        if let Ok(mut smc) = Smc::connect() {
            smc.cpu_temperature().is_ok()
        } else {
            // Fallback to NSProcessInfo (always available)
            if let Some(_mtm) = MainThreadMarker::new() {
                let process_info = NSProcessInfo::processInfo();
                let thermal_state = process_info.thermalState();
                thermal_state.0 <= 3 // Valid thermal state
            } else {
                false
            }
        }
    };
    
    // Cache the result permanently
    if let Some((_, freq, cpu_power, gpu_power)) = cache.as_ref() {
        *cache = Some((can_read, *freq, *cpu_power, *gpu_power));
    } else {
        *cache = Some((can_read, false, false, false));
    }
    
    can_read
}

fn get_cpu_temperature() -> f32 {
    // Try SMC (create connection each time - Smc is not Sync so can't cache in static)
    // But this is still cheaper than IOReport
    if let Ok(mut smc) = Smc::connect() {
        if let Ok(temps) = smc.cpu_temperature() {
            let die_temp: f64 = temps.die.into();
            let prox_temp: f64 = temps.proximity.into();
            
            let temp = if die_temp > 0.0 {
                die_temp
            } else if prox_temp > 0.0 {
                prox_temp
            } else {
                0.0
            };
            if temp > 0.0 {
                return temp as f32;
            }
        }
    }
    
    // Fallback to NSProcessInfo.thermalState (always available, user space, very cheap)
    if let Some(_mtm) = MainThreadMarker::new() {
        let process_info = NSProcessInfo::processInfo();
        let thermal_state = process_info.thermalState();
        let state_value = thermal_state.0;
        
        // Map thermal state to approximate temperature
        match state_value {
            0 => 40.0,  // Nominal
            1 => 60.0,  // Fair
            2 => 80.0,  // Serious
            3 => 95.0,  // Critical
            _ => 0.0,
        }
    } else {
        0.0
    }
}

fn can_read_frequency() -> bool {
    // Check cache first
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((_, freq, _, _)) = cache.as_ref() {
        return *freq;
    }
    
    // Check if sysctl works (much cheaper than IOReport)
    let output = Command::new("sh")
        .arg("-c")
        .arg("sysctl -n hw.cpufrequency_max 2>/dev/null || sysctl -n hw.cpufrequency 2>/dev/null || echo ''")
        .output();
    
    let can_read = if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            !stdout.trim().is_empty() && stdout.trim() != "0"
        } else {
            false
        }
    } else {
        false
    };
    
    // Update cache
    if let Some((temp, _, cpu_power, gpu_power)) = cache.as_ref() {
        *cache = Some((*temp, can_read, *cpu_power, *gpu_power));
    } else {
        *cache = Some((false, can_read, false, false));
    }
    
    can_read
}

fn get_cpu_frequency() -> f32 {
    // IOReport is too expensive to call every second - skip it for now
    // Only use sysctl fallback which is much cheaper
    let output = Command::new("sh")
        .arg("-c")
        .arg("sysctl -n hw.cpufrequency_max 2>/dev/null || sysctl -n hw.cpufrequency 2>/dev/null || echo '0'")
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            if !trimmed.is_empty() && trimmed != "0" {
                if let Ok(freq_hz) = trimmed.parse::<f64>() {
                    if freq_hz > 0.0 {
                        let freq_ghz = (freq_hz / 1_000_000_000.0) as f32;
                        if freq_ghz > 0.1 && freq_ghz < 10.0 {
                            return freq_ghz;
                        }
                    }
                }
            }
        }
    }
    
    0.0
}

fn can_read_cpu_power() -> bool {
    // Check cache first
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((_, _, cpu_power, _)) = cache.as_ref() {
        return *cpu_power;
    }
    
    // IOReport is too expensive - check once and cache
    // For now, assume false unless we can verify cheaply
    let can_read = false; // IOReport too expensive to check every time
    
    // Update cache
    if let Some((temp, freq, _, gpu_power)) = cache.as_ref() {
        *cache = Some((*temp, *freq, can_read, *gpu_power));
    } else {
        *cache = Some((false, false, can_read, false));
    }
    
    can_read
}

fn get_cpu_power() -> f32 {
    // IOReport is too expensive to call every second - return 0 for now
    // TODO: Implement proper caching if power reading is needed
    0.0
}

fn can_read_gpu_power() -> bool {
    // Check cache first
    let mut cache = ACCESS_CACHE.lock().unwrap();
    if let Some((_, _, _, gpu_power)) = cache.as_ref() {
        return *gpu_power;
    }
    
    // IOReport is too expensive - check once and cache
    let can_read = false; // IOReport too expensive to check every time
    
    // Update cache
    if let Some((temp, freq, cpu_power, _)) = cache.as_ref() {
        *cache = Some((*temp, *freq, *cpu_power, can_read));
    } else {
        *cache = Some((false, false, false, can_read));
    }
    
    can_read
}

fn get_gpu_power() -> f32 {
    // IOReport is too expensive to call every second - return 0 for now
    // TODO: Implement proper caching if power reading is needed
    0.0
}

fn get_metrics() -> SystemMetrics {
    let mut sys = SYSTEM.lock().unwrap();
    if sys.is_none() {
        *sys = Some(System::new());
    }
    let sys = sys.as_mut().unwrap();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let mut disks = DISKS.lock().unwrap();
    if disks.is_none() {
        *disks = Some(Disks::new());
    }
    let disks = disks.as_mut().unwrap();
    disks.refresh(true);

    let cpu_usage = sys.global_cpu_usage();
    let ram_usage = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;
    let mut disk_usage = 0.0;
    for disk in disks.list() {
        let total = disk.total_space() as f64;
        let available = disk.available_space() as f64;
        if total > 0.0 {
            disk_usage = ((total - available) / total * 100.0) as f32;
            break;
        }
    }
    
    let gpu_usage = get_gpu_usage();

    SystemMetrics {
        cpu: cpu_usage,
        gpu: gpu_usage,
        ram: ram_usage,
        disk: disk_usage,
    }
}

#[tauri::command]
fn get_cpu_details() -> CpuDetails {
    let mut sys = SYSTEM.lock().unwrap();
    if sys.is_none() {
        *sys = Some(System::new());
    }
    let sys = sys.as_mut().unwrap();
    sys.refresh_cpu_usage();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let usage = sys.global_cpu_usage();
    let load = sysinfo::System::load_average();
    let uptime_secs = sysinfo::System::uptime();
    
    // Get CPU temperature using macsmc (similar to macmon, user space)
    let temperature = get_cpu_temperature();
    
    // Get CPU frequency (macOS) - simplified, no expensive IOReport
    let frequency = get_cpu_frequency();
    
    // Get power consumption (macOS) - disabled for performance (IOReport too expensive)
    let cpu_power = get_cpu_power();
    let gpu_power = get_gpu_power();
    
    // Get chip information (cached, only fetched once)
    let chip_info = get_chip_info();
    
    // Only check access flags once and cache them
    // Access checks are expensive, so we do them once and cache
    let mut access_cache = ACCESS_CACHE.lock().unwrap();
    let (can_read_temperature, can_read_frequency, can_read_cpu_power, can_read_gpu_power) = 
        if let Some(cached) = access_cache.as_ref() {
            *cached
        } else {
            // First time - check all access
            let temp = can_read_temperature();
            let freq = can_read_frequency();
            let cpu_p = can_read_cpu_power();
            let gpu_p = can_read_gpu_power();
            let result = (temp, freq, cpu_p, gpu_p);
            *access_cache = Some(result);
            result
        };

    let mut processes: Vec<ProcessUsage> = sys
        .processes()
        .values()
        .map(|proc| ProcessUsage {
            name: proc.name().to_string_lossy().to_string(),
            cpu: proc.cpu_usage(),
        })
        .collect();
    processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    processes.truncate(8); // Show top 8 processes

    CpuDetails {
        usage,
        temperature,
        frequency,
        cpu_power,
        gpu_power,
        load_1: load.one,
        load_5: load.five,
        load_15: load.fifteen,
        uptime_secs,
        top_processes: processes,
        chip_info,
        can_read_temperature,
        can_read_frequency,
        can_read_cpu_power,
        can_read_gpu_power,
    }
}

fn build_status_text(metrics: &SystemMetrics) -> String {
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

fn as_any<T: objc2::Message>(obj: &T) -> &AnyObject {
    unsafe { &*(obj as *const T as *const AnyObject) }
}

fn make_attributed_title(text: &str) -> Retained<NSMutableAttributedString> {
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
    let color = NSColor::labelColor();
    let paragraph = NSMutableParagraphStyle::new();
    paragraph.setLineSpacing(-2.0);
    paragraph.setLineHeightMultiple(0.75);
    paragraph.setAlignment(NSTextAlignment::Left);
    paragraph.setDefaultTabInterval(38.0);

    let options: Retained<NSDictionary<NSTextTabOptionKey, AnyObject>> = NSDictionary::new();
    let tab1 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            38.0,
            &options,
        )
    };
    let tab2 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            76.0,
            &options,
        )
    };
    let tab3 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            114.0,
            &options,
        )
    };
    let tab4 = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Left,
            152.0,
            &options,
        )
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

fn setup_status_item() {
    let mtm = MainThreadMarker::new().unwrap();
    let status_bar = NSStatusBar::systemStatusBar();
    let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

    let handler: Retained<AnyObject> =
        unsafe { Retained::from_raw(msg_send![click_handler_class(), new]) }
            .expect("click handler");
    CLICK_HANDLER.with(|cell| {
        *cell.borrow_mut() = Some(handler.clone());
    });

    if let Some(button) = status_item.button(mtm) {
        unsafe {
            button.setTarget(Some(&*handler));
            button.setAction(Some(sel!(onStatusItemClick:)));
        }
    }

    STATUS_ITEM.with(|cell| {
        *cell.borrow_mut() = Some(status_item);
    });
}

fn click_handler_class() -> &'static AnyClass {
    static REGISTER: OnceLock<&'static AnyClass> = OnceLock::new();
    REGISTER.get_or_init(|| {
        let name = unsafe { CStr::from_bytes_with_nul_unchecked(b"MacStatsStatusHandler\0") };
        let mut builder = ClassBuilder::new(name, NSObject::class()).expect("class already exists");
        extern "C-unwind" fn on_status_item_click(
            _this: &AnyObject,
            _cmd: Sel,
            _sender: *mut AnyObject,
        ) {
            let mtm = MainThreadMarker::new().unwrap();
            let app = NSApplication::sharedApplication(mtm);
            let is_right_click = app
                .currentEvent()
                .map(|event: Retained<NSEvent>| {
                    // Check button number: 0 = left, 1 = right, 2 = middle
                    let button_number = event.buttonNumber();
                    button_number == 1
                })
                .unwrap_or(false);
            if is_right_click {
                show_about_panel();
            } else {
                show_cpu_window();
            }
        }
        unsafe {
            builder.add_method(
                sel!(onStatusItemClick:),
                on_status_item_click as extern "C-unwind" fn(_, _, _),
            );
        }
        builder.register()
    })
}

fn show_cpu_window() {
    if let Some(app) = APP_HANDLE.get() {
        let app = app.clone();
        let _ = app.run_on_main_thread({
            let app = app.clone();
            move || {
                if let Some(window) = app.get_window("cpu") {
                    // Set always on top first
                    let _ = window.set_always_on_top(true);
                    // Show and focus the window
                    let _ = window.show();
                    let _ = window.set_focus();
                    // Bring to front
                    let _ = window.unminimize();
                }
            }
        });
    }
}

fn show_about_panel() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    let name = NSString::from_str("mac_stats");
    let version = NSString::from_str(env!("CARGO_PKG_VERSION"));
    let build = NSString::from_str(BUILD_DATE);
    let authors = NSString::from_str(env!("CARGO_PKG_AUTHORS"));
    let credits_text = NSString::from_str(&format!(
        "Author: {}\nBuild: {}",
        authors, BUILD_DATE
    ));
    let credits = NSAttributedString::from_nsstring(&credits_text);

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

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_cpu_details])
        .setup(|app| {
            // Hide the main window immediately (menu bar app)
            if let Some(main_window) = app.get_window("main") {
                let _ = main_window.hide();
            }
            
            let _ = APP_HANDLE.set(app.handle());

            let cpu_window = WindowBuilder::new(
                app,
                "cpu",
                WindowUrl::App("cpu.html".into()),
            )
            .title("CPU")
            .visible(false)  // Start hidden
            .inner_size(360.0, 560.0)
            .resizable(true)
            .always_on_top(true)  // Always on top when shown
            .build();

            if let Ok(_window) = cpu_window {
                // Window is created hidden, will be shown on click
            }

            setup_status_item();
            
            // Set initial text immediately so the menu bar item is visible
            let initial_metrics = get_metrics();
            let initial_text = build_status_text(&initial_metrics);
            let initial_attributed = make_attributed_title(&initial_text);
            STATUS_ITEM.with(|cell| {
                if let Some(item) = cell.borrow().as_ref() {
                    let mtm = MainThreadMarker::new().unwrap();
                    if let Some(button) = item.button(mtm) {
                        button.setAttributedTitle(&initial_attributed);
                    }
                }
            });
            
            let app_handle = app.handle();
            std::thread::spawn(move || {
                loop {
                    // Update menu bar every 2 seconds to reduce CPU usage
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let metrics = get_metrics();
                    let text = build_status_text(&metrics);
                    let _ = app_handle.run_on_main_thread(move || {
                        let attributed = make_attributed_title(&text);
                        STATUS_ITEM.with(|cell| {
                            if let Some(item) = cell.borrow().as_ref() {
                                let mtm = MainThreadMarker::new().unwrap();
                                if let Some(button) = item.button(mtm) {
                                    button.setAttributedTitle(&attributed);
                                }
                            }
                        });
                    });
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}