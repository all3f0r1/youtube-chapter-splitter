# YouTube Authentication with Cookies

To download member-only videos or private videos from YouTube, you need to provide your YouTube authentication cookies.

## Setup Instructions

### 1. Export Your YouTube Cookies

You can use a browser extension to export your YouTube cookies in Netscape format:

- **Chrome/Edge**: [Get cookies.txt LOCALLY](https://chrome.google.com/webstore/detail/get-cookiestxt-locally/cclelndahbckbenkjhflpdbgdldlbecc)
- **Firefox**: [cookies.txt](https://addons.mozilla.org/en-US/firefox/addon/cookies-txt/)

#### Steps:
1. Install the browser extension
2. Log in to YouTube in your browser
3. Navigate to any YouTube page (e.g., https://www.youtube.com)
4. Click the extension icon and export cookies as `cookies.txt`

### 2. Place the Cookies File

Move the exported `cookies.txt` file to:

```
~/.config/ytcs/cookies.txt
```

On Linux/macOS:
```bash
mkdir -p ~/.config/ytcs
mv ~/Downloads/cookies.txt ~/.config/ytcs/cookies.txt
```

On Windows:
```powershell
mkdir $env:USERPROFILE\.config\ytcs
move $env:USERPROFILE\Downloads\cookies.txt $env:USERPROFILE\.config\ytcs\cookies.txt
```

### 3. Verify Setup

Once the cookies file is in place, `ytcs` will automatically use it for all YouTube downloads. You should now be able to download:

- Member-only videos
- Private videos (that you have access to)
- Age-restricted videos
- Any other content requiring authentication

## Security Notes

- **Keep your cookies file private!** It contains your authentication credentials.
- The cookies file should have restricted permissions (readable only by you).
- Cookies may expire after some time. If downloads start failing, export fresh cookies.

## Troubleshooting

If downloads still fail after setting up cookies:

1. **Check file location**: Ensure the file is at `~/.config/ytcs/cookies.txt`
2. **Check file format**: The file should be in Netscape format (plain text)
3. **Refresh cookies**: Your cookies may have expired. Export fresh ones from your browser.
4. **Check permissions**: Ensure the file is readable by your user account.

## Alternative Method: Using yt-dlp Directly

You can also test your cookies file with yt-dlp directly:

```bash
yt-dlp --cookies ~/.config/ytcs/cookies.txt "YOUR_VIDEO_URL"
```

If this works, then `ytcs` should work as well.
