//! Logging syscalls for TOS contracts
//!
//! This module provides the tos_log syscall that allows contracts to
//! output debug messages during execution.

use tos_tbpf::{
    declare_builtin_function,
    memory_region::MemoryMapping,
};
use tos_program_runtime::{InvokeContext, memory::translate_slice};
use thiserror::Error as ThisError;

/// Syscall error types
#[derive(Debug, ThisError)]
pub enum SyscallError {
    /// Invalid UTF-8 string in message
    #[error("Invalid UTF-8 string")]
    InvalidString,
    /// Message exceeds maximum length
    #[error("Message too long: {0} bytes (max {1})")]
    MessageTooLong(u64, u64),
    /// Insufficient compute units remaining
    #[error("Out of compute units")]
    OutOfComputeUnits,
}

/// Maximum length of a log message
pub const MAX_LOG_LENGTH: u64 = 10_000;

/// Compute units charged per byte of log message
pub const LOG_COST_PER_BYTE: u64 = 1;

declare_builtin_function!(
    /// Log a UTF-8 encoded message from a contract
    ///
    /// This syscall allows contracts to output debug messages. Messages are only
    /// displayed when the InvokeContext is in debug mode.
    ///
    /// # Arguments (from VM registers)
    /// * `msg_ptr` - Pointer to the message string in VM memory
    /// * `msg_len` - Length of the message in bytes
    /// * `_arg3` - Unused (required by calling convention)
    /// * `_arg4` - Unused (required by calling convention)
    /// * `_arg5` - Unused (required by calling convention)
    ///
    /// # Returns
    /// 0 on success
    ///
    /// # Errors
    /// - `MessageTooLong` - If the message exceeds MAX_LOG_LENGTH
    /// - `OutOfComputeUnits` - If not enough compute units remain
    /// - `InvalidString` - If the message is not valid UTF-8
    ///
    /// # Compute Cost
    /// Base cost (100 CU) + msg_len * LOG_COST_PER_BYTE
    TosLog,
    fn rust(
        invoke_context: &mut InvokeContext,
        msg_ptr: u64,
        msg_len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate message length
        if msg_len > MAX_LOG_LENGTH {
            return Err(Box::new(SyscallError::MessageTooLong(msg_len, MAX_LOG_LENGTH)));
        }

        // Charge compute units: base cost + per-byte cost
        let base_cost = 100u64;
        let total_cost = base_cost.saturating_add(msg_len.saturating_mul(LOG_COST_PER_BYTE));
        invoke_context.consume_checked(total_cost)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate message from VM memory (no alignment check needed for u8 bytes)
        let msg_bytes = translate_slice::<u8>(
            memory_mapping,
            msg_ptr,
            msg_len,
            false, // u8 doesn't need alignment checking
        )?;

        let msg = std::str::from_utf8(msg_bytes)
            .map_err(|_| Box::new(SyscallError::InvalidString) as Box<dyn std::error::Error>)?;

        // Output the log message if debug mode is enabled
        if invoke_context.debug_mode {
            log::info!(
                "[Contract {:02x}{:02x}{:02x}{:02x}...]: {}",
                invoke_context.contract_hash[0],
                invoke_context.contract_hash[1],
                invoke_context.contract_hash[2],
                invoke_context.contract_hash[3],
                msg
            );
        }

        Ok(0)
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use tos_program_runtime::{InvokeContext, NoOpStorage, NoOpAccounts};
    use tos_tbpf::{
        memory_region::{MemoryRegion, MemoryMapping},
        program::TBPFVersion,
        vm::Config,
    };

    fn create_test_mapping(data: &mut [u8]) -> MemoryMapping {
        // Leak config so it lives for 'static - this is fine for tests
        let config: &'static Config = Box::leak(Box::new(Config::default()));
        let region = MemoryRegion::new_writable(data, 0x100000000);
        MemoryMapping::new(vec![region], config, TBPFVersion::V3).unwrap()
    }

    #[test]
    fn test_tos_log_success() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(10_000, [1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], &mut storage, &mut accounts);
        context.enable_debug();

        let mut data = b"Hello, TOS!".to_vec();
        let len = data.len() as u64;
        let mut mapping = create_test_mapping(&mut data);

        let result = TosLog::rust(
            &mut context,
            0x100000000,
            len,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Check compute units were consumed
        let expected_cost = 100 + (data.len() as u64);
        assert_eq!(context.get_compute_units_consumed(), expected_cost);
    }

    #[test]
    fn test_tos_log_too_long() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(100_000, [0u8; 32], &mut storage, &mut accounts);
        let mut data = vec![0u8; 1024];
        let mut mapping = create_test_mapping(&mut data);

        // Try to log a message that's too long
        let result = TosLog::rust(
            &mut context,
            0x100000000,
            MAX_LOG_LENGTH + 1,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_tos_log_insufficient_compute() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(50, [0u8; 32], &mut storage, &mut accounts); // Very low budget
        let mut data = b"Hello".to_vec();
        let len = data.len() as u64;
        let mut mapping = create_test_mapping(&mut data);

        let result = TosLog::rust(
            &mut context,
            0x100000000,
            len,
            0,
            0,
            0,
            &mut mapping,
        );

        // Should fail due to insufficient compute units
        assert!(result.is_err());
    }

    #[test]
    fn test_tos_log_invalid_utf8() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(10_000, [0u8; 32], &mut storage, &mut accounts);
        let mut data = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8
        let len = data.len() as u64;
        let mut mapping = create_test_mapping(&mut data);

        let result = TosLog::rust(
            &mut context,
            0x100000000,
            len,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_tos_log_empty_message() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(10_000, [0u8; 32], &mut storage, &mut accounts);
        context.enable_debug();

        let mut data = b"".to_vec();
        let mut mapping = create_test_mapping(&mut data);

        let result = TosLog::rust(
            &mut context,
            0x100000000,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        // Base cost only
        assert_eq!(context.get_compute_units_consumed(), 100);
    }
}
