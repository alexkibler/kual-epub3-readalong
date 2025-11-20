#!/bin/sh

# Display help information

cat << 'EOF'
=== Immersion Reader Help ===

This extension reads EPUB 3 books with Media Overlays
(synchronized audio and text highlighting).

SETUP:
1. Connect Bluetooth headphones/speaker to your Kindle
2. Place your EPUB file in /mnt/us/documents/
3. Rename it to 'readalong.epub'
4. Launch the reader from KUAL

CONTROLS:
- Tap: Play/Pause
- Swipe Right: Next chapter
- Swipe Left: Previous chapter
- Swipe Up: Skip forward 10s
- Swipe Down: Skip backward 10s

REQUIREMENTS:
- Kindle Paperwhite 5 (11th Gen)
- EPUB 3 with Media Overlays (SMIL)
- Bluetooth audio device connected
- FBInk extension installed

TROUBLESHOOTING:
- Check log: /mnt/us/immersion_reader.log
- Ensure Bluetooth is paired and connected
- Verify EPUB has Media Overlays support

VERSION: 0.1.0
EOF

sleep 10
