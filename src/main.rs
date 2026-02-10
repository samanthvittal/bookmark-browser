use std::fs;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use tao::{
    dpi::LogicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    keyboard::{Key, ModifiersState},
    window::WindowBuilder,
};
use wry::dpi::{LogicalPosition, LogicalSize as WryLogicalSize};
use wry::{Rect, WebViewBuilder};

#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;

const SIDEBAR_WIDTH: f64 = 280.0;
const STRIP_WIDTH: f64 = 28.0;

#[derive(Debug)]
enum UserEvent {
    Navigate(String),
    ToggleFolder(usize),
    ToggleSidebar,
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
    SaveSettings {
        github_token: String,
        github_repo: String,
    },
    PushToGitHub,
    PullFromGitHub,
    AutoSync,
    SyncStatus(String),
    PushComplete(Option<String>),
    PullComplete(BookmarkStore, String),
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

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"))
        .join("bookmarks-browser")
}

fn config_path() -> PathBuf {
    config_dir().join("bookmarks.json")
}

fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Settings {
    #[serde(default)]
    sidebar_collapsed: bool,
    #[serde(default)]
    github_token: String,
    #[serde(default)]
    github_repo: String,
    /// Legacy field — read from old settings files, never written back
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    github_gist_id: String,
}

impl Settings {
    fn load() -> Settings {
        Self::load_from(&settings_path())
    }

    fn load_from(path: &Path) -> Settings {
        fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_to(&settings_path())
    }

    fn save_to(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

impl BookmarkStore {
    fn load() -> BookmarkStore {
        Self::load_from(&config_path())
    }

    fn load_from(path: &Path) -> BookmarkStore {
        fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_else(default_store)
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_to(&config_path())
    }

    fn save_to(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

fn sidebar_html(store: &BookmarkStore, settings: &Settings) -> String {
    let folders_json = serde_json::to_string(&store.folders).unwrap_or_else(|_| "[]".to_string());
    let has_token = !settings.github_token.is_empty();
    let repo = settings.github_repo.replace('\'', "\\'");
    let collapsed_class = if settings.sidebar_collapsed {
        " collapsed"
    } else {
        ""
    };
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
  body {{
    background: var(--mantle);
    color: var(--text);
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 14px;
    height: 100vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--surface0);
  }}
  #tree {{
    padding: 8px 0;
    flex: 1;
    overflow-y: auto;
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
    background: var(--mantle);
    border-top: 1px solid var(--surface0);
    padding: 8px;
    gap: 8px;
    flex-shrink: 0;
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
    overflow: hidden;
    text-overflow: ellipsis;
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
  .sync-status {{
    display: none;
    padding: 6px 8px;
    font-size: 11px;
    border-top: 1px solid var(--surface0);
    flex-shrink: 0;
    text-align: center;
    cursor: pointer;
  }}
  .sync-status.active {{
    display: block;
  }}
  .sync-status.status-progress {{
    color: var(--subtext);
  }}
  .sync-status.status-success {{
    color: var(--green);
  }}
  .sync-status.status-error {{
    color: var(--red);
  }}
  /* Collapsed sidebar mode */
  #expandBtn {{
    display: none;
    background: none;
    border: none;
    color: var(--text);
    font-size: 16px;
    cursor: pointer;
    width: 100%;
    flex: 1;
  }}
  #expandBtn:hover {{
    background: var(--surface0);
    color: var(--accent);
  }}
  body.collapsed #tree,
  body.collapsed .bottom-bar,
  body.collapsed .sync-status,
  body.collapsed .modal-overlay {{
    display: none !important;
  }}
  body.collapsed #expandBtn {{
    display: block;
  }}
  body.collapsed {{
    border-right: 1px solid var(--surface0);
  }}
