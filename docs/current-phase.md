# Current Phase: 2 — Data Model & Persistence

## Status: NOT STARTED

## Phase Dependencies
- Phase 1 — Project Bootstrap (complete)

## Context
Phase 1 gave us a compilable Cargo project with a tao window. Now we need the data
layer: Rust structs for bookmarks/folders, JSON persistence to `~/.config/bookmarks-browser/bookmarks.json`,
and sample data for first run. This phase touches only `src/main.rs` — no WebViews or UI yet.

## Key Files
- `src/main.rs` — will add structs, persistence functions, and load store on startup
- `Cargo.toml` — already has `serde`, `serde_json`, `dirs` dependencies (no changes needed)

## Tasks

- [ ] **2.1** — Define `Bookmark`, `Folder`, `BookmarkStore` structs with serde derives (`Serialize`, `Deserialize`, `Clone`, `Debug`)
- [ ] **2.2** — Implement `default_true()` helper for `Folder.expanded` serde default
- [ ] **2.3** — Implement `default_store()` returning a `BookmarkStore` with sample bookmarks (Documentation folder with Rust/Arch Wiki links, News folder with Hacker News)
- [ ] **2.4** — Implement `config_path()` using `dirs::config_dir()` returning `~/.config/bookmarks-browser/bookmarks.json`
- [ ] **2.5** — Implement `BookmarkStore::load()` — read from config path, fallback to `default_store()` if file missing or invalid
- [ ] **2.6** — Implement `BookmarkStore::save()` — write pretty JSON to config path, create parent dirs if needed
- [ ] **2.7** — Unit test: roundtrip `save()` then `load()` produces identical data

## Test Checkpoint

- [ ] `cargo build` completes without errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` reports no formatting issues
- [ ] `cargo test` passes — roundtrip test verifies save/load consistency
- [ ] Running `cargo run` still opens the window (no regression)
- [ ] After first run, `~/.config/bookmarks-browser/bookmarks.json` exists with sample data

## Notes
- Keep everything in `src/main.rs` for now — the file is still small
- Use `Result<T, Box<dyn std::error::Error>>` for persistence functions
- Use `serde_json::to_string_pretty` for human-readable JSON output
- The `load()` function should silently fall back to defaults on any error (missing file, invalid JSON)
- The unit test should use a temp directory (via `std::env::temp_dir()`) to avoid touching real config

---

## Completed Phases
- Phase 1 — Project Bootstrap (completed 2026-02-10)
