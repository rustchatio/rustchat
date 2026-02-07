//! WebSocket Actor for Mattermost-compatible connections
//!
//! Implements:
//! - Protocol-level Ping/Pong (WebSocket control frames)
//! - 60s ping interval, 100s pong timeout, 30s write deadline
//! - Session resumption support
//! - Graceful shutdown with proper close codes

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, timeout};
use tracing::{debug, error, trace, warn};
use uuid::Uuid;

use crate::mattermost_compat::models as mm;
use crate::realtime::connection_store::{ConnectionState, ConnectionStore};

// Mattermost WebSocket constants from web_conn.go
/// Write timeout for WebSocket operations (30 seconds)
const WRITE_WAIT: Duration = Duration::from_secs(30);
/// Time to wait for Pong response after Ping (100 seconds)
const PONG_WAIT: Duration = Duration::from_secs(100);
/// Interval between Ping frames (60 seconds)
const PING_INTERVAL: Duration = Duration::from_secs(60);

/// WebSocket close codes
pub mod close_codes {
    /// Normal closure
    pub const NORMAL: u16 = 1000;
    /// Going away (server restart, etc.)
    pub const GOING_AWAY: u16 = 1001;
    /// Protocol error
    pub const PROTOCOL_ERROR: u16 = 1002;
    /// Unsupported data
    pub const UNSUPPORTED_DATA: u16 = 1003;
    /// Policy violation (auth failure)
    pub const POLICY_VIOLATION: u16 = 1008;
    /// Message too big
    pub const MESSAGE_TOO_BIG: u16 = 1009;
    /// Internal server error
    pub const INTERNAL_ERROR: u16 = 1011;
    /// Service restart
    pub const SERVICE_RESTART: u16 = 1012;
    /// Try again later
    pub const TRY_AGAIN_LATER: u16 = 1013;
}

/// Commands sent to the WebSocket actor
#[derive(Debug)]
pub enum WsCommand {
    /// Send a message to the client
    SendMessage(mm::WebSocketMessage),
    /// Send raw JSON to the client
    SendRaw(serde_json::Value),
    /// Close the connection with a specific code
    Close(u16, String),
    /// Update channel subscriptions
    SubscribeChannels(Vec<Uuid>),
    /// Update team subscriptions
    SubscribeTeams(Vec<Uuid>),
}

/// Events received from the WebSocket actor
#[derive(Debug)]
pub enum WsEvent {
    /// Message received from client
    MessageReceived(String),
    /// Client sent a pong (activity detected)
    PongReceived,
    /// Connection closed
    Closed(CloseReason),
    /// Error occurred
    Error(String),
}

/// Reason for connection close
#[derive(Debug, Clone)]
pub struct CloseReason {
    pub code: u16,
    pub reason: String,
}

/// WebSocket Actor that manages a single connection
pub struct WebSocketActor {
    /// Connection ID
    pub connection_id: String,
    /// User ID
    pub user_id: Uuid,
    /// Connection state for resumption
    state: Arc<ConnectionState>,
    /// Connection store for session management
    store: Arc<ConnectionStore>,
    /// Channel for sending commands to the actor
    cmd_tx: mpsc::UnboundedSender<WsCommand>,
    /// Channel for receiving events from the actor
    event_rx: Mutex<mpsc::UnboundedReceiver<WsEvent>>,
    /// Last activity timestamp (for pong timeout)
    last_activity: Arc<std::sync::atomic::AtomicU64>,
    /// Whether the connection is closing
    is_closing: Arc<AtomicBool>,
    /// Remote address (for logging)
    remote_addr: Option<SocketAddr>,
}

