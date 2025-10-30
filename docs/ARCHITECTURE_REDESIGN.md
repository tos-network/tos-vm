# TOS VM Architecture Redesign - Solana Alignment

**Date**: 2025-10-30
**Status**: Design Phase
**Goal**: Align TOS VM architecture completely with Solana's approach

---

## Current vs Proposed Architecture

### Current Architecture (With Wrapper)

```
TOS Blockchain
     ↓
  TosVm (wrapper class)
     ↓  - new(), execute()
     ↓  - TosContext inside
     ↓
  tos-tbpf::EbpfVm
```

### Proposed Architecture (Solana-Aligned) ✅

```
TOS Blockchain
     ↓
  InvokeContext (implements ContextObject)
     ↓  - Direct use of tos-tbpf types
     ↓  - No wrapper layer
     ↓
  tos-tbpf::EbpfVm (used directly)
```

---

## Detailed Comparison

### Solana's Approach

```rust
// 1. Context implements ContextObject
pub struct InvokeContext<'a> {
    transaction_context: &'a mut TransactionContext,
    compute_meter: RefCell<u64>,
    // ... other blockchain state
}

impl ContextObject for InvokeContext<'_> {
    fn consume(&mut self, amount: u64) { ... }
    fn get_remaining(&self) -> u64 { ... }
}

// 2. Direct VM usage (no wrapper)
let mut vm = EbpfVm::new(loader, version, context, text, pc);
let result = vm.execute_program(&executable, verify);

// 3. Syscalls use macro
declare_builtin_function!(
    SyscallLog,
    fn rust(context: &mut InvokeContext, ...) -> Result<u64, Error> { ... }
);
```

### Our Aligned Approach

```rust
// 1. InvokeContext implements ContextObject
pub struct InvokeContext<'a> {
    // TOS blockchain state
    contract_hash: Hash,
    block_hash: Hash,
    tx_hash: Hash,
    compute_meter: RefCell<u64>,
    storage_provider: &'a dyn StorageProvider,
    // ... other TOS state
}

impl ContextObject for InvokeContext<'_> {
    fn consume(&mut self, amount: u64) { ... }
    fn get_remaining(&self) -> u64 { ... }
}

// 2. Direct VM usage (no TosVm wrapper)
let mut vm = EbpfVm::new(loader, version, context, text, pc);
let result = vm.execute_program(&executable, verify);

// 3. Syscalls use macro
declare_builtin_function!(
    TosLog,
    fn rust(context: &mut InvokeContext, ...) -> Result<u64, Error> { ... }
);
```

---

## New File Structure

```
tos-vm/
├── Cargo.toml                  # Workspace

├── program-runtime/            # Renamed from tbpf/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── invoke_context.rs  # Main execution context (was context.rs)
│       ├── memory.rs          # Memory translation utilities
│       └── error.rs           # Error types

├── syscalls/                   # New crate (was inside tbpf/)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── logging.rs         # tos_log (was log.rs)
│       ├── balance.rs         # tos_get_balance, tos_transfer
│       ├── storage.rs         # tos_storage_*
│       └── chain.rs           # tos_get_block_*, tos_get_tx_*

├── sdk/                        # Contract development SDK
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── entrypoint.rs      # Entry point macro
│       └── syscalls.rs        # Syscall bindings for contracts

├── examples/
├── tests/
└── docs/
```

---

## Key Changes

### 1. Remove TosVm Wrapper ❌

**Delete**: `tbpf/src/vm.rs` with `TosVm` struct

**Rationale**:
- Unnecessary abstraction
- Makes code harder to follow
- Diverges from Solana patterns

### 2. Rename TosContext → InvokeContext ✅

**Change**: `tbpf/src/context.rs` → `program-runtime/src/invoke_context.rs`

**Rationale**:
- Same naming as Solana
- More descriptive name
- Easier to reference Solana docs

### 3. Split tbpf/ into Two Crates ✅

**Rename**: `tbpf/` → `program-runtime/`
**Create**: `syscalls/` as separate crate

**Rationale**:
- Clearer separation of concerns
- program-runtime: Execution infrastructure
- syscalls: TOS-specific syscall implementations
- Matches Solana's structure exactly

### 4. Add Memory Utilities ✅

**Create**: `program-runtime/src/memory.rs`

```rust
pub fn translate_type<'a, T>(
    memory_mapping: &MemoryMapping,
    addr: u64,
) -> Result<&'a T, Error>

pub fn translate_slice<'a, T>(
    memory_mapping: &MemoryMapping,
    addr: u64,
    len: u64,
) -> Result<&'a [T], Error>
```

---

## Migration Path

### Phase 1: Restructure Files

1. Rename `tbpf/` → `program-runtime/`
2. Rename `context.rs` → `invoke_context.rs`
3. Delete `vm.rs` (remove TosVm)
4. Create `syscalls/` crate
5. Move syscall implementations to `syscalls/`

### Phase 2: Update InvokeContext

