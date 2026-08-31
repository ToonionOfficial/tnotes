use crate::db::AppDb;
use crate::keymap;
use crate::state::{AppState, AuthStore, FolderStore, NoteStore, SyncStore};
use crate::views::TNotesWorkspace;
use gpui::*;

pub fn run_app() {
    // 1. Initialize local SQLite database & persistence
    let db = AppDb::init().unwrap_or_else(|err| {
        eprintln!("Error initializing local SQLite database: {err}. Falling back to in-memory mode.");
        AppDb::in_memory().expect("In-memory SQLite initialization failed")
    });

    let platform = gpui_platform::current_platform(false);
    Application::with_platform(platform).run(move |cx: &mut App| {
        // 2. Global Shortcuts & Keybindings from Keymap System
        keymap::init_keymap(cx);

        // 3. Instantiate Reactive Stores
        let note_store = cx.new(|_cx| NoteStore::new(db.clone()));
        let folder_store = cx.new(|_cx| FolderStore::new(db.clone()));
        let auth_store = cx.new(|_cx| AuthStore::new(db.default_user_id.clone(), db.device_id.clone()));
        let sync_store = cx.new(|_cx| SyncStore::new());

        // 4. Instantiate Global AppState Coordinator
        let app_state = cx.new(|_cx| {
            AppState::new(note_store, folder_store, auth_store, sync_store)
        });

        // 5. Open Main Window
        open_main_window(app_state, cx);
    });
}

fn open_main_window(app_state: Entity<AppState>, cx: &mut App) {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(1040.0), px(680.0)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("TNotes".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        focus: true,
        inactive_frame_interval: None,
        ..Default::default()
    };

    cx.open_window(options, |window, cx| {
        let workspace = cx.new(|cx| TNotesWorkspace::new(app_state, cx));
        let focus_handle = workspace.read(cx).focus_handle.clone();
        window.focus(&focus_handle, cx);
        workspace
    })
    .unwrap();
}
