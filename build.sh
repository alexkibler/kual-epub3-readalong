#!/bin/bash

# Cross-compilation build script for Immersion Reader
# Target: Kindle Paperwhite 5 (aarch64-unknown-linux-musl)

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUST_PROJECT="$PROJECT_DIR/immersion_reader"
OUTPUT_DIR="$PROJECT_DIR/ImmersionReader/bin"
LIB_DIR="$PROJECT_DIR/ImmersionReader/lib"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Immersion Reader Build Script ===${NC}"

# Check if we're in a Docker container or have cross-compilation tools
check_toolchain() {
    echo -e "${YELLOW}Checking for cross-compilation toolchain...${NC}"

    if ! command -v aarch64-linux-musl-gcc &> /dev/null; then
        echo -e "${RED}ERROR: aarch64-linux-musl-gcc not found!${NC}"
        echo ""
        echo "You need to install the musl cross-compilation toolchain."
        echo ""
        echo "Option 1: Using Docker (recommended)"
        echo "  docker run --rm -v \$(pwd):/workspace messense/rust-musl-cross:aarch64-musl bash -c 'cd /workspace && ./build.sh'"
        echo ""
        echo "Option 2: Manual installation"
        echo "  - Download from: https://musl.cc/"
        echo "  - Or use: brew install filosottile/musl-cross/musl-cross"
        echo ""
        exit 1
    fi

    echo -e "${GREEN}✓ Toolchain found${NC}"
}

# Install Rust target if not present
install_rust_target() {
    echo -e "${YELLOW}Checking Rust target...${NC}"

    if ! rustup target list | grep -q "aarch64-unknown-linux-musl (installed)"; then
        echo "Installing aarch64-unknown-linux-musl target..."
        rustup target add aarch64-unknown-linux-musl
    fi

    echo -e "${GREEN}✓ Rust target ready${NC}"
}

# Build the project
build_project() {
    echo -e "${YELLOW}Building Immersion Reader...${NC}"

    cd "$RUST_PROJECT"

    # Build in release mode with musl target
    # Explicitly specify target to override any config
    cargo build --release --target aarch64-unknown-linux-musl --features kindle

    echo -e "${GREEN}✓ Build complete${NC}"
}

# Copy binary to output directory
package_binary() {
    echo -e "${YELLOW}Packaging binary...${NC}"

    mkdir -p "$OUTPUT_DIR"

    BINARY_SRC="$RUST_PROJECT/target/aarch64-unknown-linux-musl/release/immersion_reader"
    BINARY_DST="$OUTPUT_DIR/immersion_reader"

    if [ ! -f "$BINARY_SRC" ]; then
        echo -e "${RED}ERROR: Binary not found at $BINARY_SRC${NC}"
        exit 1
    fi

    cp "$BINARY_SRC" "$BINARY_DST"
    strip "$BINARY_DST" 2>/dev/null || aarch64-linux-musl-strip "$BINARY_DST" || true

    BINARY_SIZE=$(du -h "$BINARY_DST" | cut -f1)
    echo -e "${GREEN}✓ Binary packaged: $BINARY_SIZE${NC}"
}

# Create the final extension package
create_package() {
    echo -e "${YELLOW}Creating KUAL extension package...${NC}"

    cd "$PROJECT_DIR"

    # Create archive
    PACKAGE_NAME="ImmersionReader-v0.1.0.tar.gz"
    tar -czf "$PACKAGE_NAME" ImmersionReader/

    PACKAGE_SIZE=$(du -h "$PACKAGE_NAME" | cut -f1)
    echo -e "${GREEN}✓ Package created: $PACKAGE_NAME ($PACKAGE_SIZE)${NC}"
}

# Print installation instructions
print_instructions() {
    echo ""
    echo -e "${GREEN}=== Build Complete! ===${NC}"
    echo ""
    echo "Installation Instructions:"
    echo "1. Copy ImmersionReader-v0.1.0.tar.gz to your computer"
    echo "2. Connect Kindle via USB"
    echo "3. Extract to /mnt/us/extensions/ on the Kindle"
    echo "   tar -xzf ImmersionReader-v0.1.0.tar.gz -C /mnt/us/extensions/"
    echo "4. Safely eject Kindle"
    echo "5. Place EPUB file at /mnt/us/documents/readalong.epub"
    echo "6. Launch from KUAL menu"
    echo ""
    echo "Requirements:"
    echo "- KUAL (Kindle Unified Application Launcher)"
    echo "- FBInk extension"
    echo "- Bluetooth headphones connected"
    echo ""
}

# Main build process
main() {
    check_toolchain
    install_rust_target
    build_project
    package_binary
    create_package
    print_instructions
}

main "$@"
