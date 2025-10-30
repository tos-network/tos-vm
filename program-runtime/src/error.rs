//! Error types for TOS VM

use thiserror::Error;

/// Result type for TOS VM operations
pub type Result<T> = std::result::Result<T, TosVmError>;

/// Errors that can occur during VM execution
#[derive(Debug, Error)]
pub enum TosVmError {
    /// ELF loading or parsing error
    #[error("Failed to load ELF bytecode: {0}")]
    ElfLoadError(String),

    /// Bytecode verification failed
    #[error("Bytecode verification failed: {0}")]
    VerificationFailed(String),

    /// VM execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Out of compute units (gas exhaustion)
    #[error("Out of compute units (gas exhausted)")]
    OutOfComputeUnits,

    /// Memory access violation
    #[error("Memory access violation at address {address:#x}")]
    MemoryAccessViolation {
        /// The address that caused the violation
        address: u64
    },

    /// Invalid syscall
    #[error("Invalid syscall: {name}")]
    InvalidSyscall {
        /// The syscall name
        name: String
    },

    /// Syscall execution error
    #[error("Syscall '{syscall}' failed: {reason}")]
    SyscallError {
        /// The syscall that failed
        syscall: &'static str,
        /// The reason for failure
        reason: String,
    },

    /// Program returned error code
    #[error("Program exited with error code: {0}")]
    ProgramError(u64),

    /// TBPF engine error
    #[error("TBPF error: {0}")]
    TbpfError(String),
}

impl From<tos_tbpf::error::EbpfError> for TosVmError {
    fn from(err: tos_tbpf::error::EbpfError) -> Self {
        TosVmError::TbpfError(err.to_string())
    }
}
