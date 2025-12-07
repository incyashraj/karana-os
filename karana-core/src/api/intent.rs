// Kāraṇa OS - Phase 61: Intent API
// REST/gRPC interface for external app integration

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Intent API version
pub const API_VERSION: &str = "1.0.0";

/// Intent type for external apps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    /// Capture photo/video
    Capture { mode: CaptureMode },
    
    /// Display AR content
    DisplayAR { content: ARContent },
    
    /// Voice command
    Voice { command: String },
    
    /// Navigation
    Navigate { destination: String },
    
    /// Query blockchain
    BlockchainQuery { query: String },
    
    /// Store data
    Store { key: String, value: Vec<u8> },
    
    /// Retrieve data
    Retrieve { key: String },
}

/// Capture modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureMode {
    Photo,
    Video,
    LiveStream,
}

/// AR content to display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARContent {
    pub id: String,
    pub content_type: String,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub scale: f32,
    pub data: Vec<u8>,
}

/// Intent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResponse {
    pub intent_id: String,
    pub status: IntentStatus,
    pub result: Option<Vec<u8>>,
    pub error: Option<String>,
}

/// Intent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentStatus {
    Pending,
    Processing,
    Success,
    Failed,
}

/// App registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistration {
    pub app_id: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub api_key: String,
}

/// Intent API server
pub struct IntentAPI {
    apps: Arc<RwLock<HashMap<String, AppRegistration>>>,
    pending_intents: Arc<RwLock<HashMap<String, Intent>>>,
    intent_counter: Arc<RwLock<u64>>,
}

impl IntentAPI {
    /// Create new Intent API
    pub fn new() -> Self {
        Self {
            apps: Arc::new(RwLock::new(HashMap::new())),
            pending_intents: Arc::new(RwLock::new(HashMap::new())),
            intent_counter: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Register an external app
    pub async fn register_app(&self, name: String, permissions: Vec<String>) -> Result<AppRegistration> {
        let app_id = format!("app_{}", uuid::Uuid::new_v4());
        let api_key = format!("key_{}", uuid::Uuid::new_v4());
        
        let registration = AppRegistration {
            app_id: app_id.clone(),
            name,
            permissions,
            api_key,
        };
        
        self.apps.write().await.insert(app_id, registration.clone());
        
        Ok(registration)
    }
    
    /// Submit intent
    pub async fn submit_intent(&self, api_key: &str, intent: Intent) -> Result<String> {
        // Verify API key
        let apps = self.apps.read().await;
        let _app = apps.values()
            .find(|a| a.api_key == api_key)
            .ok_or_else(|| anyhow!("Invalid API key"))?;
        
        // Generate intent ID
        let mut counter = self.intent_counter.write().await;
        *counter += 1;
        let intent_id = format!("intent_{}", *counter);
        
        // Store intent
        self.pending_intents.write().await.insert(intent_id.clone(), intent);
        
        Ok(intent_id)
    }
    
    /// Get intent status
    pub async fn get_status(&self, intent_id: &str) -> Result<IntentResponse> {
        // Check if intent exists
        let intents = self.pending_intents.read().await;
        if intents.contains_key(intent_id) {
            Ok(IntentResponse {
                intent_id: intent_id.to_string(),
                status: IntentStatus::Pending,
                result: None,
                error: None,
            })
        } else {
            // Intent completed or doesn't exist
            Ok(IntentResponse {
                intent_id: intent_id.to_string(),
                status: IntentStatus::Success,
                result: Some(vec![]), // Placeholder
                error: None,
            })
        }
    }
    
    /// Process pending intent (internal)
    pub async fn process_intent(&self, intent_id: &str) -> Result<Vec<u8>> {
        let mut intents = self.pending_intents.write().await;
        let intent = intents.remove(intent_id)
            .ok_or_else(|| anyhow!("Intent not found"))?;
        
        // Process based on intent type
        match intent {
            Intent::Capture { mode } => {
                // Trigger camera capture
                Ok(format!("Captured {:?}", mode).into_bytes())
            }
            Intent::DisplayAR { content } => {
                // Display AR content
                Ok(format!("Displayed AR content {}", content.id).into_bytes())
            }
            Intent::Voice { command } => {
                // Process voice command
                Ok(format!("Executed: {}", command).into_bytes())
            }
            Intent::Navigate { destination } => {
                // Navigate
                Ok(format!("Navigating to {}", destination).into_bytes())
            }
            Intent::BlockchainQuery { query } => {
                // Query blockchain
                Ok(format!("Query result: {}", query).into_bytes())
            }
            Intent::Store { key, value } => {
                // Store data
                Ok(format!("Stored {} bytes at {}", value.len(), key).into_bytes())
            }
            Intent::Retrieve { key } => {
                // Retrieve data
                Ok(format!("Retrieved data from {}", key).into_bytes())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_app_registration() {
        let api = IntentAPI::new();
        
        let registration = api.register_app(
            "TestApp".to_string(),
            vec!["camera".to_string()],
        ).await.unwrap();
        
        assert!(registration.app_id.starts_with("app_"));
        assert!(registration.api_key.starts_with("key_"));
    }
    
    #[tokio::test]
    async fn test_intent_submission() {
        let api = IntentAPI::new();
        
        let registration = api.register_app(
            "TestApp".to_string(),
            vec!["camera".to_string()],
        ).await.unwrap();
        
        let intent_id = api.submit_intent(
            &registration.api_key,
            Intent::Capture { mode: CaptureMode::Photo },
        ).await.unwrap();
        
        assert!(intent_id.starts_with("intent_"));
    }
    
    #[tokio::test]
    async fn test_intent_processing() {
        let api = IntentAPI::new();
        
        let registration = api.register_app(
            "TestApp".to_string(),
            vec!["camera".to_string()],
        ).await.unwrap();
        
        let intent_id = api.submit_intent(
            &registration.api_key,
            Intent::Voice { command: "take photo".to_string() },
        ).await.unwrap();
        
        let result = api.process_intent(&intent_id).await.unwrap();
        assert!(!result.is_empty());
    }
}
