use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tnotes_core::{
    auth::token::create_session_for_device,
    db::{
        devices::upsert_device, migrations::open_in_memory, notes::get_note_by_id,
        sessions::create_session, users::create_user,
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
        last_seq: 0,
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
        last_seq: 0,
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
    assert_eq!(sync_res.cursor, 0);
    assert!(!sync_res.has_more);

    // 3. Verify note is persisted in DB
    {
        let db = state.db.lock().await;
        let saved_note = get_note_by_id(&db, &note1.id).unwrap().unwrap();
        assert_eq!(saved_note.title, "Rust Concurrency");
        assert_eq!(saved_note.body, "Fearless concurrency with Tokio");
        assert_eq!(saved_note.user_id, user.id);
    }

    // 4. Client 2 syncs with last_seq = 0 -> receives Client 1's note (seq 1)
    let pull_envelope = SyncEnvelope {
        device_id: "device_mobile".into(),
        last_seq: 0,
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
    assert_eq!(sync_res.cursor, 1);
    assert!(!sync_res.has_more);

    // 5. Client 2 syncs again with last_seq = 1 -> receives 0 changes
    let pull_envelope_2 = SyncEnvelope {
        device_id: "device_mobile".into(),
        last_seq: 1,
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
                .body(Body::from(serde_json::to_string(&pull_envelope_2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let sync_res2: SyncResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(sync_res2.changes.len(), 0);
    assert_eq!(sync_res2.cursor, 1);
    assert!(!sync_res2.has_more);
}

#[tokio::test]
async fn test_sync_api_pagination() {
    let conn = open_in_memory().unwrap();
    let config = ServerConfig {
        host: "127.0.0.1".into(),
        port: 8787,
        data_dir: ":memory:".into(),
        server_url: "http://localhost:8787".into(),
    };

    let user = User::new("pageuser", "hash");
    create_user(&conn, &user).unwrap();

    let dev1 = Device::new("dev_page1", "Device 1", "desktop", &user.id);
    let dev2 = Device::new("dev_page2", "Device 2", "mobile", &user.id);
    upsert_device(&conn, &dev1).unwrap();
    upsert_device(&conn, &dev2).unwrap();

    let session1 = create_session_for_device("dev_page1", None);
    let session2 = create_session_for_device("dev_page2", None);
    create_session(&conn, &session1).unwrap();
    create_session(&conn, &session2).unwrap();

    let state = AppState::new(conn, config);
    let app = build_router(state);

    // 1. Dev 1 pushes 520 notes
    let mut changes = Vec::with_capacity(520);
    for i in 0..520 {
        let note = Note::new(
            format!("Note {}", i),
            format!("Body {}", i),
            None,
            "dev_page1",
            &user.id,
        );
        changes.push(Change {
            entity_type: EntityType::Note,
            entity_id: note.id.clone(),
            version: note.version,
            updated_at: note.updated_at,
            tombstone: false,
            payload: serde_json::to_value(&note).unwrap(),
        });
    }

    let push_envelope = SyncEnvelope {
        device_id: "dev_page1".into(),
        last_seq: 0,
        last_sync_at: 0,
        changes,
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

    // 2. Dev 2 pulls with last_seq = 0 -> expects first page of 500 with has_more = true
    let pull_envelope_page1 = SyncEnvelope {
        device_id: "dev_page2".into(),
        last_seq: 0,
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
                .body(Body::from(
                    serde_json::to_string(&pull_envelope_page1).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let page1_res: SyncResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(page1_res.changes.len(), 500);
    assert_eq!(page1_res.cursor, 500);
    assert!(page1_res.has_more);

    // 3. Dev 2 pulls page 2 with last_seq = 500 -> expects remaining 20 with has_more = false
    let pull_envelope_page2 = SyncEnvelope {
        device_id: "dev_page2".into(),
        last_seq: 500,
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
                .body(Body::from(
                    serde_json::to_string(&pull_envelope_page2).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let page2_res: SyncResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(page2_res.changes.len(), 20);
    assert_eq!(page2_res.cursor, 520);
    assert!(!page2_res.has_more);
}
