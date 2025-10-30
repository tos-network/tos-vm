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

### Prerequisites

To compile TOS smart contracts to eBPF bytecode, you need Solana's platform tools which include a custom Rust compiler and LLVM toolchain.

#### Option 1: Install Solana Platform Tools (Recommended)

```bash
# Download Solana platform tools (contains eBPF toolchain)
# See: https://github.com/anza-xyz/platform-tools/releases
mkdir -p ~/.cache/tos/v1.50
cd ~/.cache/tos/v1.50
wget https://github.com/anza-xyz/platform-tools/releases/download/v1.50/platform-tools-linux-x86_64.tar.bz2
tar -xjf platform-tools-linux-x86_64.tar.bz2
```

#### Option 2: Use Pre-compiled Binaries

For testing purposes, you can use the pre-compiled ELF binaries in `../tos-tbpf/tests/elfs/`.

### Compile the Contract

```bash
cd examples/hello-world

# Set toolchain path
TOOLCHAIN=~/.cache/tos/v1.50/platform-tools

# Compile to .o file
$TOOLCHAIN/rust/bin/rustc --target sbf-tos-tos \
  --crate-type lib \
  -C panic=abort \
  -C opt-level=2 \
  -C target_cpu=v4 \
  -o hello_world.o \
  src/lib.rs

# Link to .so file
$TOOLCHAIN/llvm/bin/ld.lld \
  -z notext \
  -shared \
  --Bdynamic \
  -entry entry_0 \
  -Bsymbolic \
  -o hello_world.so \
  hello_world.o

# Clean up
rm hello_world.o
```

The output `hello_world.so` is the compiled contract ready to deploy.

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
