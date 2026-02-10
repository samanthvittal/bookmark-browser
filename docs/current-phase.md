# Current Phase: 7 — Polish & Release

## Status: NOT STARTED

## Phase Dependencies
- Phase 1 — Project Bootstrap (complete)
- Phase 2 — Data Model & Persistence (complete)
- Phase 3 — Dual WebView Setup (complete)
- Phase 4 — Sidebar HTML & IPC (complete)
- Phase 5 — Bookmark & Folder Management (complete)
- Phase 6 — Keyboard Shortcuts (complete)

## Context
Phases 1–6 delivered the full feature set: dual-pane layout, bookmark/folder CRUD with
persistence, sidebar tree view with modals, and keyboard shortcuts. Phase 7 is the final
polish pass — auditing error handling, fixing edge cases, verifying the release build,
updating the README, and doing a final code quality review.

## Key Files
- `src/main.rs` — audit `.unwrap()`/`.expect()` calls, edge cases, dead code
- `README.md` — update with install instructions, feature list
- `Cargo.toml` — verify metadata

## Tasks

- [ ] **7.1** — Audit all `.unwrap()` and `.expect()` calls — replace with proper error handling (`?` operator) where possible
- [ ] **7.2** — Handle edge cases: empty store, missing config dir, invalid JSON, very long bookmark names
- [ ] **7.3** — Verify release build: `cargo build --release`, check binary size (target: 3–5 MB)
- [ ] **7.4** — Update README.md with install instructions, feature list, build commands
- [ ] **7.5** — Final code review: check for dead code, unused imports, consistent naming
- [ ] **7.6** — Run full test suite and all clippy checks one final time

## Test Checkpoint

- [ ] `cargo build --release` completes without errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` reports no formatting issues
- [ ] `cargo test` passes all tests
- [ ] No `.unwrap()` calls remain in production code paths (test code is OK)
- [ ] Release binary size is under 5 MB
- [ ] README.md has install instructions and feature list
- [ ] `cargo run --release` opens the app and all features work

## Notes
- Keep `.expect()` only in the initial window/webview setup where failure is truly unrecoverable
- The `#[allow(dead_code)]` on UserEvent can be removed if unused variants are cleaned up
- Binary size target with LTO + strip: 3–5 MB

---

## Completed Phases
- Phase 1 — Project Bootstrap (completed 2026-02-10)
- Phase 2 — Data Model & Persistence (completed 2026-02-10)
- Phase 3 — Dual WebView Setup (completed 2026-02-10)
- Phase 4 — Sidebar HTML & IPC (completed 2026-02-10)
- Phase 5 — Bookmark & Folder Management (completed 2026-02-10)
- Phase 6 — Keyboard Shortcuts (completed 2026-02-10)
