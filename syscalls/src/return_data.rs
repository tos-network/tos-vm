//! Return data syscalls for TOS contracts
//!
//! This module provides syscalls for contracts to set and get return data,
//! enabling inter-contract communication (especially important for CPI).

use tos_tbpf::{
    declare_builtin_function,
    memory_region::MemoryMapping,
};
use tos_program_runtime::{InvokeContext, memory::{translate_slice, translate_slice_mut}};
use thiserror::Error as ThisError;

/// Syscall error types
#[derive(Debug, ThisError)]
pub enum SyscallError {
    /// Return data is too large
    #[error("Return data too large: {0} bytes (max {1})")]
    ReturnDataTooLarge(usize, usize),
    /// Buffer too small to hold return data
    #[error("Buffer too small: need {0} bytes, got {1}")]
    BufferTooSmall(usize, u64),
    /// Insufficient compute units remaining
    #[error("Out of compute units")]
    OutOfComputeUnits,
}

/// Maximum return data size (1024 bytes, matching Solana)
pub const MAX_RETURN_DATA: u64 = 1024;

/// Base compute cost for return data operations
pub const RETURN_DATA_SET_BASE_COST: u64 = 100;
pub const RETURN_DATA_GET_COST: u64 = 50;

/// Per-byte cost for setting return data
pub const RETURN_DATA_SET_BYTE_COST: u64 = 1;

declare_builtin_function!(
    /// Set return data
    ///
    /// Sets return data that can be retrieved by the caller.
    /// This is primarily used for cross-program invocation (CPI).
    ///
    /// # Arguments (from VM registers)
    /// * `data_ptr` - Pointer to data bytes
    /// * `data_len` - Length of the data in bytes
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 0 on success
    ///
    /// # Errors
    /// - `ReturnDataTooLarge` - If data exceeds MAX_RETURN_DATA
    /// - `OutOfComputeUnits` - If not enough compute units remain
    TosSetReturnData,
    fn rust(
        invoke_context: &mut InvokeContext,
        data_ptr: u64,
        data_len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate data size
        if data_len > MAX_RETURN_DATA {
            return Err(Box::new(SyscallError::ReturnDataTooLarge(data_len as usize, MAX_RETURN_DATA as usize)));
        }

        // Charge compute units based on data size
        let cost = RETURN_DATA_SET_BASE_COST
            .saturating_add(data_len.saturating_mul(RETURN_DATA_SET_BYTE_COST));
        invoke_context.consume_checked(cost)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate data from VM memory
        let data = translate_slice::<u8>(
            memory_mapping,
            data_ptr,
            data_len,
            false,
        )?;

        // Set return data (program_id is the current contract)
        let program_id = invoke_context.contract_hash;
        invoke_context.set_return_data(program_id, data.to_vec())?;

        Ok(0)
    }
);

declare_builtin_function!(
    /// Get return data
    ///
    /// Retrieves return data from the last invocation.
    ///
    /// # Arguments (from VM registers)
    /// * `data_ptr` - Pointer to buffer to receive the data
    /// * `data_len` - Size of the buffer
    /// * `program_id_ptr` - Pointer to receive the program ID (32 bytes)
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// The actual size of the return data in bytes (0 if no return data)
    ///
    /// # Errors
    /// - `BufferTooSmall` - If buffer is too small for the return data
    /// - `OutOfComputeUnits` - If not enough compute units remain
    TosGetReturnData,
    fn rust(
        invoke_context: &mut InvokeContext,
        data_ptr: u64,
        data_len: u64,
        program_id_ptr: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(RETURN_DATA_GET_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Get return data
        match invoke_context.get_return_data() {
            Some((program_id, data)) => {
                let actual_len = data.len() as u64;
                // SOLANA BEHAVIOR: Truncate to buffer size, don't error
                let length_to_copy = data_len.min(actual_len);

                if length_to_copy != 0 {
                    // Translate output buffers
                    let output_data = translate_slice_mut::<u8>(
                        memory_mapping,
                        data_ptr,
                        length_to_copy,
                        false,
                    )?;

                    let output_program_id = translate_slice_mut::<u8>(
                        memory_mapping,
                        program_id_ptr,
                        32,
                        false,
                    )?;

                    // Copy data (truncated if necessary) and program_id to output
                    output_data.copy_from_slice(&data[..length_to_copy as usize]);
                    output_program_id.copy_from_slice(&program_id);
                }

                // Return actual length, not copied length (Solana behavior)
                Ok(actual_len)
            }
            None => {
                // No return data available
                Ok(0)
            }
        }
    }
);


#[cfg(test)]
mod tests {
    use super::*;
    use tos_program_runtime::storage::{NoOpStorage, NoOpAccounts};
    use tos_tbpf::vm::ContextObject;

    // Basic tests at InvokeContext level (no memory mapping complexity)
    
    #[test]
    fn test_return_data_constants() {
        assert_eq!(MAX_RETURN_DATA, 1024);
        assert!(RETURN_DATA_SET_BASE_COST > 0);
        assert!(RETURN_DATA_GET_COST > 0);
    }

    #[test]
    fn test_invoke_context_return_data() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let context = InvokeContext::new(10_000, [1u8; 32], &mut storage, &mut accounts);

        // Initially no return data
        assert!(context.get_return_data().is_none());

        // Set return data
        let program_id = [5u8; 32];
        let data = vec![1, 2, 3, 4, 5];
        assert!(context.set_return_data(program_id, data.clone()).is_ok());

        // Get return data
        let result = context.get_return_data();
        assert!(result.is_some());
        let (ret_program_id, ret_data) = result.unwrap();
        assert_eq!(ret_program_id, program_id);
        assert_eq!(ret_data, data);

        // Clear return data
        context.clear_return_data();
        assert!(context.get_return_data().is_none());
    }

    #[test]
    fn test_return_data_max_size() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let context = InvokeContext::new(10_000, [1u8; 32], &mut storage, &mut accounts);

        // Exactly max size should work
        let max_data = vec![0u8; 1024];
        assert!(context.set_return_data([2u8; 32], max_data).is_ok());

        // Over max size should fail
        let too_large = vec![0u8; 2000];
        assert!(context.set_return_data([2u8; 32], too_large).is_err());
    }
}
