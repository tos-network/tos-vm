# Third-Party Review Guide for TOS-VM

**Version:** 1.0.0
**Date:** 2025-11-02
**Purpose:** Guide external auditors through TOS-VM codebase review

---

## Overview

This guide helps third-party auditors efficiently review the TOS-VM implementation. The VM is an eBPF-based execution engine for smart contracts, designed with reference to Solana/Agave's architecture.

**Estimated Review Time:** 4-8 hours for thorough review

---

## Quick Start: Priority Reading Order

### 🔴 **MUST READ** (Essential Understanding)

These documents provide the foundation for understanding TOS-VM:

1. **`README.md`** (Root directory) - 10 minutes
   - Project overview and current status
   - Quick architecture diagram
   - Build and test instructions

2. **`docs/SOLANA_AUDIT_REPORT.md`** (This repo) - 30 minutes
   - Comprehensive comparison with Solana/Agave
   - Identified issues and recommendations
   - Security analysis
   - **START HERE for auditors**

3. **`docs/ARCHITECTURE.md`** - 30 minutes
   - Complete architecture documentation
   - Design principles and patterns
   - Component interactions
   - Dependency injection system

### 🟡 **SHOULD READ** (Detailed Understanding)

For deeper technical review:

4. **`docs/INTEGRATION_GUIDE.md`** - 20 minutes
   - How to integrate VM with TOS blockchain
   - Provider trait implementation examples
   - Step-by-step integration instructions

5. **`docs/SOLANA_COMPARISON_ANALYSIS.md`** - 15 minutes
   - Detailed comparison with Solana
   - Design decision rationale
   - Trade-offs analysis

### 🟢 **OPTIONAL READ** (Historical Context)

Background and alternatives:

6. **`docs/TOS_VM_IMPLEMENTATION_GUIDE.md`** - Reference only
   - Partially outdated implementation guide
   - Historical context

7. **`docs/VM_ENGINE_INTEGRATION_PLAN.md`** - Reference only
   - Alternative approach (not implemented)
   - Why current approach was chosen

8. **`docs/TOOLCHAIN_DEVELOPMENT.md`** - For SDK developers
   - Contract compilation toolchain plan
   - LLVM/Clang setup for eBPF targets

---

## Code Review Checklist

### Phase 1: Architecture Review (1 hour)

**Goal:** Understand high-level design

**Documents:**
- ✅ `README.md` - Overview
- ✅ `docs/ARCHITECTURE.md` - Architecture details
- ✅ `docs/SOLANA_AUDIT_REPORT.md` - Audit findings

**Key Questions:**
- Is the dependency injection pattern correctly implemented?
- Does the layering make sense?
- Is the VM properly decoupled from blockchain?

**Success Criteria:**
- Understand the three-tier architecture
- Understand provider trait abstraction
- Understand compute unit tracking

---

### Phase 2: Core Implementation Review (2 hours)

**Goal:** Review critical code paths

#### 2.1 InvokeContext (30 minutes)

**Files:**
- `program-runtime/src/invoke_context.rs`
- `program-runtime/src/lib.rs`

**Review Points:**
```rust
// Key areas to examine:
1. InvokeContext struct definition (lines 21-55)
   - Compute budget tracking
   - Blockchain state storage
   - Provider references

2. ContextObject trait implementation (lines 200-211)
   - consume() method
   - get_remaining() method

3. Storage/account access methods (lines 151-193)
   - get_storage(), set_storage(), delete_storage()
   - get_balance(), transfer()
```

**Critical Questions:**
- ✅ Is compute unit tracking thread-safe?
- ✅ Are provider calls properly abstracted?
- ✅ Is there proper error handling?

---

#### 2.2 Memory Translation (30 minutes)

**Files:**
- `program-runtime/src/memory.rs`

**Review Points:**
```rust
// Key macros and functions:
1. translate_inner! macro (lines 34-43)
2. translate_type_inner! macro (lines 46-64)
3. translate_slice_inner! macro (lines 67-84)
4. address_is_aligned() function (lines 26-31)
```

