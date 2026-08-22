use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tnotes_core::{
    auth::token::create_session_for_device,
    db::{
        devices::upsert_device,
        migrations::open_in_memory,
        notes::get_note_by_id,
        sessions::create_session,
        users::create_user,
    },
    models::{device::Device, note::Note, user::User},
    sync::envelope::{Change, EntityType, SyncEnvelope, SyncResponse},
};
use tnotes_server::{
    build_router,
    state::{AppState, ServerConfig},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_sync_api_flow() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://localhost:8787".into(),
    };

    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let dev1 = Device::new("device_desktop", "Desktop", "desktop", &user.id);
    let dev2 = Device::new("device_mobile", "Mobile", "mobile", &user.id);
    upsert_device(&conn, &dev1).unwrap();
    upsert_device(&conn, &dev2).unwrap();

    let session1 = create_session_for_device("device_desktop", None);
    let session2 = create_session_for_device("device_mobile", None);

    create_session(&conn, &session1).unwrap();
    create_session(&conn, &session2).unwrap();

    let state = AppState::new(conn, config);
    let app = build_router(state.clone());

    // 1. Unauthenticated request -> 401 Unauthorized
    let envelope = SyncEnvelope {
        device_id: "device_desktop".into(),
        last_sync_at: 0,
        changes: vec![],
    };
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sync")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&envelope).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Client 1 pushes a new note
    let note1 = Note::new(
        "Rust Concurrency",
        "Fearless concurrency with Tokio",
        None,
        "device_desktop",
        &user.id,
    );
    let note1_payload = serde_json::to_value(&note1).unwrap();

    let push_envelope = SyncEnvelope {
        device_id: "device_desktop".into(),
        last_sync_at: 0,
        changes: vec![Change {
            entity_type: EntityType::Note,
            entity_id: note1.id.clone(),
            version: note1.version,
            updated_at: note1.updated_at,
            tombstone: false,
            payload: note1_payload,
        }],
    };

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sync")
                .header("authorization", format!("Bearer {}", session1.token))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&push_envelope).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let sync_res: SyncResponse = serde_json::from_slice(&body).unwrap();
    assert!(sync_res.server_time > 0);

    // 3. Verify note is persisted in DB
    {
        let db = state.db.lock().await;
        let saved_note = get_note_by_id(&db, &note1.id).unwrap().unwrap();
        assert_eq!(saved_note.title, "Rust Concurrency");
        assert_eq!(saved_note.body, "Fearless concurrency with Tokio");
        assert_eq!(saved_note.user_id, user.id);
    }

    // 4. Client 2 syncs with last_sync_at = 0 -> receives Client 1's note
    let pull_envelope = SyncEnvelope {
        device_id: "device_mobile".into(),
        last_sync_at: 0,
        changes: vec![],
    };

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sync")
                .header("authorization", format!("Bearer {}", session2.token))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&pull_envelope).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let sync_res: SyncResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(sync_res.changes.len(), 1);
    assert_eq!(sync_res.changes[0].entity_id, note1.id);
    assert_eq!(sync_res.changes[0].entity_type, EntityType::Note);
}
