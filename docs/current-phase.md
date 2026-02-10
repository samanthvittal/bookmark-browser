# Current Phase: 8 — Sidebar Polish, Desktop Integration & GitHub Sync

## Status: COMPLETE

## Phase Dependencies
- Phase 1 — Project Bootstrap (complete)
- Phase 2 — Data Model & Persistence (complete)
- Phase 3 — Dual WebView Setup (complete)
- Phase 4 — Sidebar HTML & IPC (complete)
- Phase 5 — Bookmark & Folder Management (complete)
- Phase 6 — Keyboard Shortcuts (complete)
- Phase 7 — Polish & Release (complete)

## Context
Phases 1–7 delivered a complete v0.1: dual-pane layout with GTK box containers, bookmark/folder
CRUD with JSON persistence, sidebar tree view with modals, keyboard shortcuts, and a polished
release build. Phase 8 adds three feature groups: (1) collapsible sidebar with state persistence,
(2) XDG desktop entry for Linux launcher integration, and (3) GitHub Gist sync for cloud backup
of bookmarks and settings. A new `Settings` struct is introduced as shared infrastructure.

## Key Files
- `src/main.rs` — sidebar toggle, Settings struct, GitHub sync, new UserEvent variants, new IPC actions
- `Cargo.toml` — add `ureq` dependency for HTTP
- `README.md` — update install instructions

New files:
- `assets/bookmark-browser.desktop` — XDG desktop entry
- `assets/bookmark-browser.svg` — app icon
- `install.sh` — user-level install script
- `uninstall.sh` — uninstall script

## Tasks

### Feature 1: Collapsible Sidebar

- [x] **8.1** — Sidebar toggle mechanism: add `Ctrl+B` shortcut, `sidebar_box.hide()`/`.show()` on GTK, collapse strip with `»` expand button, `«` button in sidebar header, new `toggle_sidebar` IPC action and `UserEvent::ToggleSidebar`
- [x] **8.2** — Settings persistence: create `Settings` struct (`sidebar_collapsed`, `github_token`, `github_gist_id`), persist to `~/.config/bookmarks-browser/settings.json`, restore sidebar state on startup, update help modal with `Ctrl+B`

### Feature 2: Desktop Entry

- [x] **8.3** — Create `assets/bookmark-browser.desktop` (XDG Desktop Entry spec) and `assets/bookmark-browser.svg` (Catppuccin-themed bookmark icon)
- [x] **8.4** — Create `install.sh` and `uninstall.sh` for `~/.local/` user-level install (binary + desktop file + icon), update README with install instructions

### Feature 3: GitHub Sync

- [x] **8.5** — Settings UI: add gear icon button in sidebar bottom bar, settings modal with GitHub PAT input and Gist ID display, `save_settings` IPC action, `UserEvent::SaveSettings`, add `ureq` dependency
- [x] **8.6** — Push/pull via Gist API: `POST`/`PATCH /gists` for push, `GET /gists/{id}` for pull, spawned threads with `EventLoopProxy`, `Ctrl+U` (push) / `Ctrl+I` (pull) shortcuts, push/pull buttons in sidebar
- [x] **8.7** — Sync status indicator: status area in sidebar HTML, `updateSyncStatus()` JS function, auto-dismiss messages, error handling (no token, 401, 404 deleted gist, timeout, malformed response)

## Test Checkpoint

- [ ] `cargo build` succeeds with all new code and `ureq` dependency
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] `cargo test` passes all tests
- [ ] `Ctrl+B` toggles sidebar visibility; content pane fills window when collapsed
- [ ] Collapse strip with `»` button appears when sidebar is hidden
- [ ] Sidebar collapsed state persists across app restarts via `settings.json`
- [ ] Help modal includes `Ctrl+B — Toggle sidebar`
- [ ] `desktop-file-validate assets/bookmark-browser.desktop` passes (if tool available)
- [ ] `install.sh` installs binary + desktop file + icon to `~/.local/`
- [ ] `uninstall.sh` removes all installed files
- [ ] Settings modal opens from gear button, saves GitHub PAT to `settings.json`
- [ ] Push creates/updates a private GitHub Gist with `bookmarks.json`
- [ ] Pull fetches Gist content and updates local bookmarks + re-renders sidebar
- [ ] Sync status area shows progress ("Syncing...") and results ("Last synced" / errors)
- [ ] No `.unwrap()` on network or JSON parsing code paths

## Notes
- `sidebar_box` must be extracted from the `#[cfg(target_os = "linux")]` block so it's accessible in the event loop closure
- Non-Linux: use `set_bounds` with zero-width sidebar and full-width content as fallback
- Avoid `Ctrl+Shift` combos (Wayland modifier bug — see project memory)
- `ureq` is synchronous — all network calls must run in `std::thread::spawn` to avoid blocking the event loop
- GitHub PAT stored in plain text in `settings.json` (acceptable for v1, noted as limitation)
- `#[serde(default)]` on all Settings fields for forward compatibility

---

## Completed Phases
- Phase 1 — Project Bootstrap (completed 2026-02-10)
- Phase 2 — Data Model & Persistence (completed 2026-02-10)
- Phase 3 — Dual WebView Setup (completed 2026-02-10)
- Phase 4 — Sidebar HTML & IPC (completed 2026-02-10)
- Phase 5 — Bookmark & Folder Management (completed 2026-02-10)
- Phase 6 — Keyboard Shortcuts (completed 2026-02-10)
- Phase 7 — Polish & Release (completed 2026-02-10)