**Critical Questions:**
- ✅ Are unsafe blocks properly justified?
- ✅ Is alignment checking correct?
- ✅ Are there bounds checks before pointer dereferencing?
- ✅ Could there be buffer overflows?

**Security Focus:**
- Memory alignment validation
- Integer overflow in size calculations
- Pointer arithmetic safety

---

#### 2.3 Syscall Implementations (1 hour)

**Files:**
- `syscalls/src/lib.rs` - Registration
- `syscalls/src/logging.rs` - Logging syscalls
- `syscalls/src/blockchain.rs` - Blockchain state syscalls
- `syscalls/src/balance.rs` - Balance/transfer syscalls
- `syscalls/src/storage.rs` - Storage syscalls

**Review Each Syscall:**

1. **Logging** (`syscalls/src/logging.rs`):
```rust
// TosLog syscall
- Check: Proper compute unit charging
- Check: String validation
- Check: Debug mode check
```

2. **Blockchain State** (`syscalls/src/blockchain.rs`):
```rust
// tos_get_block_hash, tos_get_block_height, etc.
- Check: Memory translation correctness
- Check: Compute costs
- Check: Return value handling
```

3. **Storage** (`syscalls/src/storage.rs`):
```rust
// tos_storage_read, tos_storage_write, tos_storage_delete
- Check: Key/value size limits (MAX_KEY_SIZE, MAX_VALUE_SIZE)
- Check: Buffer overflow protection
- Check: Compute cost scaling with data size
```

4. **Balance/Transfer** (`syscalls/src/balance.rs`):
```rust
// tos_get_balance, tos_transfer
- Check: Address validation
- Check: Transfer amount validation
- Check: Error propagation from providers
```

**Security Checklist:**
- [ ] All syscalls validate input sizes
- [ ] Compute units charged before execution
- [ ] Memory translations use proper access types
- [ ] Error handling prevents panics
- [ ] No unchecked arithmetic operations

---

### Phase 3: Provider Traits Review (30 minutes)

**Files:**
- `program-runtime/src/storage.rs`

**Review Points:**
```rust
// StorageProvider trait (lines 12-24)
- Method signatures
- Error handling
- Thread safety considerations

// AccountProvider trait (lines 26-38)
- Balance query semantics
- Transfer validation
- Error conditions

// NoOp implementations (lines 40-75)
- Test implementation correctness
- Proper error returns
```

**Critical Questions:**
- ✅ Are trait methods appropriately abstracted?
- ✅ Can providers handle concurrent access?
- ✅ Are error types sufficient?
- ⚠️ Is reentrancy considered?

---

### Phase 4: Test Review (1 hour)

**Files:**
- `program-runtime/src/invoke_context.rs` - Tests at bottom
- `program-runtime/src/memory.rs` - Memory tests
- `syscalls/src/*.rs` - Syscall tests

**Test Coverage Analysis:**

Run tests:
```bash
cd program-runtime && cargo test
cd ../syscalls && cargo test
```

**Review Test Cases:**
1. **Compute Unit Tests:**
   - Normal consumption
   - Overflow handling
   - Checked vs unchecked consumption

2. **Memory Translation Tests:**
   - Alignment checks
   - Bounds checking
   - Invalid pointers

3. **Syscall Tests:**
   - Happy path
   - Error conditions
   - Edge cases (empty keys, max sizes)

**Coverage Gaps to Note:**
- [ ] Integration tests with real VM execution
- [ ] Concurrency/race condition tests
- [ ] Fuzzing tests
- [ ] Performance benchmarks

---

## Critical Issues to Verify

Based on the audit report, verify these specific concerns:

### 🔴 **HIGH PRIORITY VERIFICATION**

#### 1. Missing CPI (Cross-Program Invocation)

**What to Look For:**
- No `tos_invoke` syscall implementation
- No mechanism for contracts to call other contracts

**Files to Check:**
- `syscalls/src/lib.rs` - Syscall registration list
- Search for "invoke" in syscalls directory

