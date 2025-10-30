# TOS Contract Compilation Toolchain

**Status**: Planning Phase
**Priority**: High
**Timeline**: 2-4 weeks

---

## Overview

To enable developers to write TOS smart contracts in Rust and compile them to eBPF bytecode, we need a complete compilation toolchain. Currently, this is the main blocker for contract development.

## Current Situation

### What We Have ✅
- ✅ TOS-VM runtime with TBPF engine
- ✅ Complete syscall system (11 syscalls)
- ✅ SDK with Rust syscall bindings
- ✅ Example contract code (hello-world)
- ✅ 41 passing tests

### What We Need ❌
- ❌ Rust compiler with eBPF/BPF target
- ❌ Custom linker for eBPF binaries
- ❌ Build tool integration (`cargo tos build`)
- ❌ Contract deployment tools

## Problem Statement

Standard Rust toolchain issues:
1. **BPF Target**: Rust nightly has `bpfel-unknown-none` target, but requires `bpf-linker`
2. **No `bpf-linker`**: This tool doesn't exist in standard Rust distribution
3. **Solana Dependency**: Currently only Solana provides a complete eBPF toolchain

**We cannot depend on Solana's toolchain** - TOS needs its own independent solution.

## Solution Options

### Option 1: Fork Solana's Toolchain ⭐ (Recommended)

**Approach**: Fork and rebrand Solana's platform-tools for TOS

**Advantages**:
- ✅ Battle-tested (used in production by Solana)
- ✅ Complete solution (Rust + LLVM + linker)
- ✅ Fast to deploy (2-3 weeks)
- ✅ Maintains compatibility with eBPF ecosystem

**Disadvantages**:
- ⚠️ Large codebase to maintain
- ⚠️ Need to track upstream changes

**Components to fork**:
1. `rust-bpf` - Custom Rust compiler fork
2. `bpf-tools` - LLVM-based tools (clang, ld.lld)
3. `cargo-build-bpf` - Cargo integration

**Implementation**:
```bash
# 1. Fork repositories
https://github.com/solana-labs/rust-bpf
https://github.com/solana-labs/platform-tools

# 2. Rebrand
- Rename sbf-solana-solana → sbf-tos-tos
- Update syscall interface
- Rebrand tooling (cargo-build-bpf → cargo-build-tos)

# 3. Release
- Build binaries for Linux/Mac/Windows
- Distribute via GitHub releases
```

### Option 2: Use `bpf-linker` Crate

**Approach**: Use existing `bpf-linker` from cargo-bpf project

**Advantages**:
- ✅ Lightweight solution
- ✅ Works with standard Rust nightly
- ✅ Community maintained

**Disadvantages**:
- ⚠️ Less mature than Solana's toolchain
- ⚠️ May have compatibility issues
- ⚠️ Limited features

**Implementation**:
```bash
# Install bpf-linker
cargo install bpf-linker

# Build contracts
cargo +nightly build --release --target bpfel-unknown-none \
  -Z build-std=core \
  -Z build-std-features=panic_immediate_abort
```

### Option 3: Custom LLVM Backend

**Approach**: Build our own LLVM-based eBPF compiler

**Advantages**:
- ✅ Full control
- ✅ TOS-specific optimizations
- ✅ No external dependencies

**Disadvantages**:
- ❌ Very time-consuming (6+ months)
- ❌ Requires LLVM expertise
- ❌ High maintenance burden
- ❌ Risky approach

## Recommended Approach

**Phase 1: Fork Solana Toolchain (Weeks 1-3)** ⭐

1. **Week 1**: Setup and rebrand
   - Fork rust-bpf and platform-tools repos
   - Rename targets: `sbf-tos-tos`
   - Update build scripts

2. **Week 2**: Integration
   - Build toolchain binaries
   - Test with hello-world contract
   - Create cargo integration

3. **Week 3**: Release
   - Package for Linux/Mac/Windows
   - Create installation scripts
   - Write documentation

**Phase 2: Long-term (Months 2-6)**

1. **Stabilization**
   - Fix bugs found in production
   - Optimize compilation speed
   - Improve error messages

2. **Enhancement**
   - Add TOS-specific optimizations
   - Improve developer experience
   - Better IDE integration

3. **Independence**
   - Gradually reduce Solana dependencies
   - Custom LLVM passes for TOS
   - TOS-specific features

