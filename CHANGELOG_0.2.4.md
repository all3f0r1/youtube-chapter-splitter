# Changelog v0.2.4

## ✨ New Features

### 1. Default Output to Music Directory
The output directory now defaults to your system's Music folder instead of `./output`.

**Behavior by OS**:
- **Linux**: `~/Music` (e.g., `/home/username/Music`)
- **macOS**: `~/Music` (e.g., `/Users/username/Music`)
- **Windows**: `%USERPROFILE%\Music` (e.g., `C:\Users\username\Music`)

**Benefits**:
- Music files are automatically organized in the standard location
- No need to specify output directory for typical usage
- Works consistently regardless of where you run the command
- Music appears automatically in your music player library

**Usage**:
```bash
# Uses ~/Music by default
ytcs "https://www.youtube.com/watch?v=..."

# Custom output directory still works
ytcs "https://www.youtube.com/watch?v=..." -o ~/Downloads
```

### 2. Lighter Binary Size
Replaced `reqwest` with `ureq` for HTTP requests, significantly reducing dependencies.

**Size comparison**:
- **Before (v0.2.3)**: Binary included tokio async runtime + reqwest
- **After (v0.2.4)**: Lighter synchronous HTTP client

**Benefits**:
- Faster compilation time
- Smaller binary size (6.3 MB vs previous versions)
- Fewer dependencies to maintain
- No async runtime overhead

## 🔧 Technical Changes

### Dependencies Removed
- ❌ `tokio` - No longer needed (was only used for async thumbnail download)
- ❌ `reqwest` - Replaced with `ureq`

### Dependencies Added
- ✅ `ureq` (2.10) - Lightweight HTTP client
- ✅ `dirs` (5.0) - Cross-platform directory detection

### Modified Files

**Cargo.toml**:
- Removed `tokio` and `reqwest` dependencies
- Added `ureq` and `dirs` dependencies
- Updated version to 0.2.4

**src/main.rs**:
- Removed `#[tokio::main]` and `async fn main()`
- Added `get_default_music_dir()` function using `dirs::audio_dir()`
- Changed `output` field from `String` to `Option<String>`
- Modified output directory logic to use Music folder by default

**src/downloader.rs**:
- Changed `download_thumbnail()` from `async fn` to regular `fn`
- Replaced `reqwest::get()` with `ureq::get()`
- Replaced `response.bytes().await` with `response.into_reader()` + `read_to_end()`

## 📊 Performance Improvements

### Compilation Time
- **Faster**: No need to compile tokio async runtime
- **Fewer crates**: Reduced dependency tree

### Binary Size
- **Smaller**: 6.3 MB (optimized release build)
- **Portable**: Single executable with minimal dependencies

### Runtime Performance
- **Simpler**: No async overhead for simple HTTP GET
- **Efficient**: Direct synchronous I/O for thumbnail download

## 🎵 Example Usage

### Before (v0.2.3)
```bash
ytcs "URL"
# Output: ./output/Artist - Album/
```

### After (v0.2.4)
```bash
ytcs "URL"
# Output: ~/Music/Artist - Album/
# (e.g., /home/alex/Music/Artist - Album/)
```

### Custom Output
```bash
ytcs "URL" -o ~/Documents/Music
# Output: ~/Documents/Music/Artist - Album/
```

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

- Works on Linux, macOS, and Windows
- Automatically detects the correct Music directory for each OS
- Falls back to `~/Music` if system audio directory is not available
- Custom output directory can still be specified with `-o` flag

## 🎯 Migration Notes

If you were relying on the default `./output` directory, you have two options:

1. **Use the new default** (recommended): Files will go to `~/Music`
2. **Keep old behavior**: Use `-o ./output` flag

The new default provides better integration with music players and system organization.
