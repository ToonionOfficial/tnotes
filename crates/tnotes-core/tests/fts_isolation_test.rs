use tnotes_core::{
    db::{
        migrations::open_in_memory,
        notes::{delete_note_permanently, insert_note, search_notes, upsert_note},
        users::create_user,
    },
    models::{note::Note, user::User},
};

#[test]
fn test_fts_zero_json_token_false_positives() {
    let conn = open_in_memory().unwrap();
    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let json_body = r#"{
        "version": 1,
        "blocks": [
            {
                "id": "01J8ABC123XYZ0000000000001",
                "type": "Heading",
                "data": {
                    "level": 1,
                    "content": { "spans": [{ "text": "Project Plan", "marks": {} }] }
                }
            },
            {
                "id": "01J8ABC123XYZ0000000000002",
                "type": "Paragraph",
                "data": {
                    "spans": [{ "text": "Deliverables for Q3 release", "marks": { "bold": true } }]
                }
            }
        ]
    }"#;

    let searchable_text = "Project Plan Deliverables for Q3 release";

    let note = Note::new(
        "Q3 Roadmap",
        json_body,
        searchable_text,
        None,
        "dev_desktop",
        &user.id,
    );
    insert_note(&conn, &note).unwrap();

    assert_eq!(search_notes(&conn, &user.id, "Paragraph").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "version").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "blocks").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "spans").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "type").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "data").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "marks").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "level").unwrap().len(), 0);
    assert_eq!(
        search_notes(&conn, &user.id, "01J8ABC123XYZ0000000000001")
            .unwrap()
            .len(),
        0
    );

    let positive_title = search_notes(&conn, &user.id, "Roadmap").unwrap();
    assert_eq!(positive_title.len(), 1);
    assert_eq!(positive_title[0].id, note.id);

    let positive_content = search_notes(&conn, &user.id, "Deliverables").unwrap();
    assert_eq!(positive_content.len(), 1);
    assert_eq!(positive_content[0].id, note.id);

    let multi_token = search_notes(&conn, &user.id, "Roadmap Deliverables").unwrap();
    assert_eq!(multi_token.len(), 1);
    assert_eq!(multi_token[0].id, note.id);
}

#[test]
fn test_fts_update_and_delete_lifecycle() {
    let conn = open_in_memory().unwrap();
    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let mut note = Note::new(
        "Cooking",
        "{\"type\":\"Recipe\"}",
        "Making fresh pasta and homemade sauce",
        None,
        "dev_mobile",
        &user.id,
    );
    insert_note(&conn, &note).unwrap();

    assert_eq!(search_notes(&conn, &user.id, "pasta").unwrap().len(), 1);
    assert_eq!(search_notes(&conn, &user.id, "baking").unwrap().len(), 0);

    note.update(
        "Baking",
        "{\"type\":\"BakingRecipe\"}",
        "Baking sourdough bread and croissants",
        None,
        "dev_mobile",
    );
    upsert_note(&conn, &note).unwrap();

    assert_eq!(search_notes(&conn, &user.id, "pasta").unwrap().len(), 0);
    assert_eq!(search_notes(&conn, &user.id, "sourdough").unwrap().len(), 1);
    assert_eq!(search_notes(&conn, &user.id, "Baking").unwrap().len(), 1);

    note.trash("dev_mobile");
    upsert_note(&conn, &note).unwrap();
    assert_eq!(search_notes(&conn, &user.id, "sourdough").unwrap().len(), 0);

    note.restore("dev_mobile");
    upsert_note(&conn, &note).unwrap();
    assert_eq!(search_notes(&conn, &user.id, "sourdough").unwrap().len(), 1);

    delete_note_permanently(&conn, &note.id).unwrap();
    assert_eq!(search_notes(&conn, &user.id, "sourdough").unwrap().len(), 0);
}

#[test]
fn test_fts_cross_user_isolation() {
    let conn = open_in_memory().unwrap();
    let user1 = User::new("user1", "hash1");
    let user2 = User::new("user2", "hash2");
    create_user(&conn, &user1).unwrap();
    create_user(&conn, &user2).unwrap();

    let note1 = Note::new(
        "Confidential Project",
        "{}",
        "Top secret strategy document",
        None,
        "dev_1",
        &user1.id,
    );
    insert_note(&conn, &note1).unwrap();

    assert_eq!(search_notes(&conn, &user1.id, "secret").unwrap().len(), 1);
    assert_eq!(search_notes(&conn, &user2.id, "secret").unwrap().len(), 0);
}
