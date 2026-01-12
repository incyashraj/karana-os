// Kāraṇa OS - Unified AI Oracle
// Connects all AI components into a cohesive intelligent assistant

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::collections::VecDeque;

use crate::ai::{
    QueryRouter, QueryIntent, ReActAgent,
    DialogueManager, ReasoningContext, ChainOfThoughtReasoner,
};
use crate::assistant::{
    tool_registry::{ToolRegistry, ToolArgs, ToolResult},
    state_context::StateContext,
    tts_service::TtsService,
};
use crate::network::ws_server::WsServer;

/// Maximum conversation history to maintain
const MAX_HISTORY: usize = 10;

/// Minimum confidence threshold for tool execution
const CONFIDENCE_THRESHOLD: f32 = 0.7;

/// Oracle response with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    /// Natural language response to user
    pub text: String,
    
    /// Intent classification result
    pub intent_type: String,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    
    /// Whether action requires user confirmation
    pub requires_confirmation: bool,
    
    /// Tools that were executed
    pub tools_executed: Vec<String>,
    
    /// Reasoning steps (for multi-step queries)
    pub reasoning_trace: Vec<String>,
    
    /// Suggested follow-up actions
    pub suggested_followup: Option<Vec<String>>,
    
    /// Execution time in milliseconds
    pub latency_ms: u64,
}

/// Conversation message for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationMessage {
    role: String,      // "user" or "assistant"
    content: String,
    intent: Option<String>,
    timestamp: i64,
}

/// Unified AI Oracle - The brain of kāraṇa-os
pub struct AIOracle {
    /// Query classification and routing
    query_router: Arc<QueryRouter>,
    
    /// Tool execution registry
    tool_registry: Arc<ToolRegistry>,
    
    /// UI state and context tracking
    state_context: Arc<RwLock<StateContext>>,
    
    /// Multi-step reasoning agent
    react_agent: Option<Arc<ReActAgent>>,
    
    /// Dialogue management
    dialogue_manager: Arc<Mutex<DialogueManager>>,
    
    /// Chain-of-thought reasoner
    reasoner: Arc<ChainOfThoughtReasoner>,
    
    /// Text-to-speech service
    tts_service: Option<Arc<TtsService>>,
    
    /// WebSocket server for real-time updates
    ws_server: Option<Arc<WsServer>>,
    
    /// Conversation history
    history: Arc<Mutex<VecDeque<ConversationMessage>>>,
    
    /// Performance mode
    mode: OracleMode,
}

/// Oracle operating modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OracleMode {
    /// Fast pattern matching only (low battery)
    Fast,
    
    /// Balanced ML + patterns (normal)
    Balanced,
    
    /// Full intelligence with ReAct (high power)
    Intelligent,
}

