# TOS-VM Architecture Documentation

**Last Updated**: 2025-10-30
**Status**: Core Implementation Complete
**Version**: 1.0.0

---

## Table of Contents

1. [Overview](#overview)
2. [Design Principles](#design-principles)
3. [Component Architecture](#component-architecture)
4. [Dependency Injection System](#dependency-injection-system)
5. [Syscall System](#syscall-system)
6. [Memory Management](#memory-management)
7. [Testing Strategy](#testing-strategy)
8. [Integration Points](#integration-points)

---

## Overview

TOS-VM is a high-performance eBPF-based virtual machine for executing smart contracts on the TOS blockchain. It is designed as an **independent, pluggable component** that can be integrated into any blockchain with minimal coupling.

### Key Features

- **eBPF Execution**: Based on TBPF (TOS Berkeley Packet Filter) for high performance
- **Dependency Injection**: Clean trait-based abstraction for blockchain integration
- **Zero Coupling**: VM core has no knowledge of blockchain internals
- **Production Ready**: 40+ tests, comprehensive error handling, full documentation

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    TOS Blockchain                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Implements Integration Traits:                      │   │
│  │  - StorageProvider (contract storage)                │   │
│  │  - AccountProvider (balance/transfer)                │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │ Inject providers
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   tos-vm (Independent)                       │
│  ┌────────────────────────────────────────────────────┐     │
│  │           InvokeContext<'a>                        │     │
│  │  - Compute budget tracking                         │     │
│  │  - Blockchain state (block/tx info)                │     │
│  │  - &mut dyn StorageProvider                        │     │
│  │  - &mut dyn AccountProvider                        │     │
│  └────────────┬───────────────────────────────────────┘     │
│               │                                              │
│  ┌────────────▼───────────────────────────────────────┐     │
│  │                  Syscalls                          │     │
│  │  - tos_log (logging)                               │     │
│  │  - tos_get_block_hash/height/tx_hash/tx_sender     │     │
│  │  - tos_get_contract_hash                           │     │
│  │  - tos_get_balance, tos_transfer                   │     │
│  │  - tos_storage_read/write/delete                   │     │
│  └────────────┬───────────────────────────────────────┘     │
│               │                                              │
│  ┌────────────▼───────────────────────────────────────┐     │
│  │            Memory Translation                      │     │
│  │  - Safe VM ↔ host memory mapping                   │     │
│  │  - Alignment checking                              │     │
│  └────────────────────────────────────────────────────┘     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              tos-tbpf (eBPF Engine)                          │
│  - ELF loader & verifier                                     │
│  - Interpreter / JIT compiler                                │
│  - Memory regions & mapping                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Design Principles

### 1. Dependency Injection

The VM receives all blockchain functionality through trait interfaces, not direct dependencies.

**Benefits:**
- VM can be tested standalone with NoOp implementations
- No coupling to specific blockchain implementation
- Easy to mock and test
- Clear separation of concerns

### 2. Trait-Based Abstraction

Two core traits define the blockchain interface:

```rust
trait StorageProvider {
    fn get(&self, contract_hash: &[u8; 32], key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn set(&mut self, contract_hash: &[u8; 32], key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&mut self, contract_hash: &[u8; 32], key: &[u8]) -> Result<bool>;
}

trait AccountProvider {
    fn get_balance(&self, address: &[u8; 32]) -> Result<u64>;
    fn transfer(&mut self, from: &[u8; 32], to: &[u8; 32], amount: u64) -> Result<()>;
}
```

### 3. Zero Coupling

The VM core knows nothing about:
- Blockchain storage implementation
- Account management
- Transaction processing
- Consensus mechanisms
- Network protocols

It only knows how to execute eBPF bytecode and call provider traits.

### 4. Easy Testing

NoOp providers enable standalone testing:

```rust
let mut storage = NoOpStorage;  // Stub implementation
let mut accounts = NoOpAccounts; // Stub implementation
let mut context = InvokeContext::new(10_000, [0u8; 32], &mut storage, &mut accounts);
```

---

## Component Architecture

### Directory Structure

```
tos-vm/
├── program-runtime/         # Core runtime (independent)
│   ├── src/
│   │   ├── lib.rs           # Public API
│   │   ├── invoke_context.rs # Execution context
│   │   ├── storage.rs       # Provider traits
│   │   ├── memory.rs        # Memory translation
│   │   └── error.rs         # Error types
│   └── Cargo.toml
│
├── syscalls/                # Syscall implementations
│   ├── src/
│   │   ├── lib.rs           # Syscall registration
│   │   ├── logging.rs       # tos_log
│   │   ├── blockchain.rs    # Block/tx info syscalls
│   │   ├── balance.rs       # Balance/transfer syscalls
│   │   └── storage.rs       # Storage syscalls
│   └── Cargo.toml
│
├── sdk/                     # Contract development SDK (TODO)
│   └── Cargo.toml
│
└── Cargo.toml               # Workspace configuration
```

### Component Dependencies

```
syscalls → program-runtime → tos-tbpf
   ↑                             ↑
   └──────── thiserror ──────────┘
```

**Key Point**: All dependencies flow towards `tos-tbpf`. No dependencies on TOS blockchain.

---

## Dependency Injection System

### InvokeContext: The Heart of Execution

`InvokeContext` is the execution environment for contracts. It holds:

1. **Compute Budget**: Gas/compute unit tracking
2. **Blockchain State**: Block hash, height, tx info, contract address
3. **Injected Providers**: Storage and account providers
4. **Debug Mode**: Enable/disable logging

```rust
pub struct InvokeContext<'a> {
    // Compute tracking
    compute_budget: u64,
    compute_meter: RefCell<u64>,

    // Blockchain state
    pub contract_hash: [u8; 32],
    pub block_hash: [u8; 32],
    pub block_height: u64,
    pub tx_hash: [u8; 32],
    pub tx_sender: [u8; 32],

    // Injected providers (dynamic dispatch)
    storage: &'a mut dyn StorageProvider,
    accounts: &'a mut dyn AccountProvider,

    // Debug mode
    pub debug_mode: bool,
}
```

### Constructor Pattern

```rust
impl<'a> InvokeContext<'a> {
    /// Basic constructor
    pub fn new(
        compute_budget: u64,
        contract_hash: [u8; 32],
        storage: &'a mut dyn StorageProvider,
        accounts: &'a mut dyn AccountProvider,
    ) -> Self { ... }

    /// Constructor with full blockchain state
    pub fn new_with_state(
        compute_budget: u64,
        contract_hash: [u8; 32],
        block_hash: [u8; 32],
        block_height: u64,
        tx_hash: [u8; 32],
        tx_sender: [u8; 32],
        storage: &'a mut dyn StorageProvider,
        accounts: &'a mut dyn AccountProvider,
    ) -> Self { ... }
}
```

### Provider Methods

`InvokeContext` wraps provider calls with contract context:

```rust
impl<'a> InvokeContext<'a> {
    /// Storage operations (automatically use self.contract_hash)
    pub fn get_storage(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.storage.get(&self.contract_hash, key)
    }

    pub fn set_storage(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.storage.set(&self.contract_hash, key, value)
    }

    /// Account operations
    pub fn get_balance(&self, address: &[u8; 32]) -> Result<u64> {
        self.accounts.get_balance(address)
    }

    pub fn transfer(&mut self, recipient: &[u8; 32], amount: u64) -> Result<()> {
        self.accounts.transfer(&self.contract_hash, recipient, amount)
    }
}
```

### Compute Unit Tracking

```rust
impl<'a> InvokeContext<'a> {
    /// Consume compute units (returns error if budget exceeded)
    pub fn consume_checked(&self, units: u64) -> Result<()> {
        let mut current = self.compute_meter.borrow_mut();
        let new_total = current.checked_add(units)
            .ok_or(EbpfError::OutOfComputeUnits)?;

        if new_total > self.compute_budget {
            return Err(EbpfError::OutOfComputeUnits);
        }

        *current = new_total;
        Ok(())
    }

    /// Get total compute units consumed
    pub fn get_compute_units_consumed(&self) -> u64 {
        *self.compute_meter.borrow()
    }
}
```

---

## Syscall System

### Overview

TOS-VM provides 11 syscalls across 4 categories:

1. **Logging** (1): `tos_log`
2. **Blockchain State** (5): Block/tx info queries
3. **Balance/Transfer** (2): Account operations
4. **Storage** (3): Key-value operations

### Syscall Registration

All syscalls are registered via a single function:

```rust
use tos_tbpf::program::BuiltinProgram;
use tos_program_runtime::InvokeContext;

pub fn register_syscalls(
    loader: &mut BuiltinProgram<InvokeContext>
) -> Result<(), ElfError> {
    // Logging
    loader.register_function("tos_log", logging::TosLog::vm)?;

    // Blockchain state
    loader.register_function("tos_get_block_hash", blockchain::TosGetBlockHash::vm)?;
    loader.register_function("tos_get_block_height", blockchain::TosGetBlockHeight::vm)?;
    loader.register_function("tos_get_tx_hash", blockchain::TosGetTxHash::vm)?;
    loader.register_function("tos_get_tx_sender", blockchain::TosGetTxSender::vm)?;
    loader.register_function("tos_get_contract_hash", blockchain::TosGetContractHash::vm)?;

    // Balance/transfer
    loader.register_function("tos_get_balance", balance::TosGetBalance::vm)?;
    loader.register_function("tos_transfer", balance::TosTransfer::vm)?;

    // Storage
    loader.register_function("tos_storage_read", storage::TosStorageRead::vm)?;
    loader.register_function("tos_storage_write", storage::TosStorageWrite::vm)?;
    loader.register_function("tos_storage_delete", storage::TosStorageDelete::vm)?;

    Ok(())
}
```

### Syscall Pattern

All syscalls follow the same pattern using `declare_builtin_function!`:

```rust
declare_builtin_function!(
    /// Syscall name and documentation
    TosLog,
    fn rust(
        invoke_context: &mut InvokeContext,
        arg1: u64,
        arg2: u64,
        arg3: u64,
        arg4: u64,
        arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // 1. Charge compute units
        invoke_context.consume_checked(COST)?;

        // 2. Translate VM memory to host memory
        let data = translate_slice::<u8>(memory_mapping, arg1, arg2, false)?;

        // 3. Perform operation using InvokeContext methods
        let result = invoke_context.some_operation(data)?;

        // 4. Return result
        Ok(result)
    }
);
```

### Compute Unit Costs

Each syscall charges compute units based on operation complexity:

| Syscall | Base Cost | Per-Byte Cost | Notes |
|---------|-----------|---------------|-------|
| `tos_log` | 100 | 1 | Logging overhead |
| `tos_get_block_hash` | 50 | 0 | Simple query |
| `tos_get_balance` | 100 | 0 | Account lookup |
| `tos_transfer` | 500 | 0 | State modification |
| `tos_storage_read` | 200 | 1 | Depends on value size |
| `tos_storage_write` | 500 | 2 | Higher for writes |
| `tos_storage_delete` | 300 | 0 | Deletion cost |

---

## Memory Management

### Overview

Safe memory translation is critical for security. TOS-VM uses utilities from the standard eBPF memory model.

### Translation Functions

```rust
// Immutable type access
pub fn translate_type<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    check_aligned: bool,
) -> Result<&'a T>

// Mutable type access
pub fn translate_type_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    check_aligned: bool,
) -> Result<&'a mut T>

// Immutable slice access
pub fn translate_slice<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    check_aligned: bool,
) -> Result<&'a [T]>

// Mutable slice access
pub fn translate_slice_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    check_aligned: bool,
) -> Result<&'a mut [T]>
```

### Access Types

- **Load**: Reading from VM memory (immutable access)
- **Store**: Writing to VM memory (mutable access)

### Alignment Checking

For primitive types (u32, u64, etc.), alignment checking is enabled:

```rust
// Check alignment for u64 access
let value = translate_type::<u64>(memory_mapping, addr, true)?;

// No alignment check needed for u8 bytes
let bytes = translate_slice::<u8>(memory_mapping, addr, len, false)?;
```

---

## Testing Strategy

### Test Coverage

- **program-runtime**: 17 tests
  - InvokeContext creation
  - Compute unit tracking
  - Provider method delegation
  - Error handling

- **syscalls**: 23 tests
  - All 11 syscalls tested
  - Error conditions (out of gas, invalid params)
  - Edge cases (empty messages, zero amounts)

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p tos-program-runtime
cargo test -p tos-syscalls

# With output
cargo test -- --nocapture
```

### Test Example

```rust
#[test]
fn test_storage_read_write() {
    let mut storage = NoOpStorage;
    let mut accounts = NoOpAccounts;
    let mut context = InvokeContext::new(10_000, [0u8; 32], &mut storage, &mut accounts);

    // Set up test data
    let key = b"test_key";
    let value = b"test_value";

    // Write
    context.set_storage(key, value).unwrap();

    // Read
    let result = context.get_storage(key).unwrap();
    assert_eq!(result, Some(value.to_vec()));
}
```

---

## Integration Points

### For TOS Blockchain Developers

See `docs/INTEGRATION_GUIDE.md` for detailed instructions on:

1. Implementing `StorageProvider` for your storage backend
2. Implementing `AccountProvider` for your account system
3. Registering syscalls
4. Executing contracts
5. Contract deployment with ELF bytecode

### Quick Integration Example

```rust
// 1. Implement providers
struct MyStorage { /* ... */ }
impl StorageProvider for MyStorage { /* ... */ }

struct MyAccounts { /* ... */ }
impl AccountProvider for MyAccounts { /* ... */ }

// 2. Create context
let mut storage = MyStorage::new();
let mut accounts = MyAccounts::new();
let mut context = InvokeContext::new_with_state(
    1_000_000,  // compute budget
    contract_hash,
    block_hash,
    block_height,
    tx_hash,
    tx_sender,
    &mut storage,
    &mut accounts,
);

// 3. Register syscalls and execute (see INTEGRATION_GUIDE.md)
```

---

## Future Work

### Phase 3: TOS Chain Integration (2-3 days)
- Implement real `StorageProvider` for TOS storage
- Implement real `AccountProvider` for TOS balances
- Wire up contract deployment
- Integration testing

### Phase 4: SDK Development (1 week)
- `entrypoint!` macro
- Syscall bindings
- Common types (Hash, PublicKey, etc.)
- Helper utilities

### Phase 5: Example Contracts (3-5 days)
- Hello World
- Token Contract
- Counter
- C and Rust versions

---

## References

### Internal Documentation
- `README.md` - Project overview and status
- `INTEGRATION_GUIDE.md` - TOS chain integration instructions
- `program-runtime/src/lib.rs` - Core runtime API
- `syscalls/src/lib.rs` - Syscall system overview

### External References
- [tos-tbpf](https://github.com/tos-network/tos-tbpf) - eBPF engine
- [eBPF Instruction Set](https://www.kernel.org/doc/html/latest/bpf/instruction-set.html)
- [Apache License 2.0](../LICENSE)

---

**Last Updated**: 2025-10-30
**Maintainer**: TOS Development Team
**License**: Apache 2.0
