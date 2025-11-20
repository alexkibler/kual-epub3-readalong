# Immersion Reader - EPUB 3 Media Overlays for Kindle

A specialized EPUB 3 reader for the Kindle Paperwhite 5 (11th Gen) that supports Media Overlays (SMIL), providing synchronized audio playback with real-time text highlighting on the E Ink display.

## Features

- **EPUB 3 Media Overlays**: Full support for SMIL-based audio synchronization
- **Real-time Text Highlighting**: Low-latency highlighting using FBInk's A2 waveform
- **Bluetooth Audio**: Streams audio via Bluetooth (bluealsa)
- **Touch Controls**:
  - Tap to play/pause
  - Swipe right/left for chapter navigation
  - Swipe up/down for 10-second skip
- **E Ink Optimized**: Direct framebuffer rendering for minimal ghosting

## Hardware Requirements

- **Device**: Kindle Paperwhite 5 (11th Gen, 2021)
  - Codename: Bellatrix/Malbec
  - SoC: MediaTek MT8113 (ARMv8/AArch64)
  - Display: E Ink Carta 1200
- **Audio**: Bluetooth headphones or speaker (paired and connected)
- **Software**:
  - KUAL (Kindle Unified Application Launcher)
  - FBInk extension

## Software Architecture

### Subsystems

1. **Display** (`display.rs`): FBInk FFI bindings for framebuffer rendering
2. **Audio** (`audio.rs`): GStreamer pipeline targeting bluealsa
3. **Input** (`input.rs`): Raw evdev touchscreen event processing
4. **Parser** (`parser.rs`): EPUB and SMIL parsing
5. **Main** (`main.rs`): Synchronization controller

### Dependencies

- **Rust**: Cross-compiled with `aarch64-unknown-linux-musl`
- **GStreamer**: Audio pipeline with ALSA sink
- **FBInk**: E Ink display library (via FFI)
- **evdev**: Touchscreen input processing

## Building

### For Development/Testing (macOS/Linux Simulator)

**New!** You can now test the application on your Mac or Linux machine without Kindle hardware:

```bash
# Install GStreamer first (macOS)
brew install gstreamer gst-plugins-base gst-plugins-good

# Build and run simulator
./build-simulator.sh

# Or manually:
cd immersion_reader
cargo run --no-default-features --features simulator -- /path/to/book.epub
```

See **[SIMULATOR.md](SIMULATOR.md)** for complete simulator documentation.

### For Kindle Deployment

#### Option 1: Using Docker (Recommended)

```bash
# Build the Docker image
docker build -t immersion-reader-builder .

# Run the build
docker run --rm -v $(pwd):/workspace immersion-reader-builder ./build.sh
```

#### Option 2: Manual Cross-Compilation

1. Install the musl cross-compilation toolchain:
   ```bash
   # macOS with Homebrew
   brew install filosottile/musl-cross/musl-cross

   # Or download from https://musl.cc/
   ```

2. Install Rust and add the target:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup target add aarch64-unknown-linux-musl
   ```

3. Run the build script:
   ```bash
   ./build.sh
   ```

This will produce `ImmersionReader-v0.1.0.tar.gz`.

## Installation

1. **Connect Kindle via USB**

2. **Install prerequisites** (if not already installed):
   - KUAL: https://www.mobileread.com/forums/showthread.php?t=203326
   - FBInk: https://github.com/NiLuJe/FBInk

3. **Extract the extension**:
   ```bash
   # On your computer, with Kindle mounted
   tar -xzf ImmersionReader-v0.1.0.tar.gz -C /path/to/kindle/extensions/
   ```

4. **Add EPUB file**:
   - Place your EPUB 3 with Media Overlays at:
     `/mnt/us/documents/readalong.epub`

5. **Connect Bluetooth audio**:
   - Pair and connect Bluetooth headphones via Kindle settings

6. **Launch**:
   - Eject Kindle safely
   - Open KUAL menu
   - Select "Immersion Reader" → "Launch Reader"

## EPUB Format Requirements

The EPUB must be **EPUB 3** with **Media Overlays** support:

### Required Structure

```
book.epub
├── META-INF/
│   └── container.xml
├── OEBPS/
│   ├── content.opf          # Package document
│   ├── chapter1.xhtml       # Content
│   ├── chapter1.smil        # Media overlay
│   ├── audio/
│   │   └── chapter1.mp3     # Audio file
│   └── ...
```

### SMIL Example

```xml
<smil xmlns="http://www.w3.org/ns/SMIL" version="3.0">
  <body>
    <par id="par1">
      <text src="chapter1.xhtml#text1"/>
      <audio src="audio/chapter1.mp3" clipBegin="0.000s" clipEnd="3.500s"/>
    </par>
    <par id="par2">
      <text src="chapter1.xhtml#text2"/>
      <audio src="audio/chapter1.mp3" clipBegin="3.500s" clipEnd="7.200s"/>
    </par>
  </body>
