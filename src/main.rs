use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::dpi::{LogicalPosition, LogicalSize as WryLogicalSize};
use wry::{Rect, WebViewBuilder};

#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;

const SIDEBAR_WIDTH: f64 = 280.0;

#[derive(Debug)]
enum UserEvent {
    Navigate(String),
    ToggleFolder(usize),
    AddFolder(String),
    AddBookmark {
        folder_index: usize,
        name: String,
        url: String,
    },
    DeleteBookmark {
        folder_index: usize,
        bookmark_index: usize,
    },
    DeleteFolder(usize),
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Bookmark {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Folder {
    name: String,
    #[serde(default = "default_true")]
    expanded: bool,
    bookmarks: Vec<Bookmark>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct BookmarkStore {
    folders: Vec<Folder>,
}

fn default_store() -> BookmarkStore {
    BookmarkStore {
        folders: vec![
            Folder {
                name: "Documentation".to_string(),
                expanded: true,
                bookmarks: vec![
                    Bookmark {
                        name: "The Rust Programming Language".to_string(),
                        url: "https://doc.rust-lang.org/book/".to_string(),
                    },
                    Bookmark {
                        name: "Arch Wiki".to_string(),
                        url: "https://wiki.archlinux.org/".to_string(),
                    },
                ],
            },
            Folder {
                name: "News".to_string(),
                expanded: true,
                bookmarks: vec![Bookmark {
                    name: "Hacker News".to_string(),
                    url: "https://news.ycombinator.com/".to_string(),
                }],
            },
        ],
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"))
        .join("bookmarks-browser")
        .join("bookmarks.json")
}

impl BookmarkStore {
    fn load() -> BookmarkStore {
        Self::load_from(&config_path())
    }

    fn load_from(path: &PathBuf) -> BookmarkStore {
        fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_else(default_store)
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_to(&config_path())
    }

    fn save_to(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

fn sidebar_html(store: &BookmarkStore) -> String {
    let folders_json = serde_json::to_string(&store.folders).unwrap_or_else(|_| "[]".to_string());
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<style>
  :root {{
    --base: #1e1e2e;
    --mantle: #181825;
    --surface0: #313244;
    --surface1: #45475a;
    --surface2: #585b70;
    --text: #cdd6f4;
    --subtext: #a6adc8;
    --accent: #cba6f7;
    --red: #f38ba8;
    --green: #a6e3a1;
    --overlay: rgba(0, 0, 0, 0.5);
  }}
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html {{ height: 100%; }}
  body {{
    background: var(--mantle);
    color: var(--text);
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 14px;
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--surface0);
  }}
  #tree {{
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }}
  .folder-header {{
    display: flex;
    align-items: center;
    padding: 6px 12px;
    cursor: pointer;
    user-select: none;
    color: var(--subtext);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }}
  .folder-header:hover {{
    background: var(--surface0);
  }}
  .folder-arrow {{
    display: inline-block;
    width: 16px;
    font-size: 10px;
    color: var(--subtext);
  }}
  .folder-name {{
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }}
  .folder-actions {{
    display: none;
    align-items: center;
    gap: 2px;
    margin-left: auto;
  }}
  .folder-header:hover .folder-actions {{
    display: flex;
  }}
  .icon-btn {{
    background: none;
    border: none;
    color: var(--subtext);
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
    line-height: 1;
  }}
  .icon-btn:hover {{
    color: var(--text);
  }}
  .icon-btn.delete:hover {{
    color: var(--red);
  }}
  .bookmark {{
    display: flex;
    align-items: center;
    padding: 6px 12px 6px 32px;
    color: var(--text);
    text-decoration: none;
    cursor: pointer;
  }}
  .bookmark:hover {{
    background: var(--surface0);
  }}
  .bookmark.active {{
    background: var(--surface0);
    color: var(--accent);
  }}
  .bookmark-name {{
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }}
  .bookmark .delete-btn {{
    display: none;
    background: none;
    border: none;
    color: var(--subtext);
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
    line-height: 1;
  }}
  .bookmark:hover .delete-btn {{
    display: inline;
  }}
  .bookmark .delete-btn:hover {{
    color: var(--red);
  }}
  .bottom-bar {{
    display: flex;
    border-top: 1px solid var(--surface0);
    padding: 8px;
    gap: 8px;
  }}
  .bar-btn {{
    flex: 1;
    background: var(--surface1);
    border: 1px solid var(--surface2);
    color: var(--text);
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
  }}
  .bar-btn:hover {{
    background: var(--surface2);
    color: var(--text);
  }}
  .modal-overlay {{
    display: none;
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: var(--overlay);
    z-index: 100;
    align-items: center;
    justify-content: center;
  }}
  .modal-overlay.active {{
    display: flex;
  }}
  .modal {{
    background: var(--base);
    border: 1px solid var(--surface1);
    border-radius: 8px;
    padding: 20px;
    width: 240px;
  }}
  .modal h3 {{
    font-size: 14px;
    margin-bottom: 12px;
    color: var(--text);
  }}
  .modal label {{
    display: block;
    font-size: 12px;
    color: var(--subtext);
    margin-bottom: 4px;
  }}
  .modal input, .modal select {{
    width: 100%;
    padding: 6px 8px;
    margin-bottom: 10px;
    background: var(--surface0);
    border: 1px solid var(--surface1);
    border-radius: 4px;
    color: var(--text);
    font-size: 13px;
    font-family: inherit;
    outline: none;
  }}
  .modal select {{
    appearance: none;
    -webkit-appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23a6adc8' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 28px;
  }}
  .modal select option {{
    background: var(--surface0);
    color: var(--text);
  }}
  .modal input:focus, .modal select:focus {{
    border-color: var(--accent);
  }}
  .modal-buttons {{
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }}
  .modal-buttons button {{
    flex: 1;
    padding: 6px 8px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    font-family: inherit;
  }}
  .btn-cancel {{
    background: var(--surface0);
    color: var(--subtext);
  }}
  .btn-cancel:hover {{
    background: var(--surface1);
    color: var(--text);
  }}
  .btn-primary {{
    background: var(--accent);
    color: var(--base);
    font-weight: 600;
  }}
  .btn-primary:hover {{
    opacity: 0.9;
  }}
  .help-table {{
    width: 100%;
    margin-bottom: 12px;
  }}
  .help-table td {{
    padding: 4px 0;
    font-size: 13px;
  }}
  .help-key {{
    color: var(--accent);
    font-family: monospace;
    padding-right: 12px;
    white-space: nowrap;
  }}
</style>
</head>
<body>
<div id="tree"></div>
<div class="bottom-bar">
  <button class="bar-btn" onclick="showAddFolderModal()">+ Folder</button>
  <button class="bar-btn" onclick="showHelpModal()">? Help</button>
</div>

<div id="addBookmarkOverlay" class="modal-overlay">
  <div class="modal">
    <h3>Add Bookmark</h3>
    <label for="bmName">Name</label>
    <input type="text" id="bmName" placeholder="Bookmark name">
    <label for="bmUrl">URL</label>
    <input type="text" id="bmUrl" placeholder="https://...">
    <label for="bmFolder">Folder</label>
    <select id="bmFolder"></select>
    <div class="modal-buttons">
      <button class="btn-cancel" onclick="closeModals()">Cancel</button>
      <button class="btn-primary" onclick="submitAddBookmark()">Add</button>
    </div>
  </div>
</div>

<div id="addFolderOverlay" class="modal-overlay">
  <div class="modal">
    <h3>Add Folder</h3>
    <label for="folderName">Name</label>
    <input type="text" id="folderName" placeholder="Folder name">
    <div class="modal-buttons">
      <button class="btn-cancel" onclick="closeModals()">Cancel</button>
      <button class="btn-primary" onclick="submitAddFolder()">Create</button>
    </div>
  </div>
</div>

<div id="helpOverlay" class="modal-overlay">
  <div class="modal">
    <h3>Keyboard Shortcuts</h3>
    <table class="help-table">
      <tr><td class="help-key">Ctrl+N</td><td>Add bookmark</td></tr>
      <tr><td class="help-key">Ctrl+Shift+N</td><td>Add folder</td></tr>
      <tr><td class="help-key">Ctrl+D</td><td>Delete bookmark</td></tr>
      <tr><td class="help-key">F5</td><td>Reload page</td></tr>
      <tr><td class="help-key">Ctrl+[</td><td>Navigate back</td></tr>
      <tr><td class="help-key">Ctrl+]</td><td>Navigate forward</td></tr>
      <tr><td class="help-key">Ctrl+Q</td><td>Quit</td></tr>
      <tr><td class="help-key">Escape</td><td>Close dialog</td></tr>
    </table>
    <div class="modal-buttons">
      <button class="btn-primary" onclick="closeModals()" style="flex:1">Close</button>
    </div>
  </div>
</div>

<script>
  let folders = {folders_json};
  let activeUrl = null;
  let activeModal = null;

  function renderBookmarks(data) {{
    folders = data;
    const tree = document.getElementById('tree');
    tree.innerHTML = '';
    folders.forEach(function(folder, fi) {{
      const header = document.createElement('div');
      header.className = 'folder-header';
      header.onclick = function() {{ toggleFolder(fi); }};

      const arrow = document.createElement('span');
      arrow.className = 'folder-arrow';
      arrow.textContent = folder.expanded ? '\u25BC' : '\u25B6';

      const name = document.createElement('span');
      name.className = 'folder-name';
      name.textContent = folder.name;

      const actions = document.createElement('span');
      actions.className = 'folder-actions';

      const addBtn = document.createElement('button');
      addBtn.className = 'icon-btn';
      addBtn.textContent = '+';
      addBtn.title = 'Add bookmark to this folder';
      addBtn.onclick = function(e) {{ e.stopPropagation(); showAddBookmarkModal(fi); }};

      const delBtn = document.createElement('button');
      delBtn.className = 'icon-btn delete';
      delBtn.textContent = '\u00D7';
      delBtn.title = 'Delete folder';
      delBtn.onclick = function(e) {{ e.stopPropagation(); deleteFolder(fi); }};

      actions.appendChild(addBtn);
      actions.appendChild(delBtn);
      header.appendChild(arrow);
      header.appendChild(name);
      header.appendChild(actions);
      tree.appendChild(header);

      if (folder.expanded) {{
        folder.bookmarks.forEach(function(bm, bi) {{
          const link = document.createElement('div');
          link.className = 'bookmark' + (bm.url === activeUrl ? ' active' : '');
          link.title = bm.url;
          link.onclick = function() {{ navigate(bm.url); }};

          const bmName = document.createElement('span');
          bmName.className = 'bookmark-name';
          bmName.textContent = bm.name;

          const bmDel = document.createElement('button');
          bmDel.className = 'delete-btn';
          bmDel.textContent = '\u00D7';
          bmDel.title = 'Delete bookmark';
          bmDel.onclick = function(e) {{ e.stopPropagation(); deleteBookmark(fi, bi); }};

          link.appendChild(bmName);
          link.appendChild(bmDel);
          tree.appendChild(link);
        }});
      }}
    }});
  }}

  function navigate(url) {{
    activeUrl = url;
    window.ipc.postMessage(JSON.stringify({{ action: 'navigate', url: url }}));
    renderBookmarks(folders);
  }}

  function toggleFolder(index) {{
    window.ipc.postMessage(JSON.stringify({{ action: 'toggle_folder', folder_index: index }}));
  }}

  function deleteFolder(fi) {{
    if (confirm('Delete folder "' + folders[fi].name + '" and all its bookmarks?')) {{
      window.ipc.postMessage(JSON.stringify({{ action: 'delete_folder', folder_index: fi }}));
    }}
  }}

  function deleteBookmark(fi, bi) {{
    if (confirm('Delete bookmark "' + folders[fi].bookmarks[bi].name + '"?')) {{
      window.ipc.postMessage(JSON.stringify({{ action: 'delete_bookmark', folder_index: fi, bookmark_index: bi }}));
    }}
  }}

  function showAddBookmarkModal(fi) {{
    const select = document.getElementById('bmFolder');
    select.innerHTML = '';
    folders.forEach(function(folder, i) {{
      const opt = document.createElement('option');
      opt.value = i;
      opt.textContent = folder.name;
      if (fi !== undefined && fi === i) opt.selected = true;
      select.appendChild(opt);
    }});
    document.getElementById('bmName').value = '';
    document.getElementById('bmUrl').value = '';
    document.getElementById('addBookmarkOverlay').classList.add('active');
    activeModal = 'addBookmark';
    document.getElementById('bmName').focus();
  }}

  function showAddFolderModal() {{
    document.getElementById('folderName').value = '';
    document.getElementById('addFolderOverlay').classList.add('active');
    activeModal = 'addFolder';
    document.getElementById('folderName').focus();
  }}

  function showHelpModal() {{
    document.getElementById('helpOverlay').classList.add('active');
    activeModal = 'help';
  }}

  function closeModals() {{
    document.getElementById('addBookmarkOverlay').classList.remove('active');
    document.getElementById('addFolderOverlay').classList.remove('active');
    document.getElementById('helpOverlay').classList.remove('active');
    activeModal = null;
  }}

  function submitAddBookmark() {{
    const name = document.getElementById('bmName').value.trim();
    const url = document.getElementById('bmUrl').value.trim();
    const fi = parseInt(document.getElementById('bmFolder').value, 10);
    if (!name || !url) return;
    window.ipc.postMessage(JSON.stringify({{ action: 'add_bookmark', folder_index: fi, name: name, url: url }}));
    closeModals();
  }}

  function submitAddFolder() {{
    const name = document.getElementById('folderName').value.trim();
    if (!name) return;
    window.ipc.postMessage(JSON.stringify({{ action: 'add_folder', name: name }}));
    closeModals();
  }}

  document.addEventListener('keydown', function(e) {{
    if (e.key === 'Escape') {{
      closeModals();
    }} else if (e.key === 'Enter' && activeModal) {{
      e.preventDefault();
      if (activeModal === 'addBookmark') submitAddBookmark();
      else if (activeModal === 'addFolder') submitAddFolder();
    }}
  }});

  renderBookmarks(folders);
</script>
</body>
</html>"#,
        folders_json = folders_json
    )
}

fn welcome_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
<style>
  :root {
    --base: #1e1e2e;
    --mantle: #181825;
    --text: #cdd6f4;
    --subtext: #a6adc8;
    --accent: #cba6f7;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    background: var(--base);
    color: var(--text);
    font-family: system-ui, -apple-system, sans-serif;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
  }
  .welcome {
    text-align: center;
  }
  .welcome h1 {
    font-size: 24px;
    font-weight: 600;
    color: var(--text);
    margin-bottom: 8px;
  }
  .welcome p {
    font-size: 14px;
    color: var(--subtext);
  }
</style>
</head>
<body>
  <div class="welcome">
    <h1>Select a bookmark</h1>
    <p>Choose a bookmark from the sidebar to get started.</p>
  </div>
</body>
</html>"#
        .to_string()
}

fn make_bounds(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        position: LogicalPosition::new(x, y).into(),
        size: WryLogicalSize::new(width, height).into(),
    }
}

