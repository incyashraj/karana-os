//! Settings & Preferences System for Kāraṇa OS AR Glasses
//!
//! Comprehensive user preferences management with persistence,
//! profiles, import/export, and real-time synchronization.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

pub mod schema;
pub mod storage;
pub mod profiles;
pub mod sync;
pub mod migration;

pub use schema::{SettingValue, SettingSchema, ValidationError};
pub use storage::{SettingsStorage, StorageBackend};
pub use profiles::{UserProfile, ProfileManager};
pub use sync::{SettingsSync, SyncStatus};
pub use migration::{MigrationRunner, MigrationVersion};

/// Settings change event
#[derive(Debug, Clone)]
pub struct SettingChange {
    /// Setting key path
    pub key: String,
    /// Previous value
    pub old_value: Option<SettingValue>,
    /// New value
    pub new_value: SettingValue,
    /// When changed
    pub timestamp: Instant,
    /// Source of change
    pub source: ChangeSource,
}

/// Source of a setting change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeSource {
    /// User changed directly
    User,
    /// Profile applied
    Profile,
    /// Cloud sync
    CloudSync,
    /// App request
    App,
    /// System default
    System,
    /// Migration
    Migration,
}

/// Settings category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsCategory {
    /// Display settings
    Display,
    /// Audio settings
    Audio,
    /// Input settings
    Input,
    /// Privacy settings
    Privacy,
    /// Network settings
    Network,
    /// Accessibility settings
    Accessibility,
    /// Power settings
    Power,
    /// Notification settings
    Notifications,
    /// AR/Spatial settings
    Spatial,
    /// Voice settings
    Voice,
    /// Haptic settings
    Haptics,
    /// System settings
    System,
}

/// Settings change listener
pub type SettingsListener = Box<dyn Fn(&SettingChange) + Send + Sync>;

/// Settings engine
#[derive(Debug)]
pub struct SettingsEngine {
    /// Current settings values
    values: HashMap<String, SettingValue>,
    /// Setting schemas
    schemas: HashMap<String, SettingSchema>,
    /// Storage backend
    storage: SettingsStorage,
    /// Profile manager
    profiles: ProfileManager,
    /// Cloud sync manager
    sync: Option<SettingsSync>,
    /// Change listeners
    listeners: Vec<(String, usize)>, // (pattern, listener_id)
    /// Next listener ID
    next_listener_id: usize,
    /// Pending changes for batch
    pending_changes: Vec<SettingChange>,
    /// Batch mode active
    batch_mode: bool,
    /// Last save time
    last_save: Instant,
    /// Auto-save interval
    auto_save_interval: Duration,
    /// Dirty flag
    dirty: bool,
}

impl SettingsEngine {
    /// Create new settings engine
    pub fn new() -> Self {
        let mut engine = Self {
            values: HashMap::new(),
            schemas: HashMap::new(),
            storage: SettingsStorage::new(),
            profiles: ProfileManager::new(),
            sync: None,
            listeners: Vec::new(),
            next_listener_id: 0,
            pending_changes: Vec::new(),
            batch_mode: false,
            last_save: Instant::now(),
            auto_save_interval: Duration::from_secs(30),
            dirty: false,
        };
        
        engine.register_default_schemas();
        engine.apply_defaults();
        engine
    }
    
