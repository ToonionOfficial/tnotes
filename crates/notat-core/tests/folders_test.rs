use notat_core::{
    db::{
        folders::{
            delete_folder_permanently, get_folder_by_id, get_folder_tree,
            insert_folder, list_all_folders, list_subfolders, upsert_folder,
        },
        migrations::open_in_memory,
    },
    models::folder::Folder,
};

#[test]
fn test_folder_crud_and_mutations() {
    let conn = open_in_memory().unwrap();

    // 1. Create a folder
    let mut folder = Folder::new("Work", Some("💼".into()), None, 0, "dev_1");
    assert_eq!(folder.version, 1);
    insert_folder(&conn, &folder).unwrap();

    let fetched = get_folder_by_id(&conn, &folder.id).unwrap().unwrap();
    assert_eq!(fetched.name, "Work");
    assert_eq!(fetched.icon, "💼");
    assert_eq!(fetched.version, 1);

    // 2. Rename and change icon
    folder.rename("Work & Projects", "dev_1");
    folder.set_icon("🚀", "dev_1");
    assert_eq!(folder.version, 3);
    upsert_folder(&conn, &folder).unwrap();

    let fetched_updated = get_folder_by_id(&conn, &folder.id).unwrap().unwrap();
    assert_eq!(fetched_updated.name, "Work & Projects");
    assert_eq!(fetched_updated.icon, "🚀");
    assert_eq!(fetched_updated.version, 3);

    // 3. Soft delete and restore
    folder.soft_delete("dev_1");
    assert!(folder.deleted_at.is_some());
    upsert_folder(&conn, &folder).unwrap();

    assert!(list_all_folders(&conn).unwrap().is_empty());

    folder.restore("dev_1");
    assert!(folder.deleted_at.is_none());
    upsert_folder(&conn, &folder).unwrap();

    assert_eq!(list_all_folders(&conn).unwrap().len(), 1);

    // 4. Hard delete
    delete_folder_permanently(&conn, &folder.id).unwrap();
    assert!(get_folder_by_id(&conn, &folder.id).unwrap().is_none());
}

#[test]
fn test_hierarchical_folder_tree_recursive_cte() {
    let conn = open_in_memory().unwrap();

    // 1. Level 0: "Work" and "Personal"
    let work = Folder::new("Work", Some("💼".into()), None, 0, "dev_1");
    let personal = Folder::new("Personal", Some("🏠".into()), None, 1, "dev_1");
    insert_folder(&conn, &work).unwrap();
    insert_folder(&conn, &personal).unwrap();

    // 2. Level 1: "Engineering" inside "Work"
    let eng = Folder::new("Engineering", Some("⚙️".into()), Some(work.id.clone()), 0, "dev_1");
    insert_folder(&conn, &eng).unwrap();

    // 3. Level 2: "Backend" inside "Engineering"
    let backend = Folder::new("Backend", Some("🦀".into()), Some(eng.id.clone()), 0, "dev_1");
    insert_folder(&conn, &backend).unwrap();

    // Test list_subfolders
    let root_folders = list_subfolders(&conn, None).unwrap();
    assert_eq!(root_folders.len(), 2);

    let work_subfolders = list_subfolders(&conn, Some(&work.id)).unwrap();
    assert_eq!(work_subfolders.len(), 1);
    assert_eq!(work_subfolders[0].id, eng.id);

    // Test recursive CTE tree query
    let tree = get_folder_tree(&conn).unwrap();
    assert_eq!(tree.len(), 4);

    let personal_node = tree.iter().find(|n| n.folder.id == personal.id).unwrap();
    assert_eq!(personal_node.depth, 0);
    assert_eq!(personal_node.path, "Personal");

    let work_node = tree.iter().find(|n| n.folder.id == work.id).unwrap();
    assert_eq!(work_node.depth, 0);
    assert_eq!(work_node.path, "Work");

    let eng_node = tree.iter().find(|n| n.folder.id == eng.id).unwrap();
    assert_eq!(eng_node.depth, 1);
    assert_eq!(eng_node.path, "Work / Engineering");

    let backend_node = tree.iter().find(|n| n.folder.id == backend.id).unwrap();
    assert_eq!(backend_node.depth, 2);
    assert_eq!(backend_node.path, "Work / Engineering / Backend");
}
