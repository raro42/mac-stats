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
