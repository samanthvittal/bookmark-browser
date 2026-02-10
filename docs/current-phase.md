# Current Phase: 6 — Keyboard Shortcuts

## Status: NOT STARTED

## Phase Dependencies
- Phase 1 — Project Bootstrap (complete)
- Phase 2 — Data Model & Persistence (complete)
- Phase 3 — Dual WebView Setup (complete)
- Phase 4 — Sidebar HTML & IPC (complete)
- Phase 5 — Bookmark & Folder Management (complete)

## Context
Phase 5 gave us full CRUD for bookmarks and folders with modal dialogs, a help modal
showing shortcuts, and delete confirmations. Now we add keyboard shortcuts handled via
tao's event loop so they work regardless of which WebView has focus. Shortcuts trigger
sidebar modals via `evaluate_script()` (Option A from the spec). This phase also adds
content pane navigation (back/forward/reload) and the Ctrl+Q quit shortcut.

## Key Files
- `src/main.rs` — add keyboard event handling in tao event loop, add `ReloadContent`, `GoBack`, `GoForward` UserEvent variants

## Tasks

- [ ] **6.1** — Add `ReloadContent`, `GoBack`, `GoForward` variants to `UserEvent` enum
- [ ] **6.2** — Track `ModifiersState` in the event loop by handling `WindowEvent::ModifiersChanged`
- [ ] **6.3** — Handle `WindowEvent::KeyboardInput` — match virtual key codes with modifier state to dispatch shortcuts
- [ ] **6.4** — Implement Ctrl+N → `sidebar.evaluate_script("showAddBookmarkModal()")` to open Add Bookmark modal
- [ ] **6.5** — Implement Ctrl+Shift+N → `sidebar.evaluate_script("showAddFolderModal()")` to open Add Folder modal
- [ ] **6.6** — Implement F1 and Ctrl+/ → `sidebar.evaluate_script("showHelpModal()")` to open Help modal
- [ ] **6.7** — Implement Ctrl+Q → set `ControlFlow::Exit` to quit
- [ ] **6.8** — Implement F5 → send `ReloadContent` event, handle by calling `content.evaluate_script("location.reload()")`
- [ ] **6.9** — Implement Ctrl+[ → send `GoBack` event, handle by calling `content.evaluate_script("history.back()")`
- [ ] **6.10** — Implement Ctrl+] → send `GoForward` event, handle by calling `content.evaluate_script("history.forward()")`
- [ ] **6.11** — Implement Escape → `sidebar.evaluate_script("closeModals()")` to close any open modal

## Test Checkpoint

- [ ] `cargo build` completes without errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` reports no formatting issues
- [ ] `cargo test` passes (existing roundtrip test still works)
- [ ] Ctrl+N opens Add Bookmark modal (even when content pane has focus)
- [ ] Ctrl+Shift+N opens Add Folder modal
- [ ] F1 or Ctrl+/ opens Help modal
- [ ] Ctrl+Q quits the application
- [ ] F5 reloads the content pane
- [ ] Ctrl+[ navigates back, Ctrl+] navigates forward in content pane
- [ ] Escape closes any open modal

## Notes
- Use Option A from the spec: handle keyboard events in tao, trigger sidebar JS via `evaluate_script`
- This ensures shortcuts work regardless of which WebView has focus
- Track modifier state via `WindowEvent::ModifiersChanged` and store in a `ModifiersState` variable
- Match key codes using `VirtualKeyCode` from tao
- Ctrl+D (delete selected bookmark) is listed in the spec but requires tracking "selected" bookmark state — defer to Phase 7 (Polish) if complex

---

## Completed Phases
- Phase 1 — Project Bootstrap (completed 2026-02-10)
- Phase 2 — Data Model & Persistence (completed 2026-02-10)
- Phase 3 — Dual WebView Setup (completed 2026-02-10)
- Phase 4 — Sidebar HTML & IPC (completed 2026-02-10)
- Phase 5 — Bookmark & Folder Management (completed 2026-02-10)