fn main() {
    let mut store = BookmarkStore::load();
    if let Err(e) = store.save() {
        eprintln!("Warning: could not save bookmarks: {e}");
    }

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Bookmarks Browser")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)
        .expect("Failed to create window");

    let inner = window.inner_size();
    let scale = window.scale_factor();
    let w = inner.width as f64 / scale;
    let h = inner.height as f64 / scale;

    let sidebar_builder = WebViewBuilder::new()
        .with_html(sidebar_html(&store))
        .with_bounds(make_bounds(0.0, 0.0, SIDEBAR_WIDTH, h))
        .with_ipc_handler(move |req: wry::http::Request<String>| {
            let body = req.body();
            let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) else {
                return;
            };
            let Some(action) = msg.get("action").and_then(|a| a.as_str()) else {
                return;
            };
            match action {
                "navigate" => {
                    if let Some(url) = msg.get("url").and_then(|u| u.as_str()) {
                        let _ = proxy.send_event(UserEvent::Navigate(url.to_string()));
                    }
                }
                "toggle_folder" => {
                    if let Some(index) = msg.get("folder_index").and_then(|i| i.as_u64()) {
                        let _ = proxy.send_event(UserEvent::ToggleFolder(index as usize));
                    }
                }
                "add_folder" => {
                    if let Some(name) = msg.get("name").and_then(|n| n.as_str()) {
                        let _ = proxy.send_event(UserEvent::AddFolder(name.to_string()));
                    }
                }
                "add_bookmark" => {
                    if let (Some(fi), Some(name), Some(url)) = (
                        msg.get("folder_index").and_then(|i| i.as_u64()),
                        msg.get("name").and_then(|n| n.as_str()),
                        msg.get("url").and_then(|u| u.as_str()),
                    ) {
                        let _ = proxy.send_event(UserEvent::AddBookmark {
                            folder_index: fi as usize,
                            name: name.to_string(),
                            url: url.to_string(),
                        });
                    }
                }
                "delete_bookmark" => {
                    if let (Some(fi), Some(bi)) = (
                        msg.get("folder_index").and_then(|i| i.as_u64()),
                        msg.get("bookmark_index").and_then(|i| i.as_u64()),
                    ) {
                        let _ = proxy.send_event(UserEvent::DeleteBookmark {
                            folder_index: fi as usize,
                            bookmark_index: bi as usize,
                        });
                    }
                }
                "delete_folder" => {
                    if let Some(index) = msg.get("folder_index").and_then(|i| i.as_u64()) {
                        let _ = proxy.send_event(UserEvent::DeleteFolder(index as usize));
                    }
                }
                _ => {}
            }
        });

    let content_builder = WebViewBuilder::new()
        .with_html(welcome_html())
        .with_bounds(make_bounds(SIDEBAR_WIDTH, 0.0, w - SIDEBAR_WIDTH, h));

    #[cfg(target_os = "linux")]
    let (sidebar, content) = {
        use gtk::prelude::*;

        let vbox = window.default_vbox().expect("Failed to get default vbox");
        let fixed = gtk::Fixed::new();
        fixed.show_all();
        vbox.pack_start(&fixed, true, true, 0);

        let sidebar = sidebar_builder
            .build_gtk(&fixed)
            .expect("Failed to create sidebar webview");
        let content = content_builder
            .build_gtk(&fixed)
            .expect("Failed to create content webview");
        (sidebar, content)
    };

    #[cfg(not(target_os = "linux"))]
    let (sidebar, content) = {
        let sidebar = sidebar_builder
            .build_as_child(&window)
            .expect("Failed to create sidebar webview");
        let content = content_builder
            .build_as_child(&window)
            .expect("Failed to create content webview");
        (sidebar, content)
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let scale = window.scale_factor();
                let w = new_size.width as f64 / scale;
                let h = new_size.height as f64 / scale;

                let _ = sidebar.set_bounds(make_bounds(0.0, 0.0, SIDEBAR_WIDTH, h));
                let _ = content.set_bounds(make_bounds(SIDEBAR_WIDTH, 0.0, w - SIDEBAR_WIDTH, h));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::Navigate(url)) => {
                let _ = content.load_url(&url);
            }
            Event::UserEvent(UserEvent::ToggleFolder(index)) => {
                if let Some(folder) = store.folders.get_mut(index) {
                    folder.expanded = !folder.expanded;
                    let _ = store.save();
                    if let Ok(json) = serde_json::to_string(&store.folders) {
                        let _ = sidebar.evaluate_script(&format!("renderBookmarks({json})"));
                    }
                }
            }
            Event::UserEvent(UserEvent::AddFolder(name)) => {
                store.folders.push(Folder {
                    name,
                    expanded: true,
                    bookmarks: vec![],
                });
                let _ = store.save();
                if let Ok(json) = serde_json::to_string(&store.folders) {
                    let _ = sidebar.evaluate_script(&format!("renderBookmarks({json})"));
                }
            }
            Event::UserEvent(UserEvent::AddBookmark {
                folder_index,
                name,
                url,
            }) => {
                if let Some(folder) = store.folders.get_mut(folder_index) {
                    folder.bookmarks.push(Bookmark { name, url });
                    let _ = store.save();
                    if let Ok(json) = serde_json::to_string(&store.folders) {
                        let _ = sidebar.evaluate_script(&format!("renderBookmarks({json})"));
                    }
                }
            }
            Event::UserEvent(UserEvent::DeleteBookmark {
                folder_index,
                bookmark_index,
            }) => {
                if let Some(folder) = store.folders.get_mut(folder_index) {
                    if bookmark_index < folder.bookmarks.len() {
                        folder.bookmarks.remove(bookmark_index);
                        let _ = store.save();
                        if let Ok(json) = serde_json::to_string(&store.folders) {
                            let _ = sidebar.evaluate_script(&format!("renderBookmarks({json})"));
                        }
                    }
                }
            }
            Event::UserEvent(UserEvent::DeleteFolder(index)) => {
                if index < store.folders.len() {
                    store.folders.remove(index);
                    let _ = store.save();
                    if let Ok(json) = serde_json::to_string(&store.folders) {
                        let _ = sidebar.evaluate_script(&format!("renderBookmarks({json})"));
                    }
                }
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn roundtrip_save_load() {
        let dir = env::temp_dir().join("bookmarks-browser-test");
        let path = dir.join("bookmarks.json");

        // Clean up from any previous run
        let _ = fs::remove_dir_all(&dir);

        let store = default_store();
        store.save_to(&path).expect("save should succeed");

        let loaded = BookmarkStore::load_from(&path);
        assert_eq!(store, loaded);

        // Clean up
        let _ = fs::remove_dir_all(&dir);
    }
}
