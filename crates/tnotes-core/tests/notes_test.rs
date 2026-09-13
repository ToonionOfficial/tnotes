use tnotes_core::{
    db::{
        folders::insert_folder,
        migrations::open_in_memory,
        notes::{
            delete_note_permanently, delete_trashed_older_than, get_note_by_id, insert_note,
            list_active_notes, list_notes_by_folder, list_trashed_notes, search_notes, upsert_note,
        },
        users::create_user,
    },
    models::{folder::Folder, note::Note, user::User},
};

#[test]
fn test_note_crud_and_mutations() {
    let conn = open_in_memory().unwrap();
    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let folder = Folder::new("Projects", None, None, 0, "dev_1", &user.id);
    insert_folder(&conn, &folder).unwrap();

    let mut note = Note::new(
        "Initial Title",
        "Initial Body",
        "Initial Searchable Text",
        None,
        "dev_1",
        &user.id,
    );
    let initial_checksum = note.checksum.clone();
    assert_eq!(note.version, 1);
    assert_eq!(note.user_id, user.id);
    assert_eq!(note.searchable_text, "Initial Searchable Text");
    assert!(!initial_checksum.is_empty());

    insert_note(&conn, &note).unwrap();

    let fetched = get_note_by_id(&conn, &note.id).unwrap().unwrap();
    assert_eq!(fetched.title, "Initial Title");
    assert_eq!(fetched.body, "Initial Body");
    assert_eq!(fetched.searchable_text, "Initial Searchable Text");
    assert_eq!(fetched.user_id, user.id);
    assert_eq!(fetched.version, 1);

    note.update(
        "Updated Title",
        "Updated Body Content",
        "Updated Searchable Text Content",
        Some(folder.id.clone()),
        "dev_1",
    );
    assert_eq!(note.version, 2);
    assert_eq!(note.searchable_text, "Updated Searchable Text Content");
    assert_ne!(note.checksum, initial_checksum);

    upsert_note(&conn, &note).unwrap();

    let fetched_updated = get_note_by_id(&conn, &note.id).unwrap().unwrap();
    assert_eq!(fetched_updated.title, "Updated Title");
    assert_eq!(fetched_updated.body, "Updated Body Content");
    assert_eq!(
        fetched_updated.searchable_text,
        "Updated Searchable Text Content"
    );
    assert_eq!(fetched_updated.folder_id, Some(folder.id.clone()));
    assert_eq!(fetched_updated.version, 2);

    note.set_pinned(true, "dev_1");
    upsert_note(&conn, &note).unwrap();
    let fetched_pinned = get_note_by_id(&conn, &note.id).unwrap().unwrap();
    assert!(fetched_pinned.pinned);

    note.trash("dev_1");
    assert!(note.trashed);
    assert!(note.deleted_at.is_some());
    upsert_note(&conn, &note).unwrap();

    assert!(list_active_notes(&conn, &user.id).unwrap().is_empty());
    assert_eq!(list_trashed_notes(&conn, &user.id).unwrap().len(), 1);

    note.restore("dev_1");
    assert!(!note.trashed);
    assert!(note.deleted_at.is_none());
    upsert_note(&conn, &note).unwrap();

    assert_eq!(list_active_notes(&conn, &user.id).unwrap().len(), 1);
    assert!(list_trashed_notes(&conn, &user.id).unwrap().is_empty());

    delete_note_permanently(&conn, &note.id).unwrap();
    assert!(get_note_by_id(&conn, &note.id).unwrap().is_none());
}

#[test]
fn test_list_notes_by_folder() {
    let conn = open_in_memory().unwrap();
    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let folder_a = Folder::new("Folder A", None, None, 0, "dev_1", &user.id);
    insert_folder(&conn, &folder_a).unwrap();

    let root_note = Note::new(
        "Root Note",
        "No folder",
        "No folder text",
        None,
        "dev_1",
        &user.id,
    );
    let folder_note = Note::new(
        "Folder Note",
        "In folder A",
        "In folder A text",
        Some(folder_a.id.clone()),
        "dev_1",
        &user.id,
    );

    insert_note(&conn, &root_note).unwrap();
    insert_note(&conn, &folder_note).unwrap();

    let root_notes = list_notes_by_folder(&conn, &user.id, None).unwrap();
    assert_eq!(root_notes.len(), 1);
    assert_eq!(root_notes[0].id, root_note.id);

    let folder_notes = list_notes_by_folder(&conn, &user.id, Some(&folder_a.id)).unwrap();
    assert_eq!(folder_notes.len(), 1);
    assert_eq!(folder_notes[0].id, folder_note.id);
}

