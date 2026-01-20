//! Safe wrappers for IOReport FFI calls
//! 
//! IOReport is a macOS framework for system performance monitoring.
//! These wrappers add null checks and error handling to prevent crashes.

use std::os::raw::c_void;
use std::time::Instant;
use core_foundation::base::{CFTypeRef, TCFType};
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation::string::{CFStringRef, CFString};
use thiserror::Error;

/// IOReport error types
/// Currently unused as direct FFI calls are used in lib.rs.
/// Kept for future migration to safer FFI patterns.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum IOReportError {
    #[error("IOReport function returned null pointer")]
    NullPointer,
    
    #[error("IOReport function returned invalid dictionary")]
    InvalidDictionary,
    
    #[error("IOReport function returned invalid string")]
    InvalidString,
    
    #[error("IOReport function failed: {0}")]
    FunctionFailed(String),
    
    #[error("IOReport channel not found")]
    ChannelNotFound,
    
    #[error("IOReport state index out of bounds")]
    StateIndexOutOfBounds,
}

/// Result type for IOReport operations
#[allow(dead_code)] // Kept for future FFI migration
pub type IOReportResult<T> = Result<T, IOReportError>;

// Raw FFI bindings (unsafe)
// Note: These are declared but currently unused as direct calls are in lib.rs.
// Kept for future migration to safer FFI patterns.
#[allow(dead_code)] // All FFI functions kept for future migration
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

/// Safe wrapper for IOReportCopyChannelsInGroup
/// 
/// Note: This function expects static string literals for group and subgroup.
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn copy_channels_in_group(
    group: &'static str,
    subgroup: &'static str,
    want_hierarchical: bool,
    want_sub_groups: bool,
    want_historical: bool,
) -> IOReportResult<CFDictionaryRef> {
    // Create CFString objects from static strings
    let group_cf = CFString::from_static_string(group);
    let subgroup_cf = CFString::from_static_string(subgroup);
    
    // Keep references alive during the unsafe call
    let group_ref = group_cf.as_concrete_TypeRef();
    let subgroup_ref = subgroup_cf.as_concrete_TypeRef();
    
    let result = unsafe {
        IOReportCopyChannelsInGroup(
            group_ref,
            subgroup_ref,
            if want_hierarchical { 1 } else { 0 },
            if want_sub_groups { 1 } else { 0 },
            if want_historical { 1 } else { 0 },
        )
    };
    
    if result.is_null() {
        Err(IOReportError::NullPointer)
    } else {
        Ok(result)
    }
}

/// Safe wrapper for IOReportMergeChannels
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn merge_channels(
    dest: CFMutableDictionaryRef,
    src: CFDictionaryRef,
) -> IOReportResult<()> {
    if dest.is_null() {
        return Err(IOReportError::NullPointer);
    }
    if src.is_null() {
        return Err(IOReportError::InvalidDictionary);
    }
    
    unsafe {
        IOReportMergeChannels(dest, src, std::ptr::null());
    }
    
    Ok(())
}

/// Safe wrapper for IOReportCreateSubscription
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn create_subscription(
    channels: CFMutableDictionaryRef,
) -> IOReportResult<(*mut c_void, CFMutableDictionaryRef)> {
    if channels.is_null() {
        return Err(IOReportError::NullPointer);
    }
    
    let mut subscription_dict: CFMutableDictionaryRef = std::ptr::null_mut();
    
    let subscription_ptr = unsafe {
        IOReportCreateSubscription(
            std::ptr::null(),
            channels,
            &mut subscription_dict,
            0,
            std::ptr::null(),
        )
    };
    
    if subscription_ptr.is_null() {
        Err(IOReportError::NullPointer)
    } else {
        Ok((subscription_ptr, subscription_dict))
    }
}

/// Safe wrapper for IOReportCreateSamples
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn create_samples(
    subscription: *const c_void,
    channels: CFMutableDictionaryRef,
) -> IOReportResult<CFDictionaryRef> {
    if subscription.is_null() {
        return Err(IOReportError::NullPointer);
    }
    if channels.is_null() {
        return Err(IOReportError::NullPointer);
    }
    
    let result = unsafe {
        IOReportCreateSamples(
            subscription,
            channels,
            std::ptr::null(),
        )
    };
    
    if result.is_null() {
        Err(IOReportError::NullPointer)
    } else {
        Ok(result)
    }
}

/// Safe wrapper for IOReportChannelGetChannelName
#[allow(dead_code)] // Kept for future FFI migration
pub fn get_channel_name(channel: CFDictionaryRef) -> IOReportResult<String> {
    if channel.is_null() {
        return Err(IOReportError::InvalidDictionary);
    }
    
    let name_ref = unsafe { IOReportChannelGetChannelName(channel) };
    
    if name_ref.is_null() {
        Err(IOReportError::InvalidString)
    } else {
        let name = unsafe { CFString::wrap_under_get_rule(name_ref) };
        Ok(name.to_string())
    }
}

/// Safe wrapper for IOReportStateGetCount
#[allow(dead_code)] // Kept for future FFI migration
pub fn get_state_count(channel: CFDictionaryRef) -> IOReportResult<i32> {
    if channel.is_null() {
        return Err(IOReportError::InvalidDictionary);
    }
    
    let count = unsafe { IOReportStateGetCount(channel) };
    
    if count < 0 {
        Err(IOReportError::FunctionFailed("Invalid state count".to_string()))
    } else {
        Ok(count)
    }
}

/// Safe wrapper for IOReportStateGetNameForIndex
#[allow(dead_code)] // Kept for future FFI migration
pub fn get_state_name_for_index(channel: CFDictionaryRef, index: i32) -> IOReportResult<String> {
    if channel.is_null() {
        return Err(IOReportError::InvalidDictionary);
    }
    
    // Validate index first
    let count = get_state_count(channel)?;
    if index < 0 || index >= count {
        return Err(IOReportError::StateIndexOutOfBounds);
    }
    
    let name_ref = unsafe { IOReportStateGetNameForIndex(channel, index) };
    
    if name_ref.is_null() {
        Err(IOReportError::InvalidString)
    } else {
        let name = unsafe { CFString::wrap_under_get_rule(name_ref) };
        Ok(name.to_string())
    }
}

/// Safe wrapper for IOReportStateGetResidency
#[allow(dead_code)] // Kept for future FFI migration
pub fn get_state_residency(channel: CFDictionaryRef, index: i32) -> IOReportResult<i64> {
    if channel.is_null() {
        return Err(IOReportError::InvalidDictionary);
    }
    
    // Validate index first
    let count = get_state_count(channel)?;
    if index < 0 || index >= count {
        return Err(IOReportError::StateIndexOutOfBounds);
    }
    
    let residency = unsafe { IOReportStateGetResidency(channel, index) };
    Ok(residency)
}

// Frequency reading functionality
// These functions extract CPU frequency information from IOReport channels

// CoreFoundation FFI functions needed for dictionary iteration
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDictionaryGetCount(theDict: CFDictionaryRef) -> i32;
    fn CFDictionaryGetKeysAndValues(
        theDict: CFDictionaryRef,
        keys: *mut *const c_void,
        values: *mut *const c_void,
    );
    fn CFGetTypeID(cf: CFTypeRef) -> u64;
    fn CFDictionaryGetTypeID() -> u64;
    fn CFArrayGetTypeID() -> u64;
    fn CFStringGetTypeID() -> u64;
    fn CFArrayGetCount(theArray: *const c_void) -> i32;
    fn CFArrayGetValueAtIndex(theArray: *const c_void, idx: i32) -> *const c_void;
    fn CFRelease(cf: CFTypeRef);
}

// IOReport FFI functions are already declared at the top of the file

/// Frequency data structure
#[derive(Debug, Default)]
pub struct FrequencyData {
    pub overall: f32,
    pub p_core: f32,
    pub e_core: f32,
}

/// Power data structure (CPU and GPU power in watts)
#[derive(Debug, Default)]
pub struct PowerData {
    pub cpu_power: f32,  // CPU power in watts
    pub gpu_power: f32, // GPU power in watts
}

/// Internal structure for accumulating frequency statistics
#[derive(Debug, Default)]
struct FrequencyAccumulator {
    max_freq_mhz: f64,
    total_residency: f64,
    weighted_freq_sum: f64,
    p_core_max_freq_mhz: f64,
    p_core_total_residency: f64,
    p_core_weighted_freq_sum: f64,
    e_core_max_freq_mhz: f64,
    e_core_total_residency: f64,
    e_core_weighted_freq_sum: f64,
}

/// Determine if a channel is a P-core or E-core channel
fn classify_channel(channel_name: &str) -> (bool, bool) {
    // Channel names are like "ECPU000", "ECPU010" (E-cores) or "PCPU000", "PCPU010" (P-cores)
    let is_p_core = channel_name.starts_with("PCPU") || 
                   channel_name.contains("P-Cluster") || 
                   (channel_name.contains("Performance") && 
                    !channel_name.contains("E-Cluster") && 
                    !channel_name.contains("Efficiency") &&
                    !channel_name.starts_with("ECPU"));
    let is_e_core = channel_name.starts_with("ECPU") ||
                   channel_name.contains("E-Cluster") || 
                   (channel_name.contains("Efficiency") && 
                    !channel_name.contains("P-Cluster") && 
                    !channel_name.contains("Performance") &&
                    !channel_name.starts_with("PCPU"));
    (is_p_core, is_e_core)
}

