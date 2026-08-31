use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tnotes_core::db::{folders, migrations, notes, users};
use tnotes_core::models::folder::Folder;
use tnotes_core::models::note::Note;
use tnotes_core::models::user::User;
use tnotes_core::rusqlite::Result;
use tnotes_core::{Connection, Ulid};

#[derive(Clone)]
pub struct AppDb {
    conn: Arc<Mutex<Connection>>,
    pub default_user_id: String,
    pub device_id: String,
}

impl AppDb {
    pub fn init() -> Result<Self> {
        let db_path = default_db_path();
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = match migrations::open_connection(&db_path) {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!(
                    "Failed to open DB at {:?}: {err}. Falling back to in-memory DB.",
                    db_path
                );
                migrations::open_in_memory()?
            }
        };

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            default_user_id: "local_user".to_string(),
            device_id: Ulid::generate().to_string(),
        };

        db.seed_defaults_if_needed()?;
        Ok(db)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = migrations::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            default_user_id: "local_user".to_string(),
            device_id: Ulid::generate().to_string(),
        };
        db.seed_defaults_if_needed()?;
        Ok(db)
    }

    fn seed_defaults_if_needed(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if users::get_user_by_id(&conn, &self.default_user_id)?.is_none() {
            let mut user = User::new("local", "");
            user.id = self.default_user_id.clone();
            let _ = users::create_user(&conn, &user);
        }

        let existing_folders = folders::list_all_folders(&conn, &self.default_user_id)?;
        if existing_folders.is_empty() {
            let initial_folders = ["Work", "Personal", "Research", "Studio"];
            for (idx, name) in initial_folders.iter().enumerate() {
                let folder = Folder::new(
                    *name,
                    None,
                    None,
                    idx as i32,
                    &self.device_id,
                    &self.default_user_id,
                );
                let _ = folders::insert_folder(&conn, &folder);
            }
        }

        let existing_notes = notes::list_active_notes(&conn, &self.default_user_id)?;
        if existing_notes.is_empty() {
            let initial_notes = [
                (
                    "Welcome to TNotes",
                    "<h1>Welcome to TNotes</h1><p>TNotes is a fast, minimal, native note-taking app with instant sync across desktop, mobile, and web.</p><blockquote><p>Simplicity is prerequisite for reliability.</p></blockquote><ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>Local SQLite persistence with instant search</p></div></li><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>Direct GPU-accelerated rendering</p></div></li><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p>Pair mobile and web clients</p></div></li></ul>",
                ),
                (
                    "Explore Keyboard Shortcuts",
                    "<h2>Explore Keyboard Shortcuts</h2><p>TNotes is built for keyboard-driven workflows with fully customizable keybindings.</p><ul><li><strong>Ctrl+P</strong> / <strong>Cmd+P</strong> — Open Command Palette to search notes and run actions</li><li><strong>Ctrl+N</strong> / <strong>Cmd+N</strong> — Instantly create a new note</li><li><strong>Ctrl+,</strong> / <strong>Cmd+,</strong> — Open Settings and Preferences screen</li><li><strong>F3</strong> — Toggle live Performance and FPS monitor HUD</li></ul><blockquote><p>Customize your keybindings anytime in <code>~/.config/tnotes/keymap.json</code>.</p></blockquote>",
                ),
                (
                    "Rich Document AST & Formatting",
                    "<h2>Rich Document AST & Formatting</h2><p>All notes in TNotes are backed by an editor-independent document AST supporting headings, quotes, code blocks, and task lists.</p><pre><code class=\"language-rust\">let doc = Document::from_html(html)?;\nlet out = doc.to_html();</code></pre><p>Enjoy distraction-free, high-performance note-taking!</p>",
                ),
            ];

            for (title, body) in initial_notes {
                let note = Note::new(
                    title,
                    body,
                    None,
                    &self.device_id,
                    &self.default_user_id,
                );
                let _ = notes::insert_note(&conn, &note);
            }
        }

        Ok(())
    }

    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let conn = self.conn.lock().unwrap();
        notes::list_active_notes(&conn, &self.default_user_id)
    }

    #[allow(dead_code)]
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let conn = self.conn.lock().unwrap();
        notes::search_notes(&conn, &self.default_user_id, query)
    }

    pub fn insert_note(&self, note: &Note) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        notes::insert_note(&conn, note)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_note(&self, note: &Note) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        notes::upsert_note(&conn, note)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_note(&self, note_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        notes::delete_note_permanently(&conn, note_id)
    }

    pub fn list_folders(&self) -> Result<Vec<Folder>> {
        let conn = self.conn.lock().unwrap();
        folders::list_all_folders(&conn, &self.default_user_id)
    }

    #[allow(dead_code)]
    pub fn insert_folder(&self, folder: &Folder) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        folders::insert_folder(&conn, folder)?;
        Ok(())
    }
}

fn default_db_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/tnotes/tnotes.db")
    } else {
        PathBuf::from("tnotes.db")
    }
}
