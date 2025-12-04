//! Notification Channels for Kāraṇa OS AR Glasses
//!
//! Channels allow apps to categorize their notifications
//! and let users configure per-channel settings.

use std::time::Duration;

/// Channel importance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelImportance {
    /// Minimum importance - no visual/audio
    Min,
    /// Low importance - collapsed by default
    Low,
    /// Default importance - standard display
    Default,
    /// High importance - expanded, sound
    High,
    /// Urgent importance - full screen, sound, vibration
    Urgent,
}

impl ChannelImportance {
    /// Does this importance show a heads-up display?
    pub fn shows_heads_up(&self) -> bool {
        *self >= ChannelImportance::High
    }
    
    /// Does this importance make sound?
    pub fn makes_sound(&self) -> bool {
        *self >= ChannelImportance::Default
    }
    
    /// Does this importance vibrate?
    pub fn vibrates(&self) -> bool {
        *self >= ChannelImportance::High
    }
    
    /// Does this show in status?
    pub fn shows_in_status(&self) -> bool {
        *self >= ChannelImportance::Low
    }
}

/// Notification channel configuration
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Show badge on app icon
    pub show_badge: bool,
    /// Allow sound
    pub allow_sound: bool,
    /// Allow vibration
    pub allow_vibration: bool,
    /// Allow lights/LED
    pub allow_lights: bool,
    /// Show on lock screen
    pub show_on_lock_screen: bool,
    /// Allow full-screen intent
    pub allow_full_screen: bool,
    /// Custom sound URI
    pub sound_uri: Option<String>,
    /// Vibration pattern (durations in ms)
    pub vibration_pattern: Option<Vec<u64>>,
    /// Light color (RGB)
    pub light_color: Option<(u8, u8, u8)>,
    /// Bypass DND
    pub bypass_dnd: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            show_badge: true,
            allow_sound: true,
            allow_vibration: true,
            allow_lights: true,
            show_on_lock_screen: true,
            allow_full_screen: false,
            sound_uri: None,
            vibration_pattern: None,
            light_color: None,
            bypass_dnd: false,
        }
    }
}

impl ChannelConfig {
    /// Config for silent channel
    pub fn silent() -> Self {
        Self {
            show_badge: true,
            allow_sound: false,
            allow_vibration: false,
            allow_lights: false,
            show_on_lock_screen: false,
            allow_full_screen: false,
            sound_uri: None,
            vibration_pattern: None,
            light_color: None,
            bypass_dnd: false,
        }
    }
    
    /// Config for urgent channel
    pub fn urgent() -> Self {
        Self {
            show_badge: true,
            allow_sound: true,
            allow_vibration: true,
            allow_lights: true,
            show_on_lock_screen: true,
            allow_full_screen: true,
            sound_uri: None,
            vibration_pattern: Some(vec![0, 200, 100, 200, 100, 200]),
            light_color: Some((255, 0, 0)),
            bypass_dnd: true,
        }
    }
}

/// Notification channel
#[derive(Debug, Clone)]
pub struct NotificationChannel {
    /// Channel ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Importance level
    pub importance: ChannelImportance,
    /// Configuration
    pub config: ChannelConfig,
    /// Is channel enabled by user
    pub enabled: bool,
    /// Is blocked by user
    pub blocked: bool,
    /// Group ID (for channel groups)
    pub group_id: Option<String>,
    /// Conversation shortcut ID (for conversation-specific channels)
    pub conversation_id: Option<String>,
}

impl NotificationChannel {
    /// Create new channel
    pub fn new(id: &str, name: &str, importance: ChannelImportance) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            importance,
            config: ChannelConfig::default(),
            enabled: true,
            blocked: false,
            group_id: None,
            conversation_id: None,
        }
    }
    
    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Set config
    pub fn with_config(mut self, config: ChannelConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Set group
    pub fn in_group(mut self, group_id: &str) -> Self {
        self.group_id = Some(group_id.to_string());
        self
    }
    
    /// Check if notifications should be shown
    pub fn should_show(&self) -> bool {
        self.enabled && !self.blocked
    }
    
    /// Check if should make sound
    pub fn should_sound(&self) -> bool {
        self.should_show() && 
        self.config.allow_sound && 
        self.importance.makes_sound()
    }
    
    /// Check if should vibrate
    pub fn should_vibrate(&self) -> bool {
        self.should_show() &&
        self.config.allow_vibration &&
        self.importance.vibrates()
    }
    
    /// Check if should show heads-up
    pub fn should_heads_up(&self) -> bool {
        self.should_show() && self.importance.shows_heads_up()
    }
}

/// Channel group for organizing channels
#[derive(Debug, Clone)]
pub struct ChannelGroup {
    /// Group ID
    pub id: String,
    /// Group name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Is blocked
    pub blocked: bool,
}

impl ChannelGroup {
    /// Create new channel group
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            blocked: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_channel_creation() {
        let channel = NotificationChannel::new("test", "Test Channel", ChannelImportance::Default);
        assert_eq!(channel.id, "test");
        assert!(channel.enabled);
    }
    
    #[test]
    fn test_channel_importance() {
        assert!(ChannelImportance::Urgent > ChannelImportance::Low);
        assert!(ChannelImportance::High.shows_heads_up());
        assert!(!ChannelImportance::Low.shows_heads_up());
    }
    
    #[test]
    fn test_channel_should_show() {
        let mut channel = NotificationChannel::new("test", "Test", ChannelImportance::Default);
        assert!(channel.should_show());
        
        channel.blocked = true;
        assert!(!channel.should_show());
    }
    
    #[test]
    fn test_channel_sound() {
        let channel = NotificationChannel::new("test", "Test", ChannelImportance::Default);
        assert!(channel.should_sound());
        
        let low_channel = NotificationChannel::new("low", "Low", ChannelImportance::Low);
        assert!(!low_channel.should_sound());
    }
    
    #[test]
    fn test_silent_config() {
        let config = ChannelConfig::silent();
        assert!(!config.allow_sound);
        assert!(!config.allow_vibration);
    }
    
    #[test]
    fn test_urgent_config() {
        let config = ChannelConfig::urgent();
        assert!(config.bypass_dnd);
        assert!(config.allow_full_screen);
    }
    
    #[test]
    fn test_channel_group() {
        let group = ChannelGroup::new("social", "Social");
        let channel = NotificationChannel::new("messages", "Messages", ChannelImportance::High)
            .in_group(&group.id);
        
        assert_eq!(channel.group_id.as_deref(), Some("social"));
    }
}
