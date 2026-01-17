//! Safe wrappers for Objective-C FFI calls
//! 
//! These wrappers add null checks and validation for Objective-C runtime calls
//! to prevent foreign exceptions from crossing into Rust.

use objc2::runtime::{AnyClass, AnyObject, Sel};
use objc2::msg_send;
use thiserror::Error;

/// Objective-C FFI error types
/// Currently unused as direct FFI calls are used in ui module.
/// Kept for future migration to safer FFI patterns.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ObjCError {
    #[error("Objective-C object is null")]
    NullObject,
    
    #[error("Objective-C class not found: {0}")]
    ClassNotFound(String),
    
    #[error("Objective-C selector not found: {0}")]
    SelectorNotFound(String),
    
    #[error("Objective-C method call failed: {0}")]
    MethodCallFailed(String),
    
    #[error("Objective-C object does not respond to selector: {0}")]
    DoesNotRespondToSelector(String),
}

/// Result type for Objective-C operations
#[allow(dead_code)] // Kept for future FFI migration
pub type ObjCResult<T> = Result<T, ObjCError>;

/// Safe wrapper to check if an object responds to a selector
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn responds_to_selector(obj: &AnyObject, sel: Sel) -> bool {
    unsafe {
        let responds: bool = msg_send![obj, respondsToSelector: sel];
        responds
    }
}

/// Safe wrapper to verify an object is not null
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn check_not_null<T>(obj: Option<&T>) -> ObjCResult<&T> {
    obj.ok_or(ObjCError::NullObject)
}

/// Safe wrapper for msg_send! that returns a Result
/// 
/// Note: This is a helper for common patterns. Most msg_send! calls
/// should still use the macro directly but with proper null checks.
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn safe_msg_send_bool(obj: &AnyObject, sel: Sel) -> ObjCResult<bool> {
    if !responds_to_selector(obj, sel) {
        return Err(ObjCError::DoesNotRespondToSelector(
            sel.name().to_string_lossy().to_string()
        ));
    }
    
    unsafe {
        let result: bool = msg_send![obj, sel];
        Ok(result)
    }
}

/// Validate that a class exists and can be used
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn validate_class(class: &'static AnyClass) -> ObjCResult<&'static AnyClass> {
    // Basic validation - class should not be null (though AnyClass is a reference)
    Ok(class)
}

/// Validate that an object instance is valid
/// Currently unused - kept for future FFI migration.
#[allow(dead_code)]
pub fn validate_object(obj: &AnyObject) -> ObjCResult<&AnyObject> {
    // Basic validation - in practice, objc2 handles null checks
    Ok(obj)
}