    /// Register default setting schemas
    fn register_default_schemas(&mut self) {
        // Display settings
        self.register_schema("display.brightness", SettingSchema::float(0.0, 1.0, 0.7));
        self.register_schema("display.contrast", SettingSchema::float(0.5, 2.0, 1.0));
        self.register_schema("display.color_temperature", SettingSchema::int(2700, 6500, 5000));
        self.register_schema("display.night_mode", SettingSchema::bool(false));
        self.register_schema("display.auto_brightness", SettingSchema::bool(true));
        self.register_schema("display.hud_opacity", SettingSchema::float(0.3, 1.0, 0.8));
        self.register_schema("display.text_scale", SettingSchema::float(0.5, 3.0, 1.0));
        
        // Audio settings
        self.register_schema("audio.volume", SettingSchema::float(0.0, 1.0, 0.7));
        self.register_schema("audio.spatial_enabled", SettingSchema::bool(true));
        self.register_schema("audio.mic_volume", SettingSchema::float(0.0, 1.0, 0.8));
        self.register_schema("audio.voice_activity_detection", SettingSchema::bool(true));
        self.register_schema("audio.balance", SettingSchema::float(-1.0, 1.0, 0.0));
        
        // Input settings
        self.register_schema("input.gaze_sensitivity", SettingSchema::float(0.1, 2.0, 1.0));
        self.register_schema("input.gesture_sensitivity", SettingSchema::float(0.1, 2.0, 1.0));
        self.register_schema("input.dwell_enabled", SettingSchema::bool(false));
        self.register_schema("input.dwell_time_ms", SettingSchema::int(500, 3000, 1000));
        self.register_schema("input.voice_wake_word", SettingSchema::string("hey karana".to_string()));
        
        // Privacy settings
        self.register_schema("privacy.camera_enabled", SettingSchema::bool(true));
        self.register_schema("privacy.location_enabled", SettingSchema::bool(true));
        self.register_schema("privacy.recording_indicator", SettingSchema::bool(true));
        self.register_schema("privacy.data_collection", SettingSchema::bool(false));
        self.register_schema("privacy.face_blur", SettingSchema::bool(true));
        
        // Notification settings
        self.register_schema("notifications.enabled", SettingSchema::bool(true));
        self.register_schema("notifications.sound", SettingSchema::bool(true));
        self.register_schema("notifications.haptic", SettingSchema::bool(true));
        self.register_schema("notifications.visual", SettingSchema::bool(true));
        self.register_schema("notifications.do_not_disturb", SettingSchema::bool(false));
        self.register_schema("notifications.priority_only", SettingSchema::bool(false));
        
        // Power settings
        self.register_schema("power.profile", SettingSchema::string("balanced".to_string()));
        self.register_schema("power.auto_sleep_minutes", SettingSchema::int(1, 60, 5));
        self.register_schema("power.battery_saver_threshold", SettingSchema::int(5, 50, 20));
        
        // Accessibility settings
        self.register_schema("accessibility.screen_reader", SettingSchema::bool(false));
        self.register_schema("accessibility.magnifier", SettingSchema::bool(false));
        self.register_schema("accessibility.high_contrast", SettingSchema::bool(false));
        self.register_schema("accessibility.reduce_motion", SettingSchema::bool(false));
        self.register_schema("accessibility.color_correction", SettingSchema::string("none".to_string()));
        
        // Spatial AR settings
        self.register_schema("spatial.persistence_enabled", SettingSchema::bool(true));
        self.register_schema("spatial.anchor_sharing", SettingSchema::bool(true));
        self.register_schema("spatial.plane_detection", SettingSchema::bool(true));
        self.register_schema("spatial.mesh_visualization", SettingSchema::bool(false));
        
        // Voice settings
        self.register_schema("voice.language", SettingSchema::string("en-US".to_string()));
        self.register_schema("voice.speech_rate", SettingSchema::int(80, 300, 150));
        self.register_schema("voice.voice_id", SettingSchema::string("default".to_string()));
    }
    
    /// Register a setting schema
    pub fn register_schema(&mut self, key: &str, schema: SettingSchema) {
        self.schemas.insert(key.to_string(), schema);
    }
    
    /// Apply default values
    fn apply_defaults(&mut self) {
        for (key, schema) in &self.schemas {
            if !self.values.contains_key(key) {
                self.values.insert(key.clone(), schema.default_value.clone());
            }
        }
    }
    
    /// Get a setting value
    pub fn get(&self, key: &str) -> Option<&SettingValue> {
        self.values.get(key)
    }
    
