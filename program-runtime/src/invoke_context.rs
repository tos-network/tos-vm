//! Program invocation context for TOS VM
//!
//! This module provides the execution context for TBPF programs, following
//! standard eBPF execution patterns. The context holds blockchain state and
//! provides access to chain data during contract execution.

use std::cell::RefCell;
use tos_tbpf::{
    error::EbpfError,
    vm::ContextObject,
};
use crate::storage::{StorageProvider, AccountProvider};

/// Program invocation context
///
/// This is the main execution context for TOS contracts, implementing
/// the ContextObject trait to integrate with the TBPF VM.
///
/// Uses dynamic dispatch for storage and account providers, allowing
/// the TOS blockchain to inject custom implementations without modifying VM code.
pub struct InvokeContext<'a> {
    // === Compute Budget Tracking ===
    /// Initial compute budget allocated for this execution
    compute_budget: u64,

    /// Current remaining compute units (wrapped in RefCell for interior mutability)
    compute_meter: RefCell<u64>,

    // === TOS Blockchain State ===
    /// Hash of the contract being executed
    pub contract_hash: [u8; 32],

    /// Current block hash
    pub block_hash: [u8; 32],

    /// Current block height
    pub block_height: u64,

    /// Transaction hash
    pub tx_hash: [u8; 32],

    /// Transaction sender's public key
    pub tx_sender: [u8; 32],

    // === External Providers ===
    /// Storage provider for reading/writing contract state
    storage: &'a mut dyn StorageProvider,

    /// Account provider for balance queries and transfers
    accounts: &'a mut dyn AccountProvider,

    // === Debug and Logging ===
    /// Debug mode (enables tos_log syscall output)
    pub debug_mode: bool,

    // === Return Data ===
    /// Return data set by the contract (used for inter-contract communication)
    /// Format: (program_id, data)
    return_data: RefCell<Option<([u8; 32], Vec<u8>)>>,
}

impl<'a> InvokeContext<'a> {
    /// Creates a new invocation context
    ///
    /// # Arguments
    /// * `compute_budget` - Maximum compute units allowed for this execution
    /// * `contract_hash` - Hash of the contract being executed
    /// * `storage` - Storage provider for contract storage operations
    /// * `accounts` - Account provider for balance/transfer operations
    ///
    /// # Returns
    /// A new InvokeContext with default/zero values for blockchain state
    pub fn new(
        compute_budget: u64,
        contract_hash: [u8; 32],
        storage: &'a mut dyn StorageProvider,
        accounts: &'a mut dyn AccountProvider,
    ) -> Self {
        Self {
            compute_budget,
            compute_meter: RefCell::new(compute_budget),
            contract_hash,
            block_hash: [0u8; 32],
            block_height: 0,
            tx_hash: [0u8; 32],
            tx_sender: [0u8; 32],
            storage,
            accounts,
            debug_mode: false,
            return_data: RefCell::new(None),
        }
    }

    /// Creates a context with full blockchain state
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_state(
        compute_budget: u64,
        contract_hash: [u8; 32],
        block_hash: [u8; 32],
        block_height: u64,
        tx_hash: [u8; 32],
        tx_sender: [u8; 32],
        storage: &'a mut dyn StorageProvider,
        accounts: &'a mut dyn AccountProvider,
    ) -> Self {
        Self {
            compute_budget,
            compute_meter: RefCell::new(compute_budget),
            contract_hash,
            block_hash,
            block_height,
            tx_hash,
            tx_sender,
            storage,
            accounts,
            debug_mode: false,
            return_data: RefCell::new(None),
        }
    }

    /// Enable debug mode (allows tos_log syscall to produce output)
    pub fn enable_debug(&mut self) {
        self.debug_mode = true;
    }

    /// Get the initial compute budget
    pub fn get_compute_budget(&self) -> u64 {
        self.compute_budget
    }

