# CLAUDE.md — Bookmark Browser Project Intelligence

## Project

Bookmark Browser is a lightweight, native two-pane bookmark browser for Linux built in Rust (GPL 3.0).
Single-binary layout: all source in `/src/`. Uses `tao` for windowing, `wry` for WebView rendering.
Full spec: `/docs/overview.md` (9 phases).

## Active Work

**Always read `/docs/current-phase.md` before doing anything.** It contains the current phase number, tasks, context, key files, and what has already been completed. This is the single source of truth for what to work on.

If `/docs/current-phase.md` does not exist or is empty, ask the user which phase to start.

## Quick Commands

```bash
# Build
cargo build                          # Debug build
cargo build --release                # Release build (LTO + strip)
cargo run                            # Build and run (debug)
cargo run --release                  # Build and run (release)

# Check & Lint
cargo check                          # Type check without building
cargo clippy -- -D warnings          # Lint with warnings as errors
cargo fmt                            # Format code
cargo fmt -- --check                 # Check formatting without modifying

# Test
cargo test                           # Run all tests
cargo test -- --nocapture            # Run tests with stdout visible
cargo test test_name                 # Run specific test

# System dependency (Arch Linux)
sudo pacman -S webkit2gtk-4.1        # Required by wry
```

## Slash Commands

| Command | Purpose |
|---------|---------|
| `/resume` | After `/clear` — read current-phase.md, find next task, report status |
| `/next-task` | Implement exactly one task, run checks, commit, update phase file |
| `/plan-phase` | Review all tasks, propose approach before coding |
| `/test-checkpoint` | Run every Test Checkpoint item, report pass/fail |
| `/complete-phase` | Merge to main, generate next phase in current-phase.md |
| `/status` | Quick overview: phase progress, build/lint status, git, next step |
| `/fix-build` | Diagnose and fix cargo build/clippy/fmt failures |
| `/context-check` | Assess context usage, suggest /clear if needed |

## Rust Conventions

- **Rust 2021 edition**, type annotations where they improve clarity
- **Single `main.rs` initially**; split into modules when file exceeds ~500 lines:
  - `main.rs` — entry point, event loop
  - `store.rs` — BookmarkStore, persistence
  - `sidebar.rs` — HTML generation
  - `events.rs` — UserEvent enum, IPC handling
- **Error handling**: Use `Result<T, Box<dyn std::error::Error>>` for the main function; prefer `?` operator over `.unwrap()`
- **Serde** for all serialization/deserialization; derive `Serialize`, `Deserialize`, `Clone`, `Debug`
- **No unsafe code** unless absolutely necessary and documented
- **Naming**: snake_case for functions/variables/modules; PascalCase for types/structs/enums; SCREAMING_SNAKE_CASE for constants
- **Imports**: Group as std → external crates → local modules; use explicit imports over glob imports

### Data Model

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
- If file doesn't exist, create with sample bookmarks
- Use `serde_json::to_string_pretty` for human-readable JSON

### IPC Protocol (Sidebar JS → Rust)

| Action | Payload | Effect |
|--------|---------|--------|
| `navigate` | `{ action: "navigate", url: "..." }` | Load URL in content pane |
| `add_folder` | `{ action: "add_folder", name: "..." }` | Append new folder to store |
| `add_bookmark` | `{ action: "add_bookmark", folder_index: N, name: "...", url: "..." }` | Add bookmark to folder N |
| `delete_bookmark` | `{ action: "delete_bookmark", folder_index: N, bookmark_index: M }` | Remove bookmark M from folder N |
| `delete_folder` | `{ action: "delete_folder", folder_index: N }` | Remove entire folder N |
| `toggle_folder` | `{ action: "toggle_folder", folder_index: N }` | Collapse/expand folder N |

### UserEvent Enum

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

## Sidebar UI Conventions

- Sidebar is a self-contained HTML page rendered in the left wry WebView
- Generated as a Rust string (`fn sidebar_html(store: &BookmarkStore) -> String`) with bookmark data embedded as JSON
- **Dark theme** (Catppuccin Mocha palette)
- NEVER hardcode colors outside of CSS variable definitions
- All colors via CSS variables: `var(--base)`, `var(--text)`, `var(--accent)`, etc.
- Dialogs are HTML modals inside the sidebar (not native dialogs)
- JS communicates with Rust via `window.ipc.postMessage(JSON.stringify({...}))`
- Rust pushes updates back via `sidebar.evaluate_script(&format!("renderBookmarks({})", json))`

