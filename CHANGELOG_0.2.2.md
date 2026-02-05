# Changelog v0.2.2

## 🐛 Bug Fixes

### 1. Fixed folder name with slashes in video title
**Problem**: When a video title contained a slash (e.g., "70s Progressive/Psychedelic Rock"), it created nested subdirectories instead of a single folder.

**Solution**: 
- Remove everything after "(FULL ALBUM)" or "[FULL ALBUM]" (case insensitive)
- Replace slashes `/` with dashes `-` in folder names
- Example: `PURPLE DREAMS - WANDERING SHADOWS (FULL ALBUM) | 70s Progressive/Psychedelic Rock` → `Purple Dreams - Wandering Shadows`

### 2. Fixed duplicate track numbers in filenames
**Problem**: Track names like "1 - Echoes of the Forgotten" resulted in filenames like "01. 1 - Echoes of the Forgotten.mp3" with double numbering.

**Solution**:
- Automatically remove track numbers from the beginning of chapter titles
- Supports various formats: "1 - ", "01. ", "Track 1: ", etc.
- Example: `1 - Echoes of the Forgotten` → `Echoes of the Forgotten` → `01. Echoes of the Forgotten.mp3`

## 📝 Changes Summary

| Issue | Before | After |
|---|---|---|
| **Folder name** | `Purple Dreams - Wandering Shadows - 70s Progressive/psychedelic Rock/` | `Purple Dreams - Wandering Shadows/` |
| **Track filename** | `01. 1 - Echoes of the Forgotten.mp3` | `01. Echoes of the Forgotten.mp3` |

## 🔧 Technical Details

### Modified Files

**src/utils.rs**:
- Added regex to remove content after "(FULL ALBUM)" or "[FULL ALBUM]"
- Added slash `/` replacement with dash `-`
- Updated tests to cover new cases

**src/chapters.rs**:
- Modified `sanitize_title()` to strip track numbers at the beginning
- Supports multiple track number formats

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

## ✅ Testing

All tests pass:
- `test_clean_folder_name` - Validates folder name cleaning
- `test_format_duration` - Validates duration formatting
- `test_format_duration_short` - Validates short duration formatting

New test case added for the PURPLE DREAMS example.
