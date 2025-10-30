//! Storage syscalls for TOS contracts
//!
//! This module provides syscalls for contract persistent storage operations:
//! reading, writing, and deleting key-value pairs.

use tos_tbpf::{
    declare_builtin_function,
    memory_region::MemoryMapping,
};
use tos_program_runtime::{InvokeContext, memory::{translate_slice, translate_slice_mut}};
use thiserror::Error as ThisError;

/// Syscall error types
#[derive(Debug, ThisError)]
pub enum SyscallError {
    /// Storage key is too large
    #[error("Key too large: {0} bytes (max {1})")]
    KeyTooLarge(u64, u64),
    /// Storage value is too large
    #[error("Value too large: {0} bytes (max {1})")]
    ValueTooLarge(u64, u64),
    /// Storage key not found
    #[error("Key not found")]
    KeyNotFound,
    /// Output buffer too small
    #[error("Buffer too small: need {0} bytes, got {1}")]
    BufferTooSmall(usize, u64),
    /// Insufficient compute units remaining
    #[error("Out of compute units")]
    OutOfComputeUnits,
}

/// Maximum storage key size (256 bytes)
pub const MAX_KEY_SIZE: u64 = 256;

/// Maximum storage value size (64 KB)
pub const MAX_VALUE_SIZE: u64 = 65_536;

/// Base compute cost for storage operations
pub const STORAGE_READ_BASE_COST: u64 = 200;
pub const STORAGE_WRITE_BASE_COST: u64 = 500;
pub const STORAGE_DELETE_COST: u64 = 300;

/// Per-byte cost for storage operations
pub const STORAGE_READ_BYTE_COST: u64 = 1;
pub const STORAGE_WRITE_BYTE_COST: u64 = 2;

declare_builtin_function!(
    /// Read a value from contract storage
    ///
    /// # Arguments (from VM registers)
    /// * `key_ptr` - Pointer to storage key bytes
    /// * `key_len` - Length of the key in bytes
    /// * `output_ptr` - Pointer to buffer to receive the value
    /// * `output_len` - Size of the output buffer
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// The actual size of the value in bytes (0 if key not found)
    ///
    /// # Errors
    /// - `KeyTooLarge` - If key exceeds MAX_KEY_SIZE
    /// - `OutOfComputeUnits` - If not enough compute units remain
    /// - `BufferTooSmall` - If output buffer is too small for the value
    TosStorageRead,
    fn rust(
        invoke_context: &mut InvokeContext,
        key_ptr: u64,
        key_len: u64,
        output_ptr: u64,
        output_len: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate key size
        if key_len > MAX_KEY_SIZE {
            return Err(Box::new(SyscallError::KeyTooLarge(key_len, MAX_KEY_SIZE)));
        }

        // Translate key from VM memory
        let key = translate_slice::<u8>(
            memory_mapping,
            key_ptr,
            key_len,
            false,
        )?;

        // Read from storage
        let value = invoke_context.get_storage(key)?;

        match value {
            Some(data) => {
                let value_len = data.len() as u64;

                // Charge compute units based on value size
                let cost = STORAGE_READ_BASE_COST
                    .saturating_add(value_len.saturating_mul(STORAGE_READ_BYTE_COST));
                invoke_context.consume_checked(cost)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

                // Check if output buffer is large enough
                if output_len < value_len {
                    return Err(Box::new(SyscallError::BufferTooSmall(data.len(), output_len)));
                }

                // Translate output buffer
                let output = translate_slice_mut::<u8>(
                    memory_mapping,
                    output_ptr,
                    value_len,
                    false,
                )?;

                // Copy value to output
                output.copy_from_slice(&data);

                Ok(value_len)
            }
            None => {
                // Key not found - charge minimal cost
                invoke_context.consume_checked(STORAGE_READ_BASE_COST)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                Ok(0)
            }
        }
    }
);

declare_builtin_function!(
    /// Write a value to contract storage
    ///
    /// # Arguments (from VM registers)
    /// * `key_ptr` - Pointer to storage key bytes
    /// * `key_len` - Length of the key in bytes
    /// * `value_ptr` - Pointer to value bytes to store
    /// * `value_len` - Length of the value in bytes
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 0 on success
    ///
    /// # Errors
    /// - `KeyTooLarge` - If key exceeds MAX_KEY_SIZE
    /// - `ValueTooLarge` - If value exceeds MAX_VALUE_SIZE
    /// - `OutOfComputeUnits` - If not enough compute units remain
    TosStorageWrite,
    fn rust(
        invoke_context: &mut InvokeContext,
        key_ptr: u64,
        key_len: u64,
        value_ptr: u64,
        value_len: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate key size
        if key_len > MAX_KEY_SIZE {
            return Err(Box::new(SyscallError::KeyTooLarge(key_len, MAX_KEY_SIZE)));
        }

        // Validate value size
        if value_len > MAX_VALUE_SIZE {
            return Err(Box::new(SyscallError::ValueTooLarge(value_len, MAX_VALUE_SIZE)));
        }

        // Charge compute units based on value size
        let cost = STORAGE_WRITE_BASE_COST
            .saturating_add(value_len.saturating_mul(STORAGE_WRITE_BYTE_COST));
        invoke_context.consume_checked(cost)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate key and value from VM memory
        let key = translate_slice::<u8>(
            memory_mapping,
            key_ptr,
            key_len,
            false,
        )?;

        let value = translate_slice::<u8>(
            memory_mapping,
            value_ptr,
            value_len,
            false,
        )?;

        // Write to storage
        invoke_context.set_storage(key, value)?;

        Ok(0)
    }
);

