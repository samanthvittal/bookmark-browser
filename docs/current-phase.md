# Current Phase: 3 — Dual WebView Setup

## Status: COMPLETE

## Phase Dependencies
- Phase 1 — Project Bootstrap (complete)
- Phase 2 — Data Model & Persistence (complete)

## Context
Phase 2 gave us the data layer: `BookmarkStore` with JSON persistence and sample bookmarks.
Now we add the visual foundation: two side-by-side `wry` WebViews — a fixed-width sidebar
(280px) and a content pane that fills the rest. The sidebar shows placeholder HTML for now;
the content pane shows a dark-themed welcome message. Both panes must resize correctly
when the window is resized. This is the first phase that uses `wry`.

## Key Files
- `src/main.rs` — will replace bare `tao` window with dual `wry` WebViews
- `Cargo.toml` — already has `wry` dependency (no changes needed)

## Tasks

- [x] **3.1** — Define `SIDEBAR_WIDTH` constant (280.0 f64)
- [x] **3.2** — Create sidebar WebView with placeholder HTML ("Sidebar placeholder") using `WebViewBuilder`
- [x] **3.3** — Create content WebView with welcome page HTML (centered "Select a bookmark" message, dark theme using Catppuccin Mocha palette)
- [x] **3.4** — Handle `WindowEvent::Resized` — recalculate and call `set_bounds()` on both webviews
- [ ] **3.5** — Verify both panes render and resize correctly (manual visual check)

## Test Checkpoint

- [ ] `cargo build` completes without errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` reports no formatting issues
- [ ] `cargo test` passes (existing roundtrip test still works)
- [ ] Running `cargo run` shows two panes side by side: sidebar (280px) and content pane
- [ ] Sidebar shows "Sidebar placeholder" text
- [ ] Content pane shows centered "Select a bookmark" welcome message with dark theme
- [ ] Resizing the window correctly repositions and resizes both panes

## Notes
- Use `WebViewBuilder::new_as_child(&window)` to create child webviews within the tao window
- Sidebar is positioned at (0, 0) with width `SIDEBAR_WIDTH` and full window height
- Content pane is positioned at (`SIDEBAR_WIDTH`, 0) with remaining width and full window height
- Both webviews need their bounds updated in the `Resized` event handler
- The welcome page HTML should use CSS variables from the Catppuccin Mocha palette
- Keep the `BookmarkStore` load/save from Phase 2 — it's not wired to the UI yet

---

## Completed Phases
- Phase 1 — Project Bootstrap (completed 2026-02-10)
- Phase 2 — Data Model & Persistence (completed 2026-02-10)
