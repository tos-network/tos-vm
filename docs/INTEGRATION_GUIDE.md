# TOS Chain Integration Guide

**Last Updated**: 2025-10-30
**Target Audience**: TOS Blockchain Developers
**Estimated Time**: 2-3 days

---

## Overview

This guide provides step-by-step instructions for integrating TOS-VM into the TOS blockchain. The VM is designed as an **independent, pluggable component** using dependency injection, requiring only that you implement two simple traits.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    TOS Blockchain                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  YOUR CODE: Implement These Traits                  │   │
│  │  - StorageProvider (contract storage)                │   │
│  │  - AccountProvider (balance/transfer)                │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │ Inject into InvokeContext
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   tos-vm (Independent)                       │
│  - InvokeContext with injected providers                     │
│  - Complete syscall system (11 syscalls)                     │
│  - TBPF execution engine                                     │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

```bash
# Add tos-vm to your blockchain's Cargo.toml
[dependencies]
tos-program-runtime = { path = "../tos-vm/program-runtime" }
tos-syscalls = { path = "../tos-vm/syscalls" }
tos-tbpf = { path = "../tos-tbpf" }
```

## Step 1: Implement StorageProvider

The `StorageProvider` trait provides the contract storage backend. You need to implement three methods that map to your blockchain's key-value storage.

### Trait Definition

```rust
use tos_program_runtime::StorageProvider;
use tos_tbpf::error::EbpfError;

/// Provides contract storage operations
pub trait StorageProvider {
    /// Read a value from contract storage
    /// Returns None if the key doesn't exist
    fn get(&self, contract_hash: &[u8; 32], key: &[u8]) -> Result<Option<Vec<u8>>, EbpfError>;

    /// Write a value to contract storage
    fn set(&mut self, contract_hash: &[u8; 32], key: &[u8], value: &[u8]) -> Result<(), EbpfError>;

    /// Delete a key from contract storage
    /// Returns true if the key existed, false otherwise
    fn delete(&mut self, contract_hash: &[u8; 32], key: &[u8]) -> Result<bool, EbpfError>;
}
```

### Example Implementation

```rust
use tos_program_runtime::StorageProvider;
use tos_tbpf::error::EbpfError;
use std::collections::HashMap;

/// TOS blockchain storage implementation
pub struct TosStorage<'a> {
    /// Reference to your blockchain's storage backend
    /// This could be a database connection, state tree, etc.
    storage_backend: &'a mut dyn YourStorageBackend,
}

impl<'a> StorageProvider for TosStorage<'a> {
    fn get(&self, contract_hash: &[u8; 32], key: &[u8]) -> Result<Option<Vec<u8>>, EbpfError> {
        // Build the full storage key: contract_hash + key
        let full_key = build_storage_key(contract_hash, key);

        // Query your blockchain's storage
        match self.storage_backend.read(&full_key) {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => Ok(None),
            Err(e) => Err(EbpfError::InvalidMemory), // Map your error appropriately
        }
    }

    fn set(&mut self, contract_hash: &[u8; 32], key: &[u8], value: &[u8]) -> Result<(), EbpfError> {
        let full_key = build_storage_key(contract_hash, key);

        // Write to your blockchain's storage
        self.storage_backend.write(&full_key, value)
            .map_err(|_| EbpfError::InvalidMemory)?;

        Ok(())
    }

    fn delete(&mut self, contract_hash: &[u8; 32], key: &[u8]) -> Result<bool, EbpfError> {
        let full_key = build_storage_key(contract_hash, key);

        // Delete from your blockchain's storage
        let existed = self.storage_backend.contains(&full_key);
        if existed {
            self.storage_backend.delete(&full_key)
                .map_err(|_| EbpfError::InvalidMemory)?;
        }

        Ok(existed)
    }
}

/// Helper: Build storage key with namespace
fn build_storage_key(contract_hash: &[u8; 32], key: &[u8]) -> Vec<u8> {
    let mut full_key = Vec::with_capacity(32 + key.len());
    full_key.extend_from_slice(contract_hash);
    full_key.extend_from_slice(key);
    full_key
}
```

### Key Considerations

