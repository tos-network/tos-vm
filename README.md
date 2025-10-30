# TOS VM - TBPF Integration (main branch)

**TOS Virtual Machine** based on TBPF (TOS Berkeley Packet Filter) - A high-performance eBPF execution engine for TOS blockchain smart contracts.

> **Note**: This is the `main` branch featuring a complete rewrite to integrate TBPF. The old stack-based VM is preserved in the `dev` branch.

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│          TOS Blockchain                 │
│  (transaction verification/execution)   │
└────────────────┬────────────────────────┘
                 │
                 │ invoke_contract()
                 ▼
┌─────────────────────────────────────────┐
│         tos-vm (this project)           │
│  ┌─────────────────────────────────┐    │
│  │       tos-vm-tbpf               │    │
│  │  - Load ELF bytecode            │    │
│  │  - Setup execution context      │    │
│  │  - Register TOS syscalls        │    │
│  └──────────┬──────────────────────┘    │
│             │                            │
│             │ execute()                  │
│             ▼                            │
│  ┌─────────────────────────────────┐    │
│  │    TOS Syscalls                 │    │
│  │  - tos_log                      │    │
│  │  - tos_get_balance              │    │
│  │  - tos_transfer                 │    │
│  │  - tos_storage_*                │    │
│  │  - ...                          │    │
│  └──────────┬──────────────────────┘    │
└─────────────┼───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│         tos-tbpf (forked rbpf)          │
│  - Interpreter / JIT compiler           │
│  - Verifier                             │
│  - Memory management                    │
└─────────────────────────────────────────┘
```

## 📦 Project Structure

```
tos-vm/
├── tbpf/                    # Core: TBPF engine integration for TOS
│   ├── src/
│   │   ├── lib.rs           # Public API
│   │   ├── vm.rs            # TosVm - main VM wrapper
│   │   ├── context.rs       # TosContext - execution context
│   │   ├── error.rs         # Error types
│   │   └── syscalls/        # TOS-specific syscalls
│   │       ├── mod.rs
│   │       └── log.rs       # tos_log syscall
│   └── Cargo.toml
│
├── sdk/                     # SDK for contract development (future)
│   ├── src/
│   │   └── lib.rs           # Macros and utilities for contracts
│   └── Cargo.toml
│
├── examples/                # Example contracts
│   ├── hello-world/         # C language example
│   └── counter-rust/        # Rust example
│
├── tests/                   # Integration tests
│   └── basic_execution.rs
│
├── docs/                    # Documentation
│   ├── VM_ENGINE_INTEGRATION_PLAN.md      # Complete integration plan
│   ├── TOS_VM_IMPLEMENTATION_GUIDE.md     # Implementation guide
│   ├── TOS_VM_TBPF_REFACTORING_PLAN.md    # Refactoring strategy
│   └── README_VM_INTEGRATION.md           # Quick start overview
│
├── Cargo.toml               # Workspace configuration
└── README.md                # This file
```

## 🚀 Current Status

### ✅ Completed

1. **Branch Setup**
   - Created `main` branch from `dev`
   - Cleaned up old VM implementation
   - Established new directory structure

2. **Core Architecture**
   - Workspace configured with `tbpf` and `sdk` crates
   - Dependency on `tos-tbpf` (forked from Solana rbpf)
   - Basic module structure defined

3. **Initial Implementation**
   - `TosVm` struct with public API design
   - `TosContext` implementing `ContextObject`
   - `TosVmError` error types
   - `tos_log` syscall (example implementation)

### ⚠️ In Progress

1. **API Adaptation** - The code currently has compilation errors due to API differences between our design and the actual `tos-tbpf` API. These need to be fixed:
   - `BuiltinProgram::new_loader()` signature
   - `RequisiteVerifier` constructor
   - `EbpfVm::new()` parameters
   - `Executable` method names

2. **Syscall Implementation** - Only `tos_log` is partially implemented. Need to add:
   - `tos_get_balance`
   - `tos_transfer`
   - `tos_storage_*`
   - etc.

### 📋 TODO

1. **Fix Compilation**
   - Study `tos-tbpf` examples and tests
   - Adapt VM initialization code
   - Fix syscall registration
   - Get basic example working

2. **Implement Syscalls** (Priority order)
   - P0: `tos_log`, `tos_get_contract_hash`
   - P1: `tos_get_balance`, `tos_transfer`
   - P2: `tos_storage_load`, `tos_storage_store`
   - P3: Block/TX info syscalls
   - P4: Asset management syscalls

3. **Testing**
   - Create hello-world contract in C
   - Write integration tests
   - Performance benchmarks

4. **Integration with TOS Chain**
   - Update `DeployContractPayload` to support ELF
   - Modify contract execution in TOS daemon
   - Update storage layer

5. **SDK Development**
   - `entrypoint!` macro
   - Syscall bindings
   - Common types (Hash, PublicKey, etc.)

## 🔧 Development

### Prerequisites

```bash
rustc >= 1.83
tos-tbpf located at ../tos-tbpf
```

### Build

```bash
cargo build --workspace
```

### Test

```bash
cargo test --workspace
```

## 📚 References

### Internal Documentation
- [VM_ENGINE_INTEGRATION_PLAN.md](docs/VM_ENGINE_INTEGRATION_PLAN.md) - Complete 10-14 week integration plan
- [TOS_VM_IMPLEMENTATION_GUIDE.md](docs/TOS_VM_IMPLEMENTATION_GUIDE.md) - Step-by-step implementation guide
- [TOS_VM_TBPF_REFACTORING_PLAN.md](docs/TOS_VM_TBPF_REFACTORING_PLAN.md) - Refactoring strategy
- [README_VM_INTEGRATION.md](docs/README_VM_INTEGRATION.md) - Quick start overview

### External References
- [tos-tbpf](../tos-tbpf) - The forked eBPF engine
- [Solana sBPF Documentation](https://solana.com/docs/programs/faq#berkeley-packet-filter-bpf)
- [eBPF Instruction Set](https://www.kernel.org/doc/html/latest/bpf/instruction-set.html)

## 📝 License

BSD 3-Clause License

This project includes code derived from third-party open source projects.
See [NOTICE](NOTICE) file for details on third-party attributions.

## 🤝 Contributing

This is currently in active development. The API will change significantly.

---

**Branch Strategy:**
- `main`: TBPF-based VM (this branch) - **Active Development**
- `dev`: Original stack-based VM - **Preserved for Reference**
