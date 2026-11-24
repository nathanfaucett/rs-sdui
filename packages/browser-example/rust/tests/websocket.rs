use std::time::Duration;

use a2ui_author::{ActionEvent, ActionMessage, ProtocolVersion};
use a2ui_browser_example::server::serve;
use futures::{SinkExt, StreamExt};
use serde_json::{Map, Value, json};
use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[tokio::test]
async fn sessions_share_updates_and_recover_from_snapshots() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test listener must bind");
    let address = listener.local_addr().expect("listener address must exist");
    let server = tokio::spawn(serve(listener));

    let mut first = connect(&address.to_string(), "browser-example").await;
    let mut second = connect(&address.to_string(), "browser-example").await;
    assert_eq!(
        next_json(&mut first)
            .await
            .pointer("/createSurface/dataModel/todos"),
        Some(&json!([]))
    );
    assert_eq!(
        next_json(&mut second)
            .await
            .pointer("/createSurface/dataModel/todos"),
        Some(&json!([]))
    );

    first
        .send(Message::Text(
            serde_json::to_string(&add_todo("Shared over WebSocket"))
                .expect("action must serialize")
                .into(),
        ))
        .await
        .expect("action must send");

    let first_update = next_json(&mut first).await;
    let second_update = next_json(&mut second).await;
    assert_eq!(first_update, second_update);

    let mut late = connect(&address.to_string(), "browser-example").await;
    let snapshot = next_json(&mut late).await;
    assert_eq!(
        snapshot.pointer("/createSurface/dataModel/todos/0/title"),
        Some(&json!("Shared over WebSocket"))
    );

    let mut isolated = connect(&address.to_string(), "another-session").await;
    let snapshot = next_json(&mut isolated).await;
    assert_eq!(
        snapshot.pointer("/createSurface/dataModel/todos"),
        Some(&json!([]))
    );

    server.abort();
}

async fn connect(address: &str, session: &str) -> Socket {
    let (socket, response) = connect_async(format!("ws://{address}/ws?session={session}"))
        .await
        .expect("WebSocket must connect");
    assert_eq!(response.status(), 101);
    socket
}

async fn next_json(socket: &mut Socket) -> Value {
    let message = timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("WebSocket read must not time out")
        .expect("WebSocket must stay open")
        .expect("WebSocket frame must be valid");
    let Message::Text(text) = message else {
        panic!("expected a standard A2UI text frame");
    };
    serde_json::from_str(text.as_str()).expect("A2UI frame must contain JSON")
}

fn add_todo(title: &str) -> ActionMessage {
    ActionMessage {
        version: ProtocolVersion::V1_0,
        action: ActionEvent {
            name: "add_todo".to_owned(),
            user_message: None,
            surface_id: "main".to_owned(),
            source_component_id: "root_3_1".to_owned(),
            timestamp: "2026-11-24T00:00:00Z".to_owned(),
            context: Map::from_iter([("title".to_owned(), json!(title))]),
            metadata: None,
        },
    }
}
