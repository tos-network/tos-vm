//! Execution context for TOS VM

use tos_tbpf::vm::ContextObject;

/// TOS-specific execution context
///
/// This struct holds all the state needed during contract execution,
/// including blockchain information, compute budget, and access to chain state.
#[derive(Debug)]
pub struct TosContext {
    /// Remaining compute units (gas)
    pub(crate) remaining_compute_units: u64,

    /// Initial compute budget
    pub(crate) compute_budget: u64,

    /// Debug mode (enables tos_log syscall)
    pub debug_mode: bool,

    // Future fields to be added when integrating with TOS blockchain:
    // - contract_hash: Hash,
    // - block_hash: Hash,
    // - block_height: u64,
    // - tx_hash: Hash,
    // - tx_sender: PublicKey,
    // - chain_state: &'a mut ChainState,
    // - storage_provider: Box<dyn StorageProvider>,
}

impl TosContext {
    /// Creates a new execution context
    pub fn new(compute_budget: u64) -> Self {
        Self {
            remaining_compute_units: compute_budget,
            compute_budget,
            debug_mode: false,
        }
    }

    /// Get the amount of compute units consumed
    pub fn compute_units_consumed(&self) -> u64 {
        self.compute_budget.saturating_sub(self.remaining_compute_units)
    }

    /// Enable debug mode (allows tos_log syscall to output)
    pub fn enable_debug(&mut self) {
        self.debug_mode = true;
    }
}

impl ContextObject for TosContext {
    fn consume(&mut self, amount: u64) {
        self.remaining_compute_units = self.remaining_compute_units.saturating_sub(amount);
    }

    fn get_remaining(&self) -> u64 {
        self.remaining_compute_units
    }
}