    /// Get a setting as bool
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }
    
    /// Get a setting as int
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_int())
    }
    
    /// Get a setting as float
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_float())
    }
    
    /// Get a setting as string
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_string())
    }
    
    /// Set a setting value
    pub fn set(&mut self, key: &str, value: SettingValue) -> Result<(), ValidationError> {
        self.set_with_source(key, value, ChangeSource::User)
    }
    
    /// Set a setting with specific source
    pub fn set_with_source(&mut self, key: &str, value: SettingValue, source: ChangeSource) -> Result<(), ValidationError> {
        // Validate against schema
        if let Some(schema) = self.schemas.get(key) {
            schema.validate(&value)?;
        }
        
        let old_value = self.values.get(key).cloned();
        
        let change = SettingChange {
            key: key.to_string(),
            old_value,
            new_value: value.clone(),
            timestamp: Instant::now(),
            source,
        };
        
        self.values.insert(key.to_string(), value);
        self.dirty = true;
        
        if self.batch_mode {
            self.pending_changes.push(change);
        } else {
            self.notify_change(&change);
        }
        
        Ok(())
    }
    
    /// Set bool value
    pub fn set_bool(&mut self, key: &str, value: bool) -> Result<(), ValidationError> {
        self.set(key, SettingValue::Bool(value))
    }
    
    /// Set int value
    pub fn set_int(&mut self, key: &str, value: i64) -> Result<(), ValidationError> {
        self.set(key, SettingValue::Int(value))
    }
    
    /// Set float value
    pub fn set_float(&mut self, key: &str, value: f64) -> Result<(), ValidationError> {
        self.set(key, SettingValue::Float(value))
    }
    
    /// Set string value
    pub fn set_string(&mut self, key: &str, value: &str) -> Result<(), ValidationError> {
        self.set(key, SettingValue::String(value.to_string()))
    }
    
    /// Start batch mode (defer notifications)
    pub fn begin_batch(&mut self) {
        self.batch_mode = true;
        self.pending_changes.clear();
    }
    
    /// End batch mode and notify all changes
    pub fn end_batch(&mut self) {
        self.batch_mode = false;
        let changes: Vec<_> = self.pending_changes.drain(..).collect();
        for change in changes {
            self.notify_change(&change);
        }
    }
    
    /// Notify listeners of a change
    fn notify_change(&self, _change: &SettingChange) {
        // In real implementation, this would call registered listeners
        // For now, this is a placeholder for the notification system
    }
    
    /// Reset a setting to default
    pub fn reset(&mut self, key: &str) -> Result<(), ValidationError> {
        if let Some(schema) = self.schemas.get(key) {
            self.set(key, schema.default_value.clone())
        } else {
            Err(ValidationError::UnknownSetting(key.to_string()))
        }
    }
    
    /// Reset all settings to defaults
    pub fn reset_all(&mut self) {
        self.begin_batch();
        
        for (key, schema) in &self.schemas.clone() {
            let _ = self.set_with_source(key, schema.default_value.clone(), ChangeSource::System);
        }
        
        self.end_batch();
    }
    
    /// Get all settings in a category
    pub fn get_category(&self, category: SettingsCategory) -> HashMap<String, &SettingValue> {
        let prefix = category_prefix(category);
        self.values
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, v)| (k.clone(), v))
            .collect()
    }
    
    /// Check if settings are dirty (unsaved)
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    /// Save settings to storage
    pub fn save(&mut self) -> Result<(), std::io::Error> {
        self.storage.save(&self.values)?;
        self.dirty = false;
        self.last_save = Instant::now();
        Ok(())
    }
    
    /// Load settings from storage
    pub fn load(&mut self) -> Result<(), std::io::Error> {
        let loaded = self.storage.load()?;
        
        self.begin_batch();
        for (key, value) in loaded {
            let _ = self.set_with_source(&key, value, ChangeSource::System);
        }
        self.end_batch();
        
        self.dirty = false;
        Ok(())
    }
    
    /// Update (auto-save if needed)
    pub fn update(&mut self) {
        if self.dirty && self.last_save.elapsed() > self.auto_save_interval {
            let _ = self.save();
        }
    }
    
    /// Get profile manager
    pub fn profiles(&self) -> &ProfileManager {
        &self.profiles
    }
    
    /// Get mutable profile manager
    pub fn profiles_mut(&mut self) -> &mut ProfileManager {
        &mut self.profiles
    }
    
    /// Apply a profile
    pub fn apply_profile(&mut self, profile_name: &str) -> Result<(), String> {
        // Clone the settings to avoid borrow issues
        let profile_settings = if let Some(profile) = self.profiles.get_profile(profile_name) {
            profile.settings.clone()
        } else {
            return Err(format!("Profile not found: {}", profile_name));
        };
        
        self.begin_batch();
        
        for (key, value) in profile_settings {
            let _ = self.set_with_source(&key, value, ChangeSource::Profile);
        }
        
        self.end_batch();
        Ok(())
    }
    
    /// Export settings to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.values)
    }
    
    /// Import settings from JSON
    pub fn import_json(&mut self, json: &str) -> Result<usize, serde_json::Error> {
        let imported: HashMap<String, SettingValue> = serde_json::from_str(json)?;
        let count = imported.len();
        
        self.begin_batch();
        for (key, value) in imported {
            let _ = self.set_with_source(&key, value, ChangeSource::User);
        }
        self.end_batch();
        
        Ok(count)
    }
    
    /// List all setting keys
    pub fn list_keys(&self) -> Vec<&str> {
        self.values.keys().map(|s| s.as_str()).collect()
    }
    
    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

