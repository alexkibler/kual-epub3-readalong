#!/bin/bash

# Build script for macOS/Linux simulator
# For development and testing without Kindle hardware

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUST_PROJECT="$PROJECT_DIR/immersion_reader"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Immersion Reader - Simulator Build ===${NC}"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}ERROR: Rust/Cargo not found!${NC}"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

# Check for GStreamer
echo -e "${YELLOW}Checking for GStreamer...${NC}"
if ! pkg-config --exists gstreamer-1.0; then
    echo -e "${YELLOW}WARNING: GStreamer not found via pkg-config${NC}"
    echo ""
    echo "Please install GStreamer:"
    echo "  macOS:   brew install gstreamer gst-plugins-base gst-plugins-good"
    echo "  Ubuntu:  sudo apt install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev"
    echo "  Fedora:  sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo -e "${GREEN}✓ GStreamer found${NC}"
fi

cd "$RUST_PROJECT"

# Build in simulator mode
echo ""
echo -e "${YELLOW}Building simulator...${NC}"
cargo build --no-default-features --features simulator

echo ""
echo -e "${GREEN}✓ Build complete!${NC}"
echo ""
echo "Binary location:"
echo "  $RUST_PROJECT/target/debug/immersion_reader"
echo ""
echo "To run:"
echo "  cd $RUST_PROJECT"
echo "  cargo run --no-default-features --features simulator -- path/to/book.epub"
echo ""
echo "Controls (in simulator):"
echo "  SPACE   = Tap (play/pause)"
echo "  →       = Next chapter"
echo "  ←       = Previous chapter"
echo "  ↑       = Skip forward 10s"
echo "  ↓       = Skip backward 10s"
echo "  Q/ESC   = Quit"
echo ""
echo "Note: You need an EPUB 3 file with Media Overlays (SMIL) to test."
echo "      Audio will play through your default audio output."
echo ""
