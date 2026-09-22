# Garmin Tracker

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/Emiliopg91/garmin-tracker-rs)

**Sync your Garmin devices and track your strength training — all in one desktop app.**

Garmin Tracker is a cross-platform desktop application built with [Tauri](https://tauri.app/), combining a Rust backend with a React + TypeScript frontend. It connects to Garmin watches over USB (MTP), imports your activity `.FIT` files, and stores your sessions, exercises, and body measurements in a local SQLite database — no cloud account required.

## Screenshots

| ![Screenshot 1](resources/screenshots/screenshot1.png) | ![Screenshot 2](resources/screenshots/screenshot2.png) |
| :----------------------------------------------------: | :----------------------------------------------------: |
| ![Screenshot 3](resources/screenshots/screenshot3.png) | ![Screenshot 4](resources/screenshots/screenshot4.png) |
| ![Screenshot 5](resources/screenshots/screenshot5.png) |

## Features

- **Device sync over USB (MTP)** — Auto-detects Garmin devices and downloads new activities.
- **Launch on device connect** — Starts the app automatically when a device is plugged in.
- **`.FIT` parsing** — Sessions, series, heart rate, GPS, speed, and laps.
- **Activities and training tracking** — Review/edit sessions, notes and sets, with 1RM estimation.
- **GPS route tracking with laps** — Interactive map, speed-colored track, satellite/street toggle, reverse-geocoded naming.
- **Heart-rate zones** — Color-coded HR chart and time-in-zone breakdown.
- **Personal record notifications** — Desktop alert on a new strength PR.
- **Body measurements** — Log, review, delete, and compare over time.
- **Database export to JSON** — Full database export for backup or analysis.
- **Cloud backup to OneDrive** — One-click backup upload via `rclone`.
- **Update check** — Notifies you when a new release is available.
- **Configurable settings** — Language, units, launch on boot, auto-sync — applied live.
- **Local database** — SQLite with versioned, auto-applied migrations.
- **Desktop notifications** — Native, localized alerts for background events.
- **Single instance** — Prevents multiple app copies from corrupting the database.
- **Rotating file logs** — Leveled logging with automatic rotation.

## Installation

### Arch Linux (via `AUR helper`/`PKGBUILD`)

Install the AUR `garmin-tracker-rs` package to get latest stable version of the application and every external dependency.

### From source

Requirements: [Rust](https://www.rust-lang.org/tools/install), [pnpm](https://pnpm.io/), and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
git clone https://github.com/Emiliopg91/garmin-tracker-rs.git
cd garmin-tracker-rs
pnpm install
make build       # or: pnpm tauri build
```

## Development

```bash
pnpm install
make run          # runs the app in dev mode
```

Development builds use [`mold`](https://github.com/rui314/mold) as the linker and [`sccache`](https://github.com/mozilla/sccache) for compilation caching to speed up iteration; install both for the best experience (or adjust the `Makefile` if you don't have them).

## License

Distributed under the GPL-2.0 license, as declared in the project's `PKGBUILD`.