/// Check if a channel name indicates a performance state channel
/// Only match specific CPU performance state channels to avoid processing unrelated channels
fn is_performance_channel(channel_name: &str) -> bool {
    channel_name.starts_with("ECPU") ||
    channel_name.starts_with("PCPU") ||
    channel_name.starts_with("E-Cluster") ||
    channel_name.starts_with("P-Cluster") ||
    channel_name.contains("CPU Core Performance States")
}

/// Extract frequency from state name
/// Handles formats like:
/// - "2400 MHz" -> 2400.0
/// - "V0P5", "V1P4", etc. (voltage/performance states) -> estimated frequency based on P-state
fn extract_frequency_from_name(state_name: &str) -> Option<f64> {
    // First try standard "MHz" format
    if state_name.contains("MHz") {
        return state_name
            .split_whitespace()
            .find_map(|s| s.parse::<f64>().ok())
            .filter(|&val| val > 0.0 && val < 10000.0);
    }
    
    // Handle voltage/performance state format: "V0P5", "V1P4", "V19P0", etc.
    // Format: V<voltage_level>P<performance_level>
    // NOTE: This is a HEURISTIC mapping and may not match actual frequencies.
    // The mapping is linear and approximate. For accurate frequencies, prefer
    // channels that expose MHz values directly, or derive mapping empirically
    // from powermetrics/IOReport frequency tables per SoC family.
    // For E-cores: P5 (lowest) to P0 (highest) - typically 0.5-2.4 GHz
    // For P-cores: P19 (lowest) to P0 (highest) - typically 0.6-4.0+ GHz
    if state_name.starts_with("V") && state_name.contains("P") {
        // Extract the P-state number (after "P")
        if let Some(p_pos) = state_name.find('P') {
            let p_state_str = &state_name[p_pos + 1..];
            // P-state might be followed by other characters, extract just the number
            let p_state_num: String = p_state_str.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(p_state) = p_state_num.parse::<i32>() {
                // HEURISTIC: Linear frequency estimation from P-state
                // This is approximate and may not match actual SoC frequencies
                // E-cores: P5=0.5GHz, P4=0.8GHz, P3=1.2GHz, P2=1.6GHz, P1=2.0GHz, P0=2.4GHz
                // P-cores: P19=0.6GHz, P15=1.2GHz, P10=2.0GHz, P5=3.0GHz, P0=4.0GHz
                if p_state <= 5 {
                    // E-core range: P5 to P0 (linear approximation)
                    let freq_mhz = 500.0 + (5 - p_state) as f64 * 380.0; // 500-2400 MHz
                    return Some(freq_mhz);
                } else {
                    // P-core range: P19 to P0 (linear approximation)
                    let freq_mhz = 600.0 + (19 - p_state) as f64 * 180.0; // 600-4000 MHz
                    return Some(freq_mhz);
                }
            }
        }
    }
    
    None
}

/// Estimate frequency from P-state (P0, P1, etc.)
fn estimate_frequency_from_pstate(state_idx: i32, is_p_core: bool, is_e_core: bool) -> f64 {
    if is_p_core {
        match state_idx {
            0 => 4000.0, // P0 = max
            1 => 3500.0, // P1
            2 => 3000.0, // P2
            _ => 2500.0, // Lower states
        }
    } else if is_e_core {
        match state_idx {
            0 => 2400.0, // E0 = max
            1 => 2000.0, // E1
            _ => 1500.0, // Lower states
        }
    } else {
        match state_idx {
            0 => 3000.0, // P0 equivalent
            _ => 2000.0,
        }
    }
}

/// Parse performance states from a channel and accumulate frequency data
unsafe fn parse_channel_states(
    channel_ref: CFDictionaryRef,
    channel_name: &str,
    is_p_core: bool,
    is_e_core: bool,
    accumulator: &mut FrequencyAccumulator,
    freq_logging: bool,
) {
    use crate::{debug1, debug2, debug3};
    
    if channel_ref.is_null() {
        debug3!("Channel '{}' reference is null, skipping state iteration", channel_name);
        return;
    }
    
    // Validate that this is a valid IOReport channel dictionary
    // Check that we can get the channel name (validates it's a channel dict)
    let name_check = IOReportChannelGetChannelName(channel_ref);
    if name_check.is_null() {
        debug2!("Channel '{}': IOReportChannelGetChannelName returned null, not a valid channel dict, skipping", channel_name);
        return;
    }
    
    // Validate state count is reasonable before iterating
    let state_count = IOReportStateGetCount(channel_ref);
    if state_count < 0 {
        debug2!("Channel '{}': IOReportStateGetCount returned negative value ({}), invalid channel, skipping", channel_name, state_count);
        return;
    }
    // Reasonable max: 100 states (typical CPUs have 8-22 states)
    if state_count > 100 {
        debug2!("Channel '{}': IOReportStateGetCount returned suspiciously high value ({}), likely invalid channel, skipping", channel_name, state_count);
        return;
    }
    
    if freq_logging {
        debug1!("Channel '{}' has {} performance states", channel_name, state_count);
    } else {
        debug3!("Channel '{}' has {} performance states", channel_name, state_count);
    }
    
    for state_idx in 0..state_count {
        let state_name_ref = IOReportStateGetNameForIndex(channel_ref, state_idx);
        if state_name_ref.is_null() {
            continue;
        }
        
        let state_name = CFString::wrap_under_get_rule(state_name_ref);
        let state_name_str = state_name.to_string();
        let residency_ns = IOReportStateGetResidency(channel_ref, state_idx);
        let residency_ratio = residency_ns as f64 / 1_000_000_000.0;
        
        if freq_logging {
            debug1!("  State {}: name='{}', residency={} ns ({:.3} s)", 
                state_idx, state_name_str, residency_ns, residency_ratio);
        } else {
            debug3!("  State {}: name='{}', residency={} ns", 
                state_idx, state_name_str, residency_ns);
        }
        
        // Skip DOWN and IDLE states (they don't represent active frequencies)
        if state_name_str == "DOWN" || state_name_str == "IDLE" {
            if freq_logging {
                debug1!("  State {}: skipping '{}' (not an active frequency state)", state_idx, state_name_str);
            }
            continue;
        }
        
        // Try to extract frequency from state name
        if let Some(mhz_val) = extract_frequency_from_name(&state_name_str) {
            // Update overall frequency
            if mhz_val > accumulator.max_freq_mhz {
                accumulator.max_freq_mhz = mhz_val;
            }
            accumulator.weighted_freq_sum += mhz_val * residency_ratio;
            accumulator.total_residency += residency_ratio;
            
            // Update P-core or E-core specific frequency
            if is_p_core {
                if mhz_val > accumulator.p_core_max_freq_mhz {
                    accumulator.p_core_max_freq_mhz = mhz_val;
                }
                accumulator.p_core_weighted_freq_sum += mhz_val * residency_ratio;
                accumulator.p_core_total_residency += residency_ratio;
            } else if is_e_core {
                if mhz_val > accumulator.e_core_max_freq_mhz {
                    accumulator.e_core_max_freq_mhz = mhz_val;
                }
                accumulator.e_core_weighted_freq_sum += mhz_val * residency_ratio;
                accumulator.e_core_total_residency += residency_ratio;
            }
            
            if freq_logging {
                debug1!("  State {}: extracted {} MHz from '{}' (weighted: {:.2} MHz)", 
                    state_idx, mhz_val, state_name_str, mhz_val * residency_ratio);
            } else {
                debug3!("  State {}: extracted {} MHz from name '{}'", 
                    state_idx, mhz_val, state_name_str);
            }
        } else if state_name_str.starts_with("P") && state_name_str.len() <= 3 {
            // Simple P-state (P0, P1, etc.) - estimate frequency
            let estimated_freq = estimate_frequency_from_pstate(state_idx, is_p_core, is_e_core);
            
            // Update overall frequency
            accumulator.weighted_freq_sum += estimated_freq * residency_ratio;
            accumulator.total_residency += residency_ratio;
            if estimated_freq > accumulator.max_freq_mhz {
                accumulator.max_freq_mhz = estimated_freq;
            }
            
            // Update P-core or E-core specific frequency
            if is_p_core {
                accumulator.p_core_weighted_freq_sum += estimated_freq * residency_ratio;
                accumulator.p_core_total_residency += residency_ratio;
                if estimated_freq > accumulator.p_core_max_freq_mhz {
                    accumulator.p_core_max_freq_mhz = estimated_freq;
                }
            } else if is_e_core {
                accumulator.e_core_weighted_freq_sum += estimated_freq * residency_ratio;
                accumulator.e_core_total_residency += residency_ratio;
                if estimated_freq > accumulator.e_core_max_freq_mhz {
                    accumulator.e_core_max_freq_mhz = estimated_freq;
                }
            }
            
            if freq_logging {
                debug1!("  State {}: estimated {} MHz from P-state '{}' (weighted: {:.2} MHz)", 
                    state_idx, estimated_freq, state_name_str, estimated_freq * residency_ratio);
            } else {
                debug3!("  State {}: estimated {} MHz from P-state '{}'", 
                    state_idx, estimated_freq, state_name_str);
            }
        } else {
            debug3!("  State {}: name '{}' doesn't match frequency patterns, skipping", 
                state_idx, state_name_str);
        }
    }
}

