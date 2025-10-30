//! TOS-specific syscalls
//!
//! This module implements all syscalls available to TOS smart contracts.

pub mod log;

pub use log::SyscallLog;

use crate::context::TosContext;
use tos_tbpf::program::BuiltinProgram;

/// Register all TOS syscalls with the VM
pub fn register_syscalls(loader: &mut BuiltinProgram<TosContext>) {
    // Basic syscalls
    loader
        .register_function_by_name(b"tos_log", SyscallLog::call)
        .expect("Failed to register tos_log");

    // TODO: Register more syscalls
    // - tos_get_contract_hash
    // - tos_get_balance
    // - tos_transfer
    // - tos_storage_load
    // - tos_storage_store
    // - tos_get_tx_hash
    // - tos_get_block_hash
    // - etc.
}
