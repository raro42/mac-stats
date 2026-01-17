//! FFI safety shims for foreign function interfaces
//! 
//! This module provides safe wrappers around unsafe FFI calls,
//! adding null checks, error handling, and preventing foreign exceptions
//! from crossing into Rust.

//! Foreign Function Interface (FFI) safety module
//! 
//! Provides safe wrappers for unsafe FFI calls:
//! - `ioreport`: Safe wrappers for IOReport framework calls
//! - `objc`: Safe wrappers for Objective-C interop
//! 
//! All FFI functions include null checking, error handling,
//! and proper lifetime management to prevent crashes.

pub mod ioreport;
pub mod objc;
