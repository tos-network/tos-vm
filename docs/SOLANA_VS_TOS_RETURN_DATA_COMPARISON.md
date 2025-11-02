# Solana vs TOS-VM Return Data Implementation Comparison

## KEY FINDINGS: 🔴 **CRITICAL DEVIATIONS FOUND**

### 1. Storage Location ❌ **WRONG**

**Solana:**
```rust
// In TransactionContext
pub struct TransactionContext {
    return_data: TransactionReturnData,  // ← Here
}

pub struct TransactionReturnData {
    pub program_id: Pubkey,
    pub data: Vec<u8>,
}
```

**TOS-VM (CURRENT - WRONG):**
```rust
// In InvokeContext
pub struct InvokeContext<'a> {
    return_data: RefCell<Option<([u8; 32], Vec<u8>)>>,  // ← Wrong location!
}
```

**Problem**: Return data should survive across different InvokeContext instances in CPI calls.
**Solution**: Move to a transaction-level context or make it accessible across invocations.

---

### 2. Program ID Source ❌ **WRONG**

**Solana:**
```rust
let program_id = *transaction_context
    .get_current_instruction_context()
    .and_then(|instruction_context| {
        instruction_context.get_program_key()
    })?;
```

**TOS-VM (CURRENT - WRONG):**
```rust
let program_id = invoke_context.contract_hash;  // ← Directly from context
```

**Problem**: Should get from instruction context, not directly from invoke context.
**Solution**: Need proper instruction context tracking.

---

### 3. Compute Cost Calculation ❌ **WRONG**

**Solana:**
```rust
// set_return_data
let cost = len
    .checked_div(execution_cost.cpi_bytes_per_unit)  // Dynamic!
    .unwrap_or(u64::MAX)
    .saturating_add(execution_cost.syscall_base_cost);

// get_return_data
let cost = length
    .saturating_add(size_of::<Pubkey>() as u64)
    .checked_div(execution_cost.cpi_bytes_per_unit)
    .unwrap_or(u64::MAX);
```

**TOS-VM (CURRENT - WRONG):**
```rust
// set_return_data
let cost = RETURN_DATA_SET_BASE_COST  // Fixed!
    .saturating_add(data_len.saturating_mul(RETURN_DATA_SET_BYTE_COST));

// get_return_data
invoke_context.consume_checked(RETURN_DATA_GET_COST)?;  // Fixed!
```

**Problem**: Using fixed costs instead of dynamic calculation based on `cpi_bytes_per_unit`.
**Solution**: Need `execution_cost` configuration in InvokeContext.

---

### 4. get_return_data Behavior ⚠️ **DIFFERENT**

**Solana:**
```rust
let length = length.min(return_data.len() as u64);  // ← Uses min!
// ... copy data ...
Ok(return_data.len() as u64)  // Returns actual length
```

**TOS-VM (CURRENT):**
```rust
if data_len < return_data_len {
    return Err(Box::new(SyscallError::BufferTooSmall(...)));  // ← Errors!
}
```

**Problem**: Solana truncates to buffer size, we error out.
**Solution**: Change to use `min` and truncate like Solana.

---

### 5. Empty Data Handling ✅ **CORRECT**

**Solana:**
```rust
let return_data = if len == 0 {
    Vec::new()
} else {
    translate_slice::<u8>(...)?
        .to_vec()
};
```

**TOS-VM:**
```rust
let data = translate_slice::<u8>(...)?;
invoke_context.set_return_data(program_id, data.to_vec())?;
```

**Status**: Both handle empty data, but Solana explicitly creates empty Vec.

---

## CRITICAL ISSUES SUMMARY

| Issue | Severity | Impact | Fix Complexity |
|-------|----------|--------|----------------|
| Storage in wrong location | 🔴 Critical | Breaks CPI | High |
| Program ID from wrong source | 🔴 Critical | Wrong tracking | Medium |
| Fixed compute costs | 🟡 High | Not scalable | Medium |
| get_return_data errors instead of truncates | 🟡 High | Incompatible behavior | Low |

---

## REQUIRED CHANGES

### Priority 1: Architecture Changes
1. Create TransactionContext-like structure
2. Move return_data to transaction level
3. Add instruction_context tracking

### Priority 2: Behavior Fixes
1. Fix get_return_data to truncate, not error
2. Get program_id from instruction context
3. Add execution_cost to InvokeContext

### Priority 3: Cost Model
1. Add `cpi_bytes_per_unit` configuration
2. Update compute cost calculations
3. Benchmark and tune values

---

## RECOMMENDATION

**We need to PAUSE and REFACTOR before continuing with CPI.**

The current implementation deviates significantly from Solana's architecture:
- Return data won't work properly with CPI
- Program ID tracking is wrong
- Compute cost model is incompatible

**Suggested Approach:**
1. Review Solana's TransactionContext architecture
2. Decide if we need similar structure for TOS-VM
3. Refactor return data implementation to match Solana
4. THEN implement CPI with correct foundation