declare_builtin_function!(
    /// Delete a key from contract storage
    ///
    /// # Arguments (from VM registers)
    /// * `key_ptr` - Pointer to storage key bytes
    /// * `key_len` - Length of the key in bytes
    /// * `_arg3` - Unused
    /// * `_arg4` - Unused
    /// * `_arg5` - Unused
    ///
    /// # Returns
    /// 1 if the key was deleted, 0 if the key didn't exist
    ///
    /// # Errors
    /// - `KeyTooLarge` - If key exceeds MAX_KEY_SIZE
    /// - `OutOfComputeUnits` - If not enough compute units remain
    TosStorageDelete,
    fn rust(
        invoke_context: &mut InvokeContext,
        key_ptr: u64,
        key_len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate key size
        if key_len > MAX_KEY_SIZE {
            return Err(Box::new(SyscallError::KeyTooLarge(key_len, MAX_KEY_SIZE)));
        }

        // Charge compute units
        invoke_context.consume_checked(STORAGE_DELETE_COST)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Translate key from VM memory
        let key = translate_slice::<u8>(
            memory_mapping,
            key_ptr,
            key_len,
            false,
        )?;

        // Delete from storage
        let existed = invoke_context.delete_storage(key)?;

        Ok(if existed { 1 } else { 0 })
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
    fn test_storage_read_not_found() {
        let mut context = InvokeContext::new(10_000, [0u8; 32]);

        let mut data = vec![0u8; 128];
        // First 32 bytes = key, rest = output buffer
        data[0..4].copy_from_slice(b"test");
        let mut mapping = create_test_mapping(&mut data);

        let result = TosStorageRead::rust(
            &mut context,
            0x100000000, // key_ptr
            4,           // key_len
            0x100000004, // output_ptr
            124,         // output_len
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // Key not found
        assert_eq!(context.get_compute_units_consumed(), STORAGE_READ_BASE_COST);
    }

    #[test]
    fn test_storage_write_success() {
        let mut context = InvokeContext::new(10_000, [0u8; 32]);

        let mut data = vec![0u8; 128];
        data[0..4].copy_from_slice(b"key!");
        data[64..69].copy_from_slice(b"value");
        let mut mapping = create_test_mapping(&mut data);

        let result = TosStorageWrite::rust(
            &mut context,
            0x100000000, // key_ptr
            4,           // key_len
            0x100000040, // value_ptr (offset 64)
            5,           // value_len
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let expected_cost = STORAGE_WRITE_BASE_COST
            .saturating_add(5u64.saturating_mul(STORAGE_WRITE_BYTE_COST));
        assert_eq!(context.get_compute_units_consumed(), expected_cost);
    }

    #[test]
    fn test_storage_delete() {
        let mut context = InvokeContext::new(10_000, [0u8; 32]);

        let mut data = vec![0u8; 128];
        data[0..4].copy_from_slice(b"key!");
        let mut mapping = create_test_mapping(&mut data);

        let result = TosStorageDelete::rust(
            &mut context,
            0x100000000, // key_ptr
            4,           // key_len
            0,
            0,
            0,
            &mut mapping,
        );

        assert!(result.is_ok());
        // Currently returns 0 (stub - key didn't exist)
        assert_eq!(result.unwrap(), 0);
        assert_eq!(context.get_compute_units_consumed(), STORAGE_DELETE_COST);
    }

    #[test]
    fn test_storage_key_too_large() {
        let mut context = InvokeContext::new(10_000, [0u8; 32]);
        let mut data = vec![0u8; 1024];
        let mut mapping = create_test_mapping(&mut data);

        let result = TosStorageRead::rust(
            &mut context,
            0x100000000,
            MAX_KEY_SIZE.saturating_add(1), // Key too large
            0x100000100,
            100,
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_storage_value_too_large() {
        let mut context = InvokeContext::new(100_000, [0u8; 32]);
        let mut data = vec![0u8; 1024];
        let mut mapping = create_test_mapping(&mut data);

        let result = TosStorageWrite::rust(
            &mut context,
            0x100000000, // key_ptr
            4,
            0x100000010, // value_ptr
            MAX_VALUE_SIZE.saturating_add(1), // Value too large
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_storage_insufficient_compute() {
        let mut context = InvokeContext::new(100, [0u8; 32]); // Low budget
        let mut data = vec![0u8; 128];
        let mut mapping = create_test_mapping(&mut data);

        let result = TosStorageWrite::rust(
            &mut context,
            0x100000000,
            4,
            0x100000010,
            100,
            0,
            &mut mapping,
        );

        assert!(result.is_err());
    }
}