**Expected Finding:** ❌ CPI not implemented (known issue)

**Recommendation:** Note this as critical missing feature for Phase 3

---

#### 2. Missing Return Data Mechanism

**What to Look For:**
- No `tos_set_return_data` / `tos_get_return_data` syscalls

**Files to Check:**
- `syscalls/src/lib.rs`
- `program-runtime/src/invoke_context.rs`

**Expected Finding:** ❌ Return data not implemented (known issue)

**Recommendation:** Required for inter-contract communication

---

### 🟡 **MEDIUM PRIORITY VERIFICATION**

#### 3. Compute Unit Pricing

**What to Review:**
```rust
// syscalls/src/storage.rs
STORAGE_READ_BASE_COST = 200
STORAGE_WRITE_BASE_COST = 500

// syscalls/src/balance.rs
BALANCE_READ_COST = 100
TRANSFER_COST = 1000
```

**Questions:**
- Are these costs realistic?
- Are they benchmarked?
- Are they configurable?

**Expected Finding:** ⚠️ Costs are placeholder values, need benchmarking

---

#### 4. Security Boundaries

**What to Review:**

1. **Size Limits:**
```rust
// syscalls/src/storage.rs:34-37
MAX_KEY_SIZE = 256
MAX_VALUE_SIZE = 65_536
```
- Are these limits reasonable?
- Are they enforced everywhere?

2. **Compute Budget:**
```rust
// Examples typically use 200_000 compute units
// Is this sufficient? Too much?
```

**Expected Finding:** ✅ Limits exist and are enforced

---

## Security-Focused Review Path

**Time Required:** 2-3 hours

For security-focused auditors, follow this path:

### 1. Attack Surface Analysis

**Entry Points:**
- 11 syscalls (see `syscalls/src/lib.rs:86-106`)
- Memory translation macros
- Provider trait methods

**Potential Attack Vectors:**
- Buffer overflows in memory translation
- Integer overflows in size calculations
- Compute unit exhaustion bypass
- Reentrancy in providers
- Race conditions in RefCell usage

### 2. Memory Safety Review

**Focus Files:**
- `program-runtime/src/memory.rs`

**Check Every `unsafe` Block:**
```bash
# Find all unsafe blocks
grep -n "unsafe" program-runtime/src/memory.rs
```

**Verify:**
- Pointer derefs have bounds checks
- Transmutes are justified
- No undefined behavior possible

### 3. Arithmetic Safety Review

**Search for unchecked operations:**
```bash
# Find potential overflow points
rg "(\+|\*|-|/)" --type rust | grep -v "saturating"
```

**Verify:**
- All size calculations use saturating ops
- Checked arithmetic for compute units
- No wrap-around vulnerabilities

### 4. Error Handling Review

**Check:**
- All Results are propagated
- No `.unwrap()` in production code
- Panics are avoided

```bash
# Find unwraps
rg "\.unwrap\(\)" --type rust
```

### 5. Concurrency Review

**Focus:**
- `RefCell<u64>` in InvokeContext
- Provider trait thread safety
- Potential data races

**Questions:**
- Is RefCell usage safe? (Yes, single-threaded execution)
- Are providers required to be thread-safe? (No, &mut reference)
- Could there be reentrancy issues? (Need to verify in CPI implementation)

---

## Quick Reference: File Structure