#[test]
fn test_delete_trashed_older_than() {
    let conn = open_in_memory().unwrap();
    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let mut note = Note::new(
        "Old Trashed Note",
        "Body",
        "Searchable",
        None,
        "dev_1",
        &user.id,
    );
    note.trash("dev_1");
    note.deleted_at = Some(1000);
    insert_note(&conn, &note).unwrap();

    let purged = delete_trashed_older_than(&conn, 5000).unwrap();
    assert_eq!(purged, 1);

    assert!(get_note_by_id(&conn, &note.id).unwrap().is_none());
}

#[test]
fn test_fts5_full_text_search() {
    let conn = open_in_memory().unwrap();
    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let note_1 = Note::new(
        "Rust Architecture",
        "{\"version\":1,\"blocks\":[{\"type\":\"Paragraph\",\"data\":{\"spans\":[{\"text\":\"GPUI Metal Vulkan\"}]}}]}",
        "Exploring GPUI framework for fast desktop rendering with Metal and Vulkan.",
        None,
        "dev_1",
        &user.id,
    );
    let note_2 = Note::new(
        "Pasta Recipe",
        "{\"version\":1,\"blocks\":[{\"type\":\"Paragraph\",\"data\":{\"spans\":[{\"text\":\"carbonara\"}]}}]}",
        "Classic Italian carbonara with guanciale, pecorino cheese, and black pepper.",
        None,
        "dev_1",
        &user.id,
    );
    let note_3 = Note::new(
        "Weekly Standup",
        "{\"version\":1,\"blocks\":[{\"type\":\"Paragraph\",\"data\":{\"spans\":[{\"text\":\"Rust sync\"}]}}]}",
        "Discussed Rust backend sync performance and React Native mobile client.",
        None,
        "dev_1",
        &user.id,
    );

    insert_note(&conn, &note_1).unwrap();
    insert_note(&conn, &note_2).unwrap();
    insert_note(&conn, &note_3).unwrap();

    let results = search_notes(&conn, &user.id, "GPUI").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, note_1.id);

    let results = search_notes(&conn, &user.id, "carbonara").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, note_2.id);

    let results = search_notes(&conn, &user.id, "Rust").unwrap();
    assert_eq!(results.len(), 2);

    let json_syntax_token_results = search_notes(&conn, &user.id, "Paragraph").unwrap();
    assert_eq!(json_syntax_token_results.len(), 0);

    let version_results = search_notes(&conn, &user.id, "version").unwrap();
    assert_eq!(version_results.len(), 0);

    let blocks_results = search_notes(&conn, &user.id, "blocks").unwrap();
    assert_eq!(blocks_results.len(), 0);

    let mut updated_note_2 = note_2.clone();
    updated_note_2.update(
        "Pizza Recipe",
        "{\"version\":1,\"blocks\":[{\"type\":\"Paragraph\",\"data\":{\"spans\":[{\"text\":\"pizza\"}]}}]}",
        "Neapolitan dough with san marzano tomatoes.",
        None,
        "dev_1",
    );
    upsert_note(&conn, &updated_note_2).unwrap();

    assert!(
        search_notes(&conn, &user.id, "carbonara")
            .unwrap()
            .is_empty()
    );
    let pizza_results = search_notes(&conn, &user.id, "pizza").unwrap();
    assert_eq!(pizza_results.len(), 1);
    assert_eq!(pizza_results[0].id, note_2.id);

    delete_note_permanently(&conn, &note_1.id).unwrap();
    assert!(search_notes(&conn, &user.id, "Vulkan").unwrap().is_empty());
}
