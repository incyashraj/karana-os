//! Oracle Context - State awareness for the Oracle

use serde::{Serialize, Deserialize};
use chrono::Timelike;

/// Context information for the Oracle to make better decisions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OracleContext {
    /// Currently visible object from vision system
    pub vision_object: Option<String>,
    /// Vision confidence score
    pub vision_confidence: Option<f32>,
    /// Current location (if available)
    pub location: Option<String>,
    /// User's current activity
    pub activity: Option<String>,
    /// Current time of day
    pub time_of_day: String,
    /// Currently open apps
    pub open_apps: Vec<String>,
    /// Active AR window ID
    pub active_window: Option<String>,
    /// User's wallet balance
    pub wallet_balance: u64,
    /// Recent transaction count
    pub recent_tx_count: u32,
    /// Current app in focus
    pub current_app: Option<String>,
}

impl OracleContext {
    pub fn new() -> Self {
        Self {
            time_of_day: Self::get_time_of_day(),
            ..Default::default()
        }
    }
    
    pub fn with_vision(mut self, object: String, confidence: f32) -> Self {
        self.vision_object = Some(object);
        self.vision_confidence = Some(confidence);
        self
    }
    
    pub fn with_balance(mut self, balance: u64) -> Self {
        self.wallet_balance = balance;
        self
    }
    
    pub fn with_open_apps(mut self, apps: Vec<String>) -> Self {
        self.open_apps = apps;
        self
    }
    
    fn get_time_of_day() -> String {
        let hour = chrono::Local::now().hour();
        match hour {
            5..=11 => "morning".to_string(),
            12..=17 => "afternoon".to_string(),
            18..=21 => "evening".to_string(),
            _ => "night".to_string(),
        }
    }
}