/// Find the IOReportChannels dictionary from the original channels dictionary
#[allow(dead_code)]
unsafe fn find_ioreport_channels(orig_channels: CFDictionaryRef) -> Option<CFDictionaryRef> {
    use crate::debug3;
    
    let channels_count = CFDictionaryGetCount(orig_channels);
    debug3!("Original channels_dict has {} channels", channels_count);
    
    if channels_count == 0 {
        return None;
    }
    
    let mut channel_keys_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count as usize];
    let mut channel_values_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count as usize];
    
    CFDictionaryGetKeysAndValues(
        orig_channels,
        channel_keys_buf.as_mut_ptr(),
        channel_values_buf.as_mut_ptr(),
    );
    
    // Find "IOReportChannels" key
    for i in 0..(channels_count as usize) {
        let channel_key_ref = channel_keys_buf[i] as CFStringRef;
        if channel_key_ref.is_null() {
            continue;
        }
        
        let key_type_id = CFGetTypeID(channel_key_ref as CFTypeRef);
        let string_type_id = CFStringGetTypeID();
        if key_type_id != string_type_id {
            continue;
        }
        
        let channel_key_str = CFString::wrap_under_get_rule(channel_key_ref);
        let key_str = channel_key_str.to_string();
        
        if key_str == "IOReportChannels" {
            let value_ptr = channel_values_buf[i];
            if value_ptr.is_null() {
                continue;
            }
            
            let value_type_id = CFGetTypeID(value_ptr as CFTypeRef);
            let dict_type_id = CFDictionaryGetTypeID();
            
            if value_type_id == dict_type_id {
                return Some(value_ptr as CFDictionaryRef);
            }
        }
    }
    
    None
}

/// Look up a channel dictionary from the original channels dictionary by matching the channel key
/// This is used as a fallback when channel name cannot be extracted directly from sample
unsafe fn lookup_channel_dict(
    channel_key: &str,
    _orig_channels: CFDictionaryRef,
    channel_keys_buf: &[*const c_void],
    channel_values_buf: &[*const c_void],
    channels_count: usize,
) -> Option<(CFDictionaryRef, CFStringRef)> {
    
    // First, try to find exact key match
    for orig_i in 0..channels_count {
        let orig_key_ref = channel_keys_buf[orig_i] as CFStringRef;
        if orig_key_ref.is_null() {
            continue;
        }
        
        let orig_key_str = CFString::wrap_under_get_rule(orig_key_ref);
        let orig_key = orig_key_str.to_string();
        
        // Skip metadata keys
        if orig_key == "QueryOpts" || orig_key == "IOReportChannels" {
            continue;
        }
        
        // Try exact key match first
        if orig_key == channel_key {
            let orig_value_ptr = channel_values_buf[orig_i];
            if orig_value_ptr.is_null() {
                continue;
            }
            
            let orig_value_type_id = CFGetTypeID(orig_value_ptr as CFTypeRef);
            let dict_type_id = CFDictionaryGetTypeID();
            if orig_value_type_id != dict_type_id {
                continue;
            }
            
            let orig_channel_ref = orig_value_ptr as CFDictionaryRef;
            let test_name_ref = IOReportChannelGetChannelName(orig_channel_ref);
            if !test_name_ref.is_null() {
                return Some((orig_channel_ref, test_name_ref));
            }
        }
    }
    
    // Fallback: if exact match not found, return first performance channel (legacy behavior)
    // This should rarely be needed since array path is dominant
    for orig_i in 0..channels_count {
        let orig_key_ref = channel_keys_buf[orig_i] as CFStringRef;
        if orig_key_ref.is_null() {
            continue;
        }
        
        let orig_value_ptr = channel_values_buf[orig_i];
        if orig_value_ptr.is_null() {
            continue;
        }
        
        let orig_value_type_id = CFGetTypeID(orig_value_ptr as CFTypeRef);
        let dict_type_id = CFDictionaryGetTypeID();
        if orig_value_type_id != dict_type_id {
            continue;
        }
        
        let orig_channel_ref = orig_value_ptr as CFDictionaryRef;
        let test_name_ref = IOReportChannelGetChannelName(orig_channel_ref);
        if test_name_ref.is_null() {
            continue;
        }
        
        let test_channel_name = CFString::wrap_under_get_rule(test_name_ref);
        let test_channel_name_str = test_channel_name.to_string();
        
        if is_performance_channel(&test_channel_name_str) {
            return Some((orig_channel_ref, test_name_ref));
        }
    }
    
    None
}

/// Process channels from an array (IOReportChannels can be an array)
/// Returns (FrequencyData, None) since we don't need to store the array sample
unsafe fn process_array_channels(
    array_ptr: *const c_void,
    array_count: i32,
    _orig_channels: CFDictionaryRef,
    _channel_keys_buf: &[*const c_void],
    _channel_values_buf: &[*const c_void],
    _channels_count: usize,
    freq_logging: bool,
) -> (FrequencyData, Option<CFDictionaryRef>) {
    use crate::{debug1, debug2};
    
    let mut accumulator = FrequencyAccumulator::default();
    debug2!("Processing {} channels from array", array_count);
    
    for i in 0..array_count {
        let channel_value_ptr = CFArrayGetValueAtIndex(array_ptr, i);
        if channel_value_ptr.is_null() {
            debug2!("Array element {} is null, skipping", i);
            continue;
        }
        
        let value_type_id = CFGetTypeID(channel_value_ptr as CFTypeRef);
        let dict_type_id = CFDictionaryGetTypeID();
        if value_type_id != dict_type_id {
            debug2!("Array element {} is not a dictionary (type_id={}), skipping", i, value_type_id);
            continue;
        }
        
        let channel_ref = channel_value_ptr as CFDictionaryRef;
        
        // Get channel name directly from the channel dictionary
        let channel_name_ref = IOReportChannelGetChannelName(channel_ref);
        if channel_name_ref.is_null() {
            debug2!("Array element {}: channel name is null, skipping", i);
            continue;
        }
        
        let channel_name = CFString::wrap_under_get_rule(channel_name_ref);
        let channel_name_str = channel_name.to_string();
        debug2!("Array element {}: Processing channel '{}'", i, channel_name_str);
        
        let (is_p_core, is_e_core) = classify_channel(&channel_name_str);
        
        if is_performance_channel(&channel_name_str) {
            debug2!("Found performance channel in array: '{}' (P-core: {}, E-core: {})", 
                channel_name_str, is_p_core, is_e_core);
            parse_channel_states(channel_ref, &channel_name_str, is_p_core, is_e_core, &mut accumulator, freq_logging);
        } else {
            debug2!("Array element {}: Channel '{}' is NOT a performance channel, skipping", i, channel_name_str);
        }
    }
    
    let result = calculate_frequencies(&accumulator, freq_logging);
    if freq_logging {
        debug1!("=== ARRAY PROCESSING END: Overall={:.2} GHz, P-core={:.2} GHz, E-core={:.2} GHz ===", 
            result.overall, result.p_core, result.e_core);
    }
    // Note: We return None for the sample since we're processing an array directly,
    // not a dictionary sample that needs to be stored
    (result, None)
}

