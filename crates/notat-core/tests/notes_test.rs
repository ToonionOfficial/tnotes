use notat_core::{
    db::{
        folders::insert_folder,
        migrations::open_in_memory,
        notes::{
            delete_note_permanently, delete_trashed_older_than, get_note_by_id,
            insert_note, list_active_notes, list_notes_by_folder, list_trashed_notes,
            search_notes, upsert_note,
        },
    },
    models::{folder::Folder, note::Note},
};

#[test]
fn test_note_crud_and_mutations() {
    let conn = open_in_memory().unwrap();

    // 0. Seed a folder
    let folder = Folder::new("Projects", None, None, 0, "dev_1");
    insert_folder(&conn, &folder).unwrap();

    // 1. Create a note
    let mut note = Note::new("Initial Title", "Initial Body", None, "dev_1");
    let initial_checksum = note.checksum.clone();
    assert_eq!(note.version, 1);
    assert!(!initial_checksum.is_empty());

    insert_note(&conn, &note).unwrap();

    // 2. Fetch note
    let fetched = get_note_by_id(&conn, &note.id).unwrap().unwrap();
    assert_eq!(fetched.title, "Initial Title");
    assert_eq!(fetched.body, "Initial Body");
    assert_eq!(fetched.version, 1);

    // 3. Update note content
    note.update("Updated Title", "Updated Body Content", Some(folder.id.clone()), "dev_1");
    assert_eq!(note.version, 2);
    assert_ne!(note.checksum, initial_checksum);

    upsert_note(&conn, &note).unwrap();

    let fetched_updated = get_note_by_id(&conn, &note.id).unwrap().unwrap();
    assert_eq!(fetched_updated.title, "Updated Title");
    assert_eq!(fetched_updated.body, "Updated Body Content");
    assert_eq!(fetched_updated.folder_id, Some(folder.id.clone()));
    assert_eq!(fetched_updated.version, 2);

    // 4. Pin note
    note.set_pinned(true, "dev_1");
    upsert_note(&conn, &note).unwrap();
    let fetched_pinned = get_note_by_id(&conn, &note.id).unwrap().unwrap();
    assert!(fetched_pinned.pinned);

    // 5. Trash and restore
    note.trash("dev_1");
    assert!(note.trashed);
    assert!(note.deleted_at.is_some());
    upsert_note(&conn, &note).unwrap();

    assert!(list_active_notes(&conn).unwrap().is_empty());
    assert_eq!(list_trashed_notes(&conn).unwrap().len(), 1);

    note.restore("dev_1");
    assert!(!note.trashed);
    assert!(note.deleted_at.is_none());
    upsert_note(&conn, &note).unwrap();

    assert_eq!(list_active_notes(&conn).unwrap().len(), 1);
    assert!(list_trashed_notes(&conn).unwrap().is_empty());

    // 6. Hard delete
    delete_note_permanently(&conn, &note.id).unwrap();
    assert!(get_note_by_id(&conn, &note.id).unwrap().is_none());
}

#[test]
fn test_list_notes_by_folder() {
    let conn = open_in_memory().unwrap();

    let folder_a = Folder::new("Folder A", None, None, 0, "dev_1");
    insert_folder(&conn, &folder_a).unwrap();

    let root_note = Note::new("Root Note", "No folder", None, "dev_1");
    let folder_note = Note::new("Folder Note", "In folder A", Some(folder_a.id.clone()), "dev_1");

    insert_note(&conn, &root_note).unwrap();
    insert_note(&conn, &folder_note).unwrap();

    // Query root notes
    let root_notes = list_notes_by_folder(&conn, None).unwrap();
    assert_eq!(root_notes.len(), 1);
    assert_eq!(root_notes[0].id, root_note.id);

    // Query folder notes
    let folder_notes = list_notes_by_folder(&conn, Some(&folder_a.id)).unwrap();
    assert_eq!(folder_notes.len(), 1);
    assert_eq!(folder_notes[0].id, folder_note.id);
}

#[test]
fn test_delete_trashed_older_than() {
    let conn = open_in_memory().unwrap();

    let mut note = Note::new("Old Trashed Note", "Body", None, "dev_1");
    note.trash("dev_1");
    // Set deleted_at in the past (timestamp = 1000)
    note.deleted_at = Some(1000);
    insert_note(&conn, &note).unwrap();

    // Purge threshold is 5000 (note deleted at 1000 should be purged)
    let purged = delete_trashed_older_than(&conn, 5000).unwrap();
    assert_eq!(purged, 1);

    assert!(get_note_by_id(&conn, &note.id).unwrap().is_none());
}

#[test]
fn test_fts5_full_text_search() {
    let conn = open_in_memory().unwrap();

    let note_1 = Note::new(
        "Rust Architecture",
        "Exploring GPUI framework for fast desktop rendering with Metal and Vulkan.",
        None,
        "dev_1",
    );
    let note_2 = Note::new(
        "Pasta Recipe",
        "Classic Italian carbonara with guanciale, pecorino cheese, and black pepper.",
        None,
        "dev_1",
    );
    let note_3 = Note::new(
        "Weekly Standup",
        "Discussed Rust backend sync performance and React Native mobile client.",
        None,
        "dev_1",
    );

    insert_note(&conn, &note_1).unwrap();
    insert_note(&conn, &note_2).unwrap();
    insert_note(&conn, &note_3).unwrap();

    // Search for "GPUI" (matches note 1)
    let results = search_notes(&conn, "GPUI").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, note_1.id);

    // Search for "carbonara" (matches note 2)
    let results = search_notes(&conn, "carbonara").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, note_2.id);

    // Search for "Rust" (matches note 1 and note 3)
    let results = search_notes(&conn, "Rust").unwrap();
    assert_eq!(results.len(), 2);

    // Test FTS update trigger: update note 2 from pasta to pizza
    let mut updated_note_2 = note_2.clone();
    updated_note_2.update("Pizza Recipe", "Neapolitan dough with san marzano tomatoes.", None, "dev_1");
    upsert_note(&conn, &updated_note_2).unwrap();

    assert!(search_notes(&conn, "carbonara").unwrap().is_empty());
    let pizza_results = search_notes(&conn, "pizza").unwrap();
    assert_eq!(pizza_results.len(), 1);
    assert_eq!(pizza_results[0].id, note_2.id);

    // Test FTS delete: permanently delete note 1, ensure FTS no longer returns it
    delete_note_permanently(&conn, &note_1.id).unwrap();
    assert!(search_notes(&conn, "Vulkan").unwrap().is_empty());
}
