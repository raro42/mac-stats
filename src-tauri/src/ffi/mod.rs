//! FFI safety shims for foreign function interfaces
//! 
//! This module provides safe wrappers around unsafe FFI calls,
//! adding null checks, error handling, and preventing foreign exceptions
//! from crossing into Rust.

pub mod ioreport;
pub mod objc;
