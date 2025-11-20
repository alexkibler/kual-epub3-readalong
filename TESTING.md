# Testing Guide for Immersion Reader

This document provides comprehensive testing procedures for the Immersion Reader KUAL extension.

## Testing Environment

### Development Testing (Limited)

Most functionality requires actual Kindle hardware, but some components can be tested on a development machine:

```bash
cd immersion_reader

# Run unit tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_smil_time
```

**Note**: Tests marked with `#[ignore]` require Kindle hardware and will be skipped.

## On-Device Testing

### Prerequisites

1. Kindle Paperwhite 5 with SSH access
   - Enable USB networking or WiFi SSH
   - Install: https://www.mobileread.com/forums/showthread.php?t=186645

2. Test EPUB file with Media Overlays
   - Sample provided in `test/` directory
   - Or use any EPUB 3 with SMIL

3. Bluetooth audio device (headphones/speaker)
   - Paired and connected before testing

### Test Suite

## 1. Binary Verification

```bash
# SSH into Kindle
ssh root@192.168.15.244  # Or your Kindle's IP

# Check binary
ls -lh /mnt/us/extensions/ImmersionReader/bin/immersion_reader

# Verify it's aarch64
file /mnt/us/extensions/ImmersionReader/bin/immersion_reader
# Expected: ELF 64-bit LSB executable, ARM aarch64

# Check dependencies
ldd /mnt/us/extensions/ImmersionReader/bin/immersion_reader
# Should show: "not a dynamic executable" (static linking)
```

## 2. Component Testing

### Test 1: Display Subsystem

```bash
# Stop Kindle framework
/etc/init.d/framework stop

# Test FBInk availability
fbink -h

# If FBInk test works, display subsystem should work
```

**Expected Results**:
- FBInk help text appears
- Screen clears and shows text

**Troubleshooting**:
- If FBInk not found: Install from https://github.com/NiLuJe/FBInk/releases
- Check library path: `export LD_LIBRARY_PATH=/mnt/us/extensions/FBInk/lib`

### Test 2: Audio Subsystem (Bluetooth)

```bash
# Check Bluetooth connection
hcitool con

# Expected output should show connected device:
# Connections:
#   < ACL XX:XX:XX:XX:XX:XX handle X state 1 lm MASTER

# List ALSA devices
aplay -L | grep bluealsa

# Expected: bluealsa or bluealsa:HCI=hci0

# Test audio playback (if you have a test .wav file)
aplay -D bluealsa /path/to/test.wav
```

**Expected Results**:
- Bluetooth device listed
- bluealsa device available
- Test audio plays through Bluetooth

**Troubleshooting**:
- No connection: Check Settings → Bluetooth
- No bluealsa: `modprobe bluealsa` or check bluez-alsa installation

### Test 3: Input Subsystem

```bash
# List input devices
ls -l /dev/input/event*

# Find touchscreen (usually event0 or event1)
evtest

# Select the touchscreen device
# Tap the screen - should see ABS_X, ABS_Y events
```

**Expected Results**:
- Touch events generate ABS_X, ABS_Y coordinates
- Syn reports mark end of each touch

**Troubleshooting**:
- No events: Try different event device numbers
- Wrong device: Look for device with "touch" or "capacitive" in name

## 3. Integration Testing

### Test 1: Launch Without EPUB

```bash
# Remove or rename EPUB
mv /mnt/us/documents/readalong.epub /mnt/us/documents/readalong.epub.bak

# Try to launch
/mnt/us/extensions/ImmersionReader/launch.sh

# Check log
tail -f /mnt/us/immersion_reader.log
```

**Expected Results**:
- Error message: "EPUB file not found"
- Framework restarts properly
- Log shows clear error

### Test 2: Launch With Valid EPUB

```bash
# Ensure EPUB is in place
ls -l /mnt/us/documents/readalong.epub

# Ensure Bluetooth connected
hcitool con

# Launch
/mnt/us/extensions/ImmersionReader/launch.sh

# Monitor log in another terminal
tail -f /mnt/us/immersion_reader.log
```

**Expected Results**:
- Framework stops
- Application launches
- Title screen appears
- Tap to continue works
- Audio plays synchronized with highlights

**Performance Metrics**:
- **Highlight latency**: < 100ms from audio to screen update
- **Touch response**: < 50ms from tap to action
- **Memory usage**: < 50MB (`top` or `free -m`)

