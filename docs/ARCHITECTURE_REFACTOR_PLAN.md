# TOS-VM Architecture Refactor Plan

**Date**: 2025-11-02
**Status**: 🔴 **CRITICAL - Required for CPI**
**Priority**: HIGH

---

## Executive Summary

After detailed comparison with Solana/Agave source code, **critical architectural deviations** have been identified in TOS-VM that will prevent proper CPI (Cross-Program Invocation) implementation.

**Key Issue**: Our return data and execution context architecture doesn't match Solana's transaction-level design.

---

## Current Architecture (PROBLEMATIC)

```
InvokeContext (per-invocation)
  ├── compute_budget: u64
  ├── compute_meter: RefCell<u64>
  ├── contract_hash: [u8; 32]
  ├── block_hash/height/tx info
  ├── storage: &mut dyn StorageProvider
  ├── accounts: &mut dyn AccountProvider
  └── return_data: RefCell<Option<...>>  ← PROBLEM!
```

**Issues:**
1. ❌ Return data tied to single invocation context
2. ❌ No instruction context tracking
3. ❌ No transaction-level state
4. ❌ Can't survive across CPI boundaries

---

## Solana Architecture (CORRECT)

```
TransactionContext (transaction-level)
  ├── transaction_accounts: Vec<KeyedAccountSharedData>
  ├── instruction_stack: Vec<usize>
  ├── instruction_trace: Vec<InstructionFrame>
  ├── return_data: TransactionReturnData  ← CORRECT!
  └── rent: Rent

InvokeContext (per-invocation)
  ├── transaction_context: &mut TransactionContext  ← References!
  ├── program_cache_for_tx_batch: &mut ProgramCacheForTxBatch
  ├── environment_config: EnvironmentConfig
  ├── compute_budget: SVMTransactionExecutionBudget
  ├── execution_cost: SVMTransactionExecutionCost
  └── compute_meter: RefCell<u64>
```

**Key Insights:**
1. ✅ Transaction-level state in TransactionContext
2. ✅ InvokeContext references TransactionContext
3. ✅ Return data survives across CPI calls
4. ✅ Instruction stack enables proper CPI

---

## Required Changes

### Phase 1: Create Transaction Context Structure

**New File**: `program-runtime/src/transaction_context.rs`

```rust
/// Transaction-level execution state
pub struct TransactionContext {
    /// Return data from last invocation
    pub return_data: TransactionReturnData,

    /// Instruction execution stack
    pub instruction_stack: Vec<InstructionFrame>,

    /// Current instruction index
    pub current_instruction_index: usize,

    /// Maximum stack depth
    pub max_stack_depth: usize,
}

/// Return data structure
#[derive(Clone, Debug, Default)]
pub struct TransactionReturnData {
    pub program_id: [u8; 32],
    pub data: Vec<u8>,
}

/// Single instruction frame
#[derive(Clone, Debug)]
pub struct InstructionFrame {
    pub program_id: [u8; 32],
    pub is_root: bool,
    pub caller_index: Option<usize>,
}
```

**Justification**: Matches Solana's design but simplified for TOS needs.

---

### Phase 2: Refactor InvokeContext

**File**: `program-runtime/src/invoke_context.rs`

**BEFORE** (Current):
```rust
pub struct InvokeContext<'a> {
    compute_budget: u64,
    compute_meter: RefCell<u64>,
    contract_hash: [u8; 32],
    storage: &'a mut dyn StorageProvider,
    accounts: &'a mut dyn AccountProvider,
    return_data: RefCell<Option<...>>,  // Remove this!
}
```

**AFTER** (Required):
```rust
pub struct InvokeContext<'a> {
    // NEW: Reference to transaction context
    transaction_context: &'a mut TransactionContext,

    // Existing fields
    compute_budget: u64,
    compute_meter: RefCell<u64>,

    // Blockchain state (can stay)
    block_hash: [u8; 32],
    block_height: u64,
    tx_hash: [u8; 32],
    tx_sender: [u8; 32],

    // Providers (stay)
    storage: &'a mut dyn StorageProvider,
    accounts: &'a mut dyn AccountProvider,

    // Debug (stay)
    debug_mode: bool,
}

impl<'a> InvokeContext<'a> {
    // NEW: Get current program ID from instruction stack
    pub fn get_current_program_id(&self) -> Result<[u8; 32], EbpfError> {
        self.transaction_context
            .get_current_instruction()
            .map(|frame| frame.program_id)
    }

    // CHANGED: Delegate to transaction_context
    pub fn set_return_data(&mut self, program_id: [u8; 32], data: Vec<u8>)
        -> Result<(), EbpfError> {
        self.transaction_context.set_return_data(program_id, data)
    }

    pub fn get_return_data(&self) -> Option<([u8; 32], Vec<u8>)> {
        self.transaction_context.get_return_data()
    }
}
```

---

### Phase 3: Update Return Data Syscalls

