//! Oracle Actions - Executable actions the Oracle can trigger

use serde::{Serialize, Deserialize};
use super::OracleIntent;

/// An action that the Oracle wants to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAction {
    pub action_type: ActionType,
    pub parameters: serde_json::Value,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // Blockchain
    SignTransaction,
    QueryBalance,
    QueryHistory,
    
    // Apps
    LaunchApp,
    CloseApp,
    FocusApp,
    
    // Media
    PlayMedia,
    PauseMedia,
    SeekMedia,
    
    // System
    SetMode,
    UpdateSettings,
    Notify,
    
    // Vision
    CaptureFrame,
    AnalyzeFrame,
    
    // Navigation
    StartNavigation,
    ShowMap,
}

impl OracleAction {
    /// Create an action from an intent
    pub fn from_intent(intent: &OracleIntent) -> Option<Self> {
        match intent {
            OracleIntent::Transfer { amount, recipient, memo } => {
                Some(Self {
                    action_type: ActionType::SignTransaction,
                    parameters: serde_json::json!({
                        "type": "transfer",
                        "amount": amount,
                        "recipient": recipient,
                        "memo": memo,
                    }),
                    requires_auth: true,
                })
            }
            
            OracleIntent::CheckBalance => {
                Some(Self {
                    action_type: ActionType::QueryBalance,
                    parameters: serde_json::json!({}),
                    requires_auth: false,
                })
            }
            
            OracleIntent::OpenBrowser { url } => {
                Some(Self {
                    action_type: ActionType::LaunchApp,
                    parameters: serde_json::json!({
                        "app": "browser",
                        "url": url,
                    }),
                    requires_auth: false,
                })
            }
            
            OracleIntent::PlayVideo { query, url } => {
                Some(Self {
                    action_type: ActionType::LaunchApp,
                    parameters: serde_json::json!({
                        "app": "video",
                        "query": query,
                        "url": url,
                    }),
                    requires_auth: false,
                })
            }
            
            OracleIntent::TakeNote { content } => {
                Some(Self {
                    action_type: ActionType::LaunchApp,
                    parameters: serde_json::json!({
                        "app": "notes",
                        "content": content,
                    }),
                    requires_auth: false,
                })
            }
            
            OracleIntent::Navigate { destination } => {
                Some(Self {
                    action_type: ActionType::StartNavigation,
                    parameters: serde_json::json!({
                        "destination": destination,
                    }),
                    requires_auth: false,
                })
            }
            
            OracleIntent::AnalyzeVision => {
                Some(Self {
                    action_type: ActionType::CaptureFrame,
                    parameters: serde_json::json!({}),
                    requires_auth: false,
                })
            }
            
            OracleIntent::CloseApp { app_id } => {
                Some(Self {
                    action_type: ActionType::CloseApp,
                    parameters: serde_json::json!({
                        "app_id": app_id,
                    }),
                    requires_auth: false,
                })
            }
            
            _ => None,
        }
    }
}
