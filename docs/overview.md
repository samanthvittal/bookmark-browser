# Bookmarks Browser — Specification

A lightweight, native two-pane bookmark browser for Linux built in Rust. Left pane shows an organized tree of folders and bookmarks; clicking a bookmark renders the page in the right pane.

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  tao Window                      │
│ ┌────────────┐ ┌──────────────────────────────┐ │
│ │  Sidebar   │ │       Content Pane           │ │
│ │  (wry      │ │       (wry WebView)          │ │
│ │  WebView   │ │                              │ │
│ │  with      │ │  Renders the actual          │ │
│ │  local     │ │  website when a              │ │
│ │  HTML)     │ │  bookmark is clicked         │ │
│ │            │ │                              │ │
│ │ 📁 Docs    │ │                              │ │
│ │   ├─ Rust  │ │                              │ │
│ │   └─ Arch  │ │                              │ │
│ │ 📁 News    │ │                              │ │
│ │   └─ HN    │ │                              │ │
│ │            │ │                              │ │
│ └────────────┘ └──────────────────────────────┘ │
│  ~280px fixed        fills remaining width       │
└─────────────────────────────────────────────────┘
```

### Technology Stack

| Component | Crate | Purpose |
|-----------|-------|---------|
| Window management | `tao` (v0.31+) | Cross-platform windowing from the Tauri team. Handles the native window, keyboard events, and resize. No GTK/Qt dependency in your code — tao uses the platform's native APIs |
| Web rendering | `wry` (v0.47+) | Thin Rust wrapper over the system's WebKitGTK. Two instances: one for sidebar (local HTML), one for content (loads real URLs) |
| Serialization | `serde` + `serde_json` | Bookmark data persistence |
| Config paths | `dirs` (v6+) | XDG-compliant config directory resolution |

### System Dependency

The only system-level dependency is **WebKitGTK**, which wry uses as its rendering backend on Linux. On Arch:

```bash
sudo pacman -S webkit2gtk-4.1
```

This is *not* a GTK app — your binary is a native Rust executable. WebKitGTK is used only as an embedded rendering component by wry, similar to how Electron uses Chromium but without the Electron overhead.

---

## Data Model

### Bookmark Store (`~/.config/bookmarks-browser/bookmarks.json`)

```json
{
  "folders": [
    {
      "name": "Documentation",
      "expanded": true,
      "bookmarks": [
        { "name": "Claude Code Docs", "url": "https://code.claude.com/docs" },
        { "name": "Arch Wiki", "url": "https://wiki.archlinux.org/" }
      ]
    },
    {
      "name": "News",
      "expanded": true,
      "bookmarks": [
        { "name": "Hacker News", "url": "https://news.ycombinator.com/" }
      ]
    }
  ]
}
```

### Rust Structs

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Bookmark {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Folder {
    name: String,
    #[serde(default = "default_true")]
    expanded: bool,
    bookmarks: Vec<Bookmark>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BookmarkStore {
    folders: Vec<Folder>,
}
```

### Persistence

- Config path: `dirs::config_dir() / "bookmarks-browser" / "bookmarks.json"`
- Load on startup; save on every mutation (add/delete/reorder/toggle)
- If file doesn't exist, create with a few sample bookmarks
- Use `serde_json::to_string_pretty` for human-readable JSON

---

## Window Layout

### Startup

- Default window size: 1200×800 logical pixels
- Window title: `"Bookmarks Browser"`
- Sidebar width: 280px (fixed)
- Content pane: fills remaining width

### Two WebView Setup (wry child views)

wry supports multiple `WebView` instances as children of the same window using `WebViewBuilder::new_as_child(&window)`. Each gets a `Rect` defining its position and size:

```rust
// Sidebar — left pane
let sidebar = WebViewBuilder::new_as_child(&window)
    .with_bounds(Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(SIDEBAR_WIDTH, window_height).into(),
    })
    .with_html(&sidebar_html)  // local HTML string
    .with_ipc_handler(...)     // receives messages from sidebar JS
    .build()?;

// Content — right pane
let content = WebViewBuilder::new_as_child(&window)
    .with_bounds(Rect {
        position: LogicalPosition::new(SIDEBAR_WIDTH, 0.0).into(),
        size: LogicalSize::new(window_width - SIDEBAR_WIDTH, window_height).into(),
    })
    .with_html(&welcome_html)  // initial welcome page
    .build()?;
```