    /// Get compute units consumed so far
    pub fn get_compute_units_consumed(&self) -> u64 {
        self.compute_budget.saturating_sub(*self.compute_meter.borrow())
    }

    /// Consume compute units with checked arithmetic
    ///
    /// Returns an error if not enough compute units remain.
    pub fn consume_checked(&mut self, amount: u64) -> Result<(), EbpfError> {
        let mut meter = self.compute_meter.borrow_mut();
        if *meter < amount {
            return Err(EbpfError::ExceededMaxInstructions);
        }
        *meter -= amount;
        Ok(())
    }

    // === Storage Methods (Stubs for now) ===
    // These will be implemented when we integrate with TOS chain storage

    /// Load data from contract storage
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Returns
    /// Storage value if exists, None otherwise
    pub fn get_storage(&self, key: &[u8]) -> Result<Option<Vec<u8>>, EbpfError> {
        self.storage.get(&self.contract_hash, key)
    }

    /// Store data to contract storage
    ///
    /// # Arguments
    /// * `key` - Storage key
    /// * `value` - Storage value
    pub fn set_storage(&mut self, key: &[u8], value: &[u8]) -> Result<(), EbpfError> {
        self.storage.set(&self.contract_hash, key, value)
    }

    /// Delete data from contract storage
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Returns
    /// `true` if the key existed and was deleted, `false` if it didn't exist
    pub fn delete_storage(&mut self, key: &[u8]) -> Result<bool, EbpfError> {
        self.storage.delete(&self.contract_hash, key)
    }

    /// Get account balance
    ///
    /// # Arguments
    /// * `address` - Account address
    ///
    /// # Returns
    /// Account balance in smallest units
    pub fn get_balance(&self, address: &[u8; 32]) -> Result<u64, EbpfError> {
        self.accounts.get_balance(address)
    }

    /// Transfer tokens from contract to another account
    ///
    /// # Arguments
    /// * `recipient` - Recipient address
    /// * `amount` - Amount to transfer
    pub fn transfer(&mut self, recipient: &[u8; 32], amount: u64) -> Result<(), EbpfError> {
        self.accounts.transfer(&self.contract_hash, recipient, amount)
    }

    // === Return Data Methods ===

    /// Set return data for this invocation
    ///
    /// Used for passing data back to the caller (important for CPI).
    ///
    /// # Arguments
    /// * `program_id` - The program ID that is setting the return data
    /// * `data` - The data to return (max 1024 bytes)
    ///
    /// # Returns
    /// Ok(()) if successful, Err if data is too large
    pub fn set_return_data(&self, program_id: [u8; 32], data: Vec<u8>) -> Result<(), EbpfError> {
        const MAX_RETURN_DATA: usize = 1024;

        if data.len() > MAX_RETURN_DATA {
            return Err(EbpfError::SyscallError(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Return data too large: {} > {}", data.len(), MAX_RETURN_DATA),
            ))));
        }

        *self.return_data.borrow_mut() = Some((program_id, data));
        Ok(())
    }

    /// Get return data from the last invocation
    ///
    /// # Returns
    /// Option containing (program_id, data) if return data was set
    pub fn get_return_data(&self) -> Option<([u8; 32], Vec<u8>)> {
        self.return_data.borrow().clone()
    }

    /// Clear return data
    pub fn clear_return_data(&self) {
        *self.return_data.borrow_mut() = None;
    }
}

/// Implementation of ContextObject trait for TBPF integration
///
/// This allows InvokeContext to be used as the context type when
/// creating and executing TBPF VMs.
impl<'a> ContextObject for InvokeContext<'a> {
    /// Consume compute units (called by TBPF VM during instruction execution)
    fn consume(&mut self, amount: u64) {
        let mut meter = self.compute_meter.borrow_mut();
        *meter = meter.saturating_sub(amount);
    }

