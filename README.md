# TOS VM - TBPF Integration (main branch)

**TOS Virtual Machine** based on TBPF (TOS Berkeley Packet Filter) - A high-performance eBPF execution engine for TOS blockchain smart contracts.

> **Note**: This is the `main` branch featuring a new TBPF-based implementation. The old stack-based VM is preserved in the `dev` branch.

## 🏗️ Architecture

TOS-VM is designed as an **independent, pluggable component** using dependency injection:

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

### Key Design Principles

1. **Dependency Injection**: TOS chain injects storage/account providers
2. **Trait-Based Abstraction**: Clear interfaces via Rust traits
3. **Zero Coupling**: VM doesn't depend on TOS chain implementation
4. **Easy Testing**: Includes NoOp providers for standalone testing

## 📦 Project Structure

```
tos-vm/
├── program-runtime/         # Core runtime (independent)
│   ├── src/
│   │   ├── lib.rs           # Public API
│   │   ├── invoke_context.rs # Execution context
│   │   ├── memory.rs        # Memory translation utilities
│   │   ├── storage.rs       # Provider traits (StorageProvider, AccountProvider)
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
├── sdk/                     # SDK for contract development (TODO)
│   ├── src/
│   │   └── lib.rs           # Macros and utilities for contracts
│   └── Cargo.toml
│
├── examples/                # Example contracts (TODO)
│   ├── hello-world/         # C language example
│   └── counter-rust/        # Rust example
│
├── docs/                    # Documentation
│   ├── VM_ENGINE_INTEGRATION_PLAN.md      # Complete integration plan
│   ├── TOS_VM_IMPLEMENTATION_GUIDE.md     # Implementation guide
│   ├── TOS_VM_TBPF_REFACTORING_PLAN.md    # Refactoring strategy
│   └── README_VM_INTEGRATION.md           # Quick start overview
│
├── Cargo.toml               # Workspace configuration
├── LICENSE                  # Apache 2.0 License
└── README.md                # This file
```

## 🚀 Current Status

### ✅ Completed (Phase 1 & 2)

1. **Core Architecture** ✅
   - Workspace configured with `program-runtime`, `syscalls`, and `sdk` crates
   - Dependency on `tos-tbpf` (eBPF execution engine)
   - Clean module structure following eBPF best practices

2. **Dependency Injection System** ✅
   - `StorageProvider` trait - contract storage interface
   - `AccountProvider` trait - balance/transfer interface
   - `NoOpStorage` and `NoOpAccounts` - testing implementations
   - `InvokeContext` - execution context with injected providers

3. **Memory Management** ✅
   - Safe VM ↔ host memory translation
   - Alignment checking for typed access
   - Support for Load/Store access patterns
   - Comprehensive macros for common operations

4. **Complete Syscall System** ✅
   - **Logging**: `tos_log` - debug output
   - **Blockchain State**:
     - `tos_get_block_hash` - current block hash
     - `tos_get_block_height` - current block height
     - `tos_get_tx_hash` - transaction hash
     - `tos_get_tx_sender` - transaction sender
     - `tos_get_contract_hash` - executing contract
   - **Balance/Transfer**:
     - `tos_get_balance` - query account balance
     - `tos_transfer` - transfer tokens
   - **Storage**:
     - `tos_storage_read` - read key-value
     - `tos_storage_write` - write key-value
     - `tos_storage_delete` - delete key
   - All with compute unit tracking and limits

5. **Testing & Quality** ✅
   - 40 passing tests (17 runtime + 23 syscalls)
   - Comprehensive test coverage for all syscalls
   - Error handling and edge cases tested
   - All code documented with rustdoc

### 📋 Next Steps (Phase 3 - Integration)

**The VM core is complete! Next steps are integration-specific:**

1. **TOS Chain Integration** (2-3 days)
   - Implement `StorageProvider` for TOS chain storage backend
   - Implement `AccountProvider` for TOS chain balance/transfer operations
   - Wire up contract deployment to load ELF bytecode
   - Update transaction execution to inject providers into `InvokeContext`
   - See `docs/INTEGRATION_GUIDE.md` for detailed instructions

2. **SDK Development** (1 week)
   - `entrypoint!` macro for contract entry points
   - Syscall bindings (Rust wrappers for all 11 syscalls)
   - Common types (Hash, PublicKey, Balance, etc.)
   - Helper functions and utilities

3. **Example Contracts** (3-5 days)
   - Hello World (logging demonstration)
   - Token Contract (storage + transfer)
   - Counter (state management)
   - Both C and Rust versions with build scripts

4. **Testing & Optimization** (ongoing)
   - End-to-end integration tests with real TOS chain
   - Performance benchmarks (compute unit costs)
   - Gas cost tuning and optimization
   - Security audit preparation

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

## 📚 Documentation

### Core Documentation
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Complete architecture overview and implementation details
- **[INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md)** - Step-by-step guide for TOS chain integration
- **[README.md](README.md)** - This file (project overview and quick start)

### Reference Documentation
- [VM_ENGINE_INTEGRATION_PLAN.md](docs/VM_ENGINE_INTEGRATION_PLAN.md) - Alternative approach (not implemented)
- [TOS_VM_IMPLEMENTATION_GUIDE.md](docs/TOS_VM_IMPLEMENTATION_GUIDE.md) - Legacy guide (partially outdated)
- [TOS_VM_TBPF_REFACTORING_PLAN.md](docs/TOS_VM_TBPF_REFACTORING_PLAN.md) - Refactoring strategy (reference)
- [README_VM_INTEGRATION.md](docs/README_VM_INTEGRATION.md) - Quick overview (reference)

### External References
- [tos-tbpf](../tos-tbpf) - The TBPF eBPF execution engine
- [eBPF Instruction Set](https://www.kernel.org/doc/html/latest/bpf/instruction-set.html) - BPF specification

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
