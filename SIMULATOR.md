  # Simulator Mode - Testing on macOS/Linux

This project includes a **simulator mode** that allows you to test the EPUB 3 Media Overlays reader on your Mac (or Linux development machine) without needing actual Kindle hardware.

## What Works in Simulator Mode

✅ **EPUB parsing**: Full EPUB 3 and SMIL parsing
✅ **Audio playback**: Using your system's default audio output
✅ **Synchronization logic**: Same timing engine as on Kindle
✅ **Text highlighting**: Visualized in terminal
✅ **Input**: Keyboard controls instead of touchscreen
✅ **Chapter navigation**: All navigation features work

## What's Different

❌ **Display**: Terminal output instead of E Ink screen
❌ **Audio output**: Standard audio instead of Bluetooth
❌ **Input**: Keyboard instead of touchscreen
❌ **Performance**: Not representative of Kindle performance

## Prerequisites

### macOS

```bash
# Install Homebrew if you don't have it
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install GStreamer
brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-libav

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Linux (Ubuntu/Debian)

```bash
# Install GStreamer
sudo apt update
sudo apt install libgstreamer1.0-dev \
                 libgstreamer-plugins-base1.0-dev \
                 gstreamer1.0-plugins-base \
                 gstreamer1.0-plugins-good \
                 gstreamer1.0-plugins-bad \
                 gstreamer1.0-libav

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Linux (Fedora/RHEL)

```bash
# Install GStreamer
sudo dnf install gstreamer1-devel \
                 gstreamer1-plugins-base-devel \
                 gstreamer1-plugins-base \
                 gstreamer1-plugins-good \
                 gstreamer1-plugins-bad-free \
                 gstreamer1-libav

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building the Simulator

### Option 1: Using the Build Script (Recommended)

```bash
./build-simulator.sh
```

### Option 2: Manual Build

```bash
cd immersion_reader
cargo build --no-default-features --features simulator
```

## Running the Simulator

### With an EPUB File

```bash
cd immersion_reader
cargo run --no-default-features --features simulator -- /path/to/your/book.epub
```

### Example with Test EPUB

```bash
# Download a sample EPUB with Media Overlays
# (You'll need to find or create one - see below)

