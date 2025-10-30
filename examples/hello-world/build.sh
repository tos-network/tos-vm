#!/bin/bash
set -e

# TOS Contract Build Script
# Compiles Rust contracts to eBPF bytecode

# Configuration
TOOLCHAIN=${TOOLCHAIN:-"$HOME/.cache/tos/v1.50/platform-tools"}
TARGET="sbf-tos-tos"
CRATE_TYPE="lib"
OPT_LEVEL="2"
TARGET_CPU="v4"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if toolchain exists
if [ ! -d "$TOOLCHAIN" ]; then
    echo -e "${RED}Error: Solana platform tools not found at $TOOLCHAIN${NC}"
    echo ""
    echo "Please install the toolchain:"
    echo "  mkdir -p ~/.cache/tos/v1.50"
    echo "  cd ~/.cache/tos/v1.50"
    echo "  wget https://github.com/anza-xyz/platform-tools/releases/download/v1.50/platform-tools-linux-x86_64.tar.bz2"
    echo "  tar -xjf platform-tools-linux-x86_64.tar.bz2"
    echo ""
    echo "Or set TOOLCHAIN environment variable to your platform tools location"
    exit 1
fi

RUSTC="$TOOLCHAIN/rust/bin/rustc"
LINKER="$TOOLCHAIN/llvm/bin/ld.lld"

# Check if tools exist
if [ ! -f "$RUSTC" ]; then
    echo -e "${RED}Error: rustc not found at $RUSTC${NC}"
    exit 1
fi

if [ ! -f "$LINKER" ]; then
    echo -e "${RED}Error: ld.lld not found at $LINKER${NC}"
    exit 1
fi

# Build
echo -e "${GREEN}Building hello-world contract...${NC}"
echo ""

# Compile
echo -e "${YELLOW}[1/3]${NC} Compiling Rust to eBPF object file..."
$RUSTC --target $TARGET \
  --crate-type $CRATE_TYPE \
  -C panic=abort \
  -C opt-level=$OPT_LEVEL \
  -C target_cpu=$TARGET_CPU \
  -C target_feature=+static-syscalls,+abi-v2 \
  --edition 2021 \
  -o hello_world.o \
  src/lib.rs

if [ $? -ne 0 ]; then
    echo -e "${RED}Compilation failed${NC}"
    exit 1
fi

# Link
echo -e "${YELLOW}[2/3]${NC} Linking eBPF object to shared library..."
$LINKER \
  -z notext \
  -shared \
  --Bdynamic \
  -entry entry_0 \
  -Bsymbolic \
  -o hello_world.so \
  hello_world.o

if [ $? -ne 0 ]; then
    echo -e "${RED}Linking failed${NC}"
    rm -f hello_world.o
    exit 1
fi

# Clean up
echo -e "${YELLOW}[3/3]${NC} Cleaning up intermediate files..."
rm -f hello_world.o

# Success
echo ""
echo -e "${GREEN}✓ Build successful!${NC}"
echo ""
echo "Output: hello_world.so"
ls -lh hello_world.so

# Show file info
echo ""
echo "File info:"
file hello_world.so || true

# Show size
SIZE=$(stat -f%z hello_world.so 2>/dev/null || stat -c%s hello_world.so 2>/dev/null)
echo "Size: $SIZE bytes"
