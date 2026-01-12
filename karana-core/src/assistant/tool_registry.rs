// Kāraṇa OS - Voice Tool Registry
// Structured tool execution system for voice commands

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use tokio::sync::Mutex;

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub confidence: f32,
    pub execution_id: String,
    pub tool_name: String,
    pub undo_state: Option<UndoState>,
    pub timestamp: u64,
}

/// State needed to undo a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoState {
    pub tool_name: String,
    pub previous_value: serde_json::Value,
    pub operation: String,
}

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: String,
    pub required: bool,
    pub default: Option<String>,
}

/// Tool arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArgs {
    pub params: HashMap<String, serde_json::Value>,
}

impl ToolArgs {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn add<T: Serialize>(&mut self, name: &str, value: T) {
        self.params.insert(
            name.to_string(),
            serde_json::to_value(value).unwrap(),
        );
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        self.params.get(name)?.as_str().map(|s| s.to_string())
    }

    pub fn get_f64(&self, name: &str) -> Option<f64> {
        self.params.get(name)?.as_f64()
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.params.get(name)?.as_bool()
    }
}

impl Default for ToolArgs {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool trait - implement this for each voice-activated tool
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (used for routing)
    fn name(&self) -> &str;
    
    /// Human-readable description
    fn description(&self) -> &str;
    
    /// Parameter definitions
    fn parameters(&self) -> Vec<ToolParameter>;
    
    /// Execute the tool
    async fn execute(&self, args: ToolArgs) -> Result<ToolResult>;
    
    /// Undo the last execution (optional)
    async fn undo(&self, _undo_state: &UndoState) -> Result<ToolResult> {
        Err(anyhow!("Undo not supported for this tool"))
    }
    
    /// Check if tool supports undo
    fn supports_undo(&self) -> bool {
        false
    }
}

/// Tool execution history entry
#[derive(Debug, Clone)]
struct ExecutionHistoryEntry {
    execution_id: String,
    tool_name: String,
    result: ToolResult,
    timestamp: u64,
}