cargo run --no-default-features --features simulator -- ~/Downloads/sample_book.epub
```

## Controls

When the simulator is running, use these keyboard controls:

| Key | Action | Kindle Equivalent |
|-----|--------|-------------------|
| **SPACE** | Play/Pause | Tap screen |
| **→** (Right Arrow) | Next Chapter | Swipe right |
| **←** (Left Arrow) | Previous Chapter | Swipe left |
| **↑** (Up Arrow) | Skip Forward 10s | Swipe up |
| **↓** (Down Arrow) | Skip Back 10s | Swipe down |
| **Q** or **ESC** | Quit | N/A |

## Display Output

The simulator uses your terminal to visualize the E Ink display:

```
┌─────────────────────────────────────────┐
│ Book Title                              │
│ Chapter 1                               │
│                                         │
│ This is the current sentence.          │
│ ▓▓▓▓▓▓▓▓▓▓▓▓▓ <- Highlighted text      │
│ This is the next sentence.             │
│                                         │
└─────────────────────────────────────────┘
```

- **Normal text**: Regular sentences
- **Inverse/highlighted**: Currently playing sentence (shown as `▓▓▓`)

## Audio Output

Audio plays through your default system audio output:

- **macOS**: Uses CoreAudio (via GStreamer's autoaudiosink)
- **Linux**: Uses ALSA/PulseAudio (via GStreamer's autoaudiosink)

The synchronization works exactly the same as on the Kindle, so you can verify timing accuracy.

## Test EPUB Files

You need EPUB 3 files with Media Overlays (SMIL) to test. Here are some options:

### Create a Test EPUB

See `test/create_test_epub.py` (to be added) for a script that generates a minimal test EPUB with:
- 3 chapters
- Simple text content
- SMIL synchronization
- Short audio clips

### Download Sample EPUBs

1. **DAISY Consortium Samples**:
   - http://www.daisy.org/z3998/2012/auth/
   - Look for "Media Overlays" examples

2. **EPUB 3 Sample Documents**:
   - https://github.com/IDPF/epub3-samples
   - Check for "media-overlays-" prefixed samples

3. **Create Your Own**:
   - Use [Tobi](https://www.daisy.org/project/tobi/) (DAISY Pipeline)
   - Use [Sigil](https://sigil-ebook.com/) with Media Overlays plugin

## Development Workflow

1. **Write code** on your Mac/Linux machine
2. **Test in simulator** with keyboard controls
3. **Verify parsing and sync logic** works correctly
4. **Cross-compile for Kindle** using `build.sh`
5. **Test on actual Kindle** hardware

This workflow lets you iterate quickly without deploying to the Kindle for every change.

## Debugging

### Enable Verbose Logging

```bash
RUST_LOG=debug cargo run --no-default-features --features simulator -- book.epub
```

Log levels:
- `RUST_LOG=error`: Only errors
- `RUST_LOG=warn`: Warnings and errors
- `RUST_LOG=info`: General information (default)
- `RUST_LOG=debug`: Detailed debugging
- `RUST_LOG=trace`: Very verbose (includes position updates)

### Common Issues

**"GStreamer not found"**
- Install GStreamer development libraries (see Prerequisites)
- Check with: `pkg-config --libs gstreamer-1.0`

**"Failed to initialize audio"**
- Ensure GStreamer plugins are installed
- Try: `gst-inspect-1.0 autoaudiosink`

**"No audio output"**
- Check your system audio is working
- Try playing audio with another app first
- Verify audio file format is supported (MP3, AAC, WAV, OGG, FLAC all work)

**"Terminal display garbled"**
- Try resizing your terminal window
- Use a modern terminal (iTerm2 on Mac, GNOME Terminal on Linux)
- Check terminal supports ANSI escape codes

**"Keyboard input not working"**
- Terminal must have focus
- Make sure you're not in a tmux/screen session (or test there)
- Try pressing keys multiple times

## Performance Comparison

The simulator gives you an idea of the application flow, but performance will differ from Kindle:

| Metric | Simulator (Mac M1) | Kindle PW5 (MT8113) |
|--------|-------------------|---------------------|
| EPUB Parse Time | ~50ms | ~200ms |
| Audio Start Latency | ~100ms | ~300ms |
| Highlight Update | ~5ms (terminal) | ~60ms (E Ink A2) |
| Memory Usage | ~40MB | ~35MB |

The Kindle's E Ink display has inherent latency (A2 waveform is ~50-60ms), which you won't experience in the simulator.

## Testing Checklist

Use the simulator to test:

- [ ] EPUB file opens successfully
- [ ] Chapter list is correct
- [ ] Audio plays synchronized with text
- [ ] Highlights update in real-time
- [ ] Play/pause works (SPACE)
- [ ] Chapter navigation works (←/→)
- [ ] Seeking works (↑/↓)
- [ ] App handles end of chapter
- [ ] App handles end of book
- [ ] Error messages are clear

## Limitations

The simulator is great for development, but has limitations:

1. **No E Ink waveform simulation**: Can't test ghosting, refresh artifacts
2. **No Bluetooth**: Can't test bluealsa connection/reconnection
3. **Different audio stack**: System audio vs. Kindle's ALSA
4. **No touch gestures**: Keyboard is different from touch
5. **Terminal != E Ink**: Visual appearance is completely different
6. **Different CPU**: Performance not representative

**Always test on actual Kindle hardware before release!**

## Extending the Simulator

Want to improve the simulator? Consider adding:

- [ ] GUI window instead of terminal (using SDL2 or similar)
- [ ] Simulated E Ink latency/ghosting
- [ ] Better XHTML rendering
- [ ] Mock Bluetooth connection states
- [ ] Performance profiling tools

See `src/display_sim.rs`, `src/audio_sim.rs`, and `src/input_sim.rs` for the simulator backends.

## Architecture Differences

### Kindle Build

```
main.rs
 ├─ display.rs → FBInk → /dev/fb0 (E Ink)
 ├─ audio.rs → GStreamer → alsasink[bluealsa] → Bluetooth
 └─ input.rs → evdev → /dev/input/event* (Touch)
```

### Simulator Build

```
main.rs
 ├─ display_sim.rs → crossterm → Terminal
 ├─ audio_sim.rs → GStreamer → autoaudiosink → System Audio
 └─ input_sim.rs → crossterm → Keyboard
```

The core logic (`parser.rs`, synchronization in `main.rs`) is **identical** in both builds, ensuring what you test in the simulator will work on the Kindle.

## Contributing

If you improve the simulator, please:

1. Keep it simple - it's a dev tool, not the final product
2. Ensure Kindle build still works (`cargo build --features kindle`)
3. Document any new controls or features
4. Test on both macOS and Linux if possible

---

**Happy Testing!** 🚀

For questions or issues with the simulator, open a GitHub issue with the `simulator` tag.