1. **Namespacing**: Each contract is isolated by `contract_hash`. Ensure your storage implementation properly namespaces keys.
2. **Error Mapping**: Map your storage errors to `EbpfError` variants appropriately.
3. **Atomicity**: Storage operations should be atomic within a transaction context.
4. **Gas Accounting**: The VM already charges compute units for storage ops - you don't need to handle that.

## Step 2: Implement AccountProvider

The `AccountProvider` trait provides balance queries and token transfers. Implement two methods that interact with your blockchain's account system.

### Trait Definition

```rust
use tos_program_runtime::AccountProvider;
use tos_tbpf::error::EbpfError;

/// Provides account balance and transfer operations
pub trait AccountProvider {
    /// Get the balance of an account
    fn get_balance(&self, address: &[u8; 32]) -> Result<u64, EbpfError>;

    /// Transfer tokens from one account to another
    /// The 'from' account is typically the executing contract
    fn transfer(&mut self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> Result<(), EbpfError>;
}
```

### Example Implementation

```rust
use tos_program_runtime::AccountProvider;
use tos_tbpf::error::EbpfError;

/// TOS blockchain account provider
pub struct TosAccounts<'a> {
    /// Reference to your blockchain's account state
    account_state: &'a mut dyn YourAccountState,
}

impl<'a> AccountProvider for TosAccounts<'a> {
    fn get_balance(&self, address: &[u8; 32]) -> Result<u64, EbpfError> {
        // Query balance from your blockchain's account state
        match self.account_state.get_balance(address) {
            Ok(balance) => Ok(balance),
            Err(_) => Ok(0), // Return 0 for non-existent accounts
        }
    }

    fn transfer(&mut self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> Result<(), EbpfError> {
        // Validate amount
        if amount == 0 {
            return Err(EbpfError::InvalidParameter);
        }

        // Check sender balance
        let sender_balance = self.account_state.get_balance(from)
            .map_err(|_| EbpfError::InvalidMemory)?;

        if sender_balance < amount {
            return Err(EbpfError::InvalidParameter); // Insufficient balance
        }

        // Perform the transfer atomically
        self.account_state.subtract_balance(from, amount)
            .map_err(|_| EbpfError::InvalidMemory)?;

        self.account_state.add_balance(to, amount)
            .map_err(|_| EbpfError::InvalidMemory)?;

        Ok(())
    }
}
```

### Key Considerations

1. **Atomicity**: Transfers must be atomic - either fully succeed or fully fail.
2. **Balance Checks**: Always verify sufficient balance before transfers.
3. **Contract Context**: The `from` address is typically the executing contract's address.
4. **Error Handling**: Return appropriate errors for insufficient balance or invalid addresses.

## Step 3: Register Syscalls

Before executing any contract, you need to register the TOS-VM syscalls with the TBPF engine.

### Registration Code

```rust
use tos_tbpf::program::BuiltinProgram;
use tos_program_runtime::InvokeContext;
use tos_syscalls;

/// Register all TOS-VM syscalls
pub fn register_tos_syscalls(
    loader: &mut BuiltinProgram<InvokeContext>
) -> Result<(), tos_tbpf::elf::ElfError> {
    tos_syscalls::register_syscalls(loader)
}
```

The `register_syscalls` function registers all 11 syscalls:
- `tos_log` - Logging
- `tos_get_block_hash` - Block hash
- `tos_get_block_height` - Block height
- `tos_get_tx_hash` - Transaction hash
- `tos_get_tx_sender` - Transaction sender
- `tos_get_contract_hash` - Contract address
- `tos_get_balance` - Query balance
- `tos_transfer` - Transfer tokens
- `tos_storage_read` - Read storage
- `tos_storage_write` - Write storage
- `tos_storage_delete` - Delete storage

## Step 4: Execute Contracts

Now you can execute contracts by creating an `InvokeContext` with your providers and running the TBPF VM.

### Complete Execution Example

