# Changelog v0.2.3

## ✨ New Features

### 1. Force Artist and Album Names
You can now override the automatic artist/album detection with custom values using CLI options.

**Usage**:
```bash
# Force both artist and album
ytcs "URL" --artist "Pink Floyd" --album "The Dark Side of the Moon"

# Force only artist (album auto-detected)
ytcs "URL" -a "Led Zeppelin"

# Force only album (artist auto-detected)
ytcs "URL" -A "Houses of the Holy"
```

**Benefits**:
- Correct metadata when video title format is non-standard
- Consistent naming across your music library
- Better organization in music players

### 2. Embedded Album Artwork in MP3 Tags
The album cover is now automatically embedded in the ID3 tags of each MP3 track!

**What this means**:
- Music players (iTunes, VLC, foobar2000, etc.) display the album artwork
- No need to manually add cover art
- Artwork travels with the MP3 file
- Professional-looking music library

**Technical details**:
- Uses ID3v2.3 tags for maximum compatibility
- Cover image is embedded as "Cover (front)"
- Image is copied without re-encoding (preserves quality)

## 🔧 Technical Changes

### Modified Files

**src/main.rs**:
- Added `--artist` and `--album` CLI options
- Implemented `parse_artist_album()` function call
- Pass artist, album, and cover path to `split_audio_by_chapters()`

**src/utils.rs**:
- Added `parse_artist_album()` function to extract artist/album from video title
- Supports formats: "ARTIST - ALBUM" and "ARTIST | ALBUM"
- Falls back to "Unknown Artist" if parsing fails

**src/audio.rs**:
- Modified `split_audio_by_chapters()` signature to accept artist, album, and cover path
- Enhanced ffmpeg command to embed cover art using `-map` and `-metadata:s:v`
- Added artist metadata to ID3 tags

**Cargo.toml**:
- Updated version to 0.2.3

## 📝 Examples

### Before (v0.2.2)
```bash
ytcs "https://www.youtube.com/watch?v=..."
# Output: ./output/Purple Dreams - Wandering Shadows/
# Tracks: 01. Echoes of the Forgotten.mp3 (no artist tag, no embedded cover)
```

### After (v0.2.3)
```bash
ytcs "https://www.youtube.com/watch?v=..."
# Output: ./output/Purple Dreams - Wandering Shadows/
# Tracks: 01. Echoes of the Forgotten.mp3
#   - Artist: Purple Dreams
#   - Album: Wandering Shadows
#   - Cover: ✅ Embedded

# Or with custom metadata:
ytcs "https://www.youtube.com/watch?v=..." -a "Purple Dreams" -A "Wandering Shadows"
```

## 🎵 ID3 Tags Now Include

Each MP3 track now contains:
- **Title**: Track name (e.g., "Echoes of the Forgotten")
- **Artist**: Auto-detected or forced (e.g., "Purple Dreams")
- **Album**: Auto-detected or forced (e.g., "Wandering Shadows")
- **Track**: Track number and total (e.g., "1/5")
- **Cover Art**: Embedded album artwork (if downloaded)

## 🚀 Upgrade Instructions

```bash
cargo install youtube_chapter_splitter
```

Or build from source:
```bash
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter
cargo build --release
```

## ✅ Compatibility

- ID3v2.3 tags are used for maximum compatibility with all music players
- Cover art embedding works with iTunes, Windows Media Player, VLC, foobar2000, and most mobile music apps
- Tested on Linux, macOS, and Windows
