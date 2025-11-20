#!/bin/sh

# Immersion Reader Launch Script for KUAL
# Kindle Paperwhite 5 (11th Gen) - Bellatrix/Malbec

EXTENSION_DIR="$(dirname "$0")"
BINARY="$EXTENSION_DIR/bin/immersion_reader"
LOG_FILE="/mnt/us/immersion_reader.log"
EPUB_FILE="/mnt/us/documents/readalong.epub"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "=== Immersion Reader Starting ==="

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    log "ERROR: Binary not found at $BINARY"
    exit 1
fi

# Check if EPUB file exists
if [ ! -f "$EPUB_FILE" ]; then
    log "ERROR: EPUB file not found at $EPUB_FILE"
    log "Please place an EPUB file named 'readalong.epub' in /mnt/us/documents/"
    exit 1
fi

# Stop the Kindle framework to free up resources and display
log "Stopping Kindle framework..."
/etc/init.d/framework stop || killall -9 cvm

# Wait for framework to fully stop
sleep 2

# Set up environment variables for GStreamer
export GST_PLUGIN_PATH="$EXTENSION_DIR/lib/gstreamer-1.0"
export LD_LIBRARY_PATH="$EXTENSION_DIR/lib:$LD_LIBRARY_PATH"

# Set up FBInk library path if bundled
export LD_LIBRARY_PATH="/mnt/us/extensions/FBInk/lib:$LD_LIBRARY_PATH"

# Enable core dumps for debugging (optional)
# ulimit -c unlimited

# Check if Bluetooth is connected
if ! hcitool con | grep -q "ACL"; then
    log "WARNING: No Bluetooth device connected!"
    log "Please connect Bluetooth headphones before running."
    # We'll still try to run - the app will retry connection
fi

# Run the application
log "Launching Immersion Reader..."
log "EPUB: $EPUB_FILE"

# Run with proper environment and capture output
"$BINARY" "$EPUB_FILE" 2>&1 | tee -a "$LOG_FILE"

EXIT_CODE=$?
log "Application exited with code $EXIT_CODE"

# Restart the Kindle framework
log "Restarting Kindle framework..."
/etc/init.d/framework start || /etc/init.d/framework restart

# Give framework time to start
sleep 3

log "=== Immersion Reader Finished ==="

exit $EXIT_CODE
