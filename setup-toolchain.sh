#!/usr/bin/env bash
# TOS Toolchain Setup Script
#
# This script sets up the environment to use the locally compiled TOS toolchain

set -e

# Determine the architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]] || [[ "$ARCH" == "aarch64" ]]; then
    HOST_TRIPLE=aarch64-apple-darwin
else
    HOST_TRIPLE=x86_64-apple-darwin
fi

# Base paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOS_NETWORK_DIR="$(dirname "$SCRIPT_DIR")"

# Toolchain paths
RUST_DIR="${TOS_NETWORK_DIR}/rust"
LLVM_DIR="${RUST_DIR}/build/${HOST_TRIPLE}/llvm"
RUSTC_BIN="${RUST_DIR}/build/${HOST_TRIPLE}/stage1/bin"
RUSTLIB="${RUST_DIR}/build/${HOST_TRIPLE}/stage1/lib"

# Verify toolchain exists
if [[ ! -f "${RUSTC_BIN}/rustc" ]]; then
    echo "Error: TOS Rust toolchain not found at ${RUSTC_BIN}/rustc"
    echo "Please compile the toolchain first by running:"
    echo "  cd ${TOS_NETWORK_DIR}/platform-tools"
    echo "  ./build.sh"
    exit 1
fi

# Print setup info
echo "==================================="
echo "TOS Toolchain Environment Setup"
echo "==================================="
echo "Architecture: ${HOST_TRIPLE}"
echo "Rust toolchain: ${RUSTC_BIN}"
echo "LLVM tools: ${LLVM_DIR}/build/bin"
echo ""

# Export environment variables
export RUSTC="${RUSTC_BIN}/rustc"
export RUSTUP_TOOLCHAIN="${RUST_DIR}/build/${HOST_TRIPLE}/stage1"
export RUSTFLAGS="-C link-arg=-z -C link-arg=notext"
export LD_LIBRARY_PATH="${RUSTLIB}:${LD_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="${RUSTLIB}:${DYLD_LIBRARY_PATH:-}"

# Add to PATH
export PATH="${RUSTC_BIN}:${LLVM_DIR}/build/bin:${PATH}"

# Verify setup
echo "Testing toolchain..."
"${RUSTC}" --version
echo ""

# Print available TOS targets
echo "Available TOS targets:"
"${RUSTC}" --print target-list | grep -E "(tbpf|bpf)" || true
echo ""

echo "✅ Toolchain setup complete!"
echo ""
echo "To use this toolchain in your current shell, run:"
echo "  source setup-toolchain.sh"
echo ""
echo "To build the hello-world example:"
echo "  cd examples/hello-world"
echo "  cargo build --release --target tbpf-tos-tos"
echo ""
