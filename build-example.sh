#!/usr/bin/env bash
# Build TOS Example Contracts
#
# This script builds the hello-world example contract using the TOS toolchain

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}TOS Contract Build Script${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Determine architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]] || [[ "$ARCH" == "aarch64" ]]; then
    HOST_TRIPLE=aarch64-apple-darwin
else
    HOST_TRIPLE=x86_64-apple-darwin
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOS_NETWORK_DIR="$(dirname "$SCRIPT_DIR")"

# Toolchain paths
RUSTC="${TOS_NETWORK_DIR}/rust/build/${HOST_TRIPLE}/stage1/bin/rustc"
LLVM_BIN="${TOS_NETWORK_DIR}/rust/build/${HOST_TRIPLE}/llvm/build/bin"

# Verify toolchain
if [[ ! -f "$RUSTC" ]]; then
    echo -e "${RED}Error: TOS Rust compiler not found at:${NC}"
    echo "  $RUSTC"
    echo ""
    echo -e "${YELLOW}Please build the toolchain first:${NC}"
    echo "  cd ${TOS_NETWORK_DIR}/platform-tools"
    echo "  ./build.sh"
    exit 1
fi

echo -e "${GREEN}✓${NC} Found TOS Rust compiler:"
"$RUSTC" --version
echo ""

# Set up environment
export RUSTC="$RUSTC"
export PATH="${LLVM_BIN}:${PATH}"

# Parse arguments
TARGET="${1:-tbpf-tos-tos}"
EXAMPLE="${2:-hello-world}"

echo -e "${GREEN}Building:${NC} $EXAMPLE"
echo -e "${GREEN}Target:${NC} $TARGET"
echo ""

# Build the example
cd "${SCRIPT_DIR}/examples/${EXAMPLE}"

echo -e "${YELLOW}Running: cargo build --release --target ${TARGET} -Zbuild-std=core -Zbuild-std-features=panic_immediate_abort${NC}"
echo ""

if cargo build --release --target "$TARGET" -Zbuild-std=core -Zbuild-std-features=panic_immediate_abort; then
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}✓ Build Successful!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""

    # Find the output file
    OUTPUT_FILE=$(find target/${TARGET}/release -name "*.so" -type f | head -1)

    if [[ -n "$OUTPUT_FILE" ]]; then
        echo -e "${GREEN}Output:${NC} $OUTPUT_FILE"
        echo -e "${GREEN}Size:${NC} $(ls -lh "$OUTPUT_FILE" | awk '{print $5}')"
        echo ""

        # Show file info
        echo -e "${YELLOW}File information:${NC}"
        file "$OUTPUT_FILE"
        echo ""

        # Show sections if llvm-objdump is available
        if command -v llvm-objdump &> /dev/null; then
            echo -e "${YELLOW}ELF sections:${NC}"
            llvm-objdump -h "$OUTPUT_FILE"
        fi
    fi
else
    echo ""
    echo -e "${RED}========================================${NC}"
    echo -e "${RED}✗ Build Failed${NC}"
    echo -e "${RED}========================================${NC}"
    exit 1
fi
