use std::{collections::HashMap, io, net::SocketAddr, sync::Arc};

use a2ui::{AgentToRendererMessage, RendererToAgentMessage};
use form_urlencoded::parse as parse_query;
use futures::{SinkExt, StreamExt, stream::SplitSink};
use sdui::Application;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, broadcast},
};
use tokio_tungstenite::{
    WebSocketStream, accept_hdr_async,
    tungstenite::{
        Message,
        handshake::server::{Request, Response},
        http::Uri,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};

use crate::{TodoApp, application};

struct Session {
    application: Application<TodoApp>,
    updates: broadcast::Sender<AgentToRendererMessage>,
}

impl Session {
    fn new() -> io::Result<Self> {
        let mut application = application();
        application.start().map_err(io::Error::other)?;
        let (updates, _) = broadcast::channel(256);
        Ok(Self {
            application,
            updates,
        })
    }
}

type SharedSession = Arc<Mutex<Session>>;
type Sessions = Arc<Mutex<HashMap<String, SharedSession>>>;
type Outbound = SplitSink<WebSocketStream<TcpStream>, Message>;

/// Runs the example WebSocket application server.
///
/// # Errors
///
/// Returns an error if the address cannot be bound or accepting a connection fails.
pub async fn run(address: SocketAddr) -> io::Result<()> {
    let listener = TcpListener::bind(address).await?;
    println!("A2UI WebSocket server: ws://{}/ws", listener.local_addr()?);
    serve(listener).await
}

/// Serves standard A2UI messages on an existing Tokio listener.
///
/// # Errors
///
/// Returns an error if accepting a connection fails. Individual connection errors
/// are logged and do not stop other sessions.
pub async fn serve(listener: TcpListener) -> io::Result<()> {
    let sessions = Sessions::default();
    loop {
        let (stream, peer) = listener.accept().await?;
        let sessions = Arc::clone(&sessions);
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, sessions).await {
                eprintln!("A2UI WebSocket connection {peer} ended: {error}");
            }
        });
    }
}

#[allow(
    clippy::result_large_err,
    reason = "the tungstenite handshake callback requires its standard error response type"
)]
async fn handle_connection(stream: TcpStream, sessions: Sessions) -> io::Result<()> {
    let mut requested_session = None;
    let socket = accept_hdr_async(stream, |request: &Request, response: Response| {
        requested_session = parse_session_id(request.uri());
        Ok(response)
    })
    .await
    .map_err(io::Error::other)?;

    let Some(session_id) = requested_session else {
        let mut socket = socket;
        socket
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Policy,
                reason: "expected /ws?session=<id>".into(),
            })))
            .await
            .map_err(io::Error::other)?;
        return Ok(());
    };
    let session = get_session(&sessions, session_id).await?;
    handle_socket(socket, session).await
}

async fn get_session(sessions: &Sessions, id: String) -> io::Result<SharedSession> {
    let mut sessions = sessions.lock().await;
    if let Some(session) = sessions.get(&id) {
        return Ok(Arc::clone(session));
    }

    let session = Arc::new(Mutex::new(Session::new()?));
    sessions.insert(id, Arc::clone(&session));
    drop(sessions);
    Ok(session)
}

async fn handle_socket(
    socket: WebSocketStream<TcpStream>,
    session: SharedSession,
) -> io::Result<()> {
    let (mut outbound, mut inbound) = socket.split();
    let (snapshot, mut updates) = {
        let session = session.lock().await;
        let updates = session.updates.subscribe();
        let snapshot = session.application.snapshot().map_err(io::Error::other)?;
        drop(session);
        (snapshot, updates)
    };
    send(&mut outbound, &snapshot).await?;

    loop {
        tokio::select! {
            incoming = inbound.next() => {
                let Some(incoming) = incoming else { break };
                let incoming = incoming.map_err(io::Error::other)?;
                match incoming {
                    Message::Text(text) => {
                        let action = match serde_json::from_str::<RendererToAgentMessage>(text.as_str()) {
                            Ok(RendererToAgentMessage::Action(action)) => action,
                            Ok(_) => {
                                eprintln!("A2UI WebSocket ignored a non-action renderer message");
                                continue;
                            }
                            Err(error) => {
                                eprintln!("A2UI WebSocket rejected invalid JSON: {error}");
                                continue;
                            }
                        };

                        let mut session = session.lock().await;
                        match session.application.handle(action).await {
                            Ok(messages) => {
                                for message in messages {
                                    let _ = session.updates.send(message);
                                }
                            }
                            Err(error) => eprintln!("A2UI WebSocket action failed: {error}"),
                        }
                        drop(session);
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            update = updates.recv() => {
                match update {
                    Ok(message) => send(&mut outbound, &message).await?,
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        eprintln!("A2UI WebSocket client missed {count} messages; reconnect for a snapshot");
                        outbound
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::Policy,
                                reason: "message replay limit exceeded; reconnect for a snapshot".into(),
                            })))
                            .await
                            .map_err(io::Error::other)?;
                        break;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
    Ok(())
}

async fn send(outbound: &mut Outbound, message: &AgentToRendererMessage) -> io::Result<()> {
    let json = serde_json::to_string(message).map_err(io::Error::other)?;
    outbound
        .send(Message::Text(json.into()))
        .await
        .map_err(io::Error::other)
}

fn parse_session_id(uri: &Uri) -> Option<String> {
    if uri.path() != "/ws" {
        return None;
    }
    let mut values = parse_query(uri.query()?.as_bytes())
        .filter(|(name, _)| name == "session")
        .map(|(_, value)| value);
    let id = values.next()?.into_owned();
    if values.next().is_some()
        || id.is_empty()
        || id.len() > 128
        || id.chars().any(char::is_control)
    {
        return None;
    }
    Some(id)
}

#[cfg(test)]
mod tests {
    use tokio_tungstenite::tungstenite::http::Uri;

    use super::parse_session_id;

    #[test]
    fn session_ids_are_required_and_validated() {
        assert_eq!(
            parse_session_id(&"/ws?session=browser-example".parse::<Uri>().unwrap()).as_deref(),
            Some("browser-example")
        );
        for uri in [
            "/ws",
            "/other?session=browser-example",
            "/ws?session=",
            "/ws?session=one&session=two",
        ] {
            assert_eq!(parse_session_id(&uri.parse::<Uri>().unwrap()), None);
        }
    }
}