### Resize Handling

On `WindowEvent::Resized`, recalculate and call `.set_bounds()` on both webviews so sidebar stays fixed-width and content fills the rest.

---

## IPC Communication

Sidebar (JavaScript) communicates with the Rust event loop via `window.ipc.postMessage(JSON.stringify({...}))`. The Rust side receives these in the `ipc_handler` closure.

### Message Protocol (Sidebar → Rust)

| Action | Payload | Effect |
|--------|---------|--------|
| `navigate` | `{ action: "navigate", url: "..." }` | Load URL in content pane |
| `add_folder` | `{ action: "add_folder", name: "..." }` | Append new folder to store |
| `add_bookmark` | `{ action: "add_bookmark", folder_index: N, name: "...", url: "..." }` | Add bookmark to folder N |
| `delete_bookmark` | `{ action: "delete_bookmark", folder_index: N, bookmark_index: M }` | Remove bookmark M from folder N |
| `delete_folder` | `{ action: "delete_folder", folder_index: N }` | Remove entire folder N |
| `toggle_folder` | `{ action: "toggle_folder", folder_index: N }` | Collapse/expand folder N |

### Rust → Sidebar (refresh after mutation)

After any data mutation, serialize the updated `BookmarkStore` to JSON and push it back to the sidebar:

```rust
let json = serde_json::to_string(&store)?;
sidebar.evaluate_script(&format!("renderBookmarks({})", json))?;
```

### User Events (tao EventLoop)

Use `EventLoopBuilder::<UserEvent>::with_user_event()` with a custom enum so the IPC handler (which runs on a different thread) can send events to the main loop via `EventLoopProxy`:

```rust
#[derive(Debug)]
enum UserEvent {
    Navigate(String),
    AddFolder(String),
    AddBookmark { folder_index: usize, name: String, url: String },
    DeleteBookmark { folder_index: usize, bookmark_index: usize },
    DeleteFolder(usize),
    ToggleFolder(usize),
}
```

---

## Keyboard Shortcuts

Handle via `tao`'s `WindowEvent::KeyboardInput` or via `DeviceEvent`. Check for modifier keys (`ModifiersState`) and virtual key codes.

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Open "Add Bookmark" dialog — two fields: URL and Display Name |
| `Ctrl+Shift+N` | Open "Add Folder" dialog — one field: Folder Name |
| `Ctrl+D` | Delete currently selected/hovered bookmark (with confirmation) |
| `Ctrl+L` | Focus URL/address input (future enhancement) |
| `Ctrl+Q` | Quit application |
| `F1` or `Ctrl+/` | Open Help dialog showing all keyboard shortcuts |
| `F5` | Reload content pane |
| `Ctrl+[` / `Ctrl+]` | Navigate back / forward in content pane |

### How Keyboard Shortcuts Trigger Dialogs

There are two approaches — pick whichever feels cleaner:

**Option A: Handle in tao, trigger dialog in sidebar via `evaluate_script`**

```rust
// In the tao event loop, on detecting Ctrl+N:
sidebar.evaluate_script("showAddBookmarkModal()")?;
// On detecting F1 or Ctrl+/:
sidebar.evaluate_script("showHelpModal()")?;
```

The sidebar HTML already contains the modal markup and JS. This is the simpler approach.

**Option B: Handle entirely in sidebar JS**

Attach a `keydown` listener in the sidebar HTML that checks for Ctrl+N. This works since the sidebar webview captures keyboard events when focused. However, it won't fire if the content pane has focus. So Option A is recommended.

---

## Sidebar UI (HTML/CSS/JS)

The sidebar is a self-contained HTML page rendered in the left wry WebView. It is generated as a Rust string (`fn sidebar_html(store: &BookmarkStore) -> String`) with the bookmark data embedded as a JSON literal.

### Visual Design

- Dark theme (Catppuccin Mocha or similar dark palette)
- Folder rows: icon + name + action buttons (add bookmark, delete folder) — buttons appear on hover
- Bookmark rows: indented under folder, name + delete button on hover
- Active bookmark highlighted with accent color
- Collapsible folders with arrow indicator
- Bottom bar with "+ Folder" button and "? Help" button
- Scrollable tree area

### Dialog Modals (inside sidebar HTML)

