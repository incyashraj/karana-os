// Kāraṇa OS - WebSocket Server for Real-time Voice AI Updates
// Enables instant UI feedback for voice commands and tool execution

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Tool execution result
    ToolResult {
        tool_name: String,
        result: String,
        confidence: f32,
        execution_id: String,
        timestamp: u64,
    },
    /// Voice transcription update (real-time)
    Transcription {
        text: String,
        is_partial: bool,
        confidence: f32,
    },
    /// Voice activity status
    VoiceActivity {
        active: bool,
        energy_level: f32,
    },
    /// System state update
    StateUpdate {
        app_state: String,
        visible_elements: Vec<String>,
    },
    /// Error notification
    Error {
        message: String,
        code: String,
    },
    /// Client connection confirmation
    Connected {
        client_id: String,
        session_id: String,
    },
    /// Ping/pong for keepalive
    Ping,
    Pong,
}

/// Client connection info
#[derive(Debug, Clone)]
struct Client {
    id: String,
    session_id: String,
    subscribed_topics: Vec<String>,
}

/// WebSocket server state
pub struct WsServer {
    clients: Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>,
    broadcast_tx: broadcast::Sender<WsMessage>,
    broadcast_rx: broadcast::Receiver<WsMessage>,
}

impl WsServer {
    /// Create new WebSocket server
    pub fn new() -> Self {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(100);
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            broadcast_rx,
        }
    }

    /// Start WebSocket server
    pub async fn start(self: Arc<Self>, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        log::info!("[WS] WebSocket server listening on {}", addr);

        while let Ok((stream, peer)) = listener.accept().await {
            log::debug!("[WS] New connection from {}", peer);
            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream).await {
                    log::error!("[WS] Connection error: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Handle incoming WebSocket connection
    async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Generate client ID
        let client_id = format!("client_{}", uuid::Uuid::new_v4());
        let session_id = format!("session_{}", uuid::Uuid::new_v4());

        // Create channel for this client
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Register client
        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id.clone(), tx);
        }

        // Send connection confirmation
        let connect_msg = WsMessage::Connected {
            client_id: client_id.clone(),
            session_id: session_id.clone(),
        };
        let msg_json = serde_json::to_string(&connect_msg)?;
        ws_sender.send(Message::Text(msg_json)).await?;

        log::info!("[WS] Client {} connected (session: {})", client_id, session_id);

        // Subscribe to broadcast channel
        let mut broadcast_rx = self.broadcast_tx.subscribe();

        // Spawn task to forward broadcast messages to client
        let client_id_clone = client_id.clone();
        tokio::spawn(async move {
            while let Ok(msg) = broadcast_rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if let Err(e) = rx.send(Message::Text(json)) {
                        log::debug!("[WS] Client {} channel closed: {}", client_id_clone, e);
                        break;
                    }
                }
            }
        });

        // Handle bidirectional communication
        loop {
            tokio::select! {
                // Receive from client
                Some(msg) = ws_receiver.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Err(e) = self.handle_client_message(&client_id, &text).await {
                                log::warn!("[WS] Error handling message: {}", e);
                            }
                        }
                        Ok(Message::Ping(data)) => {
                            ws_sender.send(Message::Pong(data)).await?;
                        }
                        Ok(Message::Close(_)) => {
                            log::info!("[WS] Client {} closed connection", client_id);
                            break;
                        }
                        Err(e) => {
                            log::error!("[WS] WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                // Send to client
                Some(msg) = rx.recv() => {
                    if let Err(e) = ws_sender.send(msg).await {
                        log::debug!("[WS] Failed to send to client {}: {}", client_id, e);
                        break;
                    }
                }
            }
        }

        // Cleanup
        {
            let mut clients = self.clients.write().await;
            clients.remove(&client_id);
        }
        log::info!("[WS] Client {} disconnected", client_id);

        Ok(())
    }

    /// Handle message from client
    async fn handle_client_message(&self, client_id: &str, text: &str) -> Result<()> {
        log::debug!("[WS] Message from {}: {}", client_id, text);
        
        // Parse message
        let msg: WsMessage = serde_json::from_str(text)?;
        
        match msg {
            WsMessage::Ping => {
                // Respond with pong
                self.send_to_client(client_id, WsMessage::Pong).await?;
            }
            _ => {
                log::debug!("[WS] Received {:?} from client", msg);
            }
        }

        Ok(())
    }

    /// Broadcast message to all connected clients
    pub async fn broadcast(&self, msg: WsMessage) -> Result<()> {
        log::debug!("[WS] Broadcasting {:?}", msg);
        self.broadcast_tx.send(msg)
            .map_err(|e| anyhow!("Broadcast error: {}", e))?;
        Ok(())
    }

    /// Send message to specific client
    pub async fn send_to_client(&self, client_id: &str, msg: WsMessage) -> Result<()> {
        let clients = self.clients.read().await;
        if let Some(tx) = clients.get(client_id) {
            let json = serde_json::to_string(&msg)?;
            tx.send(Message::Text(json))
                .map_err(|e| anyhow!("Send error: {}", e))?;
        }
        Ok(())
    }

    /// Get number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Broadcast tool execution result
    pub async fn broadcast_tool_result(
        &self,
        tool_name: String,
        result: String,
        confidence: f32,
        execution_id: String,
    ) -> Result<()> {
        let msg = WsMessage::ToolResult {
            tool_name,
            result,
            confidence,
            execution_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        };
        self.broadcast(msg).await
    }

    /// Broadcast voice transcription (real-time or final)
    pub async fn broadcast_transcription(
        &self,
        text: String,
        is_partial: bool,
        confidence: f32,
    ) -> Result<()> {
        let msg = WsMessage::Transcription {
            text,
            is_partial,
            confidence,
        };
        self.broadcast(msg).await
    }

    /// Broadcast voice activity status
    pub async fn broadcast_voice_activity(
        &self,
        active: bool,
        energy_level: f32,
    ) -> Result<()> {
        let msg = WsMessage::VoiceActivity {
            active,
            energy_level,
        };
        self.broadcast(msg).await
    }

    /// Broadcast state update
    pub async fn broadcast_state_update(
        &self,
        app_state: String,
        visible_elements: Vec<String>,
    ) -> Result<()> {
        let msg = WsMessage::StateUpdate {
            app_state,
            visible_elements,
        };
        self.broadcast(msg).await
    }

    /// Broadcast error
    pub async fn broadcast_error(&self, message: String, code: String) -> Result<()> {
        let msg = WsMessage::Error { message, code };
        self.broadcast(msg).await
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ws_server_creation() {
        let server = WsServer::new();
        assert_eq!(server.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let server = Arc::new(WsServer::new());
        
        let msg = WsMessage::VoiceActivity {
            active: true,
            energy_level: 0.5,
        };
        
        // Should not panic even with no clients
        let result = server.broadcast(msg).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::ToolResult {
            tool_name: "calculator".to_string(),
            result: "42".to_string(),
            confidence: 0.95,
            execution_id: "exec_123".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();

        match parsed {
            WsMessage::ToolResult { tool_name, .. } => {
                assert_eq!(tool_name, "calculator");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
