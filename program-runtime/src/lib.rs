//! TOS Program Runtime
//!
//! This crate provides the execution runtime for TBPF (TOS Berkeley Packet Filter)
//! programs on the TOS blockchain. It directly integrates with `tos-tbpf` following
//! standard eBPF architectural patterns.
//!
//! # Architecture
//!
//! This crate provides the execution context and infrastructure for running
//! TBPF programs without an additional wrapper layer:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │          TOS Blockchain                 │
//! │  (transaction verification/execution)   │
//! └────────────────┬────────────────────────┘
//!                  │
//!                  │ create InvokeContext
//!                  │ load Executable
//!                  │ create EbpfVm
//!                  ▼
//! ┌─────────────────────────────────────────┐
//! │    tos-program-runtime (this crate)     │
//! │  ┌─────────────────────────────────┐    │
//! │  │    InvokeContext                │    │
//! │  │  - Blockchain state             │    │
//! │  │  - Compute budget tracking      │    │
//! │  │  - Storage access               │    │
//! │  │  (implements ContextObject)     │    │
//! │  └─────────────────────────────────┘    │
//! │  ┌─────────────────────────────────┐    │
//! │  │    Memory Utilities             │    │
//! │  │  - translate_type               │    │
//! │  │  - translate_slice              │    │
//! │  └─────────────────────────────────┘    │
//! └─────────────────────────────────────────┘
//!               │
//!               ▼
//! ┌─────────────────────────────────────────┐
//! │         tos-syscalls (separate)         │
//! │  - tos_log, tos_get_balance, ...        │
//! └─────────────────────────────────────────┘
//!               │
//!               ▼
//! ┌─────────────────────────────────────────┐
//! │         tos-tbpf (eBPF engine)          │
//! │  - EbpfVm (used directly)               │
//! │  - Interpreter / JIT compiler           │
//! │  - Verifier                             │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! // Example of how to use the TOS Program Runtime
//! use tos_program_runtime::{InvokeContext, memory};
//! use tos_tbpf::{
//!     program::BuiltinProgram,
//!     vm::{Config, EbpfVm},
//!     elf::Executable,
//! };
//! use std::sync::Arc;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Create loader with syscalls
//!     let config = Config::default();
//!     let mut loader = BuiltinProgram::<InvokeContext>::new_loader(config);
//!     // Register syscalls (done by tos-syscalls crate)
//!     tos_syscalls::register_syscalls(&mut loader)?;
//!     let loader = Arc::new(loader);
//!
//!     // 2. Load executable
//!     let elf_bytes = include_bytes!("../tests/fixtures/example.so");
//!     let executable = Executable::load(elf_bytes, loader.clone())?;
//!
//!     // 3. Create invoke context
//!     let mut invoke_context = InvokeContext::new(200_000, [0u8; 32]);
//!     invoke_context.enable_debug();
//!
//!     // 4. Create VM and execute
//!     let mut vm = EbpfVm::new(
//!         executable.get_loader().clone(),
//!         executable.get_tbpf_version(),
//!         &mut invoke_context,
//!         executable.get_ro_region(),
//!         executable.get_text_bytes().1,
//!     );
//!
//!     let (instruction_count, result) = vm.execute_program(&executable, true);
//!     // Result is StableResult which contains both success and error cases
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![deny(clippy::arithmetic_side_effects)]

pub mod error;
pub mod invoke_context;
pub mod memory;
pub mod storage;

// Re-export main types
pub use error::{TosVmError, Result};
pub use invoke_context::InvokeContext;
pub use storage::{StorageProvider, AccountProvider, NoOpStorage, NoOpAccounts};
