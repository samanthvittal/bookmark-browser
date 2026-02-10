# Bookmarks Browser

A lightweight, native two-pane bookmark browser for Linux built in Rust.

Organize bookmarks into folders in the left sidebar; click any bookmark to load the page in the right content pane. Built with [tao](https://github.com/nickel-org/tao) for windowing and [wry](https://github.com/nickel-org/wry) for WebView rendering — no Electron, no heavyweight frameworks.

## Features

- **Two-pane layout** — sidebar with bookmark tree, content pane rendering real web pages
- **Folder organization** — create, expand/collapse, and delete folders
- **Bookmark management** — add and delete bookmarks via modals or keyboard shortcuts
- **Collapsible sidebar** — toggle the sidebar to maximize content space (`Ctrl+B`)
- **GitHub sync** — push/pull bookmarks to a GitHub repository for backup and cross-machine sync
- **Auto-sync** — bookmark mutations automatically sync in the background (skip-if-busy)
- **Persistent storage** — bookmarks saved as human-readable JSON in `~/.config/bookmarks-browser/`
- **Settings** — configure GitHub token and repository via in-app settings modal
- **Dark theme** — Catppuccin Mocha color palette
- **Keyboard shortcuts** — full keyboard control (see below)
- **Tiny binary** — under 1 MB release build with LTO and strip

## Requirements

- **Rust** 1.70+ (2021 edition)
- **WebKitGTK** (system dependency for wry)

### Arch Linux

```bash
sudo pacman -S webkit2gtk-4.1
```

### Ubuntu / Debian

```bash
sudo apt install libwebkit2gtk-4.1-dev
```

## Install

```bash
# Clone and install to ~/.local/
git clone https://github.com/user/bookmark-browser.git
cd bookmark-browser
./install.sh
```

This builds a release binary and installs it along with the desktop entry and icon to `~/.local/`. Make sure `~/.local/bin` is in your `PATH`. To uninstall:

```bash
./uninstall.sh
```

## Build & Run (Development)

```bash
# Debug build
cargo run

# Release build (optimized, stripped)
cargo build --release
./target/release/bookmarks-browser
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Add new bookmark |
| `Ctrl+G` | Add new folder |
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+U` | Push bookmarks to GitHub |
| `Ctrl+I` | Pull bookmarks from GitHub |
| `F1` / `Ctrl+/` | Show keyboard shortcuts |
| `F5` | Reload content pane |
| `Ctrl+[` | Navigate back |
| `Ctrl+]` | Navigate forward |
| `Ctrl+Q` | Quit |
| `Escape` | Close dialog |

## GitHub Sync

Bookmarks can be synced to a GitHub repository for backup and restoring on a fresh install.

1. Create a GitHub repository (e.g. `my-bookmarks`)
2. Generate a [personal access token](https://github.com/settings/tokens) with `repo` scope
3. Open **Settings** in the sidebar, enter your token and repository (`owner/repo`)
4. Use **Push** to upload or **Pull** to download bookmarks

Bookmark mutations (add/delete folders and bookmarks) automatically trigger a background sync. If a sync is already in progress, additional mutations are queued silently to avoid API spam.

## Data Storage

- **Bookmarks**: `~/.config/bookmarks-browser/bookmarks.json` — pretty-printed JSON, created with sample bookmarks on first launch
- **Settings**: `~/.config/bookmarks-browser/settings.json` — GitHub token, repository, and UI preferences

## Acknowledgements

- [Claude Code](https://claude.ai/claude-code) — AI-assisted development throughout the project
- [tao](https://github.com/nickel-org/tao) — cross-platform windowing library from the Tauri team
- [wry](https://github.com/nickel-org/wry) — lightweight WebView rendering library
- [serde](https://serde.rs/) — serialization/deserialization framework for Rust
- [WebKitGTK](https://webkitgtk.org/) — web rendering engine used by wry on Linux
- The Rust community for excellent documentation and tooling

## License

[MIT](LICENSE)
