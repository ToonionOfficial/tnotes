use notat_core::{
    db::{
        migrations::open_in_memory,
        themes::{
            delete_theme, get_theme_by_id, insert_theme, list_themes,
            seed_default_themes, upsert_theme,
        },
    },
    models::theme::Theme,
};

#[test]
fn test_theme_lifecycle_and_seeding() {
    let conn = open_in_memory().unwrap();

    // 1. Seed built-in themes
    seed_default_themes(&conn).unwrap();
    let themes = list_themes(&conn).unwrap();
    assert_eq!(themes.len(), 2);
    assert!(themes.iter().any(|t| t.name == "Default Dark" && t.builtin));
    assert!(themes.iter().any(|t| t.name == "Default Light" && t.builtin));

    // 2. Cannot delete built-in theme
    let dark_id = "theme_builtin_dark";
    let deleted = delete_theme(&conn, dark_id).unwrap();
    assert!(!deleted);
    assert!(get_theme_by_id(&conn, dark_id).unwrap().is_some());

    // 3. Create custom theme
    let custom_schema = themes[0].schema.clone();
    let mut custom_theme = Theme::new("Solarized Custom", custom_schema, "dev_1");
    insert_theme(&conn, &custom_theme).unwrap();

    let fetched = get_theme_by_id(&conn, &custom_theme.id).unwrap().unwrap();
    assert_eq!(fetched.name, "Solarized Custom");
    assert!(!fetched.builtin);

    // 4. Update custom theme
    custom_theme.name = "Solarized Dark Pro".into();
    custom_theme.version += 1;
    upsert_theme(&conn, &custom_theme).unwrap();

    let fetched_updated = get_theme_by_id(&conn, &custom_theme.id).unwrap().unwrap();
    assert_eq!(fetched_updated.name, "Solarized Dark Pro");
    assert_eq!(fetched_updated.version, 2);

    // 5. Delete custom theme
    let deleted_custom = delete_theme(&conn, &custom_theme.id).unwrap();
    assert!(deleted_custom);
    assert!(get_theme_by_id(&conn, &custom_theme.id).unwrap().is_none());
}
