# Current Phase: 1 — Project Bootstrap

## Status: COMPLETE

## Phase Dependencies
- None (starting point)

## Context
This is the very first phase. We are bootstrapping the Rust project from scratch.
No code exists yet. After this phase, we will have a compilable Cargo project with
the correct structure, dependencies, and a minimal `main.rs` that opens a tao window.

## Key Files to Create
- `Cargo.toml` — project manifest with all dependencies
- `src/main.rs` — minimal entry point that opens a tao window and runs the event loop
- `.gitignore` — Rust-specific ignores

## Tasks

- [x] **1.1** — Create `Cargo.toml` with all dependencies: `wry` (0.47+), `tao` (0.31+), `serde` (with derive), `serde_json`, `dirs` (6+). Include release profile with `opt-level = 3`, `lto = true`, `strip = true`
- [x] **1.2** — Create `.gitignore` for Rust projects (`/target`, `Cargo.lock` if library — but keep it since this is a binary, `*.swp`, etc.)
- [x] **1.3** — Create `src/main.rs` with a minimal tao event loop that opens a 1200x800 window titled "Bookmarks Browser" and handles `CloseRequested` to exit cleanly
- [x] **1.4** — Verify the project compiles and runs: `cargo build` succeeds, `cargo clippy -- -D warnings` passes, `cargo fmt -- --check` passes

## Test Checkpoint

After completing all tasks, manually verify each item:

- [x] `cargo build` completes without errors
- [x] `cargo clippy -- -D warnings` passes with no warnings
- [x] `cargo fmt -- --check` reports no formatting issues
- [x] Running `cargo run` opens a native window titled "Bookmarks Browser" at approximately 1200x800 size
- [x] Closing the window exits the process cleanly (no zombie process)

## Notes
- System dependency required: `webkit2gtk-4.1` (install via `sudo pacman -S webkit2gtk-4.1` on Arch)
- The window will be empty at this stage — just proving tao works
- Do NOT add wry WebViews yet — that comes in Phase 3

---

## Completed Phases
(none yet)