### Test 3: Chapter Navigation

```bash
# Launch app with multi-chapter EPUB
# Test each gesture:
```

1. **Play/Pause**: Tap center of screen
   - Audio should pause
   - Tap again to resume
   - Highlight should stop at pause position

2. **Next Chapter**: Swipe right
   - Display refreshes (GC16)
   - New chapter loads
   - Audio restarts

3. **Previous Chapter**: Swipe left
   - Returns to previous chapter
   - Can't go before chapter 0

4. **Skip Forward**: Swipe up
   - Audio jumps ahead 10 seconds
   - Highlight updates to new position

5. **Skip Backward**: Swipe down
   - Audio jumps back 10 seconds
   - Highlight updates to new position

### Test 4: Bluetooth Reconnection

```bash
# Start playback
# Turn off Bluetooth device (headphones)
# Wait 10 seconds
# Turn Bluetooth device back on
```

**Expected Results**:
- App detects disconnect
- Retries connection (logs should show retry attempts)
- Resumes playback when reconnected
- Max 5 retry attempts with exponential backoff

### Test 5: End of Book

```bash
# Let the last chapter play to completion
```

**Expected Results**:
- Audio finishes
- "Goodbye!" message appears
- Framework restarts
- Returns to Kindle home screen

## 4. Stress Testing

### Test 1: Long Session

```bash
# Play through multiple chapters without interruption
# Monitor for:
# - Memory leaks (check with `top`)
# - E Ink ghosting (should clear with chapter turns)
# - Audio drift (sync should stay < 50ms)
```

**Duration**: 30 minutes minimum

**Metrics to Track**:
```bash
# Memory usage over time
while true; do
  ps aux | grep immersion_reader | grep -v grep
  sleep 60
done
```

### Test 2: Rapid Input

```bash
# Quickly tap screen multiple times
# Rapidly swipe left/right
# Test gesture recognition accuracy
```

**Expected Results**:
- No crashes
- All gestures recognized correctly
- No input queue overflow

### Test 3: Bluetooth Interruption

```bash
# Start playback
# Disconnect Bluetooth randomly
# Reconnect
# Repeat 10 times
```

**Expected Results**:
- Graceful handling of disconnects
- Always attempts reconnection
- No permanent audio loss

## 5. Error Handling Testing

### Test 1: Corrupted EPUB

```bash
# Create invalid EPUB
echo "corrupted data" > /mnt/us/documents/readalong.epub

# Try to launch
/mnt/us/extensions/ImmersionReader/launch.sh
```

**Expected Results**:
- Clear error message
- No crash
- Framework restarts properly

### Test 2: Missing SMIL

```bash
# Use EPUB without Media Overlays
# (Regular EPUB 3 without SMIL files)
```

**Expected Results**:
- Warning logged: "No SMIL file available"
- App should still display content (if possible)
- Or gracefully exit with error

### Test 3: Missing Audio Files

```bash
# EPUB with SMIL pointing to non-existent audio files
```

**Expected Results**:
- Error message logged
- Clear indication of missing audio
- No crash

## 6. Performance Benchmarks

### Display Performance

```bash
# Measure highlight update time
# Add timing logs to display.rs:
# - Start: audio position query
# - End: fbink_refresh returns
# Target: < 100ms
```

### Audio Synchronization

```bash
# Measure audio-text sync accuracy
# Add timing logs to main.rs:
# - Audio position timestamp
# - Highlight display timestamp
# - Calculate drift
# Target: < 50ms
```

### Memory Footprint

```bash
# Check memory usage
ps aux | grep immersion_reader

# Check for leaks over time
# Memory should stabilize, not grow indefinitely
```

## 7. Regression Testing Checklist

After any code changes, verify:

- [ ] Build succeeds with no warnings
- [ ] Binary size < 5MB (optimized)
- [ ] Launches without EPUB (shows error)
- [ ] Launches with valid EPUB
- [ ] Title screen displays
- [ ] Audio plays
- [ ] Text highlights sync with audio
- [ ] All gestures work (tap, swipes)
- [ ] Chapter navigation works
- [ ] Bluetooth reconnection works
- [ ] App exits cleanly
- [ ] Framework restarts
- [ ] No memory leaks
- [ ] Log file created

## 8. Test EPUB Structure

### Minimal Test EPUB

Create a minimal test EPUB:

