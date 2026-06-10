#!/usr/bin/env python3
"""
Migrate aurora-player JSON data to SQLite (aurora.db).

Reads:
  ~/.config/aurora-player/liked.json
  ~/.config/aurora-player/history.json
  ~/.config/aurora-player/playlists/*.json

Writes:
  ~/.config/aurora-player/aurora.db  (created if absent)

Old files are renamed to *.bak after a successful migration so the
script won't re-import them if run again.  The song index (index.json)
is intentionally skipped — the daemon will rescan on first launch and
rebuild it with mtime tracking.

Usage:
  python3 migrate.py [--dry-run]
"""

import argparse
import json
import os
import shutil
import sqlite3
import sys
from pathlib import Path

CONFIG_DIR = Path.home() / ".config" / "aurora-player"
DB_PATH    = CONFIG_DIR / "aurora.db"

SCHEMA = """
CREATE TABLE IF NOT EXISTS songs (
    id       TEXT PRIMARY KEY,
    path     TEXT NOT NULL UNIQUE,
    title    TEXT NOT NULL,
    artists  TEXT NOT NULL,
    dur_ms   INTEGER NOT NULL,
    mtime    INTEGER NOT NULL,
    art_path TEXT
);
CREATE TABLE IF NOT EXISTS playlists (
    id    TEXT PRIMARY KEY,
    title TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS playlist_songs (
    playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    song_id     TEXT NOT NULL,
    position    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (playlist_id, song_id)
);
CREATE TABLE IF NOT EXISTS liked_songs (
    song_id TEXT PRIMARY KEY
);
CREATE TABLE IF NOT EXISTS history (
    song_id   TEXT PRIMARY KEY,
    played_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);
"""


def open_db(path: Path) -> sqlite3.Connection:
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(path))
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA foreign_keys=ON")
    conn.executescript(SCHEMA)
    conn.commit()
    return conn


def migrate_liked(conn: sqlite3.Connection, dry_run: bool) -> int:
    src = CONFIG_DIR / "liked.json"
    if not src.exists():
        print("  liked.json not found, skipping")
        return 0

    ids: list[str] = json.loads(src.read_text())
    print(f"  {len(ids)} liked song(s) found")

    if not dry_run:
        conn.executemany(
            "INSERT OR IGNORE INTO liked_songs (song_id) VALUES (?)",
            [(id_,) for id_ in ids],
        )
        conn.commit()
        shutil.move(str(src), str(src.with_suffix(".json.bak")))
        print(f"  → inserted, renamed to liked.json.bak")

    return len(ids)


def migrate_history(conn: sqlite3.Connection, dry_run: bool) -> int:
    src = CONFIG_DIR / "history.json"
    if not src.exists():
        print("  history.json not found, skipping")
        return 0

    ids: list[str] = json.loads(src.read_text())
    print(f"  {len(ids)} history entry/entries found")

    if not dry_run:
        # Preserve ordering: most recent entry gets the highest played_at value.
        # We use (2^62 - index) so the first element (most recent) sorts last DESC.
        base = (1 << 62)
        rows = [(id_, base - i) for i, id_ in enumerate(ids)]
        conn.executemany(
            "INSERT OR IGNORE INTO history (song_id, played_at) VALUES (?, ?)",
            rows,
        )
        conn.commit()
        shutil.move(str(src), str(src.with_suffix(".json.bak")))
        print(f"  → inserted, renamed to history.json.bak")

    return len(ids)


def migrate_playlists(conn: sqlite3.Connection, dry_run: bool) -> int:
    pl_dir = CONFIG_DIR / "playlists"
    if not pl_dir.exists():
        print("  playlists/ directory not found, skipping")
        return 0

    files = list(pl_dir.glob("*.json"))
    print(f"  {len(files)} playlist file(s) found")

    if not files:
        return 0

    if not dry_run:
        for f in files:
            try:
                pl = json.loads(f.read_text())
                pl_id    = pl["id"]
                pl_title = pl["title"]
                songs    = pl.get("songs", [])

                conn.execute(
                    "INSERT OR IGNORE INTO playlists (id, title) VALUES (?, ?)",
                    (pl_id, pl_title),
                )
                for pos, song in enumerate(songs):
                    conn.execute(
                        "INSERT OR IGNORE INTO playlist_songs "
                        "(playlist_id, song_id, position) VALUES (?, ?, ?)",
                        (pl_id, song["id"], pos),
                    )
                conn.commit()
                print(f"    {pl_title!r}: {len(songs)} song(s)")
            except Exception as exc:
                print(f"    WARNING: skipping {f.name}: {exc}", file=sys.stderr)

        bak = pl_dir.with_name("playlists.bak")
        shutil.move(str(pl_dir), str(bak))
        print(f"  → renamed playlists/ to playlists.bak/")

    return len(files)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--dry-run", action="store_true",
                        help="Parse and report without writing anything")
    args = parser.parse_args()

    if args.dry_run:
        print("DRY RUN — no files will be written or renamed\n")

    if not args.dry_run and DB_PATH.exists():
        print(f"Note: {DB_PATH} already exists — new data will be merged in.\n")

    conn = None if args.dry_run else open_db(DB_PATH)

    print("Migrating liked songs…")
    migrate_liked(conn, args.dry_run)

    print("Migrating play history…")
    migrate_history(conn, args.dry_run)

    print("Migrating playlists…")
    migrate_playlists(conn, args.dry_run)

    if conn:
        conn.close()

    print("\nDone." if not args.dry_run else "\nDry run complete — nothing written.")


if __name__ == "__main__":
    main()