## Toolchain Components

### 1. TOS Rust Compiler (`tos-rustc`)

**Based on**: Solana's rust-bpf fork
**Target**: `sbf-tos-tos`
**Features**:
- no_std support
- panic_abort
- TOS-specific syscalls
- Optimization for eBPF

### 2. TOS Linker (`tos-linker`)

**Based on**: LLVM ld.lld
**Features**:
- Link eBPF object files
- Symbol resolution
- Entry point configuration
- Size optimization

### 3. Build Tool (`cargo-build-tos`)

**Based on**: cargo-build-bpf
**Usage**:
```bash
cargo install cargo-build-tos

# Build contract
cargo tos build

# Output: target/tos/release/contract.so
```

### 4. CLI Tool (`tos-cli`)

**Purpose**: Contract deployment and interaction
**Features**:
```bash
# Deploy contract
tos-cli deploy hello_world.so

# Call contract
tos-cli call <address> <function> [args]

# Query contract state
tos-cli get <address>
```

## Installation (Future)

### From Pre-built Binaries

```bash
# Download and install
curl -sSf https://tos.network/install-toolchain.sh | sh

# Or manual download
wget https://github.com/tos-network/tos-toolchain/releases/latest/download/tos-toolchain-linux-x86_64.tar.gz
tar -xzf tos-toolchain-linux-x86_64.tar.gz
export PATH="$HOME/.tos/bin:$PATH"
```

### From Source

```bash
# Clone toolchain
git clone https://github.com/tos-network/tos-toolchain
cd tos-toolchain

# Build (requires LLVM, Rust source)
./build.sh

# Install
./install.sh
```

## Contract Build Workflow (Future)

```bash
# 1. Create new contract
cargo tos new my-contract
cd my-contract

# 2. Write contract code
# Edit src/lib.rs

# 3. Build
cargo tos build

# 4. Test locally
cargo tos test

# 5. Deploy to testnet
tos-cli deploy --network testnet target/tos/release/my_contract.so

# 6. Deploy to mainnet
tos-cli deploy --network mainnet target/tos/release/my_contract.so
```

## Testing Strategy

### Unit Tests (Within Contract)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Test contract logic
    }
}
```

### Integration Tests (With VM)
```bash
# Create test harness
cargo tos test --integration

# Run against local VM instance
```

### End-to-End Tests
```bash
# Deploy to local devnet
tos-cli devnet start
tos-cli deploy --network devnet contract.so
tos-cli call <address> <function>
```

## Timeline

| Week | Task | Deliverable |
|------|------|-------------|
| 1 | Fork and rebrand | TOS rust compiler fork |
| 2 | Build toolchain | Working binaries for Linux/Mac |
| 3 | Integration & testing | cargo-build-tos working |
| 4 | Documentation & release | Public release v0.1.0 |

## Success Criteria

- ✅ Developers can compile Rust contracts to eBPF
- ✅ `cargo tos build` works out of the box
- ✅ Pre-built binaries available for major platforms
- ✅ Documentation and examples complete
- ✅ CI/CD pipeline for automated builds

## Next Steps

1. **Immediate** (This Week):
   - Evaluate bpf-linker viability
   - Set up Solana toolchain fork repositories
   - Create initial build scripts

2. **Short-term** (Next Week):
   - Build first TOS-branded toolchain
   - Test with hello-world contract
   - Create installer scripts

3. **Medium-term** (Next Month):
   - Public beta release
   - Gather developer feedback
   - Iterate on tooling

## Resources

### Reference Projects
- [Solana Platform Tools](https://github.com/anza-xyz/platform-tools)
- [Solana rust-bpf](https://github.com/solana-labs/rust)
- [cargo-bpf](https://github.com/qmonnet/cargo-bpf)
- [bpf-linker](https://github.com/alessandrod/bpf-linker)

### Documentation
- [eBPF Instruction Set](https://www.kernel.org/doc/html/latest/bpf/instruction-set.html)
- [LLVM BPF Backend](https://llvm.org/docs/CompilerWriterInfo.html#bpf)
- [Rust BPF Target](https://doc.rust-lang.org/nightly/rustc/platform-support/bpf.html)

---

**Maintained by**: TOS Development Team
**Last Updated**: 2025-10-30
**Status**: Living Document
