use axum::http::Request;
use futures_util::{SinkExt, StreamExt};
use reqwest::header::COOKIE;
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
    middleware::SESSION_COOKIE_NAME,
    state::{AppState, ServerConfig, WsBroadcastMessage},
    ws::{WsClientMessage, WsServerMessage, WsTicketResponse},
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

    let device_a = Device::new("device_a", "Desktop", "desktop", &user.id);
    let device_b = Device::new("device_b", "Phone", "mobile", &user.id);

    upsert_device(&conn, &device_a).unwrap();
    upsert_device(&conn, &device_b).unwrap();

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

    let client = reqwest::Client::new();

    // 1. Device B requests a single-use WebSocket ticket via authenticated POST
    let ticket_res = client
        .post(format!(
            "http://{}:{}/api/ws/ticket",
            addr.ip(),
            addr.port()
        ))
        .header("authorization", format!("Bearer {}", session_b.token))
        .header("x-device-id", "device_b")
        .send()
        .await
        .unwrap();
    assert_eq!(ticket_res.status(), 200);
    let ticket_body: WsTicketResponse = ticket_res.json().await.unwrap();
    assert!(ticket_body.ticket.starts_with("tkt_"));

    // 2. Connect Device B to WebSocket using the single-use ticket
    let ws_url = format!(
        "ws://{}:{}/ws/sync?ticket={}",
        addr.ip(),
        addr.port(),
        ticket_body.ticket
    );
    let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect WS");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Reusing the same consumed ticket must fail with 401 Unauthorized
    assert!(connect_async(&ws_url).await.is_err());

    // 3. Test Ping / Pong over WebSocket
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

    // 4. Device A performs a sync push over HTTP
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

    let res = client
        .post(format!("http://{}:{}/api/sync", addr.ip(), addr.port()))
        .header("authorization", format!("Bearer {}", session_a.token))
        .json(&envelope)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // 5. Device B must receive the sync_notification on its open WebSocket
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

#[tokio::test]
async fn test_websocket_cookie_auth() {
    let conn = open_in_memory().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = ServerConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        data_dir: ":memory:".into(),
        server_url: format!("http://{}", addr),
    };

    let user = User::new("cookieuser", "hash");
    create_user(&conn, &user).unwrap();
    let device = Device::new("web-device", "Web", "web", &user.id);
    upsert_device(&conn, &device).unwrap();
    let session = create_session_for_device(&device.id, None);
    create_session(&conn, &session).unwrap();

    let state = AppState::new(conn, config);
    let app = build_router(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let req = Request::builder()
        .uri(format!("ws://{}:{}/ws/sync", addr.ip(), addr.port()))
        .header("host", format!("{}:{}", addr.ip(), addr.port()))
        .header(COOKIE, format!("{}={}", SESSION_COOKIE_NAME, session.token))
        .header("connection", "Upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(())
        .unwrap();

    let (ws_stream, _) = connect_async(req)
        .await
        .expect("Cookie WS connect should succeed");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    ws_sender
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&WsClientMessage::Ping)
                .unwrap()
                .into(),
        ))
        .await
        .unwrap();

    let pong_msg = ws_receiver.next().await.unwrap().unwrap();
    let parsed: WsServerMessage = serde_json::from_str(pong_msg.to_text().unwrap()).unwrap();
    match parsed {
        WsServerMessage::Pong => {}
        _ => panic!("Expected Pong message"),
    }
}

#[tokio::test]
async fn test_websocket_sync_required_message_serialization() {
    let sync_required = WsServerMessage::SyncRequired {
        reason: "buffer_lagged".to_string(),
        skipped: 42,
    };
    let json = serde_json::to_string(&sync_required).unwrap();
    assert_eq!(
        json,
        r#"{"type":"sync_required","data":{"reason":"buffer_lagged","skipped":42}}"#
    );

    let parsed: WsServerMessage = serde_json::from_str(&json).unwrap();
    match parsed {
        WsServerMessage::SyncRequired { reason, skipped } => {
            assert_eq!(reason, "buffer_lagged");
            assert_eq!(skipped, 42);
        }
        _ => panic!("Expected SyncRequired message, got {:?}", parsed),
    }
}

#[tokio::test]
async fn test_websocket_lag_sends_sync_required() {
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
    let device = Device::new("lagging-device", "Lagging", "desktop", &user.id);
    upsert_device(&conn, &device).unwrap();
    let session = create_session_for_device(&device.id, None);
    create_session(&conn, &session).unwrap();

    let state = AppState::new(conn, config);
    let app = build_router(state.clone());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let ticket_res = client
        .post(format!(
            "http://{}:{}/api/ws/ticket",
            addr.ip(),
            addr.port()
        ))
        .header("authorization", format!("Bearer {}", session.token))
        .header("x-device-id", "lagging-device")
        .send()
        .await
        .unwrap();
    let ticket_body: WsTicketResponse = ticket_res.json().await.unwrap();

    let ws_url = format!(
        "ws://{}:{}/ws/sync?ticket={}",
        addr.ip(),
        addr.port(),
        ticket_body.ticket
    );
    let (mut ws_stream, _) = connect_async(ws_url).await.unwrap();

    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while state.ws_sender.receiver_count() == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("websocket should subscribe to broadcasts");

    for _ in 0..129 {
        state
            .ws_sender
            .send(WsBroadcastMessage {
                sender_device_id: "other-device".into(),
                user_id: user.id.clone(),
                changes: vec![],
            })
            .unwrap();
    }

    let message = tokio::time::timeout(std::time::Duration::from_secs(1), ws_stream.next())
        .await
        .expect("websocket should receive a lag recovery message")
        .expect("websocket stream should remain open")
        .expect("websocket message should be valid");
    let parsed: WsServerMessage = serde_json::from_str(message.to_text().unwrap()).unwrap();
    match parsed {
        WsServerMessage::SyncRequired { reason, skipped } => {
            assert_eq!(reason, "buffer_lagged");
            assert!(skipped > 0);
        }
        other => panic!("expected SyncRequired after broadcast lag, got {other:?}"),
    }
}
