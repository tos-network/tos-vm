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

/// Program invocation context
///
/// This is the main execution context for TOS contracts, implementing
/// the ContextObject trait to integrate with the TBPF VM.
///
/// It follows standard eBPF execution context patterns for consistency
/// and compatibility with TBPF expectations.
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

    // === Storage Access ===
    /// Storage provider for reading/writing contract state
    /// (Using a marker for now until we define StorageProvider trait)
    storage_provider: std::marker::PhantomData<&'a ()>,

    // === Debug and Logging ===
    /// Debug mode (enables tos_log syscall output)
    pub debug_mode: bool,
}

impl<'a> InvokeContext<'a> {
    /// Creates a new invocation context
    ///
    /// # Arguments
    /// * `compute_budget` - Maximum compute units allowed for this execution
    /// * `contract_hash` - Hash of the contract being executed
    ///
    /// # Returns
    /// A new InvokeContext with default/zero values for blockchain state
    pub fn new(compute_budget: u64, contract_hash: [u8; 32]) -> Self {
        Self {
            compute_budget,
            compute_meter: RefCell::new(compute_budget),
            contract_hash,
            block_hash: [0u8; 32],
            block_height: 0,
            tx_hash: [0u8; 32],
            tx_sender: [0u8; 32],
            storage_provider: std::marker::PhantomData,
            debug_mode: false,
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
    ) -> Self {
        Self {
            compute_budget,
            compute_meter: RefCell::new(compute_budget),
            contract_hash,
            block_hash,
            block_height,
            tx_hash,
            tx_sender,
            storage_provider: std::marker::PhantomData,
            debug_mode: false,
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
    #[allow(unused_variables)]
    pub fn get_storage(&self, key: &[u8]) -> Result<Option<Vec<u8>>, EbpfError> {
        // TODO: Implement when StorageProvider trait is defined
        Ok(None)
    }

    /// Store data to contract storage
    ///
    /// # Arguments
    /// * `key` - Storage key
    /// * `value` - Storage value
    #[allow(unused_variables)]
    pub fn set_storage(&mut self, key: &[u8], value: &[u8]) -> Result<(), EbpfError> {
        // TODO: Implement when StorageProvider trait is defined
        Ok(())
    }

    /// Get account balance
    ///
    /// # Arguments
    /// * `address` - Account address
    ///
    /// # Returns
    /// Account balance in smallest units
    #[allow(unused_variables)]
    pub fn get_balance(&self, address: &[u8; 32]) -> Result<u64, EbpfError> {
        // TODO: Implement when integrated with TOS chain
        Ok(0)
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

    #[test]
    fn test_invoke_context_creation() {
        let context = InvokeContext::new(100_000, [1u8; 32]);
        assert_eq!(context.get_compute_budget(), 100_000);
        assert_eq!(context.get_remaining(), 100_000);
        assert_eq!(context.get_compute_units_consumed(), 0);
        assert_eq!(context.contract_hash, [1u8; 32]);
    }

    #[test]
    fn test_compute_consumption() {
        let mut context = InvokeContext::new(100_000, [0u8; 32]);

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
        let mut context = InvokeContext::new(100, [0u8; 32]);

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
        let mut context = InvokeContext::new(100, [0u8; 32]);

        // Consuming more than available should saturate at 0
        context.consume(150);
        assert_eq!(context.get_remaining(), 0);
        assert_eq!(context.get_compute_units_consumed(), 100);
    }

    #[test]
    fn test_debug_mode() {
        let mut context = InvokeContext::new(100_000, [0u8; 32]);
        assert!(!context.debug_mode);

        context.enable_debug();
        assert!(context.debug_mode);
    }

    #[test]
    fn test_context_with_full_state() {
        let context = InvokeContext::new_with_state(
            100_000,
            [1u8; 32],  // contract_hash
            [2u8; 32],  // block_hash
            12345,      // block_height
            [3u8; 32],  // tx_hash
            [4u8; 32],  // tx_sender
        );

        assert_eq!(context.contract_hash, [1u8; 32]);
        assert_eq!(context.block_hash, [2u8; 32]);
        assert_eq!(context.block_height, 12345);
        assert_eq!(context.tx_hash, [3u8; 32]);
        assert_eq!(context.tx_sender, [4u8; 32]);
    }
}