```rust
use tos_program_runtime::InvokeContext;
use tos_tbpf::{
    elf::Executable,
    vm::{Config, EbpfVm, TestContextObject},
    program::BuiltinProgram,
};

/// Execute a TOS contract
pub fn execute_contract(
    contract_bytecode: &[u8],
    compute_budget: u64,
    block_hash: [u8; 32],
    block_height: u64,
    tx_hash: [u8; 32],
    tx_sender: [u8; 32],
    contract_hash: [u8; 32],
    storage: &mut impl StorageProvider,
    accounts: &mut impl AccountProvider,
) -> Result<u64, Box<dyn std::error::Error>> {
    // Step 1: Create invoke context with injected providers
    let mut invoke_context = InvokeContext::new_with_state(
        compute_budget,
        contract_hash,
        block_hash,
        block_height,
        tx_hash,
        tx_sender,
        storage,
        accounts,
    );

    // Optional: Enable debug logging
    invoke_context.enable_debug();

    // Step 2: Create TBPF config
    let config = Config::default();

    // Step 3: Register syscalls
    let mut loader = BuiltinProgram::new_loader(config);
    register_tos_syscalls(&mut loader)?;

    // Step 4: Load and verify ELF bytecode
    let executable = Executable::<InvokeContext>::from_elf(
        contract_bytecode,
        std::sync::Arc::new(loader),
    )?;

    // Step 5: Verify bytecode
    executable.verify::<tos_tbpf::verifier::RequisiteVerifier>()?;

    // Step 6: Create VM and execute
    let mut vm = EbpfVm::new(
        executable.get_loader(),
        executable.get_sbpf_version(),
        &mut invoke_context,
        executable.get_memory_mapping(),
        executable.get_functions(),
    )?;

    // Execute the main entry point
    let result = vm.execute_program(
        executable.get_entrypoint_instruction_offset(),
        &mut TestContextObject::default(),
    )?;

    // Step 7: Check compute units consumed
    let units_consumed = invoke_context.get_compute_units_consumed();
    log::info!("Compute units consumed: {}", units_consumed);

    Ok(result)
}
```

### Integration Points

In your blockchain's transaction execution code:

```rust
// In your transaction processor:
pub fn execute_contract_transaction(tx: &Transaction) -> Result<(), Error> {
    // Extract transaction data
    let contract_bytecode = load_contract_bytecode(&tx.contract_address)?;
    let compute_budget = tx.gas_limit;

    // Get blockchain state
    let block_hash = current_block_hash();
    let block_height = current_block_height();
    let tx_hash = tx.hash();
    let tx_sender = tx.sender();
    let contract_hash = tx.contract_address;

    // Create providers
    let mut storage = TosStorage::new(&mut blockchain_storage);
    let mut accounts = TosAccounts::new(&mut blockchain_accounts);

    // Execute contract
    let result = execute_contract(
        &contract_bytecode,
        compute_budget,
        block_hash,
        block_height,
        tx_hash,
        tx_sender,
        contract_hash,
        &mut storage,
        &mut accounts,
    )?;

    log::info!("Contract execution result: {}", result);
    Ok(())
}
```

## Step 5: Contract Deployment

Update your contract deployment flow to accept ELF bytecode:

```rust
pub fn deploy_contract(
    bytecode: &[u8],
    deployer: &[u8; 32],
) -> Result<[u8; 32], Error> {
    // Validate ELF format
    validate_elf_bytecode(bytecode)?;

    // Calculate contract hash
    let contract_hash = hash_bytecode(bytecode);

    // Store contract bytecode
    store_contract_bytecode(contract_hash, bytecode)?;

    // Create contract account
    create_contract_account(contract_hash, deployer)?;

    Ok(contract_hash)
}

fn validate_elf_bytecode(bytecode: &[u8]) -> Result<(), Error> {
    use tos_tbpf::elf::Executable;

    // Try to parse as ELF
    let config = Config::default();
    let mut loader = BuiltinProgram::new_loader(config);
    register_tos_syscalls(&mut loader)?;

    let executable = Executable::<InvokeContext>::from_elf(
        bytecode,
        std::sync::Arc::new(loader),
    )?;

    // Verify bytecode
    executable.verify::<tos_tbpf::verifier::RequisiteVerifier>()?;

    Ok(())
}
```

