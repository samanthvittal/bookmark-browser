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
    --text: #cdd6f4;
    --subtext: #a6adc8;
    --accent: #89b4fa;
  }}
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    background: var(--mantle);
    color: var(--text);
    font-family: system-ui, -apple-system, sans-serif;
    font-size: 14px;
    height: 100vh;
    overflow-y: auto;
    border-right: 1px solid var(--surface0);
  }}
  #tree {{
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
  .bookmark {{
    display: block;
    padding: 6px 12px 6px 32px;
    color: var(--text);
    text-decoration: none;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }}
  .bookmark:hover {{
    background: var(--surface0);
  }}
  .bookmark.active {{
    background: var(--surface0);
    color: var(--accent);
  }}
</style>
</head>
<body>
<div id="tree"></div>
<script>
  let folders = {folders_json};
  let activeUrl = null;

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
      name.textContent = folder.name;
      header.appendChild(arrow);
      header.appendChild(name);
      tree.appendChild(header);

      if (folder.expanded) {{
        folder.bookmarks.forEach(function(bm) {{
          const link = document.createElement('div');
          link.className = 'bookmark' + (bm.url === activeUrl ? ' active' : '');
          link.textContent = bm.name;
          link.title = bm.url;
          link.onclick = function() {{ navigate(bm.url); }};
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
    --accent: #89b4fa;
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
    let store = BookmarkStore::load();
    if let Err(e) = store.save() {
        eprintln!("Warning: could not save bookmarks: {e}");
    }

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

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
        .with_bounds(make_bounds(0.0, 0.0, SIDEBAR_WIDTH, h));

    let content_builder = WebViewBuilder::new()
        .with_html(welcome_html())
        .with_bounds(make_bounds(SIDEBAR_WIDTH, 0.0, w - SIDEBAR_WIDTH, h));

    #[cfg(target_os = "linux")]
    let (_sidebar, _content) = {
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
    let (_sidebar, _content) = {
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

                let _ = _sidebar.set_bounds(make_bounds(0.0, 0.0, SIDEBAR_WIDTH, h));
                let _ = _content.set_bounds(make_bounds(SIDEBAR_WIDTH, 0.0, w - SIDEBAR_WIDTH, h));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(_) => {}
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
