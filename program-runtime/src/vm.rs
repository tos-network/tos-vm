//! TOS VM implementation

use crate::{
    context::TosContext,
    error::{Result, TosVmError},
    syscalls,
};
use std::sync::Arc;
use tos_tbpf::{
    elf::Executable,
    program::{BuiltinProgram, FunctionRegistry, TBPFVersion},
    verifier::RequisiteVerifier,
    vm::{Config, ContextObject, EbpfVm},
};

/// TOS Virtual Machine
///
/// This is the main entry point for executing TOS smart contracts.
/// It wraps the TBPF engine and provides TOS-specific functionality.
///
/// # Example
///
/// ```no_run
/// use tos_vm_tbpf::TosVm;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Load ELF bytecode
/// let elf_bytes = std::fs::read("contract.so")?;
///
/// // Create VM with 200,000 compute units
/// let mut vm = TosVm::new(&elf_bytes, 200_000)?;
///
/// // Enable debug logging
/// vm.enable_debug();
///
/// // Execute contract
/// let result = vm.execute(&[])?;
/// println!("Contract returned: {}", result);
/// println!("Compute units used: {}", vm.compute_units_consumed());
/// # Ok(())
/// # }
/// ```
pub struct TosVm {
    /// The loaded executable
    executable: Executable<TosContext>,

    /// Execution context
    context: TosContext,

    /// VM configuration
    config: Config,
}

impl TosVm {
    /// Default compute budget (200,000 CU, same as Solana)
    pub const DEFAULT_COMPUTE_BUDGET: u64 = 200_000;

    /// Maximum compute budget (1,400,000 CU, same as Solana)
    pub const MAX_COMPUTE_BUDGET: u64 = 1_400_000;

    /// Creates a new TOS VM from ELF bytecode
    ///
    /// # Arguments
    /// - `elf_bytes`: The ELF bytecode of the contract
    /// - `compute_budget`: Maximum compute units allowed
    ///
    /// # Returns
    /// - `Ok(TosVm)` if the bytecode is valid
    /// - `Err(TosVmError)` if loading or verification fails
    pub fn new(elf_bytes: &[u8], compute_budget: u64) -> Result<Self> {
        // Validate compute budget
        if compute_budget > Self::MAX_COMPUTE_BUDGET {
            return Err(TosVmError::ExecutionError(format!(
                "Compute budget {} exceeds maximum {}",
                compute_budget,
                Self::MAX_COMPUTE_BUDGET
            )));
        }

        // Create VM configuration
        let config = Config {
            max_call_depth: 64,
            stack_frame_size: 4096,
            enable_instruction_meter: true,
            enable_address_translation: true,
            enabled_tbpf_versions: TBPFVersion::V3..=TBPFVersion::V3,
            ..Config::default()
        };

        // Create builtin program (loader) with registered syscalls
        let mut loader = BuiltinProgram::new_loader(config.clone(), FunctionRegistry::default());
        syscalls::register_syscalls(&mut loader);

        let loader = Arc::new(loader);

        // Load and verify ELF
        let executable = Executable::from_elf(elf_bytes, loader.clone())
            .map_err(|e| TosVmError::ElfLoadError(e.to_string()))?;

        // Verify bytecode
        let mut verifier = RequisiteVerifier::new(
            executable.get_executable_bytes(),
            &config,
            &executable.get_tbpf_version(),
            executable.get_function_registry(),
            executable.get_loader().get_function_registry(),
        );

        verifier
            .verify()
            .map_err(|e| TosVmError::VerificationFailed(e.to_string()))?;

        // Create execution context
        let context = TosContext::new(compute_budget);

        Ok(Self {
            executable,
            context,
            config,
        })
    }

    /// Creates a VM with default compute budget
    pub fn new_default(elf_bytes: &[u8]) -> Result<Self> {
        Self::new(elf_bytes, Self::DEFAULT_COMPUTE_BUDGET)
    }

    /// Enable debug mode (allows tos_log output)
    pub fn enable_debug(&mut self) {
        self.context.enable_debug();
    }

    /// Execute the contract with given input
    ///
    /// # Arguments
    /// - `input`: Input data passed to the contract entrypoint
    ///
    /// # Returns
    /// - `Ok(u64)`: The return value from the contract (0 = success)
    /// - `Err(TosVmError)`: If execution fails
    pub fn execute(&mut self, input: &[u8]) -> Result<u64> {
        // Create VM instance
        let mut vm = EbpfVm::new(
            self.executable.get_loader().clone(),
            self.executable.get_tbpf_version(),
            &mut self.context,
            self.executable.get_text_bytes().1,
            0,
        );

        // Prepare input in VM memory
        let input_ptr = if !input.is_empty() {
            // For MVP, we'll use a simple memory layout
            // In full implementation, this would use proper memory regions
            input.as_ptr() as u64
        } else {
            0
        };

        // Execute the entrypoint
        let (instruction_count, result) = vm
            .execute_program(&self.executable, true);

        // Check for execution errors
        let result = result.map_err(|e| TosVmError::ExecutionError(e.to_string()))?;

        if log::log_enabled!(log::Level::Debug) {
            log::debug!(
                "Execution completed: {} instructions, result = {}",
                instruction_count,
                result
            );
        }

        // Check if program returned success
        if result != 0 {
            return Err(TosVmError::ProgramError(result));
        }

        Ok(result)
    }

    /// Get the number of compute units consumed
    pub fn compute_units_consumed(&self) -> u64 {
        self.context.compute_units_consumed()
    }

    /// Get the remaining compute units
    pub fn compute_units_remaining(&self) -> u64 {
        self.context.get_remaining()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_budget_validation() {
        let elf = &[]; // Empty ELF for test
        let result = TosVm::new(elf, TosVm::MAX_COMPUTE_BUDGET + 1);
        assert!(result.is_err());
    }
}
