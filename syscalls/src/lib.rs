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
//! ## Blockchain State
//! - `tos_get_block_hash` - Get current block hash
//! - `tos_get_block_height` - Get current block height
//! - `tos_get_tx_hash` - Get current transaction hash
//! - `tos_get_tx_sender` - Get transaction sender address
//! - `tos_get_contract_hash` - Get executing contract hash
//!
//! ## Account/Balance
//! - `tos_get_balance` - Get account balance
//! - `tos_transfer` - Transfer tokens from contract to account
//!
//! ## Storage
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
pub mod blockchain;
pub mod balance;
pub mod storage;
pub mod return_data;

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

    // Register blockchain state syscalls
    loader.register_function("tos_get_block_hash", blockchain::TosGetBlockHash::vm)?;
    loader.register_function("tos_get_block_height", blockchain::TosGetBlockHeight::vm)?;
    loader.register_function("tos_get_tx_hash", blockchain::TosGetTxHash::vm)?;
    loader.register_function("tos_get_tx_sender", blockchain::TosGetTxSender::vm)?;
    loader.register_function("tos_get_contract_hash", blockchain::TosGetContractHash::vm)?;

    // Register balance/transfer syscalls
    loader.register_function("tos_get_balance", balance::TosGetBalance::vm)?;
    loader.register_function("tos_transfer", balance::TosTransfer::vm)?;

    // Register storage syscalls
    loader.register_function("tos_storage_read", storage::TosStorageRead::vm)?;
    loader.register_function("tos_storage_write", storage::TosStorageWrite::vm)?;
    loader.register_function("tos_storage_delete", storage::TosStorageDelete::vm)?;

    // Register return data syscalls
    loader.register_function("tos_set_return_data", return_data::TosSetReturnData::vm)?;
    loader.register_function("tos_get_return_data", return_data::TosGetReturnData::vm)?;

    Ok(())
}

/// Syscall identifiers
///
/// These constants define the syscall names that contracts use to invoke syscalls.
pub mod syscall_names {
    // Logging
    /// Log a message (debug only)
    pub const TOS_LOG: &[u8] = b"tos_log";

    // Blockchain state
    /// Get current block hash
    pub const TOS_GET_BLOCK_HASH: &[u8] = b"tos_get_block_hash";
    /// Get current block height
    pub const TOS_GET_BLOCK_HEIGHT: &[u8] = b"tos_get_block_height";
    /// Get current transaction hash
    pub const TOS_GET_TX_HASH: &[u8] = b"tos_get_tx_hash";
    /// Get transaction sender
    pub const TOS_GET_TX_SENDER: &[u8] = b"tos_get_tx_sender";
    /// Get contract hash
    pub const TOS_GET_CONTRACT_HASH: &[u8] = b"tos_get_contract_hash";

    // Balance and transfers
    /// Get account balance
    pub const TOS_GET_BALANCE: &[u8] = b"tos_get_balance";
    /// Transfer tokens
    pub const TOS_TRANSFER: &[u8] = b"tos_transfer";

    // Storage
    /// Read from storage
    pub const TOS_STORAGE_READ: &[u8] = b"tos_storage_read";
    /// Write to storage
    pub const TOS_STORAGE_WRITE: &[u8] = b"tos_storage_write";
    /// Delete from storage
    pub const TOS_STORAGE_DELETE: &[u8] = b"tos_storage_delete";

    // Return data
    /// Set return data
    pub const TOS_SET_RETURN_DATA: &[u8] = b"tos_set_return_data";
    /// Get return data
    pub const TOS_GET_RETURN_DATA: &[u8] = b"tos_get_return_data";
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