/// Get category prefix
fn category_prefix(category: SettingsCategory) -> String {
    match category {
        SettingsCategory::Display => "display.".to_string(),
        SettingsCategory::Audio => "audio.".to_string(),
        SettingsCategory::Input => "input.".to_string(),
        SettingsCategory::Privacy => "privacy.".to_string(),
        SettingsCategory::Network => "network.".to_string(),
        SettingsCategory::Accessibility => "accessibility.".to_string(),
        SettingsCategory::Power => "power.".to_string(),
        SettingsCategory::Notifications => "notifications.".to_string(),
        SettingsCategory::Spatial => "spatial.".to_string(),
        SettingsCategory::Voice => "voice.".to_string(),
        SettingsCategory::Haptics => "haptics.".to_string(),
        SettingsCategory::System => "system.".to_string(),
    }
}

impl Default for SettingsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let engine = SettingsEngine::new();
        assert!(engine.contains("display.brightness"));
        assert!(engine.contains("audio.volume"));
    }
    
    #[test]
    fn test_get_set() {
        let mut engine = SettingsEngine::new();
        
        // Get default
        assert_eq!(engine.get_float("display.brightness"), Some(0.7));
        
        // Set new value
        assert!(engine.set_float("display.brightness", 0.9).is_ok());
        assert_eq!(engine.get_float("display.brightness"), Some(0.9));
    }
    
    #[test]
    fn test_validation() {
        let mut engine = SettingsEngine::new();
        
        // Valid value
        assert!(engine.set_float("display.brightness", 0.5).is_ok());
        
        // Invalid value (out of range)
        assert!(engine.set_float("display.brightness", 1.5).is_err());
    }
    
    #[test]
    fn test_reset() {
        let mut engine = SettingsEngine::new();
        
        engine.set_float("display.brightness", 0.5).unwrap();
        engine.reset("display.brightness").unwrap();
        
        assert_eq!(engine.get_float("display.brightness"), Some(0.7));
    }
    
    #[test]
    fn test_category_access() {
        let engine = SettingsEngine::new();
        
        let display_settings = engine.get_category(SettingsCategory::Display);
        assert!(display_settings.len() > 0);
        
        for (key, _) in display_settings {
            assert!(key.starts_with("display."));
        }
    }
    
    #[test]
    fn test_bool_settings() {
        let mut engine = SettingsEngine::new();
        
        assert!(engine.set_bool("display.night_mode", true).is_ok());
        assert_eq!(engine.get_bool("display.night_mode"), Some(true));
    }
    
    #[test]
    fn test_string_settings() {
        let mut engine = SettingsEngine::new();
        
        assert!(engine.set_string("voice.language", "fr-FR").is_ok());
        assert_eq!(engine.get_string("voice.language"), Some("fr-FR"));
    }
    
    #[test]
    fn test_dirty_flag() {
        let mut engine = SettingsEngine::new();
        
        assert!(!engine.is_dirty());
        engine.set_float("display.brightness", 0.5).unwrap();
        assert!(engine.is_dirty());
    }
}