impl WebSocketActor {
    /// Create a new WebSocket actor and start processing
    ///
    /// # Arguments
    /// * `socket` - The WebSocket stream
    /// * `store` - Connection store for session management
    /// * `user_id` - Authenticated user ID
    /// * `connection_id` - Existing connection ID for resumption, or None for new
    /// * `sequence_number` - Last sequence number received by client
    /// * `remote_addr` - Client remote address for logging
    pub async fn new(
        socket: WebSocket,
        store: Arc<ConnectionStore>,
        user_id: Uuid,
        connection_id: Option<String>,
        sequence_number: Option<i64>,
        remote_addr: Option<SocketAddr>,
    ) -> (Arc<Self>, Vec<mm::WebSocketMessage>) {
        // Resume or create connection state
        let (state, is_resumed, missed_messages) =
            store.resume_or_create(connection_id, user_id, sequence_number);

        let conn_id = state.connection_id.clone();

        // Create channels for actor communication
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let last_activity = Arc::new(std::sync::atomic::AtomicU64::new(
            Instant::now().elapsed().as_secs(),
        ));
        let is_closing = Arc::new(AtomicBool::new(false));

        // Convert missed SequencedMessages to WebSocketMessages
        let missed_ws_messages: Vec<mm::WebSocketMessage> = missed_messages
            .into_iter()
            .map(|msg| mm::WebSocketMessage {
                seq: Some(msg.seq),
                event: msg
                    .message
                    .get("event")
                    .and_then(|e| e.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                data: msg.message.get("data").cloned().unwrap_or(json!({})),
                broadcast: mm::Broadcast {
                    omit_users: None,
                    user_id: String::new(),
                    channel_id: String::new(),
                    team_id: String::new(),
                },
            })
            .collect();

        // Spawn the actor task
        let actor = ActorTask {
            socket,
            state: state.clone(),
            store: store.clone(),
            cmd_rx,
            event_tx,
            last_activity: last_activity.clone(),
            is_closing: is_closing.clone(),
            user_id,
            connection_id: conn_id.clone(),
        };

        tokio::spawn(actor.run());

        let actor = Arc::new(Self {
            connection_id: conn_id,
            user_id,
            state,
            store,
            cmd_tx,
            event_rx: Mutex::new(event_rx),
            last_activity,
            is_closing,
            remote_addr,
        });

        debug!(
            connection_id = %actor.connection_id,
            user_id = %user_id,
            resumed = is_resumed,
            missed_count = missed_ws_messages.len(),
            "WebSocket actor created"
        );

        (actor, missed_ws_messages)
    }

    /// Send a message to the client
    pub fn send(&self, msg: mm::WebSocketMessage) -> Result<(), String> {
        if self.is_closing.load(Ordering::SeqCst) {
            return Err("Connection is closing".to_string());
        }

        self.cmd_tx
            .send(WsCommand::SendMessage(msg))
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Send raw JSON to the client
    pub fn send_raw(&self, data: serde_json::Value) -> Result<(), String> {
        if self.is_closing.load(Ordering::SeqCst) {
            return Err("Connection is closing".to_string());
        }

        self.cmd_tx
            .send(WsCommand::SendRaw(data))
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// Close the connection
    pub fn close(&self, code: u16, reason: &str) {
        self.is_closing.store(true, Ordering::SeqCst);
        let _ = self.cmd_tx.send(WsCommand::Close(code, reason.to_string()));
    }

    /// Receive the next event from the actor
    pub async fn recv(&self) -> Option<WsEvent> {
        let mut rx = self.event_rx.lock().await;
        rx.recv().await
    }

    /// Update last activity (called when any message is received)
    fn update_activity(&self) {
        self.last_activity
            .store(Instant::now().elapsed().as_secs(), Ordering::SeqCst);
        self.state.touch();
    }

    /// Check if the connection is alive (not timed out)
    pub fn is_alive(&self) -> bool {
        !self.is_closing.load(Ordering::SeqCst)
    }

    /// Mark connection as disconnected (for session resumption)
    pub fn disconnect(&self) {
        self.is_closing.store(true, Ordering::SeqCst);
        self.store.disconnect_connection(&self.connection_id);
    }
}

/// Internal actor task that manages the WebSocket I/O
struct ActorTask {
    socket: WebSocket,
    state: Arc<ConnectionState>,
    store: Arc<ConnectionStore>,
    cmd_rx: mpsc::UnboundedReceiver<WsCommand>,
    event_tx: mpsc::UnboundedSender<WsEvent>,
    last_activity: Arc<std::sync::atomic::AtomicU64>,
    is_closing: Arc<AtomicBool>,
    user_id: Uuid,
    connection_id: String,
}

impl ActorTask {
    async fn run(mut self) {
        let (mut ws_sink, mut ws_stream) = self.socket.split();

        // Create ping interval
        let mut ping_interval = interval(PING_INTERVAL);

        // Track last pong time
        let last_pong = Arc::new(std::sync::Mutex::new(Instant::now()));

        // Spawn ping task
        let ping_last_pong = last_pong.clone();
        let ping_is_closing = self.is_closing.clone();
        let ping_connection_id = self.connection_id.clone();

        let ping_task = tokio::spawn(async move {
            loop {
                ping_interval.tick().await;

                if ping_is_closing.load(Ordering::SeqCst) {
                    break;
                }

                // Check if we've received a pong recently
                let last_pong_time = *ping_last_pong.lock().unwrap();
                if Instant::now().duration_since(last_pong_time) > PONG_WAIT {
                    warn!(
                        connection_id = %ping_connection_id,
                        "Pong timeout - closing connection"
                    );
                    ping_is_closing.store(true, Ordering::SeqCst);
                    break;
                }

                // Send ping frame
                trace!(connection_id = %ping_connection_id, "Sending Ping frame");
                // We can't easily send from here since we don't have sink access
                // The ping will be sent by the select! loop below
            }
        });

        // Main event loop
        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = ws_stream.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            *last_pong.lock().unwrap() = Instant::now();
                            self.state.touch();

                            let text_str = text.to_string();
                            trace!(
                                connection_id = %self.connection_id,
                                text = %text_str,
                                "Received text message"
                            );

                            if let Err(_) = self.event_tx.send(WsEvent::MessageReceived(text_str)) {
                                break;
                            }
                        }
                        Some(Ok(Message::Binary(bin))) => {
                            *last_pong.lock().unwrap() = Instant::now();
                            self.state.touch();

                            // Convert binary to string if possible, otherwise ignore
                            if let Ok(text) = String::from_utf8(bin.to_vec()) {
                                if let Err(_) = self.event_tx.send(WsEvent::MessageReceived(text)) {
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            trace!(connection_id = %self.connection_id, "Received Pong frame");
                            *last_pong.lock().unwrap() = Instant::now();
                            self.state.touch();

                            if let Err(_) = self.event_tx.send(WsEvent::PongReceived) {
                                break;
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            // Respond with Pong immediately
                            trace!(connection_id = %self.connection_id, "Received Ping frame, sending Pong");
                            if let Err(e) = ws_sink.send(Message::Pong(data)).await {
                                warn!(
                                    connection_id = %self.connection_id,
                                    error = %e,
                                    "Failed to send Pong"
                                );
                                break;
                            }
                        }
                        Some(Ok(Message::Close(frame))) => {
                            debug!(
                                connection_id = %self.connection_id,
                                frame = ?frame,
                                "Received Close frame"
                            );

                            let reason = frame.map(|f| CloseReason {
                                code: f.code.into(),
                                reason: f.reason.to_string(),
                            }).unwrap_or_else(|| CloseReason {
                                code: close_codes::NORMAL,
                                reason: "Client closed".to_string(),
                            });

                            let _ = self.event_tx.send(WsEvent::Closed(reason));
                            break;
                        }
                        Some(Err(e)) => {
                            error!(
                                connection_id = %self.connection_id,
                                error = %e,
                                "WebSocket error"
                            );
                            let _ = self.event_tx.send(WsEvent::Error(e.to_string()));
                            break;
                        }
                        None => {
                            debug!(
                                connection_id = %self.connection_id,
                                "WebSocket stream ended"
                            );
                            break;
                        }
                        _ => {}
                    }
                }

                // Handle commands from the actor
                cmd = self.cmd_rx.recv() => {
                    match cmd {
                        Some(WsCommand::SendMessage(msg)) => {
                            match serde_json::to_string(&msg) {
                                Ok(json) => {
                                    // Apply write timeout
                                    match timeout(
                                        WRITE_WAIT,
                                        ws_sink.send(Message::Text(json.into()))
                                    ).await {
                                        Ok(Ok(())) => {
                                            trace!(
                                                connection_id = %self.connection_id,
                                                seq = ?msg.seq,
                                                event = %msg.event,
                                                "Message sent"
                                            );
                                        }
                                        Ok(Err(e)) => {
                                            warn!(
                                                connection_id = %self.connection_id,
                                                error = %e,
                                                "Failed to send message"
                                            );
                                            break;
                                        }
                                        Err(_) => {
                                            warn!(
                                                connection_id = %self.connection_id,
                                                "Write timeout"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        connection_id = %self.connection_id,
                                        error = %e,
                                        "Failed to serialize message"
                                    );
                                }
                            }
                        }
                        Some(WsCommand::SendRaw(data)) => {
                            match serde_json::to_string(&data) {
                                Ok(json) => {
                                    match timeout(
                                        WRITE_WAIT,
                                        ws_sink.send(Message::Text(json.into()))
                                    ).await {
                                        Ok(Ok(())) => {}
                                        Ok(Err(e)) => {
                                            warn!(
                                                connection_id = %self.connection_id,
                                                error = %e,
                                                "Failed to send raw message"
                                            );
                                            break;
                                        }
                                        Err(_) => {
                                            warn!(
                                                connection_id = %self.connection_id,
                                                "Write timeout on raw message"
                                            );
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        connection_id = %self.connection_id,
                                        error = %e,
                                        "Failed to serialize raw message"
                                    );
                                }
                            }
                        }
                        Some(WsCommand::Close(code, reason)) => {
                            debug!(
                                connection_id = %self.connection_id,
                                code = code,
                                reason = %reason,
                                "Closing connection"
                            );

                            let reason_clone = reason.clone();
                            let close_frame = CloseFrame {
                                code: code.into(),
                                reason: reason.into(),
                            };

                            let _ = ws_sink.send(Message::Close(Some(close_frame))).await;
                            let _ = self.event_tx.send(WsEvent::Closed(CloseReason { code, reason: reason_clone }));
                            break;
                        }
                        Some(WsCommand::SubscribeChannels(channels)) => {
                            // Update state
                            // Note: We'd need to add proper channel tracking to ConnectionState
                            debug!(
                                connection_id = %self.connection_id,
                                channel_count = channels.len(),
                                "Subscribing to channels"
                            );
                        }
                        Some(WsCommand::SubscribeTeams(teams)) => {
                            debug!(
                                connection_id = %self.connection_id,
                                team_count = teams.len(),
                                "Subscribing to teams"
                            );
                        }
                        None => {
                            // Command channel closed
                            break;
                        }
                    }
                }

                // Send periodic pings
                _ = tokio::time::sleep(PING_INTERVAL) => {
                    trace!(connection_id = %self.connection_id, "Sending periodic Ping");

                    match timeout(
                        WRITE_WAIT,
                        ws_sink.send(Message::Ping(vec![].into()))
                    ).await {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => {
                            warn!(
                                connection_id = %self.connection_id,
                                error = %e,
                                "Failed to send Ping"
                            );
                            break;
                        }
                        Err(_) => {
                            warn!(
                                connection_id = %self.connection_id,
                                "Ping write timeout"
                            );
                            break;
                        }
                    }

                    // Check for pong timeout
                    let last_pong_time = *last_pong.lock().unwrap();
                    if Instant::now().duration_since(last_pong_time) > PONG_WAIT {
                        warn!(
                            connection_id = %self.connection_id,
                            "Pong timeout detected"
                        );
                        break;
                    }
                }
            }
        }

        // Cleanup
        self.is_closing.store(true, Ordering::SeqCst);
        ping_task.abort();

        // Mark connection as disconnected (but retain state for resumption)
        self.store.disconnect_connection(&self.connection_id);

        debug!(
            connection_id = %self.connection_id,
            user_id = %self.user_id,
            "WebSocket actor stopped"
        );
    }
}

/// Configure TCP keepalive for mobile networks
///
/// This function configures socket-level keepalive to prevent
/// mobile carriers from dropping idle connections.
#[cfg(unix)]
pub fn configure_tcp_keepalive(socket: &std::net::TcpStream) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;

    let fd = socket.as_raw_fd();

    // Enable TCP keepalive
    let enabled: libc::c_int = 1;
    unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_KEEPALIVE,
            &enabled as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }

    // Set keepalive interval to 15 seconds (keep under 30s carrier timeout)
    let interval: libc::c_int = 15;
    unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPINTVL,
            &interval as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }

    // Set probe count to 3
    let probes: libc::c_int = 3;
    unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPCNT,
            &probes as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }

    Ok(())
}

#[cfg(not(unix))]
pub fn configure_tcp_keepalive(_socket: &std::net::TcpStream) -> std::io::Result<()> {
    // Not implemented for non-Unix platforms
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_codes() {
        assert_eq!(close_codes::NORMAL, 1000);
        assert_eq!(close_codes::GOING_AWAY, 1001);
        assert_eq!(close_codes::POLICY_VIOLATION, 1008);
        assert_eq!(close_codes::INTERNAL_ERROR, 1011);
    }
}