### Sidebar JS Functions

```javascript
renderBookmarks(data)        // Re-render tree from JSON data
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

## Window Layout

- Default window size: 1200x800 logical pixels
- Window title: `"Bookmarks Browser"`
- Sidebar width: 280px (fixed, constant `SIDEBAR_WIDTH`)
- Content pane: fills remaining width
- On `WindowEvent::Resized`, recalculate bounds for both webviews

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Add new bookmark |
| `Ctrl+G` | Add new folder |
| `Ctrl+D` | Delete selected bookmark |
| `F1` or `Ctrl+/` | Show help/shortcuts |
| `F5` | Reload content pane |
| `Ctrl+[` | Navigate back |
| `Ctrl+]` | Navigate forward |
| `Ctrl+Q` | Quit |
| `Escape` | Close dialog |

Handle via tao event loop, trigger sidebar dialogs via `evaluate_script`.

## Project Structure

```
bookmarks-browser/
├── Cargo.toml
├── CLAUDE.md                # This file (always loaded)
├── .claude/
│   └── commands/            # Slash commands for workflow
│       ├── resume.md        # /resume — pick up after /clear
│       ├── next-task.md     # /next-task — implement one task
│       ├── plan-phase.md    # /plan-phase — review before coding
│       ├── test-checkpoint.md
│       ├── complete-phase.md
│       ├── status.md
│       ├── fix-build.md
│       └── context-check.md
├── src/
│   └── main.rs              # Everything in one file to start
├── docs/
│   ├── overview.md          # Full spec (reference only)
│   ├── current-phase.md     # Active phase tracking (source of truth)
│   ├── phase-template.md    # Template for new phases
│   └── workflow.md          # Workflow guide
├── LICENSE
└── README.md
```

## Git Workflow

- **Branch naming**: `phase-{N}-{short-description}` (e.g., `phase-1-project-bootstrap`)
- **Conventional Commits**: `feat:`, `fix:`, `test:`, `docs:`, `refactor:`, `chore:`
- **Commit often**: After each task within a phase, commit with descriptive message
- **Commit message format**: `feat(phase-N): task description` (e.g., `feat(phase-1): add Cargo.toml and project structure`)
- **Never commit directly to main**; always merge via branch
- **Run checks before committing**: `cargo check && cargo clippy -- -D warnings && cargo fmt -- --check`

## Phase Workflow

Each development phase follows this strict sequence:

1. **Read context**: `/resume` or read `/docs/current-phase.md` to understand where we are
2. **Check dependencies**: Verify that dependent phases have been completed (check git history or existing code)
3. **Plan**: `/plan-phase` — review all tasks, propose approach, get user approval
4. **Create branch**: `git checkout -b phase-{N}-{name}`
5. **Implement tasks**: `/next-task` for each task (implements, tests, commits, updates phase file)
6. **Test Checkpoint**: `/test-checkpoint` — run through every checkbox
7. **Complete phase**: `/complete-phase` — merge to main, prepare next phase
8. **Clear context**: `/clear` then `/resume` to start fresh for next phase

## Context Management Rules

- When context exceeds 60%, suggest running `/clear` and resuming from `/docs/current-phase.md`
- After `/clear`, always re-read `/docs/current-phase.md` first before doing anything
- Never try to hold the entire overview.md in context; only read the current phase section
- Use subagents for isolated subtasks (e.g., "generate the sidebar HTML" in a subagent)
- If a file is already written to disk, do NOT re-read it unless needed for the current task

## What NOT To Do

- Do NOT use `unsafe` code without explicit justification and documentation
- Do NOT use `.unwrap()` or `.expect()` in production code paths (use `?` or proper error handling)
- Do NOT hardcode color values outside CSS variable definitions
- Do NOT skip running `cargo clippy` before committing
- Do NOT add dependencies not listed in the spec without discussing first
- Do NOT use GTK/Qt directly — tao handles windowing, wry handles rendering
- Do NOT store HTML in the bookmark store — only TipTap JSON or plain text if applicable
- Do NOT create multiple binaries — this is a single `main.rs` (or modular single binary)
