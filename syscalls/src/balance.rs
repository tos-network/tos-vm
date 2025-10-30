//! Balance and transfer syscalls for TOS contracts
//!
//! This module provides syscalls for querying account balances and
//! transferring tokens between accounts.

use tos_tbpf::{
    declare_builtin_function,
    memory_region::MemoryMapping,
};
use tos_program_runtime::{InvokeContext, memory::translate_slice};
use thiserror::Error as ThisError;

/// Syscall error types
#[derive(Debug, ThisError)]
pub enum SyscallError {
    /// Invalid address format
    #[error("Invalid address")]
    InvalidAddress,
    /// Insufficient balance for transfer
    #[error("Insufficient balance")]
    InsufficientBalance,
    /// Invalid transfer amount
    #[error("Invalid amount")]
    InvalidAmount,
    /// Insufficient compute units remaining
    #[error("Out of compute units")]
    OutOfComputeUnits,
}

/// Compute units for balance query
pub const BALANCE_QUERY_COST: u64 = 100;

/// Compute units for transfer operation
pub const TRANSFER_COST: u64 = 500;

declare_builtin_function!(
    /// Get the balance of an account
    ///
    /// # Arguments (from VM registers)
    /// * `address_ptr` - Pointer to 32-byte account address
    /// * `_arg2` - Unused
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// The account balance as u64
    ///
    /// # Errors
    /// - `OutOfComputeUnits` - If not enough compute units remain
    /// - `InvalidAddress` - If the address is invalid
    TosGetBalance,
    fn rust(
        invoke_context: &mut InvokeContext,
        address_ptr: u64,
        _arg2: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(BALANCE_QUERY_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate address from VM memory
        let address_bytes = translate_slice::<u8>(
            memory_mapping,
            address_ptr,
            32,
            false,
        )?;

        // Convert to fixed-size array
        let mut address = [0u8; 32];
        address.copy_from_slice(address_bytes);

        // Query balance from invoke context
        let balance = invoke_context.get_balance(&address)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        Ok(balance)
    }
);

declare_builtin_function!(
    /// Transfer tokens from the contract to another account
    ///
    /// # Arguments (from VM registers)
    /// * `recipient_ptr` - Pointer to 32-byte recipient address
    /// * `amount` - Amount to transfer (u64)
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 0 on success
    ///
    /// # Errors
    /// - `OutOfComputeUnits` - If not enough compute units remain
    /// - `InvalidAddress` - If the recipient address is invalid
    /// - `InvalidAmount` - If the amount is 0
    /// - `InsufficientBalance` - If the contract doesn't have enough balance
    TosTransfer,
    fn rust(
        invoke_context: &mut InvokeContext,
        recipient_ptr: u64,
        amount: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Charge compute units
        invoke_context.consume_checked(TRANSFER_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Validate amount
        if amount == 0 {
            return Err(Box::new(SyscallError::InvalidAmount));
        }

        // Translate recipient address from VM memory
        let recipient_bytes = translate_slice::<u8>(
            memory_mapping,
            recipient_ptr,
            32,
            false,
        )?;

        // Convert to fixed-size array
        let mut recipient = [0u8; 32];
        recipient.copy_from_slice(recipient_bytes);

        // Perform transfer through invoke context
        invoke_context.transfer(&recipient, amount)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

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
    fn test_get_balance() {
        let mut context = InvokeContext::new(10_000, [0u8; 32]);
        let mut address_data = [5u8; 32].to_vec();
        let mut mapping = create_test_mapping(&mut address_data);

        let result = TosGetBalance::rust(
            &mut context,
            0x100000000,
            0,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        // Currently returns 0 (stub implementation)
        assert_eq!(result.unwrap(), 0);
        assert_eq!(context.get_compute_units_consumed(), BALANCE_QUERY_COST);
    }

    #[test]
    fn test_transfer_success() {
        let mut context = InvokeContext::new(10_000, [0u8; 32]);
        let mut recipient_data = [7u8; 32].to_vec();
        let mut mapping = create_test_mapping(&mut recipient_data);

        let result = TosTransfer::rust(
            &mut context,
            0x100000000,
            1000, // amount
            0,
            0,
            0,
            &mut mapping,
        );

        // Currently succeeds (stub implementation)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(context.get_compute_units_consumed(), TRANSFER_COST);
    }

    #[test]
    fn test_transfer_zero_amount() {
        let mut context = InvokeContext::new(10_000, [0u8; 32]);
        let mut recipient_data = [7u8; 32].to_vec();
        let mut mapping = create_test_mapping(&mut recipient_data);

        let result = TosTransfer::rust(
            &mut context,
            0x100000000,
            0, // zero amount
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_insufficient_compute_for_balance() {
        let mut context = InvokeContext::new(50, [0u8; 32]); // Low budget
        let mut address_data = [5u8; 32].to_vec();
        let mut mapping = create_test_mapping(&mut address_data);

        let result = TosGetBalance::rust(
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

    #[test]
    fn test_insufficient_compute_for_transfer() {
        let mut context = InvokeContext::new(100, [0u8; 32]); // Low budget
        let mut recipient_data = [7u8; 32].to_vec();
        let mut mapping = create_test_mapping(&mut recipient_data);

        let result = TosTransfer::rust(
            &mut context,
            0x100000000,
            1000,
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }
}
