# Hello World Contract

A minimal TOS smart contract that demonstrates basic syscall usage.

## What It Does

This contract:
- Logs "Hello, TOS!" message
- Reads the current block height
- Reads the transaction sender address
- Reads the contract's own address
- Returns success (0)

## Building

### Current Status

⚠️ **Toolchain Development in Progress**

The TOS contract compilation toolchain is currently under development. To compile contracts to eBPF bytecode, we need:

1. **eBPF Rust target support** - Rust compiler with BPF backend
2. **Custom linker** - `bpf-linker` or equivalent for linking eBPF objects
3. **LLVM tools** - For optimization and code generation

### Temporary Solution: Use Existing Tools

For now, you can use one of these approaches:

#### Option 1: Solana Toolchain (Temporary)

While we develop our own toolchain, you can use Solana's toolchain which includes all necessary eBPF tools:

```bash
# Download platform tools (Linux x86_64)
mkdir -p ~/.cache/tos/v1.50
cd ~/.cache/tos/v1.50
wget https://github.com/anza-xyz/platform-tools/releases/download/v1.50/platform-tools-linux-x86_64.tar.bz2
tar -xjf platform-tools-linux-x86_64.tar.bz2

# Compile contract
cd /path/to/tos-vm/examples/hello-world
TOOLCHAIN=~/.cache/tos/v1.50/platform-tools

$TOOLCHAIN/rust/bin/rustc --target sbf-tos-tos \
  --crate-type lib \
  -C panic=abort \
  -C opt-level=2 \
  -C target_cpu=v4 \
  -o hello_world.o \
  src/lib.rs

$TOOLCHAIN/llvm/bin/ld.lld \
  -z notext -shared --Bdynamic \
  -entry entry_0 -Bsymbolic \
  -o hello_world.so hello_world.o

rm hello_world.o
```

#### Option 2: Use Pre-compiled Test Binaries

For testing the VM, use pre-compiled ELF binaries from `tos-tbpf`:

```bash
# These files are already compiled eBPF programs
ls ../../tos-tbpf/tests/elfs/*.so
```

### Future: TOS Toolchain

**Planned components**:
- `tos-rustc` - Rust compiler with TOS-specific eBPF target
- `tos-linker` - Custom linker for TOS contracts
- `tos-build` - Build tool (`cargo tos build`)
- `tos-cli` - Contract deployment and interaction tool

This will allow compiling contracts with:
```bash
cargo tos build
# Output: target/tos/release/hello_world.so
```

## Contract Structure

```rust
#[no_mangle]
pub extern "C" fn entry_0() -> u64 {
    log("Hello, TOS!");
    // ... more code ...
    SUCCESS  // Return 0
}
```

### Key Points

- `#![no_std]` - No standard library (contracts run in constrained environment)
- `#![no_main]` - No main function (entry points are exported C functions)
- `#[no_mangle]` - Preserve function names for VM to find them
- `extern "C"` - Use C ABI for compatibility with eBPF
- `entry_0` - Entry point ID 0 (can have multiple: entry_0, entry_1, etc.)

## Syscalls Used

| Syscall | Purpose |
|---------|---------|
| `log()` | Output debug messages |
| `get_block_height()` | Read current block height |
| `get_tx_sender()` | Get transaction sender address |
| `get_contract_hash()` | Get contract's own address |

## Testing

TODO: Add instructions for testing with TOS VM runtime

## Next Steps

See other examples:
- `counter-rust`: State management with storage
- `token`: Transfer and balance management
