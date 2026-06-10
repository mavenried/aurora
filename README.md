<h1 align="center">
aurora
</h1>
<p align="center">
A music player.
</p>

Indev by [@mavenried](https://github.com/mavenried).

## Overview

Aurora is a local music player built in Rust. It follows a daemon/client architecture: a background daemon manages playback and library state, while a [Slint](https://slint.dev)-based GUI connects to it over a Unix socket.
The daemon keeps playing even when the player is closed.

**Crates:**

- `aurora-daemon` — handles audio playback (via rodio/symphonia), library scanning, queue management, playlists, liked songs, and MPRIS
- `aurora-player` — the GUI frontend built with Slint
- `aurora-protocol` — shared types and message definitions

## Features

- Library scanning with metadata read via lofty
- Queue with play-next, enqueue, reorder, and clear
- Playlists — create, rename, delete, add/remove songs
- Liked songs stored separately from playlists
- Search by title or artist
- Shuffle and repeat modes
- Volume control
- MPRIS2 support
- Album art display
- Per-song selection for bulk operations

## Running

```sh
# Kill any existing daemon, then build and launch both
./run.sh
```

Or manually:

```sh
cargo build --release
cargo run --release -p aurora-daemon
cargo run --release -p aurora-player
```

## Requirements

- Rust (edition 2024)
- A fonts — the UI uses **JetBrainsMono NFP** and **Noto Sans** (bundled in `app/assets/`)
- Linux (uses Unix sockets and MPRIS/D-Bus)