```rust
// program-runtime/src/invoke_context.rs

pub struct InvokeContext<'a> {
    // Compute tracking
    compute_budget: u64,
    compute_meter: RefCell<u64>,

    // TOS blockchain state
    pub contract_hash: Hash,
    pub block_hash: Hash,
    pub block_height: u64,
    pub tx_hash: Hash,
    pub tx_sender: PublicKey,

    // State access
    storage_provider: &'a dyn StorageProvider,

    // Logging
    log_collector: Option<Rc<RefCell<LogCollector>>>,

    // Debug mode
    pub debug_mode: bool,
}

impl<'a> InvokeContext<'a> {
    pub fn new(
        compute_budget: u64,
        contract_hash: Hash,
        storage_provider: &'a dyn StorageProvider,
    ) -> Self {
        Self {
            compute_budget,
            compute_meter: RefCell::new(compute_budget),
            contract_hash,
            // ... initialize other fields
        }
    }

    // Compute budget methods
    pub fn consume_checked(&mut self, amount: u64) -> Result<(), Error> {
        let mut meter = self.compute_meter.borrow_mut();
        if *meter < amount {
            return Err(Error::OutOfComputeUnits);
        }
        *meter -= amount;
        Ok(())
    }

    // Storage access methods
    pub fn get_storage(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        self.storage_provider.load_storage(&self.contract_hash, key)
    }

    pub fn set_storage(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.storage_provider.store_storage(&self.contract_hash, key, value)
    }
}

impl ContextObject for InvokeContext<'_> {
    fn consume(&mut self, amount: u64) {
        let mut meter = self.compute_meter.borrow_mut();
        *meter = meter.saturating_sub(amount);
    }

    fn get_remaining(&self) -> u64 {
        *self.compute_meter.borrow()
    }
}
```

### Phase 3: Update Syscalls

```rust
// syscalls/src/logging.rs

use tos_tbpf::declare_builtin_function;

declare_builtin_function!(
    /// Log a message from contract
    TosLog,
    fn rust(
        invoke_context: &mut InvokeContext,
        msg_ptr: u64,
        msg_len: u64,
        _arg3: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Error> {
        // Charge compute units
        invoke_context.consume_checked(msg_len)?;

        // Translate message from VM memory
        let msg_bytes = translate_slice::<u8>(
            memory_mapping,
            msg_ptr,
            msg_len,
        )?;

        let msg = std::str::from_utf8(msg_bytes)
            .map_err(|e| Error::InvalidUtf8)?;

        // Log if debug mode
        if invoke_context.debug_mode {
            log::info!("[Contract {}]: {}", invoke_context.contract_hash, msg);
        }

        Ok(0)
    }
);
```

### Phase 4: Update Integration Code

**Old way (with TosVm)**:
```rust
let mut vm = TosVm::new(elf_bytes, compute_budget)?;
vm.enable_debug();
let result = vm.execute(input)?;
```

**New way (Solana-aligned)**:
```rust
// 1. Create loader with syscalls
let mut loader = BuiltinProgram::new_loader(config);
syscalls::register_all(&mut loader);
let loader = Arc::new(loader);

// 2. Load executable
let executable = Executable::from_elf(elf_bytes, loader)?;

// 3. Create invoke context
let mut invoke_context = InvokeContext::new(
    compute_budget,
    contract_hash,
    storage_provider,
);
invoke_context.enable_debug();

// 4. Create VM and execute
let mut vm = EbpfVm::new(
    executable.get_loader().clone(),
    executable.get_tbpf_version(),
    &mut invoke_context,
    executable.get_text_bytes().1,
    0,
);

let (instruction_count, result) = vm.execute_program(&executable, true)?;
```

---

## Benefits of This Approach

### ✅ Advantages

1. **Solana Compatibility**
   - Easier to port Solana contracts
   - Can reference Solana documentation directly
   - Easier to use Solana tools

2. **Simpler Architecture**
   - No unnecessary wrapper layer
   - Fewer types to understand
   - More direct code flow

3. **Better Maintainability**
   - Follows proven patterns
   - Easier to sync with upstream tos-tbpf changes
   - Clear separation: program-runtime vs syscalls

4. **Developer Experience**
   - TOS developers familiar with Solana will feel at home
   - Less "TOS-specific magic"
   - Better error messages from tbpf

### ⚠️ Trade-offs

1. **More Verbose API**
   - Need to create loader, executable, context manually
   - More boilerplate in integration code
   - **Mitigation**: Provide helper functions in program-runtime

2. **Less "Rusty" API**
   - More C-style (matching eBPF model)
   - **Mitigation**: This is actually closer to the VM's reality

3. **Breaking Changes**
   - Current code will need rewrite
   - **Mitigation**: We haven't released yet, so acceptable

---

## Implementation Checklist

### File Restructuring
- [ ] Rename `tbpf/` to `program-runtime/`
- [ ] Create `syscalls/` crate
- [ ] Move syscall code to `syscalls/`
- [ ] Delete `tbpf/src/vm.rs`
- [ ] Rename `context.rs` to `invoke_context.rs`
- [ ] Create `program-runtime/src/memory.rs`

### Code Updates
- [ ] Expand InvokeContext with TOS state
- [ ] Add storage access methods
- [ ] Add logging support
- [ ] Implement memory translation helpers
- [ ] Convert syscalls to use `declare_builtin_function!`
- [ ] Update error types

### Integration
- [ ] Update TOS main chain integration code
- [ ] Remove references to TosVm
- [ ] Use EbpfVm directly
- [ ] Update tests

### Documentation
- [ ] Update README.md
- [ ] Update ARCHITECTURE diagram
- [ ] Add migration guide
- [ ] Document differences from Solana (where intentional)

---

## Timeline

**Week 1** (Now - 2025-11-06):
- Restructure files and directories
- Update InvokeContext
- Fix one syscall (tos_log) as example

**Week 2** (2025-11-07 - 2025-11-13):
- Implement all basic syscalls
- Add memory utilities
- Update tests

**Week 3** (2025-11-14 - 2025-11-20):
- Integration with TOS chain
- End-to-end testing
- Documentation

---

## Conclusion

**Decision**: ✅ **Fully align with Solana architecture**

This redesign will:
- Remove unnecessary abstraction (TosVm)
- Use InvokeContext pattern directly
- Split into program-runtime + syscalls
- Match Solana's proven architecture

**Start**: Immediately
**Target**: Production-ready in 3 weeks

---

**Approved by**: Awaiting confirmation
**Next Action**: Begin file restructuring
