//! Safe wrappers for IOReport FFI calls
//! 
//! IOReport is a macOS framework for system performance monitoring.
//! These wrappers add null checks and error handling to prevent crashes.

use std::os::raw::c_void;
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
