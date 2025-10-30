//! TOS VM SDK for smart contract development
//!
//! This SDK provides syscall bindings and utilities for writing TOS smart contracts
//! that compile to TBPF (eBPF) bytecode.
//!
//! # Example
//!
//! ```no_run
//! use tos_vm_sdk::*;
//!
//! #[no_mangle]
//! pub extern "C" fn entrypoint() -> u64 {
//!     log("Hello, TOS!");
//!     0
//! }
//! ```

#![no_std]
#![warn(missing_docs)]

pub mod syscalls;

// Re-export commonly used items
pub use syscalls::*;

/// Program entrypoint result type
pub type ProgramResult = u64;

/// Success return value
pub const SUCCESS: u64 = 0;

/// Error return value
pub const ERROR: u64 = 1;
