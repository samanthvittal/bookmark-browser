# Current Phase: 4 — Sidebar HTML & IPC

## Status: NOT STARTED

## Phase Dependencies
- Phase 1 — Project Bootstrap (complete)
- Phase 2 — Data Model & Persistence (complete)
- Phase 3 — Dual WebView Setup (complete)

## Context
Phase 3 gave us two side-by-side WebViews: a sidebar with placeholder text and a content pane
with a welcome message. Now we wire the sidebar to actually render the bookmark tree from
`BookmarkStore` data, handle click-to-navigate via IPC, and load URLs in the content pane.
This phase introduces `UserEvent`, `EventLoopProxy`, the IPC handler, and the full sidebar
HTML/CSS/JS generation function. After this phase the app will be a functional bookmark browser.

## Key Files
- `src/main.rs` — replace placeholder sidebar with `sidebar_html(store)`, add IPC handler,
  add `UserEvent` enum, switch to `EventLoopBuilder::with_user_event()`, handle navigate events

## Tasks

- [ ] **4.1** — Define `UserEvent` enum with `Navigate(String)` variant and switch `EventLoop::new()` to `EventLoopBuilder::<UserEvent>::with_user_event().build()`
- [ ] **4.2** — Implement `fn sidebar_html(store: &BookmarkStore) -> String` that generates the full sidebar HTML with folder tree, Catppuccin Mocha dark theme, CSS variables, and JS functions (`renderBookmarks`, `navigate`, `toggleFolder`)
- [ ] **4.3** — Add IPC handler to sidebar WebView that parses JSON messages and sends `UserEvent`s via `EventLoopProxy`
- [ ] **4.4** — Handle `UserEvent::Navigate` in the event loop — call `content.load_url()` to load the clicked bookmark
- [ ] **4.5** — Handle `UserEvent::ToggleFolder` — update store, re-render sidebar via `evaluate_script`
- [ ] **4.6** — Verify clicking a bookmark loads the page in the content pane (manual test)

## Test Checkpoint

- [ ] `cargo build` completes without errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` reports no formatting issues
- [ ] `cargo test` passes (existing roundtrip test still works)
- [ ] Running `cargo run` shows the bookmark tree in the sidebar with folder names and bookmark names
- [ ] Clicking a bookmark loads the URL in the content pane
- [ ] Clicking a folder name toggles its collapse/expand state
- [ ] Folders show an expand/collapse arrow indicator
- [ ] Active bookmark is highlighted with accent color

## Notes
- Use `EventLoopBuilder::<UserEvent>::with_user_event().build()` instead of `EventLoop::new()`
- The IPC handler receives a String; parse it as JSON with `serde_json::from_str::<serde_json::Value>`
- `EventLoopProxy::send_event()` sends events from the IPC handler thread to the main event loop
- After toggling a folder, serialize the updated store and push to sidebar via `evaluate_script("renderBookmarks(...)")`
- Keep `content` accessible in the event loop closure for `load_url` calls
- The sidebar JS uses `window.ipc.postMessage(JSON.stringify({...}))` to communicate with Rust

---

## Completed Phases
- Phase 1 — Project Bootstrap (completed 2026-02-10)
- Phase 2 — Data Model & Persistence (completed 2026-02-10)
- Phase 3 — Dual WebView Setup (completed 2026-02-10)
