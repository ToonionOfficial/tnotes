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
                    "<h1>Welcome to TNotes</h1><p>TNotes is a fast, minimal, native note-taking app with instant sync across desktop, mobile, and web.</p><blockquote><p>Simplicity is prerequisite for reliability.</p></blockquote><ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>HTML parser & serializer</p></div></li><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p>GPUI Desktop UI</p></div></li></ul>",
                ),
                (
                    "Architecture & Document AST",
                    "<h2>Architecture & Document AST</h2><p>The document model in <code>tnotes-document</code> provides editor-independent AST representation for rich text.</p><pre><code class=\"language-rust\">let doc = Document::from_html(html)?;\nlet out = doc.to_html();</code></pre>",
                ),
                (
                    "Bevy Engine Exploration",
                    "<h2>Bevy Engine Exploration</h2><p>ECS architecture notes and rendering pipeline ideas.</p>",
                ),
                (
                    "Neovim Shortcuts",
                    "<h2>Neovim Shortcuts</h2><p>Keybindings reference for custom workflow.</p>",
                ),
                (
                    "Release Checklist",
                    "<h2>Release Checklist</h2><ul data-type=\"taskList\"><li data-type=\"taskItem\" data-checked=\"true\"><label><input type=\"checkbox\" checked=\"checked\"><span></span></label><div><p>Verify SQLite migrations</p></div></li><li data-type=\"taskItem\" data-checked=\"false\"><label><input type=\"checkbox\"><span></span></label><div><p>Publish desktop build</p></div></li></ul>",
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

            let topics = [
                ("CRDT Sync Engine Protocol", "<h2>CRDT Sync Engine Protocol</h2><p>Real-time replication protocol over WebSockets with hybrid logical clocks.</p><ul><li>State vector exchange</li><li>Delta-based mutation envelopes</li><li>Conflict-free resolution</li></ul>"),
                ("High-Performance GPUI Shaders", "<h2>High-Performance GPUI Shaders</h2><p>Direct GPU command buffer recording using Blade backend on Vulkan/Metal.</p><pre><code class=\"language-rust\">fn render_quad(encoder: &mut CommandEncoder) {\n    encoder.draw(0..6, 0..1);\n}</code></pre>"),
                ("Database Schema & FTS5 Indexing", "<h2>Database Schema & FTS5 Indexing</h2><p>SQLite Full-Text Search 5 integration for sub-millisecond query performance over 100,000 notes.</p><blockquote><p>FTS5 uses prefix match tokens with BM25 ranking.</p></blockquote>"),
                ("TipTap HTML AST Serialization", "<h2>TipTap HTML AST Serialization</h2><p>Bidirectional lossless serializer matching standard HTML5 specs.</p>"),
                ("Mobile Offline Tombstone Reconciliation", "<h2>Mobile Offline Tombstone Reconciliation</h2><p>Handling soft deletes with timestamp-based tombstone garbage collection after 30 days.</p>"),
                ("Zero-Copy Memory Allocations in Rust", "<h2>Zero-Copy Memory Allocations in Rust</h2><p>Using <code>Arc&lt;str&gt;</code> and <code>SharedString</code> to eliminate heap allocations across render frames.</p>"),
                ("Asynchronous Background Workers", "<h2>Asynchronous Background Workers</h2><p>Running non-blocking SQLite writes using Tokio and calloop dispatch loops.</p>"),
                ("Design Tokens & Typography Scales", "<h2>Design Tokens & Typography Scales</h2><p>Dark theme palette harmonized across React Native and GPUI Desktop.</p>"),
            ];

            for i in 6..=60 {
                let (topic_title, topic_body) = topics[i % topics.len()];
                let note = Note::new(
                    format!("{topic_title} #{i}"),
                    format!("{topic_body}<p>Note ID: <code>n-{i}</code></p>"),
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