## Testing Your Integration

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tos_program_runtime::{NoOpStorage, NoOpAccounts, InvokeContext};

    #[test]
    fn test_contract_execution() {
        // Load test contract bytecode
        let bytecode = include_bytes!("../test_contracts/hello.so");

        // Create test providers
        let mut storage = NoOpStorage;
        let mut accounts = NoOpAccounts;

        // Execute
        let result = execute_contract(
            bytecode,
            10_000, // compute budget
            [1u8; 32], // block hash
            12345, // block height
            [2u8; 32], // tx hash
            [3u8; 32], // tx sender
            [4u8; 32], // contract hash
            &mut storage,
            &mut accounts,
        );

        assert!(result.is_ok());
    }
}
```

### Integration Test Checklist

- [ ] Contract deployment stores ELF bytecode correctly
- [ ] Contract execution creates proper InvokeContext
- [ ] Storage operations persist correctly
- [ ] Balance queries return correct values
- [ ] Transfers update balances atomically
- [ ] Compute units are tracked and enforced
- [ ] Out-of-gas errors are handled properly
- [ ] Invalid bytecode is rejected
- [ ] Logging works when debug mode is enabled

## Common Integration Issues

### Issue 1: Lifetime Errors

**Problem**: Borrow checker errors with providers.

**Solution**: Ensure providers outlive the `InvokeContext`:

```rust
// Good: providers declared first
let mut storage = TosStorage::new(&mut state);
let mut accounts = TosAccounts::new(&mut state);
let mut context = InvokeContext::new(1000, [0u8; 32], &mut storage, &mut accounts);

// Bad: would cause lifetime issues
let mut context = InvokeContext::new(1000, [0u8; 32],
    &mut TosStorage::new(&mut state),  // Temporary value
    &mut TosAccounts::new(&mut state)  // Temporary value
);
```

### Issue 2: EbpfError Mapping

**Problem**: Need to convert blockchain errors to `EbpfError`.

**Solution**: Create a conversion function:

```rust
impl From<YourStorageError> for EbpfError {
    fn from(err: YourStorageError) -> Self {
        match err {
            YourStorageError::NotFound => EbpfError::InvalidMemory,
            YourStorageError::PermissionDenied => EbpfError::AccessViolation,
            _ => EbpfError::InvalidMemory,
        }
    }
}
```

### Issue 3: Compute Units

**Problem**: Need to convert compute units to gas.

**Solution**: Use a conversion factor:

```rust
const COMPUTE_UNITS_PER_GAS: u64 = 100;

fn compute_units_to_gas(compute_units: u64) -> u64 {
    compute_units / COMPUTE_UNITS_PER_GAS
}

fn gas_to_compute_units(gas: u64) -> u64 {
    gas.saturating_mul(COMPUTE_UNITS_PER_GAS)
}
```

## Performance Considerations

1. **Compute Budget**: Set appropriate compute budgets based on transaction gas limits
2. **Storage Caching**: Consider caching storage reads within a transaction
3. **JIT Compilation**: Enable TBPF JIT for production (10-50x speedup)
4. **Memory Limits**: Configure TBPF memory limits appropriately

## Security Checklist

- [ ] Contract bytecode is verified before execution
- [ ] Compute units prevent infinite loops
- [ ] Storage keys are properly namespaced per contract
- [ ] Transfer checks prevent negative amounts
- [ ] Balance checks prevent overdrafts
- [ ] Memory access is bounds-checked by TBPF
- [ ] Syscalls validate all parameters

## Next Steps

After successful integration:

1. **SDK Development**: Create Rust SDK for contract developers
2. **Example Contracts**: Provide reference implementations
3. **Documentation**: Write contract development guide
4. **Tooling**: Build contract compilation and testing tools
5. **Audit**: Security audit of the integration

## Support

For questions or issues:
- Review the code examples in `program-runtime/src/lib.rs`
- Check the syscall implementations in `syscalls/src/`
- Look at test cases in `program-runtime/src/invoke_context.rs`
- File issues in the TOS-VM repository

---

**Last Updated**: 2025-10-30
**Version**: 1.0.0
**License**: Apache 2.0