    /// Get remaining compute units
    fn get_remaining(&self) -> u64 {
        *self.compute_meter.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{NoOpStorage, NoOpAccounts};

    #[test]
    fn test_invoke_context_creation() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let context = InvokeContext::new(100_000, [1u8; 32], &mut storage, &mut accounts);
        assert_eq!(context.get_compute_budget(), 100_000);
        assert_eq!(context.get_remaining(), 100_000);
        assert_eq!(context.get_compute_units_consumed(), 0);
        assert_eq!(context.contract_hash, [1u8; 32]);
    }

    #[test]
    fn test_compute_consumption() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(100_000, [0u8; 32], &mut storage, &mut accounts);

        // Consume some units
        context.consume(30_000);
        assert_eq!(context.get_remaining(), 70_000);
        assert_eq!(context.get_compute_units_consumed(), 30_000);

        // Consume more
        context.consume(20_000);
        assert_eq!(context.get_remaining(), 50_000);
        assert_eq!(context.get_compute_units_consumed(), 50_000);
    }

    #[test]
    fn test_consume_checked() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(100, [0u8; 32], &mut storage, &mut accounts);

        // Should succeed
        assert!(context.consume_checked(50).is_ok());
        assert_eq!(context.get_remaining(), 50);

        // Should fail - not enough units
        assert!(context.consume_checked(100).is_err());
        assert_eq!(context.get_remaining(), 50); // Unchanged

        // Should succeed with exact remaining amount
        assert!(context.consume_checked(50).is_ok());
        assert_eq!(context.get_remaining(), 0);
    }

    #[test]
    fn test_saturating_consumption() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(100, [0u8; 32], &mut storage, &mut accounts);

        // Consuming more than available should saturate at 0
        context.consume(150);
        assert_eq!(context.get_remaining(), 0);
        assert_eq!(context.get_compute_units_consumed(), 100);
    }

    #[test]
    fn test_debug_mode() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let mut context = InvokeContext::new(100_000, [0u8; 32], &mut storage, &mut accounts);
        assert!(!context.debug_mode);

        context.enable_debug();
        assert!(context.debug_mode);
    }

    #[test]
    fn test_context_with_full_state() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let context = InvokeContext::new_with_state(
            100_000,
            [1u8; 32],  // contract_hash
            [2u8; 32],  // block_hash
            12345,      // block_height
            [3u8; 32],  // tx_hash
            [4u8; 32],  // tx_sender
            &mut storage,
            &mut accounts,
        );

        assert_eq!(context.contract_hash, [1u8; 32]);
        assert_eq!(context.block_hash, [2u8; 32]);
        assert_eq!(context.block_height, 12345);
        assert_eq!(context.tx_hash, [3u8; 32]);
        assert_eq!(context.tx_sender, [4u8; 32]);
    }

    #[test]
    fn test_return_data() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let context = InvokeContext::new(100_000, [1u8; 32], &mut storage, &mut accounts);

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
    fn test_return_data_too_large() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let context = InvokeContext::new(100_000, [1u8; 32], &mut storage, &mut accounts);

        // Try to set return data larger than MAX_RETURN_DATA (1024 bytes)
        let program_id = [5u8; 32];
        let large_data = vec![0u8; 2000];
        let result = context.set_return_data(program_id, large_data);
        assert!(result.is_err());

        // Return data should still be None
        assert!(context.get_return_data().is_none());
    }

    #[test]
    fn test_return_data_max_size() {
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;
        let context = InvokeContext::new(100_000, [1u8; 32], &mut storage, &mut accounts);

        // Set return data at exactly max size (1024 bytes)
        let program_id = [5u8; 32];
        let max_data = vec![0u8; 1024];
        assert!(context.set_return_data(program_id, max_data.clone()).is_ok());

        let result = context.get_return_data();
        assert!(result.is_some());
        let (_ret_program_id, ret_data) = result.unwrap();
        assert_eq!(ret_data.len(), 1024);
    }
}

