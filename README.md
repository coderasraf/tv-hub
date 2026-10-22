# TV Hub

Live TV in one window: Bangladeshi, Bengali and free sports channels from M3U playlists, plus the official YouTube live streams of Bangladeshi broadcasters. Every stream is checked on a schedule, so only channels that are on air show up.

- **Channels**: from the iptv-org Bangladesh and Bengali playlists, filtered to leave out pay-TV channels that public lists carry without permission.
- **Sports**: free sports channels (FIFA+, Red Bull TV, DD Sports, beIN SPORTS XTRA, CBS Sports Golazo, Real Madrid TV and others), plus the T Sports, BCB, ICC and FIFA YouTube channels when they're live.
- **YouTube live**: Somoy, Jamuna, Channel 24, Ekattor, DBC, ATN News, Independent, News24, NTV, RTV, Channel i, Desh TV and more. The app looks up each channel's current live video, with no API key needed.
- **Pay TV**: links to official subscription services (Toffee, RabbitHole, Tapmad, Bioscope+), which open in the browser.
- **Your own sources**: add any M3U playlist (for example your ISP's), single stream links or YouTube channels in Settings.

Streams play in the window through a local proxy, which works around missing CORS headers and adds the headers some streams need. Each channel's backup sources are tried in turn. Anything the window can't decode opens in VLC.

## Player size

- **Default view**: the video fills the width beside the channel list and stays pinned while you browse. Drag the player's bottom edge to resize it; double-click the edge to fit it to the window again. The size is remembered.
- **Theater mode**: the video spans the full width, with the controls and a channel strip underneath. Scroll down to browse, and the video keeps playing in a corner mini player.
- **Full screen**: fills the whole screen.
- **Mini player**: opening another page or searching while a channel plays shrinks it into the bottom-right corner, YouTube style, so the page gets the whole window. The mini player has previous/next channel, expand and close. Clicking a channel card, the mini player's title or the expand button brings back the full player. Playback never restarts when the player changes size.

## Keys

| Key | Action |
| --- | --- |
| ↑ / ↓ or PgUp / PgDn | Previous / next channel |
| ← / → | Volume |
| F or double-click | Full screen |
| T | Theater mode (full-width video; scrolling down turns it into a mini player) |
| M | Mute |
| Space | Pause |
| / | Search |
| Esc | Leave full screen |

## Build and install

```
cargo build --release
```

`target\release\tv-hub.exe` is the app; `target\release\tvhub.exe` is the CLI (`tvhub serve`, `tvhub check --verbose`). The app is installed as `%LOCALAPPDATA%\Programs\TVHub\TVHub.exe` and the CLI next to it as `tvhub-cli.exe`. The CLI can't be called `tvhub.exe` there, because Windows file names ignore case and it would overwrite `TVHub.exe`.

Settings and state are in `%APPDATA%\TVHub`: `config.toml`, `state.json` (favorites, recents, check results), downloaded playlists and the WebView profile.

The icon comes from `assets/make_icon.py` (needs Pillow).