```
test.epub
├── mimetype                    # "application/epub+zip"
├── META-INF/
│   └── container.xml          # Points to content.opf
└── OEBPS/
    ├── content.opf            # Package document with media-overlay
    ├── chapter1.xhtml         # Content with IDs
    ├── chapter1.smil          # Sync points
    └── audio/
        └── chapter1.mp3       # Short audio (5-10 seconds)
```

**chapter1.xhtml** (simplified):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter 1</title></head>
<body>
  <p id="text1">This is sentence one.</p>
  <p id="text2">This is sentence two.</p>
  <p id="text3">This is sentence three.</p>
</body>
</html>
```

**chapter1.smil**:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<smil xmlns="http://www.w3.org/ns/SMIL" version="3.0">
  <body>
    <par id="par1">
      <text src="chapter1.xhtml#text1"/>
      <audio src="audio/chapter1.mp3" clipBegin="0s" clipEnd="2s"/>
    </par>
    <par id="par2">
      <text src="chapter1.xhtml#text2"/>
      <audio src="audio/chapter1.mp3" clipBegin="2s" clipEnd="4s"/>
    </par>
    <par id="par3">
      <text src="chapter1.xhtml#text3"/>
      <audio src="audio/chapter1.mp3" clipBegin="4s" clipEnd="6s"/>
    </par>
  </body>
</smil>
```

## 9. Automated Testing (Future)

### CI/CD Integration

```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build with Docker
        run: |
          docker build -t builder .
          docker run --rm -v $(pwd):/workspace builder ./build.sh
      - name: Run unit tests
        run: cd immersion_reader && cargo test
      - name: Upload artifact
        uses: actions/upload-artifact@v2
        with:
          name: immersion-reader
          path: ImmersionReader-v*.tar.gz
```

## 10. User Acceptance Testing

### Beta Testing Checklist

Distribute to 5-10 beta testers:

- [ ] Can install from .tar.gz
- [ ] KUAL menu appears
- [ ] Launch script works
- [ ] Can read complete book
- [ ] Touch controls intuitive
- [ ] Audio quality acceptable
- [ ] Battery life acceptable (> 4 hours)
- [ ] No crashes during normal use
- [ ] Clear error messages

### Feedback Collection

- Crash reports
- Performance issues
- UI/UX suggestions
- Feature requests

## 11. Troubleshooting Guide

### Common Issues

**Issue**: "Binary not found"
- Check: `ls -l /mnt/us/extensions/ImmersionReader/bin/`
- Fix: Re-extract package or rebuild

**Issue**: "No Bluetooth device"
- Check: `hcitool con`
- Fix: Pair and connect in Kindle settings

**Issue**: "FBInk failed to initialize"
- Check: FBInk installed and in PATH
- Fix: Install FBInk or update LD_LIBRARY_PATH

**Issue**: Screen not updating
- Check: `/dev/fb0` permissions
- Fix: Run as root or fix permissions

**Issue**: No audio output
- Check: `aplay -L | grep bluealsa`
- Fix: Restart bluez-alsa or re-pair device

## 12. Log Analysis

### Reading Logs

```bash
# View real-time logs
tail -f /mnt/us/immersion_reader.log

# Search for errors
grep ERROR /mnt/us/immersion_reader.log

# Check timestamps
grep "Launching" /mnt/us/immersion_reader.log
```

### Log Levels

- **INFO**: Normal operation
- **DEBUG**: Detailed state changes
- **TRACE**: Very verbose (position updates)
- **WARN**: Potential issues
- **ERROR**: Failures

### Expected Log Sequence

```
[INFO] === Immersion Reader Starting ===
[INFO] Initializing FBInk display subsystem
[INFO] Display initialized: 1448x1072 @ 300 dpi
[INFO] Initializing GStreamer audio subsystem
[INFO] Using Bluetooth audio device: bluealsa
[INFO] Initializing touchscreen input handler
[INFO] Opening EPUB: /mnt/us/documents/readalong.epub
[INFO] Successfully opened EPUB: Book Title (12 chapters)
[INFO] Loading chapter 0: Chapter 1
[INFO] Playing chapter with synchronization
[DEBUG] Starting playback
[TRACE] Highlighted text element: text1
...
```

---

**Document Version**: 1.0
**Last Updated**: 2025
**Target Device**: Kindle Paperwhite 5 (11th Gen)
