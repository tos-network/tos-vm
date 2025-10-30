//! tos_log syscall - Debug logging

use crate::{context::TosContext, error::TosVmError};
use tos_tbpf::{
    memory_region::{AccessType, MemoryMapping},
    vm::{ContextObject, EbpfVm},
};

/// Syscall: tos_log
///
/// Logs a message from the contract. Only outputs if debug mode is enabled.
///
/// # Arguments
/// - r1: Pointer to message string
/// - r2: Length of message string
///
/// # Returns
/// - 0 on success
///
/// # Example (C)
/// ```c
/// const char *msg = "Hello from contract!";
/// tos_log((uint64_t)msg, strlen(msg));
/// ```
pub struct SyscallLog;

impl SyscallLog {
    /// Execute the tos_log syscall
    pub fn call(
        _vm: &mut EbpfVm<TosContext>,
        msg_ptr: u64,
        msg_len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
        context: &mut TosContext,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate message length to prevent DoS
        const MAX_LOG_LENGTH: u64 = 1024;
        if msg_len > MAX_LOG_LENGTH {
            return Err(Box::new(TosVmError::SyscallError {
                syscall: "tos_log",
                reason: format!("Message length {} exceeds maximum {}", msg_len, MAX_LOG_LENGTH),
            }));
        }

        // Read message from VM memory
        let msg_bytes = translate_slice::<u8>(
            memory_mapping,
            msg_ptr,
            msg_len,
            false, // not writable
        )?;

        // Convert to UTF-8 string
        let msg = std::str::from_utf8(msg_bytes).map_err(|e| {
            Box::new(TosVmError::SyscallError {
                syscall: "tos_log",
                reason: format!("Invalid UTF-8: {}", e),
            }) as Box<dyn std::error::Error>
        })?;

        // Log if debug mode is enabled
        if context.debug_mode {
            log::info!("[Contract] {}", msg);
        }

        // Charge compute units for logging (1 CU per byte)
        context.consume(msg_len);

        Ok(0)
    }
}

/// Helper function to translate VM pointer to host slice
fn translate_slice<'a, T>(
    memory_mapping: &'a MemoryMapping,
    vm_addr: u64,
    len: u64,
    writable: bool,
) -> Result<&'a [T], Box<dyn std::error::Error>> {
    let len_bytes = len
        .checked_mul(std::mem::size_of::<T>() as u64)
        .ok_or_else(|| {
            Box::new(TosVmError::MemoryAccessViolation {
                address: vm_addr,
            }) as Box<dyn std::error::Error>
        })?;

    let access_type = if writable {
        AccessType::Store
    } else {
        AccessType::Load
    };

    let host_addr = memory_mapping
        .map(access_type, vm_addr, len_bytes)
        .map_err(|e| {
            Box::new(TosVmError::SyscallError {
                syscall: "translate_slice",
                reason: e.to_string(),
            }) as Box<dyn std::error::Error>
        })?
        .host_addr;

    Ok(unsafe {
        std::slice::from_raw_parts(host_addr as *const T, len as usize)
    })
}
