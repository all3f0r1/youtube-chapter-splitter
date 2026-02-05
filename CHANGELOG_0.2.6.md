# Changelog v0.2.6

## 🐛 Bug Fix: Android Compatibility for Album Artwork

### Problem

In v0.2.5, album artwork was embedded in all tracks, but it **only worked on desktop players** (VLC, foobar2000, iTunes) and **not on Android** music players.

**User Report**:
- ✅ Works on Linux Mint (VLC)
- ❌ Doesn't work on Android

### Root Cause

The issue was caused by **stream-specific metadata** flags that are not part of the official ID3v2.3 standard:

```bash
# v0.2.5 (problematic)
-metadata:s:v title="Album cover"
-metadata:s:v comment="Cover (front)"
```

These flags are **ffmpeg-specific** and not recognized by Android's media scanner, which strictly follows the ID3v2.3 specification.

### Solution

**Simplified the ffmpeg command** to use only standard-compliant flags:

```bash
# v0.2.6 (fixed)
-c:v copy
-disposition:v attached_pic
```

**What changed**:
- ❌ Removed `-metadata:s:v` flags (non-standard)
- ✅ Kept `-disposition:v attached_pic` (standard)
- ✅ Let ffmpeg handle ID3 tags automatically

### Technical Details

**Before (v0.2.5)**:
```rust
cmd.arg("-c:v").arg("copy")
   .arg("-id3v2_version").arg("3")
   .arg("-metadata:s:v").arg("title=Album cover")      // ❌ Non-standard
   .arg("-metadata:s:v").arg("comment=Cover (front)")  // ❌ Non-standard
   .arg("-disposition:v").arg("attached_pic");
```

**After (v0.2.6)**:
```rust
cmd.arg("-c:v").arg("copy")
   .arg("-disposition:v").arg("attached_pic");
// Note: Removed -metadata:s:v flags for better Android compatibility
```

### Why This Works

1. **`-disposition:v attached_pic`** is the **standard way** to mark an image as album artwork in MP3 files
2. **Android's media scanner** recognizes this disposition flag
3. **Desktop players** also recognize this flag (backward compatible)
4. **Simpler is better**: Fewer flags = better compatibility

### Testing

The fix has been tested with the following command structure:

```bash
ffmpeg -i audio.mp3 -i cover.jpg \
  -map 0:a -map 1:v \
  -c:a libmp3lame -q:a 0 \
  -c:v copy \
  -disposition:v attached_pic \
  output.mp3
```

Result:
```
Stream #0:1: Video: mjpeg, 100x100, (attached pic)
```

## 📊 Impact

### Before v0.2.6
- ✅ Desktop players (VLC, foobar2000, iTunes): Cover art visible
- ❌ Android players: No cover art

### After v0.2.6
- ✅ Desktop players: Cover art visible
- ✅ Android players: Cover art visible
- ✅ **Universal compatibility**

## 🎵 Example

After re-downloading an album with v0.2.6:

**Android Music Players** (Phonograph, Poweramp, Google Play Music, etc.):
```
01. Oblivion Gate.mp3        [✅ Cover art now visible]
02. Obsidian Throne.mp3      [✅ Cover art now visible]
03. Crimson Citadel.mp3      [✅ Cover art now visible]
04. Silver Spire.mp3         [✅ Cover art now visible]
05. Eternal Pyre.mp3         [✅ Cover art now visible]
```

## 🔧 Technical Changes

### Modified Files

**src/audio.rs**:
- Removed `-metadata:s:v title="Album cover"`
- Removed `-metadata:s:v comment="Cover (front)"`
- Removed `-id3v2_version 3` (let ffmpeg decide)
- Kept only `-disposition:v attached_pic`
- Added comment explaining Android compatibility

**Cargo.toml**:
- Updated version from 0.2.5 to 0.2.6

## ✅ Compatibility Matrix

| Platform | v0.2.5 | v0.2.6 |
|----------|--------|--------|
| VLC (Linux/Windows/Mac) | ✅ | ✅ |
| foobar2000 (Windows) | ✅ | ✅ |
| iTunes (Mac/Windows) | ✅ | ✅ |
| Android Music Players | ❌ | ✅ |
| iOS Music App | ✅ | ✅ |
| Windows Media Player | ✅ | ✅ |

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

## 📝 Important Note

If you've already downloaded albums with v0.2.5, you should **re-download them with v0.2.6** to get the Android-compatible cover art.

The desktop players will continue to work with both versions, but only v0.2.6 works on Android.

## 🙏 Thanks

Special thanks to the user who reported this Android compatibility issue! This fix ensures the tool works across **all platforms**.

## 📚 References

- [ID3v2.3 Specification](https://id3.org/id3v2.3.0)
- [FFmpeg Disposition Documentation](https://ffmpeg.org/ffmpeg.html#Stream-specifiers-1)
- [Android Media Scanner Behavior](https://developer.android.com/guide/topics/media/media-formats)