</smil>
```

### Creating EPUB 3 with Media Overlays

Tools for creating compatible EPUBs:

1. **Tobi** (DAISY Pipeline): https://www.daisy.org/project/tobi/
2. **Sigil with Media Overlays plugin**: https://sigil-ebook.com/
3. **Calibre** (with plugins): https://calibre-ebook.com/

## Usage

### Controls

| Gesture | Action |
|---------|--------|
| Single Tap | Play/Pause |
| Swipe Right | Next Chapter |
| Swipe Left | Previous Chapter |
| Swipe Up | Skip Forward 10s |
| Swipe Down | Skip Backward 10s |

### Troubleshooting

**No audio output:**
- Check Bluetooth connection: Settings → Bluetooth
- Verify device is paired and connected
- Check log: `/mnt/us/immersion_reader.log`

**Display issues:**
- Ensure FBInk is installed
- Check FBInk library path in `launch.sh`

**App doesn't start:**
- View log: `cat /mnt/us/immersion_reader.log`
- Ensure EPUB exists at `/mnt/us/documents/readalong.epub`
- Verify binary permissions: `ls -l /mnt/us/extensions/ImmersionReader/bin/`

**Bluetooth device asleep:**
- The app will retry connection automatically (up to 5 attempts)
- Wake the device and wait for reconnection

## Testing Strategy

### Unit Tests

Run on development machine (limited - hardware-dependent):
```bash
cd immersion_reader
cargo test
```

### On-Device Testing

1. **Basic Functionality Test**:
   ```bash
   # Via SSH on Kindle
   /mnt/us/extensions/ImmersionReader/bin/immersion_reader /mnt/us/documents/test.epub
   ```

2. **Display Test**:
   - Verify text rendering
   - Check highlight performance (should be < 100ms)
   - Test full refresh after chapter navigation

3. **Audio Test**:
   - Verify bluealsa device detection
   - Test playback start/pause
   - Verify synchronization accuracy (< 50ms drift)

4. **Input Test**:
   - Test all touch gestures
   - Verify touch event recognition

### Test EPUB

A minimal test EPUB is included in `test/sample.epub` (to be created):
- Single chapter
- 5 synchronized sentences
- Short audio clip (30 seconds)

## Development Notes

### Why musl libc?

The Kindle's glibc is old (circa 2016). Using `musl` provides:
- Static linking of libc
- Smaller binary size
- Broader compatibility

### FBInk Integration

FBInk provides direct framebuffer access:
- **A2 waveform**: 2-bit, ~50ms refresh (for highlights)
- **GC16 waveform**: 16-level grayscale, ~450ms refresh (for page turns)

### Audio Pipeline

GStreamer pipeline:
```
filesrc → decodebin → audioconvert → audioresample → alsasink[bluealsa]
```

### Performance Constraints

- **Highlight latency**: < 100ms from audio position query to display update
- **Memory**: < 50MB total (Kindle has limited RAM)
- **CPU**: Minimize during playback (battery life)

## Known Limitations

1. **MVP Limitations**:
   - No complex XHTML layout engine (fixed text positions)
   - No support for images in EPUB
   - Single Bluetooth device support

2. **Hardware Constraints**:
   - E Ink refresh rate limits animation
   - No local audio output (Bluetooth only)
   - Limited CPU for real-time processing

## Future Enhancements

- [ ] Full XHTML rendering engine
- [ ] Support for EPUB 2 with external SMIL
- [ ] Bookmarks and reading progress
- [ ] Multiple Bluetooth device profiles
- [ ] Configuration UI
- [ ] Support for other Kindle models

## License

MIT License - see LICENSE file

## Credits

- **FBInk**: NiLuJe - https://github.com/NiLuJe/FBInk
- **KUAL**: twobob - https://www.mobileread.com/forums/showthread.php?t=203326
- **GStreamer**: GStreamer team - https://gstreamer.freedesktop.org/

## Contributing

Contributions welcome! Please test on actual Kindle hardware before submitting PRs.

### Development Setup

1. Clone repository
2. Install Rust and cross-compilation tools
3. Run `./build.sh`
4. Test on Kindle device

## Support

For issues and questions:
- GitHub Issues: (your repo URL)
- MobileRead Forums: (thread URL)

---

**Version**: 0.1.0
**Target Device**: Kindle Paperwhite 5 (11th Gen, 2021)
**Last Updated**: 2025
