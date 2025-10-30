//! TOS VM based on TBPF (TOS Berkeley Packet Filter)
//!
//! This crate provides a high-performance virtual machine for executing eBPF bytecode
//! on the TOS blockchain. It wraps the `tos-tbpf` engine and provides TOS-specific
//! syscalls and execution context.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │          TOS Blockchain                 │
//! │  (transaction verification/execution)   │
//! └────────────────┬────────────────────────┘
//!                  │
//!                  │ invoke_contract()
//!                  ▼
//! ┌─────────────────────────────────────────┐
//! │         tos-vm-tbpf (this crate)        │
//! │  ┌─────────────────────────────────┐    │
//! │  │       TosVm                     │    │
//! │  │  - Load ELF bytecode            │    │
//! │  │  - Setup execution context      │    │
//! │  │  - Register syscalls            │    │
//! │  └──────────┬──────────────────────┘    │
//! │             │                            │
//! │             │ execute()                  │
//! │             ▼                            │
//! │  ┌─────────────────────────────────┐    │
//! │  │    TOS Syscalls                 │    │
//! │  │  - tos_log                      │    │
//! │  │  - tos_get_balance              │    │
//! │  │  - tos_transfer                 │    │
//! │  │  - tos_storage_*                │    │
//! │  │  - ...                          │    │
//! │  └──────────┬──────────────────────┘    │
//! └─────────────┼───────────────────────────┘
//!               │
//!               ▼
//! ┌─────────────────────────────────────────┐
//! │         tos-tbpf (eBPF engine)          │
//! │  - Interpreter / JIT compiler           │
//! │  - Verifier                             │
//! │  - Memory management                    │
//! └─────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]
#![deny(clippy::arithmetic_side_effects)]

pub mod error;
pub mod syscalls;
pub mod vm;
pub mod context;

// Re-export main types
pub use error::{TosVmError, Result};
pub use vm::TosVm;
pub use context::TosContext;
