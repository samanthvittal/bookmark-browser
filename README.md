# Bookmarks Browser

A lightweight, native two-pane bookmark browser for Linux built in Rust.

Organize bookmarks into folders in the left sidebar; click any bookmark to load the page in the right content pane. Built with [tao](https://github.com/nickel-org/tao) for windowing and [wry](https://github.com/nickel-org/wry) for WebView rendering — no Electron, no heavyweight frameworks.

## Features

- **Two-pane layout** — sidebar with bookmark tree, content pane rendering real web pages
- **Folder organization** — create, expand/collapse, and delete folders
- **Bookmark management** — add and delete bookmarks via modals or keyboard shortcuts
- **Persistent storage** — bookmarks saved as human-readable JSON in `~/.config/bookmarks-browser/`
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

## Build & Run

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
| `Ctrl+Shift+N` | Add new folder |
| `F1` / `Ctrl+/` | Show keyboard shortcuts |
| `F5` | Reload content pane |
| `Ctrl+[` | Navigate back |
| `Ctrl+]` | Navigate forward |
| `Ctrl+Q` | Quit |
| `Escape` | Close dialog |

## Data Storage

Bookmarks are stored in `~/.config/bookmarks-browser/bookmarks.json` as pretty-printed JSON. On first launch, sample bookmarks are created automatically.

## Upcoming Features

- Drag-and-drop reordering of bookmarks and folders
- Import/export bookmarks (HTML bookmark format or JSON)
- Search/filter bar in sidebar (`Ctrl+F`)
- Resizable sidebar width via drag handle
- Tabs in content pane for opening multiple bookmarks
- Favicon fetching and display next to bookmark names
- Keyboard navigation within the sidebar tree (arrow keys)
- Custom CSS injection for reader mode

## Acknowledgements

- [Claude Code](https://claude.ai/claude-code) — AI-assisted development throughout the project
- [tao](https://github.com/nickel-org/tao) — cross-platform windowing library from the Tauri team
- [wry](https://github.com/nickel-org/wry) — lightweight WebView rendering library
- [serde](https://serde.rs/) — serialization/deserialization framework for Rust
- [WebKitGTK](https://webkitgtk.org/) — web rendering engine used by wry on Linux
- The Rust community for excellent documentation and tooling

## License

[MIT](LICENSE)