/// Process actual channels and extract frequency data
unsafe fn process_actual_channels(
    actual_channels_ref: CFDictionaryRef,
    orig_channels: CFDictionaryRef,
    channel_keys_buf: &[*const c_void],
    channel_values_buf: &[*const c_void],
    channels_count: usize,
    accumulator: &mut FrequencyAccumulator,
    freq_logging: bool,
) {
    use crate::debug2;
    
    let actual_channels_count = CFDictionaryGetCount(actual_channels_ref);
    debug2!("IOReportChannels contains {} actual channels", actual_channels_count);
    
    if actual_channels_count == 0 {
        debug2!("No channels found in IOReportChannels, cannot process frequencies");
        return;
    }
    
    let mut actual_channel_keys: Vec<*const c_void> = vec![std::ptr::null(); actual_channels_count as usize];
    let mut actual_channel_values: Vec<*const c_void> = vec![std::ptr::null(); actual_channels_count as usize];
    
    CFDictionaryGetKeysAndValues(
        actual_channels_ref,
        actual_channel_keys.as_mut_ptr(),
        actual_channel_values.as_mut_ptr(),
    );
    
    debug2!("Iterating through {} actual channels to find performance states", actual_channels_count);
    
    let mut channels_processed = 0;
    let mut performance_channels_found = 0;
    
    for i in 0..(actual_channels_count as usize) {
        let actual_key_ref = actual_channel_keys[i] as CFStringRef;
        if actual_key_ref.is_null() {
            debug2!("Entry {}: key is null, skipping", i);
            continue;
        }
        
        let channel_key_str = CFString::wrap_under_get_rule(actual_key_ref);
        let channel_key = channel_key_str.to_string();
        debug2!("Entry {}: channel_key='{}'", i, channel_key);
        
        // Get the channel value from the sample (contains residency data)
        let sample_channel_value = actual_channel_values[i];
        if sample_channel_value.is_null() {
            debug2!("Entry {}: channel value is null, skipping", i);
            continue;
        }
        
        let value_type_id = CFGetTypeID(sample_channel_value as CFTypeRef);
        let dict_type_id = CFDictionaryGetTypeID();
        if value_type_id != dict_type_id {
            debug2!("Entry {}: channel value is not a dict (type_id={}), skipping", i, value_type_id);
            continue;
        }
        
        let sample_channel_ref = sample_channel_value as CFDictionaryRef;
        
        // Get channel name from the sample channel (or fallback to orig_channels lookup)
        let channel_name_ref = IOReportChannelGetChannelName(sample_channel_ref);
        let channel_name_str = if !channel_name_ref.is_null() {
            let channel_name = CFString::wrap_under_get_rule(channel_name_ref);
            channel_name.to_string()
        } else {
            debug2!("Entry {}: channel_name_ref is null, trying fallback lookup", i);
            // Fallback: try to get name from orig_channels
            match lookup_channel_dict(
                &channel_key,
                orig_channels,
                &channel_keys_buf,
                &channel_values_buf,
                channels_count,
            ) {
                Some((_, name_ref)) => {
                    let channel_name = CFString::wrap_under_get_rule(name_ref);
                    channel_name.to_string()
                }
                None => {
                    debug2!("Entry {}: Could not get channel name from fallback, skipping", i);
                    continue;
                }
            }
        };
        
        debug2!("Entry {}: Processing channel: name='{}'", i, channel_name_str);
        
        let (is_p_core, is_e_core) = classify_channel(&channel_name_str);
        
        debug2!("Entry {}: Channel '{}' classification: is_p_core={}, is_e_core={}", 
            i, channel_name_str, is_p_core, is_e_core);
        
        channels_processed += 1;
        
        let is_perf = is_performance_channel(&channel_name_str);
        debug2!("Entry {}: is_performance_channel('{}') = {}", i, channel_name_str, is_perf);
        
        if is_perf {
            performance_channels_found += 1;
            debug2!("Found performance state channel: '{}' (P-core: {}, E-core: {})", 
                channel_name_str, is_p_core, is_e_core);
            
            // CRITICAL: Use sample_channel_ref (from sample) which has residency data
            // not channel_ref from orig_channels which only has definitions
            parse_channel_states(sample_channel_ref, &channel_name_str, is_p_core, is_e_core, accumulator, freq_logging);
        } else {
            debug2!("Entry {}: Channel '{}' is NOT a performance channel, skipping", i, channel_name_str);
        }
    }
    
    debug2!("Processed {} channels, found {} performance channels", channels_processed, performance_channels_found);
}

/// Calculate final frequencies from accumulator
fn calculate_frequencies(accumulator: &FrequencyAccumulator, freq_logging: bool) -> FrequencyData {
    use crate::{debug1, debug2, debug3};
    
    let mut result = FrequencyData::default();
    
    // Calculate overall frequency
    if accumulator.total_residency > 0.0 {
        result.overall = (accumulator.weighted_freq_sum / accumulator.total_residency / 1000.0) as f32;
        if freq_logging {
            debug1!("Overall frequency: {:.2} GHz (weighted average, total_residency={:.3} s, weighted_sum={:.2} MHz)", 
                result.overall, accumulator.total_residency, accumulator.weighted_freq_sum);
        } else {
            debug2!("IOReport frequency parsed: {:.2} GHz (weighted average)", result.overall);
        }
    } else if accumulator.max_freq_mhz > 0.0 {
        result.overall = (accumulator.max_freq_mhz / 1000.0) as f32;
        if freq_logging {
            debug1!("Overall frequency: {:.2} GHz (max frequency)", result.overall);
        } else {
            debug2!("IOReport frequency parsed: {:.2} GHz (max frequency)", result.overall);
        }
    } else {
        if freq_logging {
            debug1!("Could not extract overall frequency from IOReport");
        } else {
            debug3!("Could not extract overall frequency from IOReport");
        }
    }
    
    // Calculate P-core frequency
    if accumulator.p_core_total_residency > 0.0 {
        result.p_core = (accumulator.p_core_weighted_freq_sum / accumulator.p_core_total_residency / 1000.0) as f32;
        if freq_logging {
            debug1!("P-core frequency: {:.2} GHz (weighted average, total_residency={:.3} s, weighted_sum={:.2} MHz)", 
                result.p_core, accumulator.p_core_total_residency, accumulator.p_core_weighted_freq_sum);
        } else {
            debug2!("IOReport P-core frequency parsed: {:.2} GHz (weighted average)", result.p_core);
        }
    } else if accumulator.p_core_max_freq_mhz > 0.0 {
        result.p_core = (accumulator.p_core_max_freq_mhz / 1000.0) as f32;
        if freq_logging {
            debug1!("P-core frequency: {:.2} GHz (max frequency)", result.p_core);
        } else {
            debug2!("IOReport P-core frequency parsed: {:.2} GHz (max frequency)", result.p_core);
        }
    } else {
        if freq_logging {
            debug1!("P-core frequency: NOT FOUND (p_core_total_residency={:.3} s, p_core_max_freq_mhz={:.2} MHz)", 
                accumulator.p_core_total_residency, accumulator.p_core_max_freq_mhz);
        }
    }
    
    // Calculate E-core frequency
    if accumulator.e_core_total_residency > 0.0 {
        result.e_core = (accumulator.e_core_weighted_freq_sum / accumulator.e_core_total_residency / 1000.0) as f32;
        if freq_logging {
            debug1!("E-core frequency: {:.2} GHz (weighted average, total_residency={:.3} s, weighted_sum={:.2} MHz)", 
                result.e_core, accumulator.e_core_total_residency, accumulator.e_core_weighted_freq_sum);
        } else {
            debug2!("IOReport E-core frequency parsed: {:.2} GHz (weighted average)", result.e_core);
        }
    } else if accumulator.e_core_max_freq_mhz > 0.0 {
        result.e_core = (accumulator.e_core_max_freq_mhz / 1000.0) as f32;
        if freq_logging {
            debug1!("E-core frequency: {:.2} GHz (max frequency)", result.e_core);
        } else {
            debug2!("IOReport E-core frequency parsed: {:.2} GHz (max frequency)", result.e_core);
        }
    } else {
        if freq_logging {
            debug1!("E-core frequency: NOT FOUND (e_core_total_residency={:.3} s, e_core_max_freq_mhz={:.2} MHz)", 
                accumulator.e_core_total_residency, accumulator.e_core_max_freq_mhz);
        }
    }
    
    result
}

