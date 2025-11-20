#!/bin/sh

# Simple file browser for selecting EPUB files
# This is a placeholder - KUAL doesn't have a built-in file picker

EPUB_DIR="/mnt/us/documents"
LOG_FILE="/mnt/us/immersion_reader.log"

echo "Available EPUB files in $EPUB_DIR:" | tee -a "$LOG_FILE"
find "$EPUB_DIR" -name "*.epub" -type f | tee -a "$LOG_FILE"

echo ""
echo "To select an EPUB file, rename it to 'readalong.epub' in $EPUB_DIR"
echo "Then use 'Launch Reader' from the menu."

# Keep the dialog open for a moment
sleep 5