**File**: `syscalls/src/return_data.rs`

**Changes:**
1. Get program_id from instruction context:
```rust
// OLD (Wrong):
let program_id = invoke_context.contract_hash;

// NEW (Correct):
let program_id = invoke_context.get_current_program_id()?;
```

2. Use transaction_context for storage:
```rust
invoke_context.transaction_context.set_return_data(program_id, data)?;
```

3. Add proper compute cost calculation (future):
```rust
let cost = len
    .checked_div(invoke_context.get_cpi_bytes_per_unit())
    .unwrap_or(u64::MAX)
    .saturating_add(invoke_context.get_syscall_base_cost());
```

---

### Phase 4: Enable CPI Support

Once transaction context exists, CPI becomes possible:

**New File**: `syscalls/src/cpi.rs`

```rust
declare_builtin_function!(
    TosInvoke,
    fn rust(
        invoke_context: &mut InvokeContext,
        instruction_ptr: u64,
        account_infos_ptr: u64,
        account_infos_len: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // 1. Parse instruction from memory
        // 2. Push new frame to instruction_stack
        // 3. Execute callee program
        // 4. Pop frame
        // 5. Propagate return data
    }
);
```

---

## Migration Strategy

### Step 1: Add TransactionContext (Non-Breaking)
- Create `transaction_context.rs`
- Define structures
- Write tests
- **No changes to existing code yet**

### Step 2: Add Optional TransactionContext to InvokeContext
```rust
pub struct InvokeContext<'a> {
    // Keep old fields
    return_data: RefCell<Option<...>>,  // Deprecated but keep

    // Add new field (optional during migration)
    transaction_context: Option<&'a mut TransactionContext>,
}
```

### Step 3: Gradual Migration
- New code uses `transaction_context`
- Old code still works
- Tests updated incrementally

### Step 4: Remove Old Return Data
- Remove `return_data` from InvokeContext
- Update all syscalls
- Clean up tests

---

## Testing Strategy

### Unit Tests
- ✅ Test TransactionContext in isolation
- ✅ Test InstructionStack push/pop
- ✅ Test return data flow

### Integration Tests
- ✅ Test return data across "CPI boundaries"
- ✅ Test instruction context tracking
- ✅ Test error propagation

### Compatibility Tests
- ✅ Ensure existing tests still pass
- ✅ Add tests for Solana compatibility
- ✅ Benchmark performance impact

---

## Timeline Estimate

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| 1. TransactionContext | 1 day | None |
| 2. InvokeContext Refactor | 2 days | Phase 1 |
| 3. Update Syscalls | 1 day | Phase 2 |
| 4. CPI Implementation | 3-4 days | Phase 3 |
| **Total** | **7-8 days** | |

---

## Risks and Mitigation

### Risk 1: Breaking Existing Code
**Mitigation**: Use optional/gradual migration approach

### Risk 2: Performance Impact
**Mitigation**: Benchmark before and after, optimize if needed

### Risk 3: Complexity Increase
**Mitigation**: Comprehensive documentation and examples

---

## Alternative: Minimal Changes Approach

If full refactor is not feasible, minimal changes:

1. ✅ **DONE**: Fix get_return_data to truncate (Solana behavior)
2. ⏳ Add `instruction_stack: Vec<[u8; 32]>` to InvokeContext
3. ⏳ Track program_id in stack instead of flat field
4. ⏳ Make return_data accessible across stack frames

**Pros**: Less refactoring, faster implementation
**Cons**: Still not fully Solana-compatible, harder to add CPI later

---

## Decision Required

**Option A: Full Refactor** (Recommended)
- ✅ Solana-compatible architecture
- ✅ Proper foundation for CPI
- ✅ Easier maintenance long-term
- ⚠️ More upfront work (7-8 days)

**Option B: Minimal Changes** (Quick Fix)
- ✅ Faster (2-3 days)
- ✅ Less breaking changes
- ⚠️ Not fully Solana-compatible
- ⚠️ Will need refactor for complex CPI

---

## Recommendation

**Go with Option A: Full Refactor**

**Rationale**:
1. We're still early in development
2. CPI is critical feature
3. Solana compatibility is important
4. Better to fix architecture now than later
5. 7-8 days is reasonable investment

**Next Steps**:
1. Get stakeholder approval for refactor
2. Create detailed implementation PRs
3. Start with Phase 1 (TransactionContext)
4. Proceed phase by phase with tests

---

## References

- Solana TransactionContext: `agave/transaction-context/src/lib.rs`
- Solana InvokeContext: `agave/program-runtime/src/invoke_context.rs`
- Solana Return Data Syscalls: `agave/syscalls/src/lib.rs`
- TOS-VM Comparison: `docs/SOLANA_VS_TOS_RETURN_DATA_COMPARISON.md`

---

**Status**: Awaiting decision on refactor approach
**Owner**: TOS-VM Development Team
**Review Date**: 2025-11-02
