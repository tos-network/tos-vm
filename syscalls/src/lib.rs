//! TOS Syscalls
//!
//! This crate provides syscall implementations for TOS contracts running
//! in the TBPF VM. Syscalls allow contracts to interact with the blockchain,
//! access storage, transfer tokens, and perform other operations.
//!
//! # Architecture
//!
//! Syscalls are implemented using the `declare_builtin_function!` macro
//! from tos-tbpf. Each syscall:
//!
//! 1. Takes `InvokeContext` as first parameter for blockchain state access
//! 2. Takes 5 u64 arguments (standard eBPF calling convention)
//! 3. Takes `MemoryMapping` for translating VM memory to host memory
//! 4. Returns `Result<u64, EbpfError>`
//!
//! # Available Syscalls
//!
//! ## Logging
//! - `tos_log` - Output a debug message (only in debug mode)
//!
//! ## Blockchain State (TODO)
//! - `tos_get_block_hash` - Get current block hash
//! - `tos_get_block_height` - Get current block height
//! - `tos_get_tx_hash` - Get current transaction hash
//! - `tos_get_tx_sender` - Get transaction sender address
//!
//! ## Account/Balance (TODO)
//! - `tos_get_balance` - Get account balance
//! - `tos_transfer` - Transfer tokens between accounts
//!
//! ## Storage (TODO)
//! - `tos_storage_read` - Read from contract storage
//! - `tos_storage_write` - Write to contract storage
//! - `tos_storage_delete` - Delete from contract storage
//!
//! # Usage
//!
//! ```rust,ignore
//! use tos_syscalls;
//! use tos_tbpf::program::BuiltinProgram;
//! use tos_tbpf::vm::Config;
//! use tos_program_runtime::InvokeContext;
//!
//! let config = Config::default();
//! let mut loader = BuiltinProgram::<InvokeContext>::new_loader(config);
//!
//! // Register all TOS syscalls
//! tos_syscalls::register_syscalls(&mut loader).unwrap();
//! ```

#![warn(missing_docs)]
#![deny(clippy::arithmetic_side_effects)]

pub mod logging;

use tos_tbpf::{
    program::BuiltinProgram,
    elf::ElfError,
};
use tos_program_runtime::InvokeContext;

/// Register all TOS syscalls with the builtin program loader
///
/// This function registers all available TOS syscalls so they can be
/// called from TBPF contracts.
///
/// # Arguments
/// * `loader` - The builtin program loader to register syscalls with
///
/// # Example
///
/// ```rust,no_run
/// use tos_syscalls;
/// use tos_tbpf::program::BuiltinProgram;
/// use tos_tbpf::vm::Config;
/// use tos_program_runtime::InvokeContext;
///
/// let mut loader = BuiltinProgram::<InvokeContext>::new_loader(Config::default());
/// tos_syscalls::register_syscalls(&mut loader).unwrap();
/// ```
pub fn register_syscalls(loader: &mut BuiltinProgram<InvokeContext>) -> Result<(), ElfError> {
    // Register logging syscalls
    loader.register_function("tos_log", logging::TosLog::vm)?;

    // TODO: Register other syscalls as they are implemented:
    // - Blockchain state syscalls
    // - Balance/transfer syscalls
    // - Storage syscalls

    Ok(())
}

/// Syscall identifiers
///
/// These constants define the syscall names that contracts use to invoke syscalls.
pub mod syscall_names {
    /// Log a message (debug only)
    pub const TOS_LOG: &[u8] = b"tos_log";

    // TODO: Add other syscall names as implemented
    // pub const TOS_GET_BLOCK_HASH: &[u8] = b"tos_get_block_hash";
    // pub const TOS_GET_BALANCE: &[u8] = b"tos_get_balance";
    // pub const TOS_TRANSFER: &[u8] = b"tos_transfer";
    // pub const TOS_STORAGE_READ: &[u8] = b"tos_storage_read";
    // pub const TOS_STORAGE_WRITE: &[u8] = b"tos_storage_write";
}

#[cfg(test)]
mod tests {
    use super::*;
    use tos_tbpf::vm::Config;

    #[test]
    fn test_register_syscalls() {
        let mut loader = BuiltinProgram::<InvokeContext>::new_loader(Config::default());

        // Should not panic or error
        register_syscalls(&mut loader).expect("Failed to register syscalls");

        // Syscalls are registered (no public API to verify, but we got here without error)
    }
}
