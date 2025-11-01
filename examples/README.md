# TOS VM Examples

This directory contains example smart contracts and tools for the TOS Virtual Machine.

## Prerequisites

Before running the examples, you need:

1. **TOS Rust Toolchain**: Custom Rust compiler with TBF (TOS Berkeley Packet Filter) target support
2. **Cargo**: Rust package manager (comes with Rust)

### Building the TOS Rust Toolchain

```bash
cd ~/tos-network/rust
./build.sh
```

This will take approximately 5 minutes to complete.

## Examples

### Hello World

A minimal smart contract that demonstrates:
- Logging messages using the `tos_log` syscall
- Returning a success status
- Basic contract structure

#### Compile the Contract

```bash
cd ~/tos-network/tos-vm/examples/hello-world

# Set the custom Rust compiler
export RUSTC=~/tos-network/rust/build/aarch64-apple-darwin/stage1/bin/rustc

# Build for TBF target
cargo build --target tbpf-tos-tos --release
```

**Output**: `target/tbpf-tos-tos/release/hello_world.so` (approximately 1.8 KB)

#### Run the Contract

First, build the test runner:

```bash
cd ~/tos-network/tos-vm/examples
cargo build --bin run_hello_world
```

Then execute the contract:

```bash
cd ~/tos-network/tos-vm
./examples/target/debug/run_hello_world
```

**Expected Output**:

```
=== TOS VM - Hello World Test Runner ===

1. Creating TBPF loader and registering syscalls...
   ✓ Loader created with syscalls registered

2. Loading hello-world.so...
   ✓ Executable loaded (1808 bytes)

3. Creating execution context...
   ✓ Context created with 200,000 compute units

4. Executing contract...
   --- Contract Output ---

   [Contract 04040404...]: Hello, TOS!

   --- End of Output ---

5. Execution Results:
   Instructions executed: 116
   Return value: Ok(0)
   Compute units used: 116

✅ Contract executed successfully! Return value: SUCCESS (0)
```

## Contract Structure

### Required Components

All TOS smart contracts must include:

```rust
#![no_std]
#![no_main]

use tos_vm_sdk::*;

/// Contract entrypoint - must be named "entrypoint"
#[no_mangle]
pub extern "C" fn entrypoint() -> u64 {
    // Your contract logic here
    SUCCESS  // Return 0 for success
}

/// Panic handler (required for no_std)
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

### Available Syscalls

The TOS VM SDK provides the following syscalls:

- **`log(message: &str)`** - Output debug messages
- **`get_block_hash() -> [u8; 32]`** - Get current block hash
- **`get_block_height() -> u64`** - Get current block height
- **`get_tx_hash() -> [u8; 32]`** - Get transaction hash
- **`get_tx_sender() -> [u8; 32]`** - Get transaction sender address
- **`get_contract_hash() -> [u8; 32]`** - Get contract's own address
- **Storage operations**: `storage_read`, `storage_write`, `storage_delete`
- **Account operations**: `get_balance`, `transfer`

### Return Values

- **`SUCCESS` (0)** - Contract executed successfully
- **`ERROR` (1)** - Generic error
- **Custom values** - You can return any `u64` value

## Creating Your Own Contract

1. **Create a new contract directory**:
   ```bash
   cd ~/tos-network/tos-vm/examples
   mkdir my-contract
   cd my-contract
   ```

2. **Create `Cargo.toml`**:
   ```toml
   [package]
   name = "my-contract"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   tos-vm-sdk = { path = "../../sdk" }

   [lib]
   crate-type = ["cdylib"]

   [profile.release]
   opt-level = 3
   lto = true
   ```

3. **Create `src/lib.rs`**:
   ```rust
   #![no_std]
   #![no_main]

   use tos_vm_sdk::*;

   #[no_mangle]
   pub extern "C" fn entrypoint() -> u64 {
       log("My first contract!");
       SUCCESS
   }

   #[panic_handler]
   fn panic(_info: &core::panic::PanicInfo) -> ! {
       loop {}
   }
   ```

4. **Compile**:
   ```bash
   export RUSTC=~/tos-network/rust/build/aarch64-apple-darwin/stage1/bin/rustc
   cargo build --target tbpf-tos-tos --release
   ```

## Troubleshooting

### Linking Errors

If you encounter linking errors like "incompatible", ensure you're using the latest TOS Rust toolchain:

```bash
cd ~/tos-network/rust
git pull origin main
./build.sh
```

### RUSTC Not Found

Make sure the RUSTC environment variable is set correctly:

```bash
export RUSTC=~/tos-network/rust/build/aarch64-apple-darwin/stage1/bin/rustc
which $RUSTC
```

### Contract Won't Load

Verify the entrypoint function is named exactly `entrypoint`:

```rust
#[no_mangle]
pub extern "C" fn entrypoint() -> u64 {
    // ...
}
```

### Debug Mode

To see detailed execution logs, set the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug ./examples/target/debug/run_hello_world
```

## Technical Details

### ELF Format

Compiled contracts use the ELF format with:
- **Machine Type**: EM_TBF (263 = 0x107)
- **Architecture**: 64-bit, little-endian
- **Format**: Shared object (.so)

### Compute Budget

Each contract execution has a compute budget (default: 200,000 units). Operations consume units:
- Basic instructions: 1 CU
- Syscalls: Variable (e.g., `log`: 100 + message length)
- Memory operations: Variable based on size

### Memory Layout

Contracts execute in a sandboxed memory environment:
- **Stack**: Located at `0x200000000`, configurable size (default: 4KB)
- **Read-only region**: Contains contract code and constants
- **Heap**: Not currently implemented (use stack only)

## Next Steps

- Explore more complex contracts in the examples directory
- Read the [TOS VM Documentation](../docs/)
- Learn about [syscall implementation](../syscalls/src/)
- Understand the [program runtime](../program-runtime/src/)

## Resources

- **TOS VM Repository**: https://github.com/tos-network/tos-vm
- **Rust Toolchain**: https://github.com/tos-network/rust
- **TBPF Engine**: https://github.com/tos-network/tos-tbpf
