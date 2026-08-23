use rusqlite::{params, Connection, OptionalExtension, Result};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleCheckResult {
    NoCycle,
    CycleDetected,
    DepthLimitExceeded,
}

/// Checks a proposed parent link before it is written to the database.
///
/// `folder_id` is included in the visited set so assigning one of a folder's
/// descendants as its parent is detected without first persisting the change.
pub fn check_proposed_folder_parent(
    conn: &Connection,
    folder_id: &str,
    parent_id: Option<&str>,
    max_depth: usize,
) -> Result<CycleCheckResult> {
    let mut current_id = match parent_id {
        Some(parent_id) => parent_id.to_owned(),
        None => return Ok(CycleCheckResult::NoCycle),
    };
    let mut visited = HashSet::from([folder_id.to_owned()]);

    for _ in 0..max_depth {
        if !visited.insert(current_id.clone()) {
            return Ok(CycleCheckResult::CycleDetected);
        }

        let next_parent_id: Option<String> = conn
            .query_row(
                "SELECT parent_id FROM folders WHERE id = ?1",
                params![current_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        match next_parent_id {
            Some(parent_id) => current_id = parent_id,
            None => return Ok(CycleCheckResult::NoCycle),
        }
    }

    Ok(CycleCheckResult::DepthLimitExceeded)
}

pub fn check_folder_cycle(
    conn: &Connection,
    folder_id: &str,
    max_depth: usize,
) -> Result<CycleCheckResult> {
    let mut current_id = folder_id.to_string();
    let mut visited = HashSet::new();

    visited.insert(current_id.clone());

    for _ in 0..max_depth {
        let parent_id: Option<String> = conn
            .query_row(
                "SELECT parent_id FROM folders WHERE id = ?1",
                params![current_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        match parent_id {
            Some(parent_id) => {
                if !visited.insert(parent_id.clone()) {
                    return Ok(CycleCheckResult::CycleDetected);
                }

                current_id = parent_id;
            }
            None => return Ok(CycleCheckResult::NoCycle),
        }
    }

    Ok(CycleCheckResult::DepthLimitExceeded)
}

pub fn has_folder_cycle(conn: &Connection, folder_id: &str, max_depth: usize) -> Result<bool> {
    check_folder_cycle(conn, folder_id, max_depth).map(|r| r == CycleCheckResult::CycleDetected)
}

pub fn break_folder_cycle_if_needed(conn: &Connection, folder_id: &str) -> Result<bool> {
    if check_folder_cycle(conn, folder_id, 50)? == CycleCheckResult::CycleDetected {
        conn.execute(
            "UPDATE folders SET parent_id = NULL WHERE id = ?1",
            params![folder_id],
        )?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::folders::insert_folder;
    use crate::db::migrations::open_in_memory;
    use crate::db::users::create_user;
    use crate::models::folder::Folder;
    use crate::models::user::User;

    #[test]
    fn test_linear_folder_hierarchy_no_cycle() {
        let conn = open_in_memory().unwrap();
        let user = User::new("user1", "hash");
        create_user(&conn, &user).unwrap();

        let mut root = Folder::new("Root", None, None, 0, "dev1", &user.id);
        let mut sub = Folder::new("Sub", None, Some(root.id.clone()), 0, "dev1", &user.id);
        let mut sub_sub = Folder::new("SubSub", None, Some(sub.id.clone()), 0, "dev1", &user.id);

        root.id = "root_id".into();
        sub.id = "sub_id".into();
        sub.parent_id = Some("root_id".into());
        sub_sub.id = "sub_sub_id".into();
        sub_sub.parent_id = Some("sub_id".into());

        insert_folder(&conn, &root).unwrap();
        insert_folder(&conn, &sub).unwrap();
        insert_folder(&conn, &sub_sub).unwrap();

        assert!(!has_folder_cycle(&conn, "sub_sub_id", 50).unwrap());
        assert!(!break_folder_cycle_if_needed(&conn, "sub_sub_id").unwrap());
    }

    #[test]
    fn test_direct_two_node_cycle_detection_and_repair() {
        let conn = open_in_memory().unwrap();
        let user = User::new("user1", "hash");
        create_user(&conn, &user).unwrap();

        let mut folder_a = Folder::new("Folder A", None, None, 0, "dev1", &user.id);
        folder_a.id = "folder_a".into();
        let mut folder_b = Folder::new(
            "Folder B",
            None,
            Some("folder_a".into()),
            0,
            "dev1",
            &user.id,
        );
        folder_b.id = "folder_b".into();

        insert_folder(&conn, &folder_a).unwrap();
        insert_folder(&conn, &folder_b).unwrap();

        // Create cycle A -> B -> A
        conn.execute(
            "UPDATE folders SET parent_id = 'folder_b' WHERE id = 'folder_a'",
            [],
        )
        .unwrap();

        assert!(has_folder_cycle(&conn, "folder_a", 50).unwrap());
        assert!(break_folder_cycle_if_needed(&conn, "folder_a").unwrap());

        // Cycle should now be resolved
        assert!(!has_folder_cycle(&conn, "folder_a", 50).unwrap());
        assert!(!has_folder_cycle(&conn, "folder_b", 50).unwrap());

        // folder_a should now be a root folder (parent_id is NULL)
        let parent: Option<String> = conn
            .query_row(
                "SELECT parent_id FROM folders WHERE id = 'folder_a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(parent.is_none());
    }

    #[test]
    fn test_multi_hop_cycle_detection() {
        let conn = open_in_memory().unwrap();
        let user = User::new("user1", "hash");
        create_user(&conn, &user).unwrap();

        // A -> B -> C -> A
        let mut f_a = Folder::new("A", None, None, 0, "dev1", &user.id);
        f_a.id = "a".into();
        let mut f_b = Folder::new("B", None, Some("a".into()), 0, "dev1", &user.id);
        f_b.id = "b".into();
        let mut f_c = Folder::new("C", None, Some("b".into()), 0, "dev1", &user.id);
        f_c.id = "c".into();

        insert_folder(&conn, &f_a).unwrap();
        insert_folder(&conn, &f_b).unwrap();
        insert_folder(&conn, &f_c).unwrap();

        // Form loop: A's parent becomes C
        conn.execute("UPDATE folders SET parent_id = 'c' WHERE id = 'a'", [])
            .unwrap();

        assert!(has_folder_cycle(&conn, "a", 50).unwrap());
        assert!(has_folder_cycle(&conn, "b", 50).unwrap());
        assert!(has_folder_cycle(&conn, "c", 50).unwrap());

        // Break on A
        assert!(break_folder_cycle_if_needed(&conn, "a").unwrap());
        assert!(!has_folder_cycle(&conn, "a", 50).unwrap());
        assert!(!has_folder_cycle(&conn, "b", 50).unwrap());
        assert!(!has_folder_cycle(&conn, "c", 50).unwrap());
    }
}