</style>
</head>
<body class="{collapsed_class}">
<button id="expandBtn" onclick="expandSidebar()" title="Expand sidebar (Ctrl+B)">&raquo;</button>
<div id="tree"></div>
<div id="syncStatus" class="sync-status"></div>
<div class="bottom-bar" style="flex-wrap:wrap;">
  <button class="bar-btn" onclick="pushToGitHub()" title="Push to GitHub (Ctrl+U)">&#x2191; Push</button>
  <button class="bar-btn" onclick="pullFromGitHub()" title="Pull from GitHub (Ctrl+I)">&#x2193; Pull</button>
  <button class="bar-btn" onclick="showAddFolderModal()">+ Folder</button>
  <button class="bar-btn" onclick="showSettingsModal()" title="Settings">&#x2699; Settings</button>
  <button class="bar-btn" onclick="showHelpModal()">? Help</button>
  <button class="bar-btn" onclick="collapseSidebar()" title="Collapse sidebar (Ctrl+B)">&laquo;</button>
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
      <tr><td class="help-key">Ctrl+G</td><td>Add folder</td></tr>
      <tr><td class="help-key">F5</td><td>Reload page</td></tr>
      <tr><td class="help-key">Ctrl+[</td><td>Navigate back</td></tr>
      <tr><td class="help-key">Ctrl+]</td><td>Navigate forward</td></tr>
      <tr><td class="help-key">Ctrl+B</td><td>Toggle sidebar</td></tr>
      <tr><td class="help-key">Ctrl+U</td><td>Push to GitHub</td></tr>
      <tr><td class="help-key">Ctrl+I</td><td>Pull from GitHub</td></tr>
      <tr><td class="help-key">Ctrl+Q</td><td>Quit</td></tr>
      <tr><td class="help-key">Escape</td><td>Close dialog</td></tr>
    </table>
    <div class="modal-buttons">
      <button class="btn-primary" onclick="closeModals()" style="flex:1">Close</button>
    </div>
  </div>
</div>

