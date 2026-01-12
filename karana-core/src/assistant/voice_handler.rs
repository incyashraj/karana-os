// Kāraṇa OS - Voice Command Handler
// Connects voice pipeline → query router → tool execution → WebSocket broadcast

use crate::assistant::{ToolRegistry, ToolArgs, StateContext, TtsService};
use crate::ai::query_router::{QueryRouter, RouteDecision};
use crate::network::WsServer;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;

/// Voice command handler - processes voice input and executes tools
pub struct VoiceCommandHandler {
    tool_registry: Arc<ToolRegistry>,
    state_context: Arc<RwLock<StateContext>>,
    ws_server: Arc<WsServer>,
    tts_service: Option<Arc<TtsService>>,
    query_router: Arc<QueryRouter>,
}

impl VoiceCommandHandler {
    /// Create new voice command handler
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        state_context: Arc<RwLock<StateContext>>,
        ws_server: Arc<WsServer>,
    ) -> Self {
        Self {
            tool_registry,
            state_context,
            ws_server,
            tts_service: None,
            query_router: Arc::new(QueryRouter::new()),
        }
    }

    /// Create with TTS support
    pub fn with_tts(
        tool_registry: Arc<ToolRegistry>,
        state_context: Arc<RwLock<StateContext>>,
        ws_server: Arc<WsServer>,
        tts_service: Arc<TtsService>,
    ) -> Self {
        Self {
            tool_registry,
            state_context,
            ws_server,
            tts_service: Some(tts_service),
            query_router: Arc::new(QueryRouter::new()),
        }
    }

    /// Handle voice input (transcribed text)
    pub async fn handle_voice_input(&self, transcript: &str) -> Result<()> {
        log::info!("[VOICE] Processing: '{}'", transcript);

        // Broadcast transcription
        self.ws_server.broadcast_transcription(transcript.to_string(), false, 0.9).await?;

        // Parse with context
        let parsed = {
            let state = self.state_context.read().await;
            state.parse_with_context(transcript).await?
        };

        // Route query
        let route = self.query_router.route(transcript, "").await?;

        match route.decision {
            RouteDecision::Direct(_) | RouteDecision::SingleTool(_, _) => {
                self.handle_direct_tool(transcript, parsed.resolved_target).await?;
            }
            RouteDecision::ReActChain(_) => {
                self.handle_complex_query(transcript).await?;
            }
            RouteDecision::Conversational => {
                self.handle_conversational(transcript).await?;
            }
        }

        Ok(())
    }

    async fn handle_direct_tool(&self, transcript: &str, _resolved: Option<String>) -> Result<()> {
        let args = ToolArgs::new();
        let result = self.tool_registry.execute("navigate", args).await?;
        self.ws_server.broadcast_tool_result(&result.tool_name, &result.output, result.confidence).await?;
        Ok(())
    }

    async fn handle_complex_query(&self, transcript: &str) -> Result<()> {
        let response = format!("Complex query: {}", transcript);
        self.ws_server.broadcast_tool_result("react", &response, 0.7).await?;
        Ok(())
    }

    async fn handle_conversational(&self, transcript: &str) -> Result<()> {
        let response = if transcript.to_lowercase().contains("hello") {
            "Hello! How can I help?"
        } else {
            "I'm here to assist."
        };
        self.ws_server.broadcast_tool_result("conversation", response, 1.0).await?;
        Ok(())
    }
}
