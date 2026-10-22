# TV Hub

TV Hub: live TV from IPTV playlists and YouTube live channels, checked and playable in one window. A Rust app with a command-line interface and a desktop front end, set up out of the box for Bangladeshi and Bengali-language channels plus free sports.

![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-async-463F3F)
![hls.js](https://img.shields.io/badge/hls.js-1.5-1F6FEB)

> **Status:** early and in progress. `Cargo.toml` declares two binaries, `src/main.rs` (the `tvhub` CLI) and `src/desktop.rs` (the `tv-hub` desktop app), and `src/lib.rs` declares a `web` module, but those entry files are not in the repo yet. Building may fail until they are added. The shared library code described below is in place.

## Overview

TV Hub collects channels from three places: M3U playlists (such as the public iptv-org lists), official YouTube channels that stream live, and channels you add yourself. It merges them into one catalog grouped by category and region, checks which streams are actually on air, and plays them in an embedded web view through a local HLS proxy. Streams the window can't decode are handed to an external player such as VLC.

## Features

- **M3U parsing** - Reads extended M3U playlists as written by iptv-org and most IPTV providers: `tvg-id`, `tvg-logo`, `group-title`, User-Agent and Referer hints, quality suffixes like `(720p)`, and tags like `[Not 24/7]` and `[Geo-blocked]`.
- **Channel catalog**
  - Merges duplicate channels across and within playlists into one entry with several fallback sources.
  - Maps `group-title` values onto app categories: News, Sports, Entertainment, Music, Kids, Religious, Movies, Documentary.
  - Detects region from the `tvg-id` country code (Bangladesh, India, International).
  - Per-playlist curation: `none` (everything), `free` (drops pay-TV brands that appear in public lists as unauthorized rebroadcasts), or `sports` (only free or official sports channels).
  - Caches downloaded playlists on disk so the app starts with channels even before (or without) the network. A failed download keeps the previous copy.
- **Health checks** - Checks every stream concurrently (32 at a time, 12 s timeout). For HLS master playlists it also fetches the first variant, because dead channels often keep serving a stale master. The re-check interval is configurable (default 6 hours) and progress is exposed for the UI.
- **YouTube live** - Finds a channel's current live stream from `youtube.com/@handle/live` without an API key. Reports whether it is live, the video ID, title, avatar, and whether the broadcaster allows embedded playback.
- **Local HLS proxy** - Streams play through a local `/stream` endpoint because most IPTV servers send no CORS headers and some need a specific User-Agent or Referer. Playlists are rewritten so every segment, key and variant also goes through the proxy. Each proxied URL carries a keyed hash, so the proxy only fetches URLs the app itself handed out.
- **External player** - Opens streams in VLC (or any configured command) when the window can't play them, such as HEVC or raw MPEG-TS. It passes User-Agent and Referer to VLC, accepts only http(s) URLs, and runs the player without a shell.
- **Persistent state** - Favorites, recently watched channels (last 30), volume, and the latest check results are kept in one JSON file that is rewritten atomically.
- **Configuration** - A TOML file of playlists, YouTube channels, your own channels, the external player and links to official paid streaming services (Toffee, RabbitHole, Tapmad, Bioscope+), which open in the browser. The file is created with sensible defaults on first run and validated on load.
- **Windows icon** - `build.rs` embeds the app icon into the Windows executables.

## Tech Stack

- **Rust** (2021 edition), **Tokio** async runtime
- **axum** for the local proxy and web endpoints, **reqwest** for HTTP
- **tao** + **wry** for the native window and WebView, **rfd** for native file dialogs
- **hls.js** (vendored in `src/ui/hls.min.js`) for HLS playback in the window
- **serde**, **toml**, **serde_json** for config and state; **clap** for the CLI; **regex**, **url**

## Getting Started

### Prerequisites

- A recent stable Rust toolchain (the code uses `std::sync::LazyLock`, so Rust 1.80 or newer)
- Optional: [VLC](https://www.videolan.org/vlc/) for streams the window can't play
- On Linux, WebKitGTK development packages are needed for the `wry` WebView

### Build and run

```bash
git clone https://github.com/coderasraf/tv-hub.git
cd tv-hub

# Run the library's unit tests
cargo test --lib

# Once src/main.rs and src/desktop.rs are added:
cargo run --release --bin tv-hub   # desktop app
cargo run --release --bin tvhub    # command-line interface
```

### Configuration and data

Settings, playlist caches and the WebView profile live in a `TVHub` folder under `%APPDATA%` on Windows, or `$XDG_CONFIG_HOME` / `~/.config` elsewhere. The default config includes three iptv-org playlists (Bangladesh, Bengali language, free sports) and about 20 official YouTube channels from Bangladeshi broadcasters, plus ICC and FIFA.

A minimal config looks like this:

```toml
check_interval_hours = 6

[player]
command = "vlc"
args = ["{url}"]

[[playlists]]
name = "Bangladesh (iptv-org)"
url = "https://iptv-org.github.io/iptv/countries/bd.m3u"
curation = "free"

[[youtube]]
name = "Somoy TV"
handle = "somoynews360"
category = "News"

[[channels]]
name = "My stream"
url = "https://example.com/live/index.m3u8"
category = "Entertainment"
```

Playlists can be an http(s) URL or a full path to a local `.m3u` file.

## Project Structure

```text
tv-hub/
├── Cargo.toml        # library + tvhub (CLI) and tv-hub (desktop) binaries
├── build.rs          # embeds the Windows icon
├── assets/           # app icon files and make_icon.py (Pillow) generator
└── src/
    ├── lib.rs        # module list and shared User-Agent
    ├── config.rs     # TOML config, defaults, validation, data dir
    ├── m3u.rs        # extended M3U parser
    ├── catalog.rs    # channel list: curation, categories, merging, cache
    ├── health.rs     # concurrent stream and YouTube checks
    ├── youtube.rs    # live-stream lookup by handle
    ├── proxy.rs      # signed local HLS proxy with playlist rewriting
    ├── player.rs     # external player and browser launching
    ├── store.rs      # favorites, recents, volume, check results
    └── ui/           # vendored hls.js
```

## Author

Built by **Ashraf Siddik**, Shopify developer & CRO specialist.

- Website: [ashraf-siddik.com](https://ashraf-siddik.com)
- GitHub: [@coderasraf](https://github.com/coderasraf)