/// Read CPU frequencies from IOReport
/// 
/// This is the main entry point for frequency reading. It handles the entire
/// process of creating samples, finding channels, and extracting frequencies.
/// 
/// If last_sample is provided, computes a delta sample for recent frequency.
/// Otherwise, uses the raw sample (absolute counters since boot).
pub unsafe fn read_frequencies_from_ioreport(
    subscription_ptr: *const c_void,
    channels_ref: CFMutableDictionaryRef,
    orig_channels: Option<CFDictionaryRef>,
    last_sample: Option<CFDictionaryRef>,
    freq_logging: bool,
) -> (FrequencyData, Option<CFDictionaryRef>) {
    use crate::{debug1, debug2, debug3};
    
    if freq_logging {
        debug1!("=== FREQUENCY READ START ===");
    }
    
    let mut accumulator = FrequencyAccumulator::default();
    
    // Create current sample from subscription
    // CRITICAL: The sample contains the actual state data with residency times
    // CRITICAL: IOReportCreateSamples returns a CFDictionaryRef that must be released
    let current_sample = IOReportCreateSamples(
        subscription_ptr,
        channels_ref,
        std::ptr::null(),
    );
    
    if current_sample.is_null() {
        debug2!("Failed to create IOReport sample (sample is null)");
        return (FrequencyData::default(), None);
    }
    
    // Use a guard to ensure current_sample is released on all exit paths
    // But we'll release it manually if we need to return it for storage
    struct SampleGuard(CFDictionaryRef, bool);
    impl Drop for SampleGuard {
        fn drop(&mut self) {
            if !self.1 && !self.0.is_null() {
                unsafe {
                    CFRelease(self.0 as CFTypeRef);
                }
            }
        }
    }
    let mut sample_guard = SampleGuard(current_sample, false);
    
    // Compute delta sample if we have a last sample (for recent frequency)
    // Otherwise use the raw sample (absolute counters)
    let sample_to_parse = if let Some(last) = last_sample {
        if freq_logging {
            debug1!("Computing delta sample from last sample");
        }
        let delta = IOReportCreateSamplesDelta(
            last,
            sample_guard.0,
            std::ptr::null(),
        );
        
        if delta.is_null() {
            debug2!("Failed to create delta sample, using raw sample");
            sample_guard.0
        } else {
            if freq_logging {
                debug1!("Using delta sample for recent frequency calculation");
            }
            // We'll parse the delta, but keep current_sample for next iteration
            delta
        }
    } else {
        if freq_logging {
            debug1!("No last sample available, using raw sample (absolute counters)");
        }
        sample_guard.0
    };
    
    // Guard for delta sample (if we created one)
    // We need to keep track of whether we created a delta to release it later
    // CRITICAL: Use a guard to ensure delta sample stays alive during processing
    let created_delta = sample_to_parse != sample_guard.0;
    let delta_guard = if created_delta {
        Some(SampleGuard(sample_to_parse, false))
    } else {
        None
    };
    
    let sample = sample_to_parse;
    
    // Get original channels dictionary (for channel name lookup)
    let orig_channels = match orig_channels {
        Some(ch) => ch,
        None => {
            debug2!("Original channels_dict not available, cannot parse frequency");
            // Release delta if we created one (guard will drop and release)
            drop(delta_guard);
            // Release current sample
            sample_guard.1 = true; // Prevent automatic release
            unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
            return (FrequencyData::default(), None);
        }
    };
    
    // Get channel keys and values from orig_channels for name lookup (needed for array processing too)
    let channels_count = CFDictionaryGetCount(orig_channels) as usize;
    if channels_count == 0 {
        debug3!("Original channels_dict is empty (no channels)");
        // Release delta if we created one (guard will drop and release)
        drop(delta_guard);
        // Release current sample
        sample_guard.1 = true; // Prevent automatic release
        unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
        return (FrequencyData::default(), None);
    }
    
    let mut channel_keys_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count];
    let mut channel_values_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count];
    
    CFDictionaryGetKeysAndValues(
        orig_channels,
        channel_keys_buf.as_mut_ptr(),
        channel_values_buf.as_mut_ptr(),
    );
    
    // CRITICAL: Extract IOReportChannels dictionary from sample
    // The sample has structure: { "IOReportChannels" -> [array of channel_dicts] or {dict of channel_dicts} }
    let sample_keys_count = CFDictionaryGetCount(sample);
    debug2!("Sample dictionary has {} keys", sample_keys_count);
    
    let mut sample_keys_buf: Vec<*const c_void> = vec![std::ptr::null(); sample_keys_count as usize];
    let mut sample_values_buf: Vec<*const c_void> = vec![std::ptr::null(); sample_keys_count as usize];
    CFDictionaryGetKeysAndValues(sample, sample_keys_buf.as_mut_ptr(), sample_values_buf.as_mut_ptr());
    
    // Find IOReportChannels value
    let sample_channels_ref = {
        let mut found: Option<CFDictionaryRef> = None;
        for i in 0..(sample_keys_count as usize) {
            let key_ref = sample_keys_buf[i] as CFStringRef;
            if !key_ref.is_null() {
                let key_type_id = CFGetTypeID(key_ref as CFTypeRef);
                let string_type_id = CFStringGetTypeID();
                if key_type_id == string_type_id {
                    let key_str = CFString::wrap_under_get_rule(key_ref);
                    let key_name = key_str.to_string();
                    debug2!("Sample key[{}]: '{}'", i, key_name);
                    if key_name == "IOReportChannels" {
                        let value_ptr = sample_values_buf[i];
                        if !value_ptr.is_null() {
                            let value_type_id = CFGetTypeID(value_ptr as CFTypeRef);
                            let dict_type_id = CFDictionaryGetTypeID();
                            let array_type_id = CFArrayGetTypeID();
                            debug2!("IOReportChannels value: type_id={}, dict_type_id={}, array_type_id={}", value_type_id, dict_type_id, array_type_id);
                            
                            if value_type_id == dict_type_id {
                                debug2!("Successfully extracted IOReportChannels dictionary from sample");
                                found = Some(value_ptr as CFDictionaryRef);
                                break;
                            } else if value_type_id == array_type_id {
                                // IOReportChannels is an array of channel dictionaries
                                debug2!("IOReportChannels value is an array - processing directly");
                                let array_ptr = value_ptr as *const c_void;
                                let array_count = CFArrayGetCount(array_ptr);
                                debug2!("IOReportChannels array has {} elements", array_count);
                                // Process array first (while delta sample is still valid - delta_guard keeps it alive)
                                let (result, _) = process_array_channels(
                                    array_ptr,
                                    array_count,
                                    orig_channels,
                                    &channel_keys_buf,
                                    &channel_values_buf,
                                    channels_count,
                                    freq_logging,
                                );
                                // Release delta if we created one (after processing is done - guard will drop)
                                drop(delta_guard);
                                // Return current sample for storage
                                sample_guard.1 = true; // Prevent release
                                return (result, Some(sample_guard.0));
                            } else {
                                debug2!("IOReportChannels value is not a dictionary or array (type_id={}, expected_dict={})", value_type_id, dict_type_id);
                            }
                        } else {
                            debug2!("IOReportChannels value pointer is null");
                        }
                    }
                } else {
                    debug2!("Sample key[{}] is not a string (type_id={})", i, key_type_id);
                }
            }
        }
        match found {
            Some(ch) => ch,
            None => {
                debug2!("Failed to extract IOReportChannels from sample, cannot parse frequency");
                // Release delta if we created one (guard will drop and release)
                drop(delta_guard);
                // Release current sample (we won't store it if we can't parse)
                sample_guard.1 = true; // Prevent automatic release
                unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
                return (FrequencyData::default(), None);
            }
        }
    };
    
    // Process actual channels from SAMPLE (which has residency data)
    // but use orig_channels for name/metadata lookup
    process_actual_channels(
        sample_channels_ref,
        orig_channels,
        &channel_keys_buf,
        &channel_values_buf,
        channels_count,
        &mut accumulator,
        freq_logging,
    );
    
    // Debug: Check accumulator state
    if freq_logging || accumulator.total_residency == 0.0 {
        debug2!("Accumulator after processing: total_residency={:.3}, max_freq={:.2} MHz, p_core_residency={:.3}, e_core_residency={:.3}",
            accumulator.total_residency, accumulator.max_freq_mhz, accumulator.p_core_total_residency, accumulator.e_core_total_residency);
    }
    
    // Calculate final frequencies
    let result = calculate_frequencies(&accumulator, freq_logging);
    
    if freq_logging {
        debug1!("=== FREQUENCY READ END: Overall={:.2} GHz, P-core={:.2} GHz, E-core={:.2} GHz ===", 
            result.overall, result.p_core, result.e_core);
    }
    
    // Release delta sample if we created one (guard will drop and release automatically)
    drop(delta_guard);
    
    // Return the current sample for storage (don't release it yet)
    sample_guard.1 = true; // Prevent automatic release
    (result, Some(sample_guard.0))
}

