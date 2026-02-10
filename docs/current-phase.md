# Current Phase: 5 — Bookmark & Folder Management

## Status: ALL TASKS COMPLETE — pending test checkpoint

## Phase Dependencies
- Phase 1 — Project Bootstrap (complete)
- Phase 2 — Data Model & Persistence (complete)
- Phase 3 — Dual WebView Setup (complete)
- Phase 4 — Sidebar HTML & IPC (complete)

## Context
Phase 4 gave us a functional bookmark browser: the sidebar renders the bookmark tree,
clicking a bookmark navigates the content pane, and folders collapse/expand. Now we add
CRUD operations — adding and deleting bookmarks and folders. This phase extends the
`UserEvent` enum with `AddBookmark`, `AddFolder`, `DeleteBookmark`, and `DeleteFolder`
variants, adds HTML modal dialogs in the sidebar for user input, and wires everything
through IPC. After this phase the user can fully manage their bookmark collection.

## Key Files
- `src/main.rs` — extend UserEvent enum, add event handlers, add modal HTML/JS to sidebar_html

## Tasks

- [x] **5.1** — Extend `UserEvent` enum with `AddFolder(String)`, `AddBookmark { folder_index, name, url }`, `DeleteBookmark { folder_index, bookmark_index }`, `DeleteFolder(usize)` variants
- [x] **5.2** — Add "Add Bookmark" modal HTML/CSS to sidebar (name input, URL input, folder selector dropdown, Cancel/Add buttons)
- [x] **5.3** — Add "Add Folder" modal HTML/CSS to sidebar (name input, Cancel/Create buttons)
- [x] **5.4** — Implement `showAddBookmarkModal(fi?)`, `showAddFolderModal()`, `closeModals()`, `submitAddBookmark()`, `submitAddFolder()` JS functions with IPC messages
- [x] **5.5** — Add sidebar UI elements: "+" button on folder headers to add bookmark to that folder, bottom bar with "+ Folder" and "? Help" buttons, delete buttons on hover for bookmarks and folders
- [x] **5.6** — Extend IPC handler to parse `add_folder`, `add_bookmark`, `delete_bookmark`, `delete_folder` actions and send corresponding UserEvents
- [x] **5.7** — Handle `AddFolder` event — append new folder to store, save, refresh sidebar
- [x] **5.8** — Handle `AddBookmark` event — push bookmark to target folder, save, refresh sidebar
- [x] **5.9** — Handle `DeleteBookmark` event — remove bookmark from folder, save, refresh sidebar
- [x] **5.10** — Handle `DeleteFolder` event — remove entire folder from store, save, refresh sidebar
- [x] **5.11** — Add confirmation prompt in sidebar JS for delete operations before sending IPC
- [x] **5.12** — Handle Enter key to submit and Escape key to close in modal dialogs

## Test Checkpoint

- [ ] `cargo build` completes without errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` reports no formatting issues
- [ ] `cargo test` passes (existing roundtrip test still works)
- [ ] Clicking "+ Folder" in bottom bar opens Add Folder modal, submitting creates a new folder
- [ ] Clicking "+" on a folder header opens Add Bookmark modal pre-selecting that folder
- [ ] Submitting Add Bookmark modal adds the bookmark under the selected folder
- [ ] Hovering a bookmark shows a delete button; clicking it (after confirmation) removes it
- [ ] Hovering a folder shows a delete button; clicking it (after confirmation) removes the entire folder
- [ ] Escape closes any open modal
- [ ] Enter submits the active modal
- [ ] Changes persist after closing and reopening the app

## Notes
- Modal dialogs are HTML overlays inside the sidebar WebView, not native OS dialogs
- Use `position: fixed` overlay with backdrop for modals
- Delete confirmation can use a simple inline "Are you sure?" prompt or a small confirmation modal
- The folder selector in Add Bookmark modal should be a `<select>` element populated from the current folders array
- All mutations follow the pattern: IPC → UserEvent → update store → save → evaluate_script(renderBookmarks)
- The "? Help" button in the bottom bar is a placeholder for now — it will be wired in Phase 6 (Keyboard Shortcuts)

---

## Completed Phases
- Phase 1 — Project Bootstrap (completed 2026-02-10)
- Phase 2 — Data Model & Persistence (completed 2026-02-10)
- Phase 3 — Dual WebView Setup (completed 2026-02-10)
- Phase 4 — Sidebar HTML & IPC (completed 2026-02-10)