Two modal overlays, toggled via JS:

**Add Bookmark Modal** (triggered by Ctrl+N or folder "+" button):
- Input 1: Display Name (text, autofocused)
- Input 2: URL (text, with placeholder `https://...`)
- Hidden field: target folder index (pre-filled if triggered from a folder's "+" button; defaults to first folder if triggered via Ctrl+N)
- Buttons: Cancel, Add
- Enter key in URL field submits
- Escape key closes

**Add Folder Modal** (triggered by Ctrl+Shift+N or bottom "+ Folder" button):
- Input: Folder Name (text, autofocused)
- Buttons: Cancel, Create
- Enter key submits
- Escape key closes

**Help / Keyboard Shortcuts Modal** (triggered by F1 or Ctrl+/):
- Title: "Keyboard Shortcuts"
- Close button: "✕" at top-right corner of the dialog
- No input fields — display only
- Renders a two-column table of all shortcuts:

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Add new bookmark |
| `Ctrl+Shift+N` | Add new folder |
| `Ctrl+D` | Delete selected bookmark |
| `F1` or `Ctrl+/` | Show this help |
| `F5` | Reload page |
| `Ctrl+[` | Navigate back |
| `Ctrl+]` | Navigate forward |
| `Ctrl+Q` | Quit |
| `Escape` | Close dialog |

- Styled consistently with the other modals (dark overlay, rounded card)
- Shortcut keys rendered in `<kbd>` styled elements (monospace, subtle border/background to look like keycaps)
- Escape key closes
- Clicking outside the dialog (on the overlay) also closes it

### Sidebar JS Functions

```javascript
renderBookmarks(data)        // Re-render the tree from JSON data
navigate(url)                // Send navigate IPC + highlight active
toggleFolder(index)          // Send toggle IPC
deleteFolder(index)          // Send delete_folder IPC
deleteBookmark(fi, bi)       // Send delete_bookmark IPC
showAddBookmarkModal(fi?)    // Show modal, optionally pre-select folder
showAddFolderModal()         // Show modal
showHelpModal()              // Show keyboard shortcuts dialog
submitAddBookmark()          // Read inputs, send add_bookmark IPC
submitAddFolder()            // Read input, send add_folder IPC
closeModals()                // Hide all modals
```

---

## Content Pane

### Welcome Page

On startup (before any bookmark is clicked), show a minimal centered message:

```
    📖
  Select a bookmark
  Pick an item from the sidebar to start reading
```

Dark background matching sidebar theme.

### Navigation

When a bookmark is clicked, load the URL in the content webview:

```rust
content.load_url(&url)?;
```

The content pane is a full web renderer (WebKitGTK via wry) — it will render modern sites including JS, CSS, images, etc.

---

## Cargo.toml

```toml
[package]
name = "bookmarks-browser"
version = "0.1.0"
edition = "2021"
description = "A lightweight two-pane bookmark browser for Linux"

[dependencies]
wry = "0.47"
tao = "0.31"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "6"

[profile.release]
opt-level = 3
lto = true
strip = true
```

---

## Build & Run

```bash
# Install system dependency (Arch)
sudo pacman -S webkit2gtk-4.1

# Build
cargo build --release

# Binary location
./target/release/bookmarks-browser
```

Expected release binary size: ~3–5 MB (after strip + LTO).

---

## Project Structure

```
bookmarks-browser/
├── Cargo.toml
├── src/
│   └── main.rs          # Everything in one file to start
└── README.md
```

Keep it as a single `main.rs` for now. The total code should be ~400–500 lines. If it grows, consider splitting into:

```
src/
├── main.rs              # Entry point, event loop
├── store.rs             # BookmarkStore, persistence
├── sidebar.rs           # HTML generation
└── events.rs            # UserEvent enum, IPC handling
```

---

## Future Enhancements (not for v0.1)

- Drag-and-drop reordering of bookmarks and folders
- Import/export bookmarks (HTML bookmark format or JSON)
- Search/filter bar in sidebar (`Ctrl+F`)
- Resizable sidebar width via drag handle
- Collapsible sidebar (expand/collapse toggle)
- Tabs in content pane for opening multiple bookmarks
- Custom CSS injection for reader mode
- Favicon fetching and display next to bookmark names
- Keyboard navigation within the sidebar tree (arrow keys)
- Bookmark tags in addition to folders
