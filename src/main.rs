use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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

fn main() {
    let store = BookmarkStore::load();
    if let Err(e) = store.save() {
        eprintln!("Warning: could not save bookmarks: {e}");
    }

    let event_loop = EventLoop::new();

    let _window = WindowBuilder::new()
        .with_title("Bookmarks Browser")
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)
        .expect("Failed to create window");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
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
