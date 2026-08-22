use tnotes_core::{
    auth::{password::{hash_password, verify_password}, token::create_session_for_device},
    db::{
        devices::upsert_device,
        migrations::open_in_memory,
        sessions::{
            cleanup_expired_sessions, create_session, delete_session,
            delete_sessions_by_device, get_session,
        },
        users::{create_user, get_user_by_id, get_user_by_username, has_any_user, update_password},
    },
    models::{device::Device, user::User},
};

#[test]
fn test_user_flow_and_auth() {
    let conn = open_in_memory().unwrap();

    // 1. Initially no users exist
    assert!(!has_any_user(&conn).unwrap());

    // 2. Hash password and create user
    let password = "my_secure_passphrase";
    let password_hash = hash_password(password).unwrap();
    let user = User::new("admin", password_hash);

    create_user(&conn, &user).unwrap();

    // 3. Verify user exists
    assert!(has_any_user(&conn).unwrap());

    let fetched = get_user_by_username(&conn, "admin").unwrap().unwrap();
    assert_eq!(fetched.id, user.id);
    assert_eq!(fetched.username, "admin");
    assert!(verify_password(password, &fetched.password_hash).unwrap());

    let by_id = get_user_by_id(&conn, &user.id).unwrap().unwrap();
    assert_eq!(by_id.username, "admin");

    // 4. Update password
    let new_pass = "new_super_pass";
    let new_hash = hash_password(new_pass).unwrap();
    update_password(&conn, &user.id, &new_hash).unwrap();

    let updated = get_user_by_id(&conn, &user.id).unwrap().unwrap();
    assert!(!verify_password(password, &updated.password_hash).unwrap());
    assert!(verify_password(new_pass, &updated.password_hash).unwrap());
}

#[test]
fn test_session_lifecycle() {
    let conn = open_in_memory().unwrap();
    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let dev_phone = Device::new("phone_1", "Phone", "mobile", &user.id);
    let dev_laptop = Device::new("laptop_1", "Laptop", "desktop", &user.id);
    upsert_device(&conn, &dev_phone).unwrap();
    upsert_device(&conn, &dev_laptop).unwrap();

    let session_1 = create_session_for_device("phone_1", None);
    let session_2 = create_session_for_device("laptop_1", None);
    let expired_session = create_session_for_device("phone_1", Some(-5000)); // already expired

    create_session(&conn, &session_1).unwrap();
    create_session(&conn, &session_2).unwrap();
    create_session(&conn, &expired_session).unwrap();

    // Verify lookup
    let fetched = get_session(&conn, &session_1.token).unwrap().unwrap();
    assert_eq!(fetched.device_id, "phone_1");

    // Cleanup expired sessions (now timestamp = current_time_ms)
    let now = tnotes_core::models::current_time_ms();
    let cleaned = cleanup_expired_sessions(&conn, now).unwrap();
    assert_eq!(cleaned, 1);
    assert!(get_session(&conn, &expired_session.token).unwrap().is_none());

    // Single session delete
    delete_session(&conn, &session_2.token).unwrap();
    assert!(get_session(&conn, &session_2.token).unwrap().is_none());

    // Delete by device
    delete_sessions_by_device(&conn, "phone_1").unwrap();
    assert!(get_session(&conn, &session_1.token).unwrap().is_none());
}
