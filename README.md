# TOS VM - TBPF Integration (main branch)

**TOS Virtual Machine** based on TBPF (TOS Berkeley Packet Filter) - A high-performance eBPF execution engine for TOS blockchain smart contracts.

> **Note**: This is the `main` branch featuring a new TBPF-based implementation. The old stack-based VM is preserved in the `dev` branch.

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

3. **Syscall Implementation**
   - `InvokeContext` implementing `ContextObject`
   - Memory translation utilities
   - All core syscalls implemented:
     - Logging: `tos_log`
     - Blockchain: `tos_get_block_hash`, `tos_get_block_height`, `tos_get_tx_hash`, `tos_get_tx_sender`, `tos_get_contract_hash`
     - Balance: `tos_get_balance`, `tos_transfer`
     - Storage: `tos_storage_read`, `tos_storage_write`, `tos_storage_delete`
   - 38 passing tests (15 in program-runtime, 23 in syscalls)

### ⚠️ In Progress

1. **Storage Backend Integration** - Syscalls are implemented but use stub storage:
   - Need to integrate with TOS chain storage layer
   - Need to implement actual balance tracking
   - Need to implement actual transfer logic

2. **SDK Development** - Contract development toolkit needed:
   - `entrypoint!` macro
   - Type definitions
   - Helper functions

### 📋 TODO

1. **Storage Backend**
   - Design StorageProvider trait
   - Implement in-memory storage for testing
   - Integrate with TOS chain storage

2. **Balance & Transfer**
   - Implement balance tracking in InvokeContext
   - Implement transfer logic with balance checks
   - Add transaction effect recording

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
- [tos-tbpf](../tos-tbpf) - The TBPF eBPF engine
- [eBPF Instruction Set](https://www.kernel.org/doc/html/latest/bpf/instruction-set.html)

## 📝 License

Apache License 2.0

Copyright 2025 TOS Network

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

## 🤝 Contributing

This is currently in active development. The API will change significantly.

---

**Branch Strategy:**
- `main`: TBPF-based VM (this branch) - **Active Development**
- `dev`: Original stack-based VM - **Preserved for Reference**