/// Tool registry - manages all available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    execution_history: Arc<Mutex<Vec<ExecutionHistoryEntry>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            execution_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Execute a tool by name
    pub async fn execute(&self, tool_name: &str, args: ToolArgs) -> Result<ToolResult> {
        let tool = self.get_tool(tool_name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", tool_name))?;

        let result = tool.execute(args).await?;

        // Record in history
        let entry = ExecutionHistoryEntry {
            execution_id: result.execution_id.clone(),
            tool_name: tool_name.to_string(),
            result: result.clone(),
            timestamp: result.timestamp,
        };
        self.execution_history.lock().await.push(entry);

        Ok(result)
    }

    /// Undo last execution
    pub async fn undo_last(&self) -> Result<ToolResult> {
        let mut history = self.execution_history.lock().await;
        
        let last = history.pop()
            .ok_or_else(|| anyhow!("No executions to undo"))?;

        let tool = self.get_tool(&last.tool_name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", last.tool_name))?;

        if let Some(undo_state) = &last.result.undo_state {
            tool.undo(undo_state).await
        } else {
            Err(anyhow!("Tool '{}' does not support undo", last.tool_name))
        }
    }

    /// Get execution history
    pub async fn get_history(&self, limit: usize) -> Vec<ToolResult> {
        let history = self.execution_history.lock().await;
        history.iter()
            .rev()
            .take(limit)
            .map(|e| e.result.clone())
            .collect()
    }

    /// Clear execution history
    pub async fn clear_history(&self) {
        self.execution_history.lock().await.clear();
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Core Tools Implementation
// ============================================================================

/// Navigate UI tool
pub struct NavigateTool;

#[async_trait]
impl Tool for NavigateTool {
    fn name(&self) -> &str {
        "navigate"
    }

    fn description(&self) -> &str {
        "Navigate to different screens or go back"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "destination".to_string(),
                description: "Where to navigate (home, settings, back, etc.)".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ]
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult> {
        let destination = args.get_string("destination")
            .ok_or_else(|| anyhow!("Missing destination parameter"))?;

        let execution_id = format!("nav_{}", uuid::Uuid::new_v4());
        
        log::info!("[TOOL] Navigate to: {}", destination);

        Ok(ToolResult {
            success: true,
            output: format!("Navigated to {}", destination),
            confidence: 1.0,
            execution_id,
            tool_name: self.name().to_string(),
            undo_state: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        })
    }
}

/// Launch app tool
pub struct LaunchAppTool;

#[async_trait]
impl Tool for LaunchAppTool {
    fn name(&self) -> &str {
        "launch_app"
    }

    fn description(&self) -> &str {
        "Launch an application (camera, wallet, browser, etc.)"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "app_name".to_string(),
                description: "Name of app to launch".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ]
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult> {
        let app_name = args.get_string("app_name")
            .ok_or_else(|| anyhow!("Missing app_name parameter"))?;

        let execution_id = format!("launch_{}", uuid::Uuid::new_v4());
        
        log::info!("[TOOL] Launching app: {}", app_name);

        Ok(ToolResult {
            success: true,
            output: format!("Launched {}", app_name),
            confidence: 1.0,
            execution_id,
            tool_name: self.name().to_string(),
            undo_state: Some(UndoState {
                tool_name: self.name().to_string(),
                previous_value: serde_json::json!({"app": app_name}),
                operation: "close_app".to_string(),
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        })
    }

    fn supports_undo(&self) -> bool {
        true
    }
}

/// Create task tool
pub struct CreateTaskTool {
    tasks: Arc<Mutex<Vec<String>>>,
}

impl CreateTaskTool {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl Tool for CreateTaskTool {
    fn name(&self) -> &str {
        "create_task"
    }

    fn description(&self) -> &str {
        "Create a new task or todo item"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "task".to_string(),
                description: "Task description".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ]
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult> {
        let task = args.get_string("task")
            .ok_or_else(|| anyhow!("Missing task parameter"))?;

        let execution_id = format!("task_{}", uuid::Uuid::new_v4());
        
        // Add to tasks
        let mut tasks = self.tasks.lock().await;
        let index = tasks.len();
        tasks.push(task.clone());

        log::info!("[TOOL] Created task: {}", task);

        Ok(ToolResult {
            success: true,
            output: format!("✓ Created task: \"{}\"", task),
            confidence: 1.0,
            execution_id: execution_id.clone(),
            tool_name: self.name().to_string(),
            undo_state: Some(UndoState {
                tool_name: self.name().to_string(),
                previous_value: serde_json::json!({"index": index}),
                operation: "delete_task".to_string(),
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        })
    }

    async fn undo(&self, undo_state: &UndoState) -> Result<ToolResult> {
        let index = undo_state.previous_value["index"].as_u64()
            .ok_or_else(|| anyhow!("Invalid undo state"))? as usize;

        let mut tasks = self.tasks.lock().await;
        if index < tasks.len() {
            let task = tasks.remove(index);
            Ok(ToolResult {
                success: true,
                output: format!("⟲ Undone: Removed task \"{}\"", task),
                confidence: 1.0,
                execution_id: format!("undo_{}", uuid::Uuid::new_v4()),
                tool_name: self.name().to_string(),
                undo_state: None,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
            })
        } else {
            Err(anyhow!("Task index out of bounds"))
        }
    }

    fn supports_undo(&self) -> bool {
        true
    }
}

/// Weather query tool (uses existing weather service)
pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "Get current weather for a location"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "location".to_string(),
                description: "City or location (use 'current' for auto-detect)".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: Some("current".to_string()),
            },
        ]
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult> {
        let location = args.get_string("location")
            .unwrap_or_else(|| "current".to_string());

        let execution_id = format!("weather_{}", uuid::Uuid::new_v4());
        
        // TODO: Integrate with actual weather service
        // For now, return mock data
        let weather_info = format!("Weather in {}: 72°F, Sunny", location);

        log::info!("[TOOL] Weather query: {}", location);

        Ok(ToolResult {
            success: true,
            output: weather_info,
            confidence: 0.9,
            execution_id,
            tool_name: self.name().to_string(),
            undo_state: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        })
    }
}

/// Wallet balance tool
pub struct WalletTool;

#[async_trait]
impl Tool for WalletTool {
    fn name(&self) -> &str {
        "wallet"
    }

    fn description(&self) -> &str {
        "Check wallet balance or send tokens"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "action".to_string(),
                description: "Action: 'balance', 'send', or 'receive'".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: Some("balance".to_string()),
            },
        ]
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResult> {
        let action = args.get_string("action")
            .unwrap_or_else(|| "balance".to_string());

        let execution_id = format!("wallet_{}", uuid::Uuid::new_v4());
        
        let output = match action.as_str() {
            "balance" => "Your balance: 100.0 KARA tokens".to_string(),
            "send" => "Send transaction prepared".to_string(),
            "receive" => "Receive address: kara:1234...".to_string(),
            _ => "Unknown wallet action".to_string(),
        };

        log::info!("[TOOL] Wallet action: {}", action);

        Ok(ToolResult {
            success: true,
            output,
            confidence: 1.0,
            execution_id,
            tool_name: self.name().to_string(),
            undo_state: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        })
    }
}

/// Create a default tool registry with core tools
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    registry.register(NavigateTool);
    registry.register(LaunchAppTool);
    registry.register(CreateTaskTool::new());
    registry.register(WeatherTool);
    registry.register(WalletTool);
    
    log::info!("[TOOLS] Registered {} tools", registry.list_tools().len());
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(NavigateTool);
        
        assert_eq!(registry.list_tools().len(), 1);
        assert!(registry.get_tool("navigate").is_some());
    }

    #[tokio::test]
    async fn test_navigate_tool() {
        let tool = NavigateTool;
        let mut args = ToolArgs::new();
        args.add("destination", "home");
        
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("home"));
    }

    #[tokio::test]
    async fn test_create_task_tool() {
        let tool = CreateTaskTool::new();
        let mut args = ToolArgs::new();
        args.add("task", "Review PR");
        
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.supports_undo());
        assert!(result.undo_state.is_some());
    }

    #[tokio::test]
    async fn test_tool_execution_history() {
        let mut registry = ToolRegistry::new();
        registry.register(NavigateTool);
        
        let mut args = ToolArgs::new();
        args.add("destination", "settings");
        
        registry.execute("navigate", args).await.unwrap();
        
        let history = registry.get_history(10).await;
        assert_eq!(history.len(), 1);
    }
}