impl AIOracle {
    /// Create a new AI Oracle instance
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        state_context: Arc<RwLock<StateContext>>,
        tts_service: Option<Arc<TtsService>>,
        ws_server: Option<Arc<WsServer>>,
    ) -> Self {
        let query_router = Arc::new(QueryRouter::new());
        let dialogue_manager = Arc::new(Mutex::new(DialogueManager::new()));
        let reasoner = Arc::new(ChainOfThoughtReasoner::new());
        let history = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_HISTORY)));
        
        Self {
            query_router,
            tool_registry,
            state_context,
            react_agent: None,  // Will be set when KaranaAI is available
            dialogue_manager,
            reasoner,
            tts_service,
            ws_server,
            history,
            mode: OracleMode::Balanced,
        }
    }
    
    /// Set the ReAct agent (requires KaranaAI LLM)
    pub fn with_react_agent(mut self, agent: Arc<ReActAgent>) -> Self {
        self.react_agent = Some(agent);
        self
    }
    
    /// Set operating mode
    pub fn set_mode(&mut self, mode: OracleMode) {
        self.mode = mode;
        log::info!("[Oracle] Mode changed to {:?}", mode);
    }
    
    /// Process user input and return intelligent response
    pub async fn process(&self, input: &str) -> Result<OracleResponse> {
        let start_time = std::time::Instant::now();
        
        log::info!("[Oracle] Processing: '{}'", input);
        
        // 1. Add to conversation history
        self.add_to_history("user", input, None).await;
        
        // 2. Classify intent
        let intent = self.classify_intent(input).await?;
        log::debug!("[Oracle] Intent: {:?} (confidence: {})", intent, intent.confidence);
        
        // 3. Check confidence threshold
        if intent.confidence < CONFIDENCE_THRESHOLD {
            return self.handle_low_confidence(input).await;
        }
        
        // 4. Check if requires multi-step reasoning
        let requires_reasoning = self.requires_reasoning(&intent);
        
        // 5. Execute query
        let result = if requires_reasoning && self.react_agent.is_some() {
            self.execute_with_reasoning(input, intent).await?
        } else {
            self.execute_direct(input, intent).await?
        };
        
        // 6. Add response to history
        self.add_to_history("assistant", &result.text, Some(&result.intent_type)).await;
        
        // 7. Send TTS if available
        if let Some(ref tts) = self.tts_service {
            if let Err(e) = tts.speak(&result.text).await {
                log::warn!("[Oracle] TTS failed: {}", e);
            }
        }
        
        // 8. Broadcast via WebSocket
        if let Some(ref ws) = self.ws_server {
            if let Err(e) = ws.broadcast_tool_result(
                &result.tools_executed.join(", "),
                &result.text,
                result.confidence
            ).await {
                log::warn!("[Oracle] WebSocket broadcast failed: {}", e);
            }
        }
        
        let latency = start_time.elapsed().as_millis() as u64;
        log::info!("[Oracle] Response generated in {}ms (confidence: {})", latency, result.confidence);
        
        Ok(OracleResponse {
            latency_ms: latency,
            ..result
        })
    }
    
    /// Classify user intent using query router
    async fn classify_intent(&self, input: &str) -> Result<ClassifiedIntent> {
        // Get conversation context
        let history = self.get_recent_history().await;
        let context_str = history.iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n");
        
        // Route query
        let route = self.query_router.route(input, &context_str).await?;
        
        Ok(ClassifiedIntent {
            intent_type: format!("{:?}", route.intent),
            confidence: route.confidence,
            entities: route.entities,
            requires_context: route.requires_context,
            route,
        })
    }
    
    /// Check if query requires multi-step reasoning
    fn requires_reasoning(&self, intent: &ClassifiedIntent) -> bool {
        // Only use ReAct for complex queries in Intelligent mode
        if self.mode != OracleMode::Intelligent {
            return false;
        }
        
        // Check if intent type suggests complexity
        matches!(intent.route.intent, 
            QueryIntent::MultiHop(_) | 
            QueryIntent::FactualCurrent(_)
        ) || intent.confidence < 0.85
    }
    
    /// Execute query with multi-step reasoning (ReAct agent)
    async fn execute_with_reasoning(
        &self,
        input: &str,
        intent: ClassifiedIntent,
    ) -> Result<OracleResponse> {
        log::info!("[Oracle] Using ReAct agent for complex query");
        
        let agent = self.react_agent.as_ref()
            .ok_or_else(|| anyhow!("ReAct agent not available"))?;
        
        // Build reasoning context
        let context = self.build_reasoning_context(input, &intent).await;
        
        // Run ReAct loop
        let agent_response = agent.run(input, &context).await?;
        
        Ok(OracleResponse {
            text: agent_response.answer,
            intent_type: intent.intent_type,
            confidence: agent_response.total_confidence,
            requires_confirmation: false,
            tools_executed: agent_response.chain.iter()
                .filter_map(|step| step.action.as_ref().map(|a| a.tool_name.clone()))
                .collect(),
            reasoning_trace: agent_response.chain.iter()
                .map(|step| step.thought.clone())
                .collect(),
            suggested_followup: None,
            latency_ms: 0, // Will be set by caller
        })
    }
    
    /// Execute query directly using tool registry
    async fn execute_direct(
        &self,
        input: &str,
        intent: ClassifiedIntent,
    ) -> Result<OracleResponse> {
        log::info!("[Oracle] Direct tool execution");
        
        // Determine which tool to use
        let tool_name = self.map_intent_to_tool(&intent)?;
        
        // Extract arguments from entities
        let args = self.extract_tool_args(&intent);
        
        // Execute tool
        let tool_result = self.tool_registry.execute(&tool_name, args).await?;
        
        // Generate natural language response
        let response_text = self.generate_response(input, &tool_name, &tool_result, &intent);
        
        // Check if action is reversible
        let has_undo = tool_result.undo_state.is_some();
        
        Ok(OracleResponse {
            text: response_text,
            intent_type: intent.intent_type,
            confidence: tool_result.confidence,
            requires_confirmation: false, // ToolResult doesn't have this field - always false for now
            tools_executed: vec![tool_name],
            reasoning_trace: vec![format!("Classified as: {}", intent.intent_type)],
            suggested_followup: if has_undo {
                Some(vec!["Undo that".to_string()])
            } else {
                None
            },
            latency_ms: 0,
        })
    }
    
    /// Map intent to tool name
    fn map_intent_to_tool(&self, intent: &ClassifiedIntent) -> Result<String> {
        match &intent.route.intent {
            QueryIntent::Computational(_) => Ok("calculator".to_string()),
            QueryIntent::FactualCurrent(query) => {
                match query.query_type {
                    crate::ai::query_router::LiveQueryType::Weather => Ok("weather".to_string()),
                    _ => Ok("web_search".to_string()),
                }
            }
            QueryIntent::Personal(query) => {
                match query.query_type {
                    crate::ai::query_router::UserQueryType::Calendar => Ok("calendar".to_string()),
                    crate::ai::query_router::UserQueryType::Files => Ok("file_manager".to_string()),
                    _ => Ok("system_control".to_string()),
                }
            }
            QueryIntent::Blockchain(cmd) => {
                // ChainCommand only has command_type: String
                if cmd.command_type.contains("balance") {
                    Ok("wallet".to_string())
                } else if cmd.command_type.contains("transfer") {
                    Ok("wallet".to_string())
                } else {
                    Ok("wallet".to_string())
                }
            }
            QueryIntent::Conversational(_) => Ok("dialogue".to_string()),
            _ => {
                // Try to infer from entities
                if intent.entities.contains_key("app") {
                    Ok("launch_app".to_string())
                } else if intent.entities.contains_key("destination") {
                    Ok("navigate".to_string())
                } else if intent.entities.contains_key("task") {
                    Ok("create_task".to_string())
                } else {
                    Err(anyhow!("Could not map intent to tool"))
                }
            }
        }
    }
    
    /// Extract tool arguments from intent entities
    fn extract_tool_args(&self, intent: &ClassifiedIntent) -> ToolArgs {
        let mut args = ToolArgs::new();
        
        for (key, value) in &intent.entities {
            args.add(key.clone(), serde_json::Value::String(value.clone()));
        }
        
        args
    }
    
    /// Generate natural language response
    fn generate_response(
        &self,
        input: &str,
        tool_name: &str,
        result: &ToolResult,
        intent: &ClassifiedIntent,
    ) -> String {
        if !result.success {
            return format!("❌ {}", result.output);
        }
        
        // Conversational responses based on tool
        match tool_name {
            "launch_app" => {
                if let Some(app) = intent.entities.get("app") {
                    format!("✓ Launched {}", app)
                } else {
                    result.output.clone()
                }
            }
            "navigate" => {
                if let Some(dest) = intent.entities.get("destination") {
                    format!("✓ Navigated to {}", dest)
                } else {
                    result.output.clone()
                }
            }
            "create_task" => {
                format!("✓ Created task: '{}'", result.output)
            }
            "weather" => {
                result.output.clone()
            }
            "wallet" => {
                result.output.clone()
            }
            "dialogue" => {
                result.output.clone()
            }
            _ => {
                format!("✓ {}", result.output)
            }
        }
    }
    
    /// Handle low confidence queries
    async fn handle_low_confidence(&self, input: &str) -> Result<OracleResponse> {
        log::warn!("[Oracle] Low confidence query: {}", input);
        
        Ok(OracleResponse {
            text: format!("I'm not sure I understood that. Could you rephrase? You said: '{}'", input),
            intent_type: "clarification_needed".to_string(),
            confidence: 0.0,
            requires_confirmation: false,
            tools_executed: vec![],
            reasoning_trace: vec!["Low confidence in intent classification".to_string()],
            suggested_followup: Some(vec![
                "Try rephrasing your request".to_string(),
                "Say 'help' for available commands".to_string(),
            ]),
            latency_ms: 0,
        })
    }
    
    /// Build reasoning context for ReAct agent
    async fn build_reasoning_context(&self, input: &str, intent: &ClassifiedIntent) -> ReasoningContext {
        let history = self.get_recent_history().await;
        let state = self.state_context.read().await;
        
        ReasoningContext {
            query: input.to_string(),
            domain: "os_control".to_string(),
            available_tools: self.tool_registry.list_tools(),
            conversation_history: history.iter()
                .map(|msg| format!("{}: {}", msg.role, msg.content))
                .collect(),
            ui_context: format!("UI Elements: {:?}", state.get_all_elements()),
            user_preferences: vec![],
        }
    }
    
    /// Add message to conversation history
    async fn add_to_history(&self, role: &str, content: &str, intent: Option<&str>) {
        let mut history = self.history.lock().await;
        
        let msg = ConversationMessage {
            role: role.to_string(),
            content: content.to_string(),
            intent: intent.map(|s| s.to_string()),
            timestamp: chrono::Utc::now().timestamp(),
        };
        
        if history.len() >= MAX_HISTORY {
            history.pop_front();
        }
        
        history.push_back(msg);
    }
    
    /// Get recent conversation history
    async fn get_recent_history(&self) -> Vec<ConversationMessage> {
        let history = self.history.lock().await;
        history.iter().cloned().collect()
    }
    
    /// Clear conversation history
    pub async fn clear_history(&self) {
        let mut history = self.history.lock().await;
        history.clear();
        log::info!("[Oracle] Conversation history cleared");
    }
    
    /// Get conversation history as formatted string
    pub async fn get_history_formatted(&self) -> String {
        let history = self.get_recent_history().await;
        history.iter()
            .map(|msg| {
                format!(
                    "[{}] {}: {}",
                    chrono::DateTime::from_timestamp(msg.timestamp, 0)
                        .map(|dt| dt.format("%H:%M:%S").to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    msg.role,
                    msg.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Internal classified intent with full context
#[derive(Debug)]
struct ClassifiedIntent {
    intent_type: String,
    confidence: f32,
    entities: std::collections::HashMap<String, String>,
    requires_context: bool,
    route: crate::ai::query_router::RouteDecision,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_oracle_basic_query() {
        let tool_registry = Arc::new(ToolRegistry::new());
        let state_context = Arc::new(RwLock::new(StateContext::new()));
        
        let oracle = AIOracle::new(
            tool_registry,
            state_context,
            None,
            None,
        );
        
        let response = oracle.process("open camera").await;
        assert!(response.is_ok());
        
        let resp = response.unwrap();
        assert!(resp.confidence > 0.7);
        assert_eq!(resp.tools_executed.len(), 1);
    }
    
    #[tokio::test]
    async fn test_conversation_history() {
        let tool_registry = Arc::new(ToolRegistry::new());
        let state_context = Arc::new(RwLock::new(StateContext::new()));
        
        let oracle = AIOracle::new(
            tool_registry,
            state_context,
            None,
            None,
        );
        
        oracle.add_to_history("user", "Hello", None).await;
        oracle.add_to_history("assistant", "Hi! How can I help?", Some("greeting")).await;
        
        let history = oracle.get_recent_history().await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, "user");
        assert_eq!(history[1].role, "assistant");
    }
}
