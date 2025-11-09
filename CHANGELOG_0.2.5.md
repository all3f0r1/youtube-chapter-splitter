# Changelog v0.2.5

## 🐛 Bug Fixes

### 1. Fixed Album Artwork Embedding in All Tracks

**Problem**: The album cover was only embedded in the first track's ID3 tags, not in subsequent tracks (track 2, 3, 4, etc.).

**Root Cause**: The ffmpeg command structure was not properly marking the cover image as an attached picture for all tracks.

**Solution**: 
- Reorganized ffmpeg arguments for better stream mapping
- Added `-disposition:v attached_pic` flag to explicitly mark the image as an attached picture
- Improved stream mapping order to ensure consistent behavior across all tracks

**Result**: Now **all MP3 files** include the embedded album artwork, not just the first one.

**Technical Details**:
```bash
# Before (only worked for first track)
ffmpeg -i audio.mp3 -i cover.jpg -ss 0 -t 300 -map 0:a -map 1:v -c:v copy ...

# After (works for all tracks)
ffmpeg -i audio.mp3 -i cover.jpg -ss 0 -t 300 -map 0:a -map 1:v -c:v copy -disposition:v attached_pic ...
```

### 2. Improved Track Title Cleaning

**Problem**: Track titles starting with just a number and space (e.g., "1 sands of heaven") were not being cleaned properly.

**Previous Behavior**:
- "1 - Echoes" → "Echoes" ✅
- "01. Shadows" → "Shadows" ✅
- "Track 1: Fading" → "Fading" ✅
- "1 sands of heaven" → "1 sands of heaven" ❌ (not cleaned)

**New Behavior**:
- "1 sands of heaven" → "sands of heaven" ✅

**Solution**: Updated the regex pattern to make the separator character optional:
```rust
// Before
r"^\s*(?:Track\s+)?\d+\s*[-.:)]\s*"

// After
r"^\s*(?:Track\s+)?\d+\s*[-.:)]?\s+"
```

The `?` after `[-.:)]` makes the separator optional, so it now matches:
- `1 - Title` (with separator)
- `1 Title` (without separator)
- `01. Title` (with dot)
- `Track 1: Title` (with "Track" prefix)

## 📊 Impact

### Album Artwork Fix
- **Before**: Only track 1 had embedded cover art
- **After**: All tracks (1, 2, 3, ..., N) have embedded cover art
- **Benefit**: Music players display artwork for all tracks, not just the first one

### Title Cleaning Improvement
- **Before**: Some track titles retained leading numbers
- **After**: All common number formats are removed
- **Benefit**: Cleaner, more professional track titles in music libraries

## 🎵 Example

### Before v0.2.5
```
01. Oblivion Gate.mp3        [✅ Has cover art]
02. Obsidian Throne.mp3      [❌ No cover art]
03. Crimson Citadel.mp3      [❌ No cover art]
04. Silver Spire.mp3         [❌ No cover art]
05. Eternal Pyre.mp3         [❌ No cover art]
```

Track title: "1 sands of heaven" → Filename: "01. 1 sands of heaven.mp3" ❌

### After v0.2.5
```
01. Oblivion Gate.mp3        [✅ Has cover art]
02. Obsidian Throne.mp3      [✅ Has cover art]
03. Crimson Citadel.mp3      [✅ Has cover art]
04. Silver Spire.mp3         [✅ Has cover art]
05. Eternal Pyre.mp3         [✅ Has cover art]
```

Track title: "1 sands of heaven" → Filename: "01. sands of heaven.mp3" ✅

## 🔧 Technical Changes

### Modified Files

**src/audio.rs**:
- Reorganized ffmpeg command structure
- Added `-disposition:v attached_pic` flag
- Improved stream mapping order
- Better comments for clarity

**src/chapters.rs**:
- Updated `sanitize_title()` regex pattern
- Made separator character optional in track number detection
- Added comment explaining the new pattern

**Cargo.toml**:
- Updated version from 0.2.4 to 0.2.5

## ✅ Testing

Both fixes have been tested and verified:

1. **Album artwork**: All tracks now display cover art in music players
2. **Title cleaning**: Various number formats are correctly removed

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

## 📝 Notes

These are important bug fixes that improve the user experience:
- Music libraries now look more professional with artwork on all tracks
- Track titles are cleaner and more consistent

If you've already downloaded albums with v0.2.4 or earlier, you may want to re-download them to get the embedded artwork on all tracks.

## 🙏 Thanks

Thanks to the user who reported these issues! Bug reports help make the tool better for everyone.