<div id="settingsOverlay" class="modal-overlay">
  <div class="modal">
    <h3>Settings</h3>
    <label for="ghToken">GitHub Personal Access Token</label>
    <input type="password" id="ghToken" placeholder="ghp_...">
    <label for="ghRepo">Repository (owner/repo)</label>
    <input type="text" id="ghRepo" placeholder="username/my-bookmarks">
    <div class="modal-buttons">
      <button class="btn-cancel" onclick="closeModals()">Cancel</button>
      <button class="btn-primary" onclick="submitSaveSettings()">Save</button>
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
    if (folders.length === 0) {{
      alert('Create a folder first before adding bookmarks.');
      return;
    }}
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
    document.getElementById('settingsOverlay').classList.remove('active');
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

  function collapseSidebar() {{
    window.ipc.postMessage(JSON.stringify({{ action: 'toggle_sidebar' }}));
  }}

  function expandSidebar() {{
    window.ipc.postMessage(JSON.stringify({{ action: 'toggle_sidebar' }}));
  }}

  function setSidebarCollapsed(collapsed) {{
    if (collapsed) {{
      document.body.classList.add('collapsed');
    }} else {{
      document.body.classList.remove('collapsed');
    }}
  }}

  let savedHasToken = {has_token};
  let savedRepo = '{repo}';

  function showSettingsModal() {{
    document.getElementById('ghToken').value = '';
    document.getElementById('ghToken').placeholder = savedHasToken ? '(token saved - enter new to change)' : 'ghp_...';
    document.getElementById('ghRepo').value = savedRepo;
    document.getElementById('settingsOverlay').classList.add('active');
    activeModal = 'settings';
    document.getElementById('ghToken').focus();
  }}

  function submitSaveSettings() {{
    const token = document.getElementById('ghToken').value.trim();
    const repo = document.getElementById('ghRepo').value.trim();
    window.ipc.postMessage(JSON.stringify({{
      action: 'save_settings',
      github_token: token,
      github_repo: repo
    }}));
    if (token) savedHasToken = true;
    savedRepo = repo;
    closeModals();
  }}

  function updateSettings(hasToken, repo) {{
    savedHasToken = hasToken;
    savedRepo = repo;
  }}

  function pushToGitHub() {{
    window.ipc.postMessage(JSON.stringify({{ action: 'push_to_github' }}));
  }}

  function pullFromGitHub() {{
    window.ipc.postMessage(JSON.stringify({{ action: 'pull_from_github' }}));
  }}

  let syncTimer = null;

  function updateSyncStatus(msg, type) {{
    var el = document.getElementById('syncStatus');
    if (syncTimer) {{ clearTimeout(syncTimer); syncTimer = null; }}
    el.className = 'sync-status';
    if (msg) {{
      el.textContent = msg;
      if (!type) {{
        if (msg.indexOf('successfully') !== -1 || msg.indexOf('Last synced') !== -1) type = 'success';
        else if (msg.indexOf('failed') !== -1 || msg.indexOf('error') !== -1 || msg.indexOf('No token') !== -1 || msg.indexOf('No repo') !== -1 || msg.indexOf('not found') !== -1) type = 'error';
        else type = 'progress';
      }}
      el.classList.add('active', 'status-' + type);
      if (type === 'success') {{
        syncTimer = setTimeout(function() {{ el.classList.remove('active'); }}, 5000);
      }} else if (type === 'error') {{
        syncTimer = setTimeout(function() {{ el.classList.remove('active'); }}, 8000);
      }}
    }} else {{
      el.classList.remove('active');
    }}
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

  document.getElementById('syncStatus').addEventListener('click', function() {{
    updateSyncStatus(null);
  }});

  renderBookmarks(folders);
</script>
</body>
</html>"#,
        folders_json = folders_json,
        has_token = has_token,
        repo = repo
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

fn format_ureq_error(e: ureq::Error) -> String {
    match e {
        ureq::Error::StatusCode(401) => "Invalid or expired GitHub token".to_string(),
        ureq::Error::StatusCode(404) => {
            "Repository or file not found — check owner/repo".to_string()
        }
        ureq::Error::StatusCode(409) => "SHA conflict — pull first, then push again".to_string(),
        ureq::Error::StatusCode(422) => "Validation error — check repo name format".to_string(),
        ureq::Error::StatusCode(code) => format!("GitHub API error (HTTP {code})"),
        ureq::Error::Timeout(_) => "Request timed out — try again".to_string(),
        ureq::Error::HostNotFound => "Could not reach GitHub — check your connection".to_string(),
        ureq::Error::ConnectionFailed => "Connection failed — check your connection".to_string(),
        _ => format!("{e}"),
    }
}

fn get_file_sha(token: &str, repo: &str) -> Result<Option<String>, String> {
    let url = format!("https://api.github.com/repos/{repo}/contents/bookmarks.json");
    let agent = ureq::Agent::new_with_defaults();

    match agent
        .get(&url)
        .header("Authorization", &format!("token {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "bookmarks-browser")
        .call()
    {
        Ok(mut response) => {
            let body = response
                .body_mut()
                .read_to_string()
                .map_err(|e| format!("Failed to read response: {e}"))?;
            let parsed: serde_json::Value = serde_json::from_str(&body)
                .map_err(|_| "Malformed response from GitHub".to_string())?;
            Ok(parsed.get("sha").and_then(|v| v.as_str()).map(String::from))
        }
        Err(ureq::Error::StatusCode(404)) => Ok(None),
        Err(e) => Err(format_ureq_error(e)),
    }
}

fn do_push(
    token: &str,
    repo: &str,
    bookmarks_json: &str,
    sha: Option<&str>,
) -> Result<String, String> {
    let encoded = BASE64.encode(bookmarks_json.as_bytes());

    let sha = match sha {
        Some(s) => Some(s.to_string()),
        None => get_file_sha(token, repo)?,
    };

    let mut payload = serde_json::json!({
        "message": "Update bookmarks",
        "content": encoded,
    });
    if let Some(ref sha_val) = sha {
        payload["sha"] = serde_json::json!(sha_val);
    }

    let url = format!("https://api.github.com/repos/{repo}/contents/bookmarks.json");
    let agent = ureq::Agent::new_with_defaults();

    let mut response = agent
        .put(&url)
        .header("Authorization", &format!("token {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "bookmarks-browser")
        .send_json(&payload)
        .map_err(format_ureq_error)?;

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response: {e}"))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "Malformed response from GitHub".to_string())?;

    parsed
        .get("content")
        .and_then(|c| c.get("sha"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Malformed response from GitHub".to_string())
}

fn do_pull(token: &str, repo: &str) -> Result<(BookmarkStore, String), String> {
    let url = format!("https://api.github.com/repos/{repo}/contents/bookmarks.json");
    let agent = ureq::Agent::new_with_defaults();

    let mut response = agent
        .get(&url)
        .header("Authorization", &format!("token {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "bookmarks-browser")
        .call()
        .map_err(format_ureq_error)?;

    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("Failed to read response: {e}"))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "Malformed response from GitHub".to_string())?;

    let sha = parsed
        .get("sha")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing SHA in response".to_string())?
        .to_string();

    let encoded = parsed
        .get("content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| "bookmarks.json not found in repository".to_string())?;

    // GitHub returns base64 with newlines — strip them before decoding
    let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    let decoded = BASE64
        .decode(&cleaned)
        .map_err(|e| format!("Failed to decode content: {e}"))?;
    let content = String::from_utf8(decoded).map_err(|e| format!("Invalid UTF-8 content: {e}"))?;

    let store = serde_json::from_str::<BookmarkStore>(&content)
        .map_err(|e| format!("Failed to parse bookmarks: {e}"))?;

    Ok((store, sha))
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

    let mut settings = Settings::load();
    let initial_collapsed = settings.sidebar_collapsed;

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
        .with_html(sidebar_html(&store, &settings))
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
                "toggle_sidebar" => {
                    let _ = proxy.send_event(UserEvent::ToggleSidebar);
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
                "push_to_github" => {
                    let _ = proxy.send_event(UserEvent::PushToGitHub);
                }
                "pull_from_github" => {
                    let _ = proxy.send_event(UserEvent::PullFromGitHub);
                }
                "save_settings" => {
                    let token = msg
                        .get("github_token")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();
                    let repo = msg
                        .get("github_repo")
                        .and_then(|g| g.as_str())
                        .unwrap_or("")
                        .to_string();
                    let _ = proxy.send_event(UserEvent::SaveSettings {
                        github_token: token,
                        github_repo: repo,
                    });
                }
                _ => {}
            }
        });

    let content_builder = WebViewBuilder::new()
        .with_html(welcome_html())
        .with_bounds(make_bounds(SIDEBAR_WIDTH, 0.0, w - SIDEBAR_WIDTH, h))
        .with_user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36");

    #[cfg(target_os = "linux")]
    let (sidebar, content, sidebar_gtk_box) = {
        use gtk::prelude::*;

        let vbox = window.default_vbox().expect("Failed to get default vbox");
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        vbox.pack_start(&hbox, true, true, 0);

        let sidebar_width = if initial_collapsed {
            STRIP_WIDTH
        } else {
            SIDEBAR_WIDTH
        };
        let sidebar_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar_box.set_size_request(sidebar_width as i32, -1);
        hbox.pack_start(&sidebar_box, false, false, 0);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        hbox.pack_start(&content_box, true, true, 0);

        hbox.show_all();

        let sidebar = sidebar_builder
            .build_gtk(&sidebar_box)
            .expect("Failed to create sidebar webview");
        let content = content_builder
            .build_gtk(&content_box)
            .expect("Failed to create content webview");

        (sidebar, content, sidebar_box)
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

    let sync_proxy = event_loop.create_proxy();

    let mut modifiers = ModifiersState::empty();
    let mut sidebar_collapsed = initial_collapsed;
    let mut remote_sha: Option<String> = None;
    let mut sync_in_progress = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(new_modifiers),
                ..
            } => {
                modifiers = new_modifiers;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event: ref key_event,
                        ..
                    },
                ..
            } if key_event.state == ElementState::Pressed => {
                let ctrl = modifiers.control_key();
                let key = &key_event.logical_key;

                if ctrl && *key == Key::Character("b") {
                    sidebar_collapsed = !sidebar_collapsed;
                    settings.sidebar_collapsed = sidebar_collapsed;
                    let _ = settings.save();
                    let _ = sidebar
                        .evaluate_script(&format!("setSidebarCollapsed({})", sidebar_collapsed));
                    #[cfg(target_os = "linux")]
                    {
                        use gtk::prelude::*;
                        let new_width = if sidebar_collapsed {
                            STRIP_WIDTH
                        } else {
                            SIDEBAR_WIDTH
                        };
                        sidebar_gtk_box.set_size_request(new_width as i32, -1);
                        sidebar_gtk_box.queue_resize();
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        let scale = window.scale_factor();
                        let inner = window.inner_size();
                        let w = inner.width as f64 / scale;
                        let h = inner.height as f64 / scale;
                        if sidebar_collapsed {
                            let _ = sidebar.set_bounds(make_bounds(0.0, 0.0, 0.0, h));
                            let _ = content.set_bounds(make_bounds(0.0, 0.0, w, h));
                        } else {
                            let _ = sidebar.set_bounds(make_bounds(0.0, 0.0, SIDEBAR_WIDTH, h));
                            let _ = content.set_bounds(make_bounds(
                                SIDEBAR_WIDTH,
                                0.0,
                                w - SIDEBAR_WIDTH,
                                h,
                            ));
                        }
                    }
                } else if ctrl && *key == Key::Character("u") {
                    let _ = sync_proxy.send_event(UserEvent::PushToGitHub);
                } else if ctrl && *key == Key::Character("i") {
                    let _ = sync_proxy.send_event(UserEvent::PullFromGitHub);
                } else if ctrl && *key == Key::Character("g") {
                    let _ = sidebar.evaluate_script("showAddFolderModal()");
                } else if ctrl && *key == Key::Character("n") {
                    let _ = sidebar.evaluate_script("showAddBookmarkModal()");
                } else if *key == Key::F1 || (ctrl && *key == Key::Character("/")) {
                    let _ = sidebar.evaluate_script("showHelpModal()");
                } else if ctrl && *key == Key::Character("q") {
                    *control_flow = ControlFlow::Exit;
                } else if *key == Key::F5 {
                    let _ = content.evaluate_script("location.reload()");
                } else if ctrl && *key == Key::Character("[") {
                    let _ = content.evaluate_script("history.back()");
                } else if ctrl && *key == Key::Character("]") {
                    let _ = content.evaluate_script("history.forward()");
                } else if *key == Key::Escape {
                    let _ = sidebar.evaluate_script("closeModals()");
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                let scale = window.scale_factor();
                let w = new_size.width as f64 / scale;
                let h = new_size.height as f64 / scale;

                if sidebar_collapsed {
                    let _ = sidebar.set_bounds(make_bounds(0.0, 0.0, 0.0, h));
                    let _ = content.set_bounds(make_bounds(0.0, 0.0, w, h));
                } else {
                    let _ = sidebar.set_bounds(make_bounds(0.0, 0.0, SIDEBAR_WIDTH, h));
                    let _ =
                        content.set_bounds(make_bounds(SIDEBAR_WIDTH, 0.0, w - SIDEBAR_WIDTH, h));
                }
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
            Event::UserEvent(UserEvent::ToggleSidebar) => {
                sidebar_collapsed = !sidebar_collapsed;
                settings.sidebar_collapsed = sidebar_collapsed;
                let _ = settings.save();
                let _ =
                    sidebar.evaluate_script(&format!("setSidebarCollapsed({})", sidebar_collapsed));
                #[cfg(target_os = "linux")]
                {
                    use gtk::prelude::*;
                    let new_width = if sidebar_collapsed {
                        STRIP_WIDTH
                    } else {
                        SIDEBAR_WIDTH
                    };
                    sidebar_gtk_box.set_size_request(new_width as i32, -1);
                    sidebar_gtk_box.queue_resize();
                }
                #[cfg(not(target_os = "linux"))]
                {
                    let scale = window.scale_factor();
                    let inner = window.inner_size();
                    let w = inner.width as f64 / scale;
                    let h = inner.height as f64 / scale;
                    if sidebar_collapsed {
                        let _ = sidebar.set_bounds(make_bounds(0.0, 0.0, 0.0, h));
                        let _ = content.set_bounds(make_bounds(0.0, 0.0, w, h));
                    } else {
                        let _ = sidebar.set_bounds(make_bounds(0.0, 0.0, SIDEBAR_WIDTH, h));
                        let _ = content.set_bounds(make_bounds(
                            SIDEBAR_WIDTH,
                            0.0,
                            w - SIDEBAR_WIDTH,
                            h,
                        ));
                    }
                }
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
                let _ = sync_proxy.send_event(UserEvent::AutoSync);
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
                    let _ = sync_proxy.send_event(UserEvent::AutoSync);
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
                        let _ = sync_proxy.send_event(UserEvent::AutoSync);
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
                    let _ = sync_proxy.send_event(UserEvent::AutoSync);
                }
            }
            Event::UserEvent(UserEvent::SaveSettings {
                github_token,
                github_repo,
            }) => {
                if !github_token.is_empty() {
                    settings.github_token = github_token;
                }
                if settings.github_repo != github_repo {
                    remote_sha = None;
                }
                settings.github_repo = github_repo;
                let _ = settings.save();
                let has_token = !settings.github_token.is_empty();
                let repo = settings.github_repo.replace('\'', "\\'");
                let _ = sidebar.evaluate_script(&format!("updateSettings({has_token}, '{repo}')"));
            }
            Event::UserEvent(UserEvent::PushToGitHub) => {
                if settings.github_token.is_empty() {
                    let _ = sidebar
                        .evaluate_script("updateSyncStatus('No token configured — open Settings')");
                    return;
                }
                if settings.github_repo.is_empty() {
                    let _ = sidebar
                        .evaluate_script("updateSyncStatus('No repo configured — open Settings')");
                    return;
                }
                if sync_in_progress {
                    return;
                }
                sync_in_progress = true;
                let token = settings.github_token.clone();
                let repo = settings.github_repo.clone();
                let sha = remote_sha.clone();
                let bookmarks_json = serde_json::to_string_pretty(&store).unwrap_or_default();
                let proxy = sync_proxy.clone();
                let _ = sidebar.evaluate_script("updateSyncStatus('Pushing...')");
                std::thread::spawn(move || {
                    match do_push(&token, &repo, &bookmarks_json, sha.as_deref()) {
                        Ok(new_sha) => {
                            let _ = proxy.send_event(UserEvent::PushComplete(Some(new_sha)));
                        }
                        Err(e) => {
                            let _ = proxy
                                .send_event(UserEvent::SyncStatus(format!("Push failed: {e}")));
                        }
                    }
                });
            }
            Event::UserEvent(UserEvent::PullFromGitHub) => {
                if settings.github_token.is_empty() {
                    let _ = sidebar
                        .evaluate_script("updateSyncStatus('No token configured — open Settings')");
                    return;
                }
                if settings.github_repo.is_empty() {
                    let _ = sidebar
                        .evaluate_script("updateSyncStatus('No repo configured — open Settings')");
                    return;
                }
                if sync_in_progress {
                    return;
                }
                sync_in_progress = true;
                let token = settings.github_token.clone();
                let repo = settings.github_repo.clone();
                let proxy = sync_proxy.clone();
                let _ = sidebar.evaluate_script("updateSyncStatus('Pulling...')");
                std::thread::spawn(move || match do_pull(&token, &repo) {
                    Ok((new_store, sha)) => {
                        let _ = proxy.send_event(UserEvent::PullComplete(new_store, sha));
                    }
                    Err(e) => {
                        let _ =
                            proxy.send_event(UserEvent::SyncStatus(format!("Pull failed: {e}")));
                    }
                });
            }
            Event::UserEvent(UserEvent::SyncStatus(msg)) => {
                sync_in_progress = false;
                let escaped = msg.replace('\\', "\\\\").replace('\'', "\\'");
                let _ = sidebar.evaluate_script(&format!("updateSyncStatus('{escaped}')"));
            }
            Event::UserEvent(UserEvent::PushComplete(new_sha)) => {
                sync_in_progress = false;
                remote_sha = new_sha;
                let _ = sidebar.evaluate_script("updateSyncStatus('Pushed successfully')");
            }
            Event::UserEvent(UserEvent::PullComplete(new_store, sha)) => {
                sync_in_progress = false;
                remote_sha = Some(sha);
                store = new_store;
                let _ = store.save();
                if let Ok(json) = serde_json::to_string(&store.folders) {
                    let _ = sidebar.evaluate_script(&format!("renderBookmarks({json})"));
                }
                let _ = sidebar.evaluate_script("updateSyncStatus('Pulled successfully')");
            }
            Event::UserEvent(UserEvent::AutoSync) => {
                if sync_in_progress
                    || settings.github_token.is_empty()
                    || settings.github_repo.is_empty()
                {
                    return;
                }
                sync_in_progress = true;
                let token = settings.github_token.clone();
                let repo = settings.github_repo.clone();
                let sha = remote_sha.clone();
                let bookmarks_json = serde_json::to_string_pretty(&store).unwrap_or_default();
                let proxy = sync_proxy.clone();
                std::thread::spawn(move || {
                    match do_push(&token, &repo, &bookmarks_json, sha.as_deref()) {
                        Ok(new_sha) => {
                            let _ = proxy.send_event(UserEvent::PushComplete(Some(new_sha)));
                        }
                        Err(e) => {
                            let _ = proxy
                                .send_event(UserEvent::SyncStatus(format!("Sync failed: {e}")));
                        }
                    }
                });
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

    #[test]
    fn settings_roundtrip() {
        let dir = env::temp_dir().join("bookmarks-browser-settings-test");
        let path = dir.join("settings.json");

        let _ = fs::remove_dir_all(&dir);

        let settings = Settings {
            sidebar_collapsed: true,
            github_token: "test-token".to_string(),
            github_repo: "user/bookmarks".to_string(),
            ..Default::default()
        };
        settings.save_to(&path).expect("save should succeed");

        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.sidebar_collapsed, true);
        assert_eq!(loaded.github_token, "test-token");
        assert_eq!(loaded.github_repo, "user/bookmarks");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_default_on_missing_file() {
        let path = env::temp_dir().join("nonexistent-settings-dir/settings.json");
        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.sidebar_collapsed, false);
        assert!(loaded.github_token.is_empty());
        assert!(loaded.github_repo.is_empty());
    }

    #[test]
    fn settings_migration_from_gist() {
        let dir = env::temp_dir().join("bookmarks-browser-migration-test");
        let path = dir.join("settings.json");

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create dir");

        // Write an old-format settings file with github_gist_id
        let old_json =
            r#"{"sidebar_collapsed":false,"github_token":"tok","github_gist_id":"abc123"}"#;
        fs::write(&path, old_json).expect("write old settings");

        // Load should succeed and populate the legacy field
        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.github_token, "tok");
        assert_eq!(loaded.github_gist_id, "abc123");
        assert!(loaded.github_repo.is_empty());

        // Re-save should drop github_gist_id
        loaded.save_to(&path).expect("save should succeed");
        let raw = fs::read_to_string(&path).expect("read saved");
        assert!(!raw.contains("github_gist_id"));

        let _ = fs::remove_dir_all(&dir);
    }
}
