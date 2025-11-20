# Quick Start Guide - Immersion Reader

Get up and running in 5 minutes!

## Prerequisites

1. **Kindle Paperwhite 5** (11th Gen, 2021)
2. **KUAL** installed - [Download here](https://www.mobileread.com/forums/showthread.php?t=203326)
3. **FBInk** installed - [Download here](https://github.com/NiLuJe/FBInk/releases)
4. **Bluetooth headphones** paired and connected
5. **EPUB 3 with Media Overlays** (SMIL)

## Step 1: Build the Extension

### Using Docker (Easiest)

```bash
docker build -t immersion-reader-builder .
docker run --rm -v $(pwd):/workspace immersion-reader-builder ./build.sh
```

This creates `ImmersionReader-v0.1.0.tar.gz`.

### Without Docker

Install musl cross-compiler and run:
```bash
./build.sh
```

## Step 2: Install on Kindle

1. **Connect Kindle via USB**

2. **Extract the extension**:
   ```bash
   tar -xzf ImmersionReader-v0.1.0.tar.gz -C /path/to/kindle/extensions/
   ```

   On Kindle, it should be at:
   ```
   /mnt/us/extensions/ImmersionReader/
   ```

3. **Add your EPUB**:
   - Copy your EPUB 3 with Media Overlays to Kindle
   - Rename it to: `/mnt/us/documents/readalong.epub`

4. **Eject Kindle safely**

## Step 3: Connect Bluetooth

1. On Kindle: **Settings → Bluetooth**
2. **Turn on Bluetooth**
3. **Pair your headphones** (if not already paired)
4. **Connect** to the device

Verify connection:
```bash
# Via SSH (optional)
hcitool con
# Should show: ACL XX:XX:XX:XX:XX:XX
```

## Step 4: Launch the Reader

1. Press **Home** button on Kindle
2. Open **KUAL** (via Search or Home screen icon)
3. Navigate to **"Immersion Reader"**
4. Select **"Launch Reader"**

The app will:
- Stop the Kindle framework
- Launch the reader
- Show the book title
- Wait for you to tap to start

## Step 5: Start Reading!

### Controls

- **Tap**: Play/Pause
- **Swipe Right**: Next chapter
- **Swipe Left**: Previous chapter
- **Swipe Up**: Skip forward 10s
- **Swipe Down**: Skip back 10s

The current sentence will be highlighted in sync with the audio!

## Troubleshooting

### "Binary not found"
- Re-extract the .tar.gz file
- Check: `/mnt/us/extensions/ImmersionReader/bin/immersion_reader` exists

### "EPUB file not found"
- Verify: `/mnt/us/documents/readalong.epub` exists
- Check the filename exactly (case-sensitive)

### "No Bluetooth device"
- Go to Settings → Bluetooth
- Ensure device is **connected** (not just paired)
- Turn device off and on

### No audio output
```bash
# Via SSH, check bluealsa
aplay -L | grep bluealsa
```

### Black screen / Display issues
- Ensure FBInk is installed
- Check: `/mnt/us/extensions/FBInk/lib/libfbink.so` exists

### View Logs
```bash
# Via SSH
tail -f /mnt/us/immersion_reader.log

# Or from USB
# Connect Kindle
cat /Volumes/Kindle/immersion_reader.log
```

## Where to Get EPUB 3 with Media Overlays

### Free Sources
- **Project Gutenberg** (some titles): https://www.gutenberg.org/
- **Librivox** (audiobook + text): https://librivox.org/
- **DAISY Consortium samples**: https://daisy.org/

### Commercial Sources
- **Google Play Books** (some titles have read-along)
- **Learning Ally** (educational content)
- **Bookshare** (accessible books)

### Create Your Own
- **Tobi** (DAISY Pipeline): https://www.daisy.org/project/tobi/
- **Sigil** with plugins: https://sigil-ebook.com/

## Example EPUB Structure

Your EPUB must have this structure:

```
book.epub
├── META-INF/container.xml
├── OEBPS/
│   ├── content.opf          # Must have media-overlay attribute
│   ├── chapter1.xhtml       # Content with IDs
│   ├── chapter1.smil        # Synchronization data
│   └── audio/
│       └── chapter1.mp3     # Audio files
```

The `content.opf` manifest items must have:
```xml
<item id="chapter1" href="chapter1.xhtml"
      media-overlay="chapter1_smil" />
<item id="chapter1_smil" href="chapter1.smil"
      media-type="application/smil+xml"/>
```

## Performance Tips

- **Battery**: A full charge should last 4-6 hours of reading
- **Sync accuracy**: Highlights should appear within 50ms of audio
- **Memory**: The app uses ~30-40MB of RAM
- **Storage**: Binary is ~2-3MB

## Exiting the App

The app will automatically exit when:
- The book finishes
- You complete all chapters
- An error occurs

The Kindle framework will restart automatically.

## Next Steps

- Read the full **README.md** for detailed information
- Check **TESTING.md** for troubleshooting
- Customize `launch.sh` if needed
- Create issues on GitHub for bugs

---

**Enjoy your immersive reading experience!** 📖🎧
