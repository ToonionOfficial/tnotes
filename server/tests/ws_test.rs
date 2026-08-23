use futures_util::{SinkExt, StreamExt};
use tnotes_core::{
    auth::token::create_session_for_device,
    db::{
        devices::upsert_device, migrations::open_in_memory, sessions::create_session,
        users::create_user,
    },
    models::{device::Device, note::Note, user::User},
    sync::envelope::{Change, EntityType, SyncEnvelope},
};
use tnotes_server::{
    build_router,
    state::{AppState, ServerConfig},
    ws::{WsClientMessage, WsServerMessage},
};
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn test_websocket_realtime_broadcast() {
    let conn = open_in_memory().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let config = ServerConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        data_dir: ":memory:".into(),
        server_url: format!("http://{}", addr),
    };

    let user = User::new("testuser", "hash");
    create_user(&conn, &user).unwrap();

    let dev_a = Device::new("device_a", "Device A", "desktop", &user.id);
    let dev_b = Device::new("device_b", "Device B", "mobile", &user.id);
    upsert_device(&conn, &dev_a).unwrap();
    upsert_device(&conn, &dev_b).unwrap();

    let session_a = create_session_for_device("device_a", None);
    let session_b = create_session_for_device("device_b", None);

    create_session(&conn, &session_a).unwrap();
    create_session(&conn, &session_b).unwrap();

    let state = AppState::new(conn, config);
    let app = build_router(state.clone());

    // Spawn the test server
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 1. Connect Device B to WebSocket with token
    let ws_url = format!(
        "ws://{}:{}/ws/sync?token={}",
        addr.ip(),
        addr.port(),
        session_b.token
    );
    let (ws_stream, _) = connect_async(ws_url).await.expect("Failed to connect WS");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // 2. Test Ping / Pong over WebSocket
    let ping = WsClientMessage::Ping;
    ws_sender
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&ping).unwrap().into(),
        ))
        .await
        .unwrap();

    let pong_msg = ws_receiver.next().await.unwrap().unwrap();
    let pong_text = pong_msg.to_text().unwrap();
    let parsed_pong: WsServerMessage = serde_json::from_str(pong_text).unwrap();
    match parsed_pong {
        WsServerMessage::Pong => {}
        _ => panic!("Expected Pong message, got {:?}", parsed_pong),
    }

    // 3. Device A performs a sync push over HTTP
    let note = Note::new(
        "Test WS Note",
        "Realtime sync test",
        None,
        "device_a",
        &user.id,
    );
    let envelope = SyncEnvelope {
        device_id: "device_a".into(),
        last_seq: 0,
        last_sync_at: 0,
        changes: vec![Change {
            entity_type: EntityType::Note,
            entity_id: note.id.clone(),
            version: note.version,
            updated_at: note.updated_at,
            tombstone: false,
            payload: serde_json::to_value(&note).unwrap(),
        }],
    };

    let client = reqwest::Client::new();
    let res = client
        .post(format!("http://{}:{}/api/sync", addr.ip(), addr.port()))
        .header("authorization", format!("Bearer {}", session_a.token))
        .json(&envelope)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // 4. Device B must receive the sync_notification on its open WebSocket
    let notification_msg = ws_receiver.next().await.unwrap().unwrap();
    let notification_text = notification_msg.to_text().unwrap();
    let parsed_notification: WsServerMessage = serde_json::from_str(notification_text).unwrap();

    match parsed_notification {
        WsServerMessage::SyncNotification {
            sender_device_id,
            count,
        } => {
            assert_eq!(sender_device_id, "device_a");
            assert_eq!(count, 1);
        }
        _ => panic!("Expected SyncNotification, got {:?}", parsed_notification),
    }
}
