//! TOS VM syscall bindings for smart contracts
//!
//! This module provides safe Rust wrappers around the TOS VM syscalls.

/// Maximum log message length (10KB)
pub const MAX_LOG_LENGTH: usize = 10_000;

// ============================================================================
// Logging Syscalls
// ============================================================================

/// Log a message from the contract
///
/// The message will only be displayed when the VM is in debug mode.
///
/// # Arguments
/// * `message` - UTF-8 string to log (max 10KB)
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::log;
///
/// log("Contract initialized successfully");
/// ```
pub fn log(message: &str) {
    let bytes = message.as_bytes();
    let len = bytes.len().min(MAX_LOG_LENGTH);

    unsafe {
        syscall_log(bytes.as_ptr(), len as u64);
    }
}

// ============================================================================
// Blockchain State Syscalls
// ============================================================================

/// Get the current block hash
///
/// # Returns
/// 32-byte block hash
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::get_block_hash;
///
/// let block_hash = get_block_hash();
/// ```
pub fn get_block_hash() -> [u8; 32] {
    let mut hash = [0u8; 32];
    unsafe {
        syscall_get_block_hash(hash.as_mut_ptr());
    }
    hash
}

/// Get the current block height
///
/// # Returns
/// Block height as u64
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::get_block_height;
///
/// let height = get_block_height();
/// ```
pub fn get_block_height() -> u64 {
    unsafe { syscall_get_block_height() }
}

/// Get the current transaction hash
///
/// # Returns
/// 32-byte transaction hash
pub fn get_tx_hash() -> [u8; 32] {
    let mut hash = [0u8; 32];
    unsafe {
        syscall_get_tx_hash(hash.as_mut_ptr());
    }
    hash
}

/// Get the transaction sender address
///
/// # Returns
/// 32-byte sender address
pub fn get_tx_sender() -> [u8; 32] {
    let mut address = [0u8; 32];
    unsafe {
        syscall_get_tx_sender(address.as_mut_ptr());
    }
    address
}

/// Get the executing contract address
///
/// # Returns
/// 32-byte contract address
pub fn get_contract_hash() -> [u8; 32] {
    let mut hash = [0u8; 32];
    unsafe {
        syscall_get_contract_hash(hash.as_mut_ptr());
    }
    hash
}

// ============================================================================
// Account Balance Syscalls
// ============================================================================

/// Get the balance of an account
///
/// # Arguments
/// * `address` - 32-byte account address
///
/// # Returns
/// Account balance as u64
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::{get_balance, get_tx_sender};
///
/// let sender = get_tx_sender();
/// let balance = get_balance(&sender);
/// ```
pub fn get_balance(address: &[u8; 32]) -> u64 {
    unsafe { syscall_get_balance(address.as_ptr()) }
}

/// Transfer tokens from the contract to another account
///
/// # Arguments
/// * `recipient` - 32-byte recipient address
/// * `amount` - Amount to transfer
///
/// # Returns
/// * `Ok(())` - Transfer succeeded
/// * `Err(u64)` - Transfer failed (insufficient balance or invalid params)
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::transfer;
///
/// let recipient = [1u8; 32];
/// transfer(&recipient, 1000).expect("Transfer failed");
/// ```
pub fn transfer(recipient: &[u8; 32], amount: u64) -> Result<(), u64> {
    let result = unsafe { syscall_transfer(recipient.as_ptr(), amount) };
    if result == 0 {
        Ok(())
    } else {
        Err(result)
    }
}

// ============================================================================
// Storage Syscalls
// ============================================================================

/// Read a value from contract storage
///
/// # Arguments
/// * `key` - Storage key (max 256 bytes)
/// * `buffer` - Buffer to receive the value
///
/// # Returns
/// Number of bytes read, or 0 if key not found
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::storage_read;
///
/// let key = b"counter";
/// let mut buffer = [0u8; 8];
/// let len = storage_read(key, &mut buffer);
/// if len > 0 {
///     // Value found
/// }
/// ```
pub fn storage_read(key: &[u8], buffer: &mut [u8]) -> u64 {
    unsafe {
        syscall_storage_read(
            key.as_ptr(),
            key.len() as u64,
            buffer.as_mut_ptr(),
            buffer.len() as u64,
        )
    }
}

/// Write a value to contract storage
///
/// # Arguments
/// * `key` - Storage key (max 256 bytes)
/// * `value` - Value to store (max 64KB)
///
/// # Returns
/// * `Ok(())` - Write succeeded
/// * `Err(u64)` - Write failed (key/value too large or out of gas)
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::storage_write;
///
/// let key = b"counter";
/// let value = 42u64.to_le_bytes();
/// storage_write(key, &value).expect("Write failed");
/// ```
pub fn storage_write(key: &[u8], value: &[u8]) -> Result<(), u64> {
    let result = unsafe {
        syscall_storage_write(
            key.as_ptr(),
            key.len() as u64,
            value.as_ptr(),
            value.len() as u64,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(result)
    }
}

/// Delete a key from contract storage
///
/// # Arguments
/// * `key` - Storage key to delete
///
/// # Returns
/// * `true` - Key was deleted
/// * `false` - Key didn't exist
///
/// # Example
///
/// ```no_run
/// use tos_vm_sdk::storage_delete;
///
/// let key = b"counter";
/// let existed = storage_delete(key);
/// ```
pub fn storage_delete(key: &[u8]) -> bool {
    let result = unsafe {
        syscall_storage_delete(key.as_ptr(), key.len() as u64)
    };
    result == 1
}

// ============================================================================
// Raw syscall declarations (extern "C")
// ============================================================================

extern "C" {
    // Logging
    #[link_name = "tos_log"]
    fn syscall_log(msg_ptr: *const u8, msg_len: u64);

    // Blockchain state
    #[link_name = "tos_get_block_hash"]
    fn syscall_get_block_hash(output_ptr: *mut u8);

    #[link_name = "tos_get_block_height"]
    fn syscall_get_block_height() -> u64;

    #[link_name = "tos_get_tx_hash"]
    fn syscall_get_tx_hash(output_ptr: *mut u8);

    #[link_name = "tos_get_tx_sender"]
    fn syscall_get_tx_sender(output_ptr: *mut u8);

    #[link_name = "tos_get_contract_hash"]
    fn syscall_get_contract_hash(output_ptr: *mut u8);

    // Account operations
    #[link_name = "tos_get_balance"]
    fn syscall_get_balance(address_ptr: *const u8) -> u64;

    #[link_name = "tos_transfer"]
    fn syscall_transfer(recipient_ptr: *const u8, amount: u64) -> u64;

    // Storage operations
    #[link_name = "tos_storage_read"]
    fn syscall_storage_read(
        key_ptr: *const u8,
        key_len: u64,
        output_ptr: *mut u8,
        output_len: u64,
    ) -> u64;

    #[link_name = "tos_storage_write"]
    fn syscall_storage_write(
        key_ptr: *const u8,
        key_len: u64,
        value_ptr: *const u8,
        value_len: u64,
    ) -> u64;

    #[link_name = "tos_storage_delete"]
    fn syscall_storage_delete(key_ptr: *const u8, key_len: u64) -> u64;
}