```
tos-vm/
├── README.md                    # 🔴 START HERE
├── docs/
│   ├── SOLANA_AUDIT_REPORT.md   # 🔴 MUST READ - Audit findings
│   ├── ARCHITECTURE.md          # 🔴 MUST READ - Architecture
│   ├── INTEGRATION_GUIDE.md     # 🟡 Integration instructions
│   ├── SOLANA_COMPARISON_ANALYSIS.md  # 🟡 Comparison details
│   └── THIRD_PARTY_REVIEW_GUIDE.md    # 🔴 THIS FILE
│
├── program-runtime/             # 🔴 Core VM runtime
│   └── src/
│       ├── invoke_context.rs    # 🔴 Execution context
│       ├── memory.rs            # 🔴 Memory translation
│       ├── storage.rs           # 🔴 Provider traits
│       ├── error.rs             # Error types
│       └── lib.rs               # Public API
│
├── syscalls/                    # 🔴 Syscall implementations
│   └── src/
│       ├── lib.rs               # 🔴 Registration
│       ├── logging.rs           # 🟡 Log syscalls
│       ├── blockchain.rs        # 🟡 Blockchain state
│       ├── balance.rs           # 🟡 Balance/transfer
│       └── storage.rs           # 🟡 Storage operations
│
├── sdk/                         # 🟢 Contract SDK (in progress)
│   └── src/
│       ├── lib.rs
│       └── syscalls.rs          # Syscall bindings
│
└── examples/                    # 🟢 Example contracts
    └── hello-world/             # Basic example
```

**Legend:**
- 🔴 Critical - Must review
- 🟡 Important - Should review
- 🟢 Optional - Can skip in initial review

---

## Common Questions from Auditors

### Q1: Is this a fork of Solana?

**A:** No. TOS-VM is an independent implementation that references Solana/Agave's architecture patterns. Key differences:
- TOS-VM uses dependency injection (traits) instead of direct coupling
- Simplified transaction context
- Different syscall set (TOS-specific)
- Memory translation macros are similar (standard eBPF pattern)

### Q2: What's the relationship with `tos-tbpf`?

**A:** `tos-tbpf` is the underlying eBPF execution engine (similar to Solana's `solana-sbpf`). TOS-VM builds the smart contract runtime layer on top of it.

### Q3: Why is CPI missing?

**A:** Current implementation is Phase 2 (core VM complete). CPI is planned for Phase 3 integration. See `docs/SOLANA_AUDIT_REPORT.md` for details.

### Q4: Are the compute unit costs final?

**A:** No. Current costs are placeholders. Final costs require:
- Benchmarking on target hardware
- Economic model analysis
- Testnet validation

### Q5: How does this compare to Ethereum's EVM?

**A:** TOS-VM uses eBPF (like Solana), not EVM bytecode. Key advantages:
- Native performance (JIT compilation)
- Standard toolchain (LLVM)
- Better verification (eBPF verifier)
- No gas limit stack depth issues

### Q6: What testing has been done?

**A:** Current status:
- ✅ Unit tests (40 passing)
- ❌ Integration tests (pending TOS chain integration)
- ❌ Fuzzing (planned for Phase 4)
- ❌ Security audit (this is pre-audit review)

---

## Audit Report Template

Use this template to structure your findings:

```markdown
# TOS-VM Third-Party Audit Report

**Auditor:** [Your Name/Organization]
**Date:** [Date]
**Version Reviewed:** [Commit Hash]

## Executive Summary
[Overall assessment]

## Findings

### Critical Issues
[Severity: Critical - Requires immediate fix]
- Finding 1
- Finding 2

### High Priority Issues
[Severity: High - Should fix before production]
- Finding 1
- Finding 2

### Medium Priority Issues
[Severity: Medium - Should address]
- Finding 1
- Finding 2

### Low Priority / Informational
[Severity: Low - Nice to have]
- Finding 1
- Finding 2

## Positive Findings
[What was done well]

## Recommendations
[Actionable recommendations]

## Conclusion
[Final verdict and go/no-go recommendation]
```

---

## Contact for Questions

If you have questions during your review:

1. **Technical Questions:** Review existing documentation first
2. **Architecture Clarifications:** See `docs/ARCHITECTURE.md`
3. **Comparison with Solana:** See `docs/SOLANA_AUDIT_REPORT.md`
4. **Integration Questions:** See `docs/INTEGRATION_GUIDE.md`

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-02 | Initial release |

---

**Happy Auditing!**

This guide is maintained to help external reviewers efficiently assess TOS-VM. If you find this guide unclear or incomplete, please provide feedback to improve it.
