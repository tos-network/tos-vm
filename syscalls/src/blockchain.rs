//! Blockchain state syscalls for TOS contracts
//!
//! This module provides syscalls for accessing blockchain state information
//! such as block hash, block height, transaction hash, and sender address.

use tos_tbpf::{
    declare_builtin_function,
    memory_region::MemoryMapping,
};
use tos_program_runtime::{InvokeContext, memory::translate_slice_mut};
use thiserror::Error as ThisError;

/// Syscall error types
#[derive(Debug, ThisError)]
pub enum SyscallError {
    /// Invalid output buffer
    #[error("Invalid output buffer")]
    InvalidBuffer,
    /// Insufficient compute units remaining
    #[error("Out of compute units")]
    OutOfComputeUnits,
}

/// Compute units for blockchain state queries
pub const BLOCKCHAIN_QUERY_COST: u64 = 50;

declare_builtin_function!(
    /// Get the current block hash
    ///
    /// # Arguments (from VM registers)
    /// * `output_ptr` - Pointer to 32-byte buffer to receive block hash
    /// * `_arg2` - Unused
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 0 on success
    ///
    /// # Errors
    /// - `OutOfComputeUnits` - If not enough compute units remain
    /// - `InvalidBuffer` - If the output buffer is invalid
    TosGetBlockHash,
    fn rust(
        invoke_context: &mut InvokeContext,
        output_ptr: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(BLOCKCHAIN_QUERY_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate output buffer
        let output = translate_slice_mut::<u8>(
            memory_mapping,
            output_ptr,
            32,
            false, // u8 doesn't need alignment
        )?;

        // Copy block hash to output
        output.copy_from_slice(&invoke_context.block_hash);

        Ok(0)
    }
);

declare_builtin_function!(
    /// Get the current block height
    ///
    /// # Returns
    /// The current block height as u64
    TosGetBlockHeight,
    fn rust(
        invoke_context: &mut InvokeContext,
        _arg1: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        _memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(BLOCKCHAIN_QUERY_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok(invoke_context.block_height)
    }
);

declare_builtin_function!(
    /// Get the current transaction hash
    ///
    /// # Arguments (from VM registers)
    /// * `output_ptr` - Pointer to 32-byte buffer to receive transaction hash
    /// * `_arg2` - Unused
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 0 on success
    TosGetTxHash,
    fn rust(
        invoke_context: &mut InvokeContext,
        output_ptr: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(BLOCKCHAIN_QUERY_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate output buffer
        let output = translate_slice_mut::<u8>(
            memory_mapping,
            output_ptr,
            32,
            false,
        )?;

        // Copy transaction hash to output
        output.copy_from_slice(&invoke_context.tx_hash);

        Ok(0)
    }
);

declare_builtin_function!(
    /// Get the transaction sender address
    ///
    /// # Arguments (from VM registers)
    /// * `output_ptr` - Pointer to 32-byte buffer to receive sender address
    /// * `_arg2` - Unused
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 0 on success
    TosGetTxSender,
    fn rust(
        invoke_context: &mut InvokeContext,
        output_ptr: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(BLOCKCHAIN_QUERY_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate output buffer
        let output = translate_slice_mut::<u8>(
            memory_mapping,
            output_ptr,
            32,
            false,
        )?;

        // Copy sender address to output
        output.copy_from_slice(&invoke_context.tx_sender);

        Ok(0)
    }
);

declare_builtin_function!(
    /// Get the current contract hash
    ///
    /// # Arguments (from VM registers)
    /// * `output_ptr` - Pointer to 32-byte buffer to receive contract hash
    /// * `_arg2` - Unused
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 0 on success
    TosGetContractHash,
    fn rust(
        invoke_context: &mut InvokeContext,
        output_ptr: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(BLOCKCHAIN_QUERY_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate output buffer
        let output = translate_slice_mut::<u8>(
            memory_mapping,
            output_ptr,
            32,
            false,
        )?;

        // Copy contract hash to output
        output.copy_from_slice(&invoke_context.contract_hash);

        Ok(0)
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use tos_program_runtime::InvokeContext;
    use tos_tbpf::{
        memory_region::{MemoryRegion, MemoryMapping},
        program::TBPFVersion,
        vm::Config,
    };

    fn create_test_mapping(data: &mut [u8]) -> MemoryMapping {
        let config: &'static Config = Box::leak(Box::new(Config::default()));
        let region = MemoryRegion::new_writable(data, 0x100000000);
        MemoryMapping::new(vec![region], config, TBPFVersion::V3).unwrap()
    }

    #[test]
    fn test_get_block_hash() {
        let mut context = InvokeContext::new_with_state(
            10_000,
            [1u8; 32],
            [2u8; 32],  // block_hash
            12345,
            [3u8; 32],
            [4u8; 32],
        );

        let mut output = vec![0u8; 32];
        let mut mapping = create_test_mapping(&mut output);

        let result = TosGetBlockHash::rust(
            &mut context,
            0x100000000,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(output, [2u8; 32]);
        assert_eq!(context.get_compute_units_consumed(), BLOCKCHAIN_QUERY_COST);
    }

    #[test]
    fn test_get_block_height() {
        let mut context = InvokeContext::new_with_state(
            10_000,
            [1u8; 32],
            [2u8; 32],
            12345,  // block_height
            [3u8; 32],
            [4u8; 32],
        );

        let mut dummy_data = vec![0u8; 32];
        let mut mapping = create_test_mapping(&mut dummy_data);

        let result = TosGetBlockHeight::rust(
            &mut context,
            0,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12345);
        assert_eq!(context.get_compute_units_consumed(), BLOCKCHAIN_QUERY_COST);
    }

    #[test]
    fn test_get_tx_hash() {
        let mut context = InvokeContext::new_with_state(
            10_000,
            [1u8; 32],
            [2u8; 32],
            12345,
            [3u8; 32],  // tx_hash
            [4u8; 32],
        );

        let mut output = vec![0u8; 32];
        let mut mapping = create_test_mapping(&mut output);

        let result = TosGetTxHash::rust(
            &mut context,
            0x100000000,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(output, [3u8; 32]);
    }

    #[test]
    fn test_get_tx_sender() {
        let mut context = InvokeContext::new_with_state(
            10_000,
            [1u8; 32],
            [2u8; 32],
            12345,
            [3u8; 32],
            [4u8; 32],  // tx_sender
        );

        let mut output = vec![0u8; 32];
        let mut mapping = create_test_mapping(&mut output);

        let result = TosGetTxSender::rust(
            &mut context,
            0x100000000,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(output, [4u8; 32]);
    }

    #[test]
    fn test_get_contract_hash() {
        let mut context = InvokeContext::new_with_state(
            10_000,
            [1u8; 32],  // contract_hash
            [2u8; 32],
            12345,
            [3u8; 32],
            [4u8; 32],
        );

        let mut output = vec![0u8; 32];
        let mut mapping = create_test_mapping(&mut output);

        let result = TosGetContractHash::rust(
            &mut context,
            0x100000000,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(output, [1u8; 32]);
    }

    #[test]
    fn test_insufficient_compute_units() {
        let mut context = InvokeContext::new(20, [0u8; 32]); // Very low budget
        let mut output = vec![0u8; 32];
        let mut mapping = create_test_mapping(&mut output);

        let result = TosGetBlockHash::rust(
            &mut context,
            0x100000000,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }
}
