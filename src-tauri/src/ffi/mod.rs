//! FFI safety shims for foreign function interfaces
//!
//! This module provides safe wrappers around unsafe FFI calls,
//! adding null checks, error handling, and preventing foreign exceptions
//! from crossing into Rust.
//!
//! ## IOReport and SMC unsafe usage
//!
//! Unsafe blocks that call IOReport or SMC (in this crate and in `lib.rs`) rely on
//! documented invariants: CF ownership rules (Create/Copy = release; Get = do not release),
//! null checks on CF types, and single-thread or thread-local use where required. When
//! changing them, preserve those invariants and prefer migrating to the safe wrappers
//! in `ffi/` (e.g. `ioreport`) where feasible.

pub mod ioreport;
pub mod objc;
