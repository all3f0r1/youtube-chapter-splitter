# Changelog v0.2.1

## Fixed

- **Folder names**: Pipe characters (`|`) are now replaced with dashes (`-`) in folder names
  - Example: `ECLIPSERA | Circle of the Endless Earth` → `Eclipsera - Circle Of The Endless Earth`

- **Title display**: The title shown after "Fetching video information..." is now cleaned using the same rules as folder names
  - Removes `[]` and `()` with their content
  - Replaces `_` and `|` with `-`
  - Capitalizes each word properly (including words after dashes)

- **Capitalization**: Improved capitalization to handle hyphenated words correctly
  - Example: `Artist-Name - Album-Title` instead of `Artist-name - Album-title`

## Known Issue

**Background jobs with URLs containing `&`**: When pasting YouTube URLs with `&` characters (like playlist or radio parameters), the shell interprets `&` as a background job operator, causing `[1]`, `[2]` messages.

**Solution**: Always quote URLs when using ytcs:
```bash
ytcs "https://www.youtube.com/watch?v=VIDEO_ID&list=..."
```

See `URL_USAGE.md` for more details.

## Version

- Updated from 0.2.0 to 0.2.1