/// Read CPU and GPU power consumption from IOReport
/// 
/// This function reads power/energy channels from IOReport and calculates
/// power consumption in watts by computing energy deltas over time.
/// 
/// Power channels are typically in groups like:
/// - "CPU Stats" / "CPU Power" or "CPU Energy"
/// - "GPU Stats" / "GPU Power" or "GPU Energy"
/// 
/// Returns (PowerData, Option<CFDictionaryRef>) where the dictionary is the
/// current sample for delta calculation on next call.
pub unsafe fn read_power_from_ioreport(
    subscription_ptr: *const c_void,
    channels_ref: CFMutableDictionaryRef,
    orig_channels: Option<CFDictionaryRef>,
    last_sample: Option<CFDictionaryRef>,
    last_read_time: Option<Instant>,
    power_logging: bool,
) -> (PowerData, Option<CFDictionaryRef>) {
    use crate::{debug1, debug2};
    
    debug1!("=== POWER READ START ===");
    debug1!("subscription_ptr={:p}, channels_ref={:p}, orig_channels.is_some()={}", 
        subscription_ptr, channels_ref, orig_channels.is_some());
    
    // Validate inputs
    if subscription_ptr.is_null() {
        debug1!("ERROR: subscription_ptr is null!");
        return (PowerData::default(), None);
    }
    if channels_ref.is_null() {
        debug1!("ERROR: channels_ref is null!");
        return (PowerData::default(), None);
    }
    
    let mut cpu_energy_total: i64 = 0;
    let mut gpu_energy_total: i64 = 0;
    
    // Create current sample from subscription
    debug1!("Creating IOReport power sample...");
    let current_sample = IOReportCreateSamples(
        subscription_ptr,
        channels_ref,
        std::ptr::null(),
    );
    
    if current_sample.is_null() {
        debug1!("ERROR: Failed to create IOReport power sample (sample is null)");
        return (PowerData::default(), None);
    }
    debug1!("IOReport power sample created successfully: {:p}", current_sample);
    
    // Use a guard to ensure current_sample is released on all exit paths
    struct SampleGuard(CFDictionaryRef, bool);
    impl Drop for SampleGuard {
        fn drop(&mut self) {
            if !self.1 && !self.0.is_null() {
                unsafe {
                    CFRelease(self.0 as CFTypeRef);
                }
            }
        }
    }
    let mut sample_guard = SampleGuard(current_sample, false);
    
    // Compute delta sample if we have a last sample (for recent power)
    // Power = Energy / Time, so we need delta energy and delta time
    let (sample_to_parse, time_delta_secs) = if let (Some(last), Some(last_time)) = (last_sample, last_read_time) {
        let now = Instant::now();
        let time_delta = now.duration_since(last_time).as_secs_f64();
        
        if time_delta > 0.0 && time_delta < 60.0 {
            // Valid time delta (between 0 and 60 seconds)
            if power_logging {
                debug1!("Computing delta power sample (time delta: {:.2}s)", time_delta);
            }
            let delta = IOReportCreateSamplesDelta(
                last,
                sample_guard.0,
                std::ptr::null(),
            );
            
            if delta.is_null() {
                debug2!("Failed to create delta power sample, using raw sample");
                (sample_guard.0, time_delta)
            } else {
                (delta, time_delta)
            }
        } else {
            debug2!("Invalid time delta ({:.2}s), using raw sample", time_delta);
            (sample_guard.0, 0.0)
        }
    } else {
        debug1!("No last sample available, using raw sample (absolute counters)");
        (sample_guard.0, 0.0)
    };
    
    debug1!("Sample to parse: {:p}, time_delta={:.2}s", sample_to_parse, time_delta_secs);
    
    // Guard for delta sample (if we created one)
    let created_delta = sample_to_parse != sample_guard.0;
    let delta_guard = if created_delta {
        Some(SampleGuard(sample_to_parse, false))
    } else {
        None
    };
    
    let sample = sample_to_parse;
    debug1!("Using sample: {:p} for power parsing", sample);
    
    // Get original channels dictionary (for channel name lookup)
    debug1!("Checking original channels dict... orig_channels.is_some()={}", orig_channels.is_some());
    let orig_channels = match orig_channels {
        Some(ch) => {
            debug1!("Original power channels_dict available: {:p}", ch);
            ch
        },
        None => {
            debug1!("ERROR: Original power channels_dict not available, cannot parse power - returning 0.0W");
            drop(delta_guard);
            sample_guard.1 = true;
            unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
            return (PowerData::default(), None);
        }
    };
    
    debug1!("Extracting IOReportChannels from sample (sample={:p})...", sample);
    // Extract IOReportChannels from sample
    let sample_keys_count = CFDictionaryGetCount(sample);
    debug1!("Power sample dictionary has {} keys", sample_keys_count);
    
    if sample_keys_count == 0 {
        debug1!("Power sample dictionary is empty!");
        drop(delta_guard);
        sample_guard.1 = true;
        unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
        return (PowerData::default(), None);
    }
    
    let mut sample_keys_buf: Vec<*const c_void> = vec![std::ptr::null(); sample_keys_count as usize];
    let mut sample_values_buf: Vec<*const c_void> = vec![std::ptr::null(); sample_keys_count as usize];
    CFDictionaryGetKeysAndValues(sample, sample_keys_buf.as_mut_ptr(), sample_values_buf.as_mut_ptr());
    
    // Log sample keys for debugging
    for i in 0..(sample_keys_count as usize) {
        let key_ref = sample_keys_buf[i] as CFStringRef;
        if !key_ref.is_null() {
            let key_type_id = CFGetTypeID(key_ref as CFTypeRef);
            let string_type_id = CFStringGetTypeID();
            if key_type_id == string_type_id {
                let key_str = CFString::wrap_under_get_rule(key_ref);
                let key_name = key_str.to_string();
                debug1!("Sample key[{}]: '{}'", i, key_name);
                
                // Check value type
                let value_ptr = sample_values_buf[i];
                if !value_ptr.is_null() {
                    let value_type_id = CFGetTypeID(value_ptr as CFTypeRef);
                    let dict_type_id = CFDictionaryGetTypeID();
                    let array_type_id = CFArrayGetTypeID();
                    debug1!("  Value type_id={}, dict_type_id={}, array_type_id={}", value_type_id, dict_type_id, array_type_id);
                    if value_type_id == array_type_id {
                        unsafe {
                            extern "C" {
                                fn CFArrayGetCount(theArray: *const c_void) -> i32;
                            }
                            let array_count = CFArrayGetCount(value_ptr as *const c_void);
                            debug1!("  Array has {} elements", array_count);
                        }
                    }
                }
            }
        }
    }
    
    // Find IOReportChannels value (can be dict or array)
    // CRITICAL: Energy Model uses arrays, so we need to handle both
    let (sample_channels_ref, is_array, array_ptr_opt) = {
        let mut found: Option<(CFDictionaryRef, bool, Option<*const c_void>)> = None;
        for i in 0..(sample_keys_count as usize) {
            let key_ref = sample_keys_buf[i] as CFStringRef;
            if !key_ref.is_null() {
                let key_type_id = CFGetTypeID(key_ref as CFTypeRef);
                let string_type_id = CFStringGetTypeID();
                if key_type_id == string_type_id {
                    let key_str = CFString::wrap_under_get_rule(key_ref);
                    let key_name = key_str.to_string();
                    if key_name == "IOReportChannels" {
                        let value_ptr = sample_values_buf[i];
                        if !value_ptr.is_null() {
                            let value_type_id = CFGetTypeID(value_ptr as CFTypeRef);
                            let dict_type_id = CFDictionaryGetTypeID();
                            let array_type_id = CFArrayGetTypeID();
                            if value_type_id == dict_type_id {
                                found = Some((value_ptr as CFDictionaryRef, false, None));
                                break;
                            } else if value_type_id == array_type_id {
                                // For arrays, we'll process them directly
                                unsafe {
                                    extern "C" {
                                        fn CFArrayGetCount(theArray: *const c_void) -> i32;
                                    }
                                    let array_count = CFArrayGetCount(value_ptr as *const c_void);
                                    if power_logging {
                                        debug1!("IOReportChannels in sample is an array with {} elements", array_count);
                                    }
                                    // Store array pointer for processing
                                    found = Some((std::ptr::null_mut(), true, Some(value_ptr as *const c_void)));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        match found {
            Some((ch, is_arr, arr_ptr)) => (ch, is_arr, arr_ptr),
            None => {
                debug2!("Failed to extract IOReportChannels from power sample");
                drop(delta_guard);
                sample_guard.1 = true;
                unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
                return (PowerData::default(), None);
            }
        }
    };
    
    // If IOReportChannels is an array, process it directly
    if is_array {
        let array_ptr = match array_ptr_opt {
            Some(arr) => arr,
            None => {
                debug2!("Array pointer is None");
                drop(delta_guard);
                sample_guard.1 = true;
                unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
                return (PowerData::default(), None);
            }
        };
        
        // Process array channels directly
        unsafe {
            extern "C" {
                fn CFArrayGetCount(theArray: *const c_void) -> i32;
                fn CFArrayGetValueAtIndex(theArray: *const c_void, idx: i32) -> *const c_void;
            }
            
            let array_count = CFArrayGetCount(array_ptr);
            // Always log channel count (not just when power_logging is enabled)
            debug1!("Processing {} channels from array for power reading", array_count);
            
            // Safety: Process channels but be defensive
            // Process all channels, but stop early if we hit too many errors
            // This allows us to find GPU channels that might be anywhere in the array
            let max_channels = array_count;
            
            // Track all CPU/GPU-related channels for debugging
            let mut cpu_candidates: Vec<String> = Vec::new();
            let mut gpu_candidates: Vec<String> = Vec::new();
            let mut power_candidates: Vec<String> = Vec::new();
            let mut error_count = 0;
            const MAX_ERRORS: i32 = 50; // Stop processing if we hit too many errors (increased to allow more channels)
            let mut consecutive_errors = 0;
            const MAX_CONSECUTIVE_ERRORS: i32 = 20; // Stop if we hit too many consecutive errors
            
            for i in 0..max_channels {
                // Basic bounds check (shouldn't be needed but be safe)
                if i < 0 {
                    debug2!("Invalid array index {} (negative), stopping", i);
                    break;
                }
                
                let channel_value_ptr = CFArrayGetValueAtIndex(array_ptr, i);
                if channel_value_ptr.is_null() {
                    error_count += 1;
                    consecutive_errors += 1;
                    if error_count > MAX_ERRORS || consecutive_errors > MAX_CONSECUTIVE_ERRORS {
                        debug1!("Too many errors encountered (total: {}, consecutive: {}), stopping processing at channel {}", 
                            error_count, consecutive_errors, i);
                        break;
                    }
                    continue;
                }
                consecutive_errors = 0; // Reset on success
                
                // Validate that this is actually a dictionary
                let value_type_id = CFGetTypeID(channel_value_ptr as CFTypeRef);
                let dict_type_id = CFDictionaryGetTypeID();
                if value_type_id != dict_type_id {
                    error_count += 1;
                    consecutive_errors += 1;
                    if error_count > MAX_ERRORS || consecutive_errors > MAX_CONSECUTIVE_ERRORS {
                        debug1!("Too many non-dictionary channels encountered (total: {}, consecutive: {}), stopping processing at channel {}", 
                            error_count, consecutive_errors, i);
                        break;
                    }
                    continue;
                }
                consecutive_errors = 0; // Reset on success
                
                let channel_dict = channel_value_ptr as CFDictionaryRef;
                
                // Get channel name
                let channel_name_ref = IOReportChannelGetChannelName(channel_dict);
                if channel_name_ref.is_null() {
                    error_count += 1;
                    consecutive_errors += 1;
                    if error_count > MAX_ERRORS || consecutive_errors > MAX_CONSECUTIVE_ERRORS {
                        debug1!("Too many channels with null names (total: {}, consecutive: {}), stopping processing at channel {}", 
                            error_count, consecutive_errors, i);
                        break;
                    }
                    continue;
                }
                consecutive_errors = 0; // Reset on success
                
                // Wrap the CFString safely - this should not panic if channel_name_ref is valid
                let channel_name = CFString::wrap_under_get_rule(channel_name_ref);
                let channel_name_str = channel_name.to_string();
                
                // Check if this is a power/energy channel
                // Energy Model channels might have names like "CPU Power", "GPU Power", "Package Power", etc.
                let is_power_channel = channel_name_str.contains("Power") || 
                                      channel_name_str.contains("Energy") ||
                                      channel_name_str.contains("Watt") ||
                                      channel_name_str.contains("Package") ||
                                      channel_name_str.contains("SoC");
                
                // Classify as CPU or GPU (check this BEFORE the is_power_channel check)
                // CPU channels might not have "Power" in the name
                let is_cpu = channel_name_str.contains("CPU") || 
                            channel_name_str.contains("P-Core") || 
                            channel_name_str.contains("E-Core") ||
                            channel_name_str.contains("Package") ||
                            channel_name_str.contains("P-CPU") ||
                            channel_name_str.contains("E-CPU");
                let is_gpu = channel_name_str.contains("GPU");
                
                // Track candidates for debugging
                if is_cpu {
                    cpu_candidates.push(channel_name_str.clone());
                }
                if is_gpu {
                    gpu_candidates.push(channel_name_str.clone());
                }
                if is_power_channel {
                    power_candidates.push(channel_name_str.clone());
                }
                
                // Process if it's a power channel OR if it's CPU/GPU (even without "Power" in name)
                // This is important because some CPU channels might not explicitly say "Power"
                // CRITICAL: Always process GPU channels, even if they have state_count=-1
                // GPU channels like "GPU Energy" were working before and need to be processed
                if is_power_channel || is_cpu || is_gpu {
                    if power_logging {
                        debug1!("Found channel in array: '{}' (is_power={}, is_cpu={}, is_gpu={})", 
                            channel_name_str, is_power_channel, is_cpu, is_gpu);
                    }
                    
                    // Check state count - if it's -1, the channel doesn't support state-based operations
                    // but IOReportSimpleGetIntegerValue might still work (this is how GPU power was working before)
                    let state_count_raw = IOReportStateGetCount(channel_dict);
                    
                    // Get energy value - try index 0 first
                    // This should work even if state_count is -1 (GPU channels work this way)
                    // However, some channels might still crash, so we need to be careful
                    let mut energy_value = IOReportSimpleGetIntegerValue(channel_dict, 0);
                    
                    // Only use state-based operations if state_count is valid
                    // Channels with state_count=-1 don't support state operations, but may have valid energy values
                    let state_count = if state_count_raw < 0 || state_count_raw > 1000 {
                        // Invalid state count - skip state-based operations but still use energy_value
                        0
                    } else {
                        state_count_raw
                    };
                    
                    // Safety check: If we got a suspiciously large energy value, it might be invalid
                    // Energy values are typically in micro-joules or nano-joules, so very large values
                    // (like > 1e15) are likely invalid/corrupted data
                    if energy_value > 1_000_000_000_000_000 || energy_value < -1_000_000_000_000_000 {
                        debug2!("Suspicious energy value {} for channel '{}', treating as 0", energy_value, channel_name_str);
                        energy_value = 0;
                    }
                    
                    // If we got 0 and state_count is valid, we might need to look at states
                    // But if state_count is -1, skip state operations entirely
                    
                    // Only process states if state_count is valid (not -1)
                    if state_count > 0 {
                        if power_logging {
                            debug1!("  Channel '{}' has {} states", channel_name_str, state_count);
                        }
                        
                        // For CPU channels, try extracting energy from states if simple value is 0
                        // Some CPU channels might store energy in state residency
                        if is_cpu && energy_value == 0 {
                            // Try summing state residencies as a fallback
                            let mut state_energy_sum: i64 = 0;
                            for state_idx in 0..state_count {
                                let residency = IOReportStateGetResidency(channel_dict, state_idx);
                                state_energy_sum += residency;
                            }
                            if state_energy_sum > 0 && power_logging {
                                debug1!("  Channel '{}': using state residency sum={} (simple value was 0)", 
                                    channel_name_str, state_energy_sum);
                                energy_value = state_energy_sum;
                            }
                        }
                    }
                    
                    // Try index 1 if index 0 returned 0 (some channels might use different indices)
                    // Only do this if state_count is valid - channels with state_count=-1 might not support multiple indices
                    if energy_value == 0 && state_count_raw != -1 {
                        let energy_value_1 = IOReportSimpleGetIntegerValue(channel_dict, 1);
                        // Safety check: validate the value before using it
                        if energy_value_1 != 0 && energy_value_1 < 1_000_000_000_000_000 && energy_value_1 > -1_000_000_000_000_000 {
                            if power_logging {
                                debug1!("  Channel '{}': index 0 was 0, trying index 1: {}", channel_name_str, energy_value_1);
                            }
                            energy_value = energy_value_1;
                        }
                    }
                    
                    // Always try to classify and add, even if energy_value is 0
                    // Some channels might have 0 energy but still be valid power channels
                    if is_cpu {
                        cpu_energy_total += energy_value;
                        if power_logging && energy_value != 0 {
                            debug1!("  Added to CPU: energy={} (total: {})", energy_value, cpu_energy_total);
                        }
                    } else if is_gpu {
                        gpu_energy_total += energy_value;
                        if power_logging && energy_value != 0 {
                            debug1!("  Added to GPU: energy={} (total: {})", energy_value, gpu_energy_total);
                        }
                    } else if power_logging && (energy_value != 0 || state_count > 0) {
                        debug2!("  Channel '{}' has energy={} but is not CPU or GPU (skipping)", channel_name_str, energy_value);
                    }
                }
                
                // Log progress every 50 channels to help identify crash location
                if (i + 1) % 50 == 0 {
                    debug1!("Processed {} / {} channels (CPU energy: {}, GPU energy: {})", 
                        i + 1, max_channels, cpu_energy_total, gpu_energy_total);
                }
            }
            
            // Log summary of candidates found (always log, not just when power_logging is enabled)
            // This is critical for debugging CPU power issues
            debug1!("Channel summary: {} CPU candidates, {} GPU candidates, {} power candidates, cpu_energy_total={}, gpu_energy_total={}, time_delta={:.2}s", 
                cpu_candidates.len(), gpu_candidates.len(), power_candidates.len(), cpu_energy_total, gpu_energy_total, time_delta_secs);
            if !cpu_candidates.is_empty() {
                // Limit to first 10 CPU candidates to avoid log spam
                let display_candidates: Vec<String> = cpu_candidates.iter().take(10).cloned().collect();
                debug1!("CPU candidate channels (first 10): {:?}", display_candidates);
                if cpu_candidates.len() > 10 {
                    debug1!("... and {} more CPU candidates", cpu_candidates.len() - 10);
                }
            } else {
                debug1!("WARNING: No CPU candidate channels found! This explains why CPU power is 0W");
            }
            if !gpu_candidates.is_empty() {
                // Limit to first 10 GPU candidates to avoid log spam
                let display_candidates: Vec<String> = gpu_candidates.iter().take(10).cloned().collect();
                debug1!("GPU candidate channels (first 10): {:?}", display_candidates);
            }
        }
        
        // Calculate power from energy totals
        let mut result = PowerData::default();
        if time_delta_secs > 0.0 && time_delta_secs < 60.0 {
            // Energy Model might report in different units
            // CPU and GPU energy values can be in different units, so we need to try multiple conversions
            
            // Calculate CPU power
            // Based on research: CPU energy from IOReport Energy Model is typically in MILLIJOULES (mJ)
            // Power (W) = Energy (mJ) / Time (s) / 1000
            if cpu_energy_total > 0 {
                // Try millijoules first (most common for CPU on Apple Silicon)
                result.cpu_power = (cpu_energy_total as f64 / time_delta_secs / 1_000.0) as f32;
                
                // Sanity check: CPU power should be reasonable (0.1W to 100W for Apple Silicon)
                // If unreasonably high, try microjoules as fallback
                if result.cpu_power > 100.0 {
                    result.cpu_power = (cpu_energy_total as f64 / time_delta_secs / 1_000_000.0) as f32;
                    if power_logging {
                        debug1!("CPU power: millijoules gave {:.2}W (too high), trying microjoules: {:.2}W", 
                            (cpu_energy_total as f64 / time_delta_secs / 1_000.0) as f32, result.cpu_power);
                    }
                }
            }
            
            // Calculate GPU power
            // Based on research: GPU energy from IOReport Energy Model is typically in MICROJOULES (μJ)
            // Power (W) = Energy (μJ) / Time (s) / 1,000,000
            if gpu_energy_total > 0 {
                // Try microjoules first (most common for GPU on Apple Silicon)
                result.gpu_power = (gpu_energy_total as f64 / time_delta_secs / 1_000_000.0) as f32;
                
                // Sanity check: GPU power should be reasonable (0.1W to 200W for Apple Silicon)
                // If unreasonably high, try nanojoules as fallback
                if result.gpu_power > 200.0 {
                    result.gpu_power = (gpu_energy_total as f64 / time_delta_secs / 1_000_000_000.0) as f32;
                    if power_logging {
                        debug1!("GPU power: microjoules gave {:.2}W (too high), trying nanojoules: {:.2}W", 
                            (gpu_energy_total as f64 / time_delta_secs / 1_000_000.0) as f32, result.gpu_power);
                    }
                }
            }
            
            if power_logging {
                debug1!("Power calculated: CPU={:.2}W, GPU={:.2}W (energy: CPU={}, GPU={}, time={:.2}s)", 
                    result.cpu_power, result.gpu_power, cpu_energy_total, gpu_energy_total, time_delta_secs);
            }
        } else {
            // time_delta is 0 or invalid - cannot calculate power
            // Return 0.0 for both (cache update logic will preserve previous values)
            if power_logging {
                debug1!("Cannot calculate power: time_delta={:.2}s (energy: CPU={}, GPU={})", 
                    time_delta_secs, cpu_energy_total, gpu_energy_total);
            }
        }
        
        drop(delta_guard);
        sample_guard.1 = true;
        return (result, Some(sample_guard.0));
    }
    
    // Continue with dictionary processing (original code)
    
    // Get channel keys and values from orig_channels
    let channels_count = CFDictionaryGetCount(orig_channels) as usize;
    if channels_count == 0 {
        debug2!("Original power channels_dict is empty");
        drop(delta_guard);
        sample_guard.1 = true;
        unsafe { CFRelease(sample_guard.0 as CFTypeRef); }
        return (PowerData::default(), None);
    }
    
    let mut channel_keys_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count];
    let mut channel_values_buf: Vec<*const c_void> = vec![std::ptr::null(); channels_count];
    CFDictionaryGetKeysAndValues(orig_channels, channel_keys_buf.as_mut_ptr(), channel_values_buf.as_mut_ptr());
    
    // Process channels from sample (dictionary case)
    // Note: Array case is handled above
    let sample_channels_count = CFDictionaryGetCount(sample_channels_ref);
    debug2!("Power sample contains {} channels (dictionary)", sample_channels_count);
    
    let mut sample_channel_keys: Vec<*const c_void> = vec![std::ptr::null(); sample_channels_count as usize];
    let mut sample_channel_values: Vec<*const c_void> = vec![std::ptr::null(); sample_channels_count as usize];
    CFDictionaryGetKeysAndValues(sample_channels_ref, sample_channel_keys.as_mut_ptr(), sample_channel_values.as_mut_ptr());
    
    // Iterate through channels and extract power/energy values
    if power_logging {
        debug1!("Processing {} power channels from sample", sample_channels_count);
    }
    
    for i in 0..(sample_channels_count as usize) {
        let channel_key_ref = sample_channel_keys[i] as CFStringRef;
        if channel_key_ref.is_null() {
            continue;
        }
        
        let channel_key_str = CFString::wrap_under_get_rule(channel_key_ref);
        let channel_key = channel_key_str.to_string();
        
        let channel_value = sample_channel_values[i];
        if channel_value.is_null() {
            continue;
        }
        
        // Get channel name from original channels
        let channel_name = {
            let mut found_name: Option<String> = None;
            for j in 0..channels_count {
                let orig_key_ref = channel_keys_buf[j] as CFStringRef;
                if orig_key_ref.is_null() {
                    continue;
                }
                let orig_key_str = CFString::wrap_under_get_rule(orig_key_ref);
                if orig_key_str.to_string() == channel_key {
                    let orig_value = channel_values_buf[j];
                    if !orig_value.is_null() {
                        let orig_value_type_id = CFGetTypeID(orig_value as CFTypeRef);
                        let dict_type_id = CFDictionaryGetTypeID();
                        if orig_value_type_id == dict_type_id {
                            let channel_dict = orig_value as CFDictionaryRef;
                            let name_ref = IOReportChannelGetChannelName(channel_dict);
                            if !name_ref.is_null() {
                                let name = CFString::wrap_under_get_rule(name_ref);
                                found_name = Some(name.to_string());
                                break;
                            }
                        }
                    }
                }
            }
            found_name
        };
        
        if let Some(name) = channel_name {
            if power_logging {
                debug1!("Power channel found: '{}' (key: '{}')", name, channel_key);
            }
            
            // Check if this is a power/energy channel
            // Be more flexible with matching - Energy Model channels might have various names
            // CPU channels: "CPU", "P-Core", "E-Core", "Package", "P-CPU", "E-CPU" (with or without "Power"/"Energy")
            // GPU channels: "GPU" (with or without "Power"/"Energy")
            let is_power_channel = name.contains("Power") || 
                                  name.contains("Energy") ||
                                  name.contains("Package") ||
                                  name.contains("SoC");
            
            let is_cpu_power = (name.contains("CPU") || 
                               name.contains("P-Core") || 
                               name.contains("E-Core") ||
                               name.contains("Package") ||
                               name.contains("P-CPU") ||
                               name.contains("E-CPU")) && 
                               (is_power_channel || name.contains("CPU") || name.contains("Core") || name.contains("Package"));
            
            let is_gpu_power = name.contains("GPU") && (is_power_channel || name.contains("GPU"));
            
            if is_cpu_power || is_gpu_power {
                // Try to extract energy value from channel
                // IOReportSimpleGetIntegerValue can get integer values from channels
                // Energy is typically in micro-joules or nano-joules
                let energy_value = IOReportSimpleGetIntegerValue(channel_value as CFDictionaryRef, 0);
                
                if power_logging {
                    debug1!("Power channel '{}': energy={} (raw value), is_cpu={}, is_gpu={}", 
                        name, energy_value, is_cpu_power, is_gpu_power);
                }
                
                if is_cpu_power {
                    cpu_energy_total += energy_value;
                    if power_logging {
                        debug1!("Added to CPU energy total: {} (new total: {})", energy_value, cpu_energy_total);
                    }
                } else if is_gpu_power {
                    gpu_energy_total += energy_value;
                    if power_logging {
                        debug1!("Added to GPU energy total: {} (new total: {})", energy_value, gpu_energy_total);
                    }
                }
            } else {
                // Log all channels for debugging (even non-power channels) to help diagnose CPU power issue
                if power_logging {
                    debug2!("Channel '{}' is not a power channel (skipping) - is_power_channel={}, contains CPU={}, contains GPU={}", 
                        name, is_power_channel, name.contains("CPU"), name.contains("GPU"));
                }
            }
        } else if power_logging {
            debug2!("Could not find channel name for key '{}'", channel_key);
        }
    }
    
    if power_logging {
        debug1!("Total energy: CPU={}, GPU={}, time_delta={:.2}s", cpu_energy_total, gpu_energy_total, time_delta_secs);
    }
    
    // Calculate power in watts
    // Energy is typically in micro-joules (μJ), so power = energy_μJ / time_s / 1_000_000
    // Or if in nano-joules: power = energy_nJ / time_s / 1_000_000_000
    // We'll assume micro-joules for now (common in IOReport)
    let mut result = PowerData::default();
    
    if time_delta_secs > 0.0 && time_delta_secs < 60.0 {
        // Convert energy (assumed micro-joules) to watts
        // Power (W) = Energy (μJ) / Time (s) / 1,000,000
        if cpu_energy_total > 0 {
            result.cpu_power = (cpu_energy_total as f64 / time_delta_secs / 1_000_000.0) as f32;
            if power_logging {
                debug1!("CPU power: {:.2}W (energy={} μJ, time={:.2}s)", result.cpu_power, cpu_energy_total, time_delta_secs);
            }
        }
        
        if gpu_energy_total > 0 {
            result.gpu_power = (gpu_energy_total as f64 / time_delta_secs / 1_000_000.0) as f32;
            if power_logging {
                debug1!("GPU power: {:.2}W (energy={} μJ, time={:.2}s)", result.gpu_power, gpu_energy_total, time_delta_secs);
            }
        }
    } else {
        debug2!("Cannot calculate power: invalid time delta ({:.2}s)", time_delta_secs);
    }
    
    if power_logging {
        debug1!("=== POWER READ END: CPU={:.2}W, GPU={:.2}W ===", result.cpu_power, result.gpu_power);
    }
    
    // Release delta sample if we created one
    drop(delta_guard);
    
    // Return the current sample for storage
    sample_guard.1 = true;
    (result, Some(sample_guard.0))
}
