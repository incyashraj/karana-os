//! User profiles for settings

use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::schema::SettingValue;

/// User profile
#[derive(Debug, Clone)]
pub struct UserProfile {
    /// Profile name
    pub name: String,
    /// Profile description
    pub description: String,
    /// Profile icon
    pub icon: Option<String>,
    /// Settings values
    pub settings: HashMap<String, SettingValue>,
    /// Creation time
    pub created_at: Instant,
    /// Last modified time
    pub modified_at: Instant,
    /// Whether this is a system profile
    pub is_system: bool,
    /// Whether profile is locked (cannot be deleted)
    pub is_locked: bool,
    /// Tags for organization
    pub tags: Vec<String>,
}

impl UserProfile {
    /// Create new user profile
    pub fn new(name: &str) -> Self {
        let now = Instant::now();
        Self {
            name: name.to_string(),
            description: String::new(),
            icon: None,
            settings: HashMap::new(),
            created_at: now,
            modified_at: now,
            is_system: false,
            is_locked: false,
            tags: Vec::new(),
        }
    }
    
    /// Create system profile
    pub fn system(name: &str, description: &str) -> Self {
        let now = Instant::now();
        Self {
            name: name.to_string(),
            description: description.to_string(),
            icon: None,
            settings: HashMap::new(),
            created_at: now,
            modified_at: now,
            is_system: true,
            is_locked: true,
            tags: Vec::new(),
        }
    }
    
    /// Set a setting value
    pub fn set(&mut self, key: &str, value: SettingValue) {
        self.settings.insert(key.to_string(), value);
        self.modified_at = Instant::now();
    }
    
    /// Get a setting value
    pub fn get(&self, key: &str) -> Option<&SettingValue> {
        self.settings.get(key)
    }
    
    /// Remove a setting
    pub fn remove(&mut self, key: &str) {
        self.settings.remove(key);
        self.modified_at = Instant::now();
    }
    
    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
    
    /// Set icon
    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }
    
    /// Add tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }
}

/// Profile manager
#[derive(Debug)]
pub struct ProfileManager {
    /// All profiles
    profiles: HashMap<String, UserProfile>,
    /// Active profile name
    active_profile: Option<String>,
    /// Default profile name
    default_profile: String,
    /// Max profiles allowed
    max_profiles: usize,
}

impl ProfileManager {
    /// Create new profile manager
    pub fn new() -> Self {
        let mut manager = Self {
            profiles: HashMap::new(),
            active_profile: None,
            default_profile: "default".to_string(),
            max_profiles: 20,
        };
        
        manager.create_default_profiles();
        manager
    }
    
    /// Create default system profiles
    fn create_default_profiles(&mut self) {
        // Default profile
        let mut default = UserProfile::system("default", "Default settings");
        default.settings.insert("display.brightness".to_string(), SettingValue::Float(0.7));
        default.settings.insert("audio.volume".to_string(), SettingValue::Float(0.7));
        self.profiles.insert("default".to_string(), default);
        
        // Night mode profile
        let mut night = UserProfile::system("night", "Reduced brightness for dark environments");
        night.settings.insert("display.brightness".to_string(), SettingValue::Float(0.3));
        night.settings.insert("display.night_mode".to_string(), SettingValue::Bool(true));
        night.settings.insert("display.color_temperature".to_string(), SettingValue::Int(2700));
        night.tags.push("lighting".to_string());
        self.profiles.insert("night".to_string(), night);
        
        // Outdoor profile
        let mut outdoor = UserProfile::system("outdoor", "High brightness for outdoor use");
        outdoor.settings.insert("display.brightness".to_string(), SettingValue::Float(1.0));
        outdoor.settings.insert("display.auto_brightness".to_string(), SettingValue::Bool(true));
        outdoor.settings.insert("audio.volume".to_string(), SettingValue::Float(0.9));
        outdoor.tags.push("environment".to_string());
        self.profiles.insert("outdoor".to_string(), outdoor);
        
        // Privacy profile
        let mut privacy = UserProfile::system("privacy", "Maximum privacy settings");
        privacy.settings.insert("privacy.camera_enabled".to_string(), SettingValue::Bool(false));
        privacy.settings.insert("privacy.location_enabled".to_string(), SettingValue::Bool(false));
        privacy.settings.insert("privacy.data_collection".to_string(), SettingValue::Bool(false));
        privacy.settings.insert("privacy.recording_indicator".to_string(), SettingValue::Bool(true));
        privacy.tags.push("privacy".to_string());
        self.profiles.insert("privacy".to_string(), privacy);
        
        // Battery saver profile
        let mut battery = UserProfile::system("battery_saver", "Extended battery life");
        battery.settings.insert("display.brightness".to_string(), SettingValue::Float(0.4));
        battery.settings.insert("power.profile".to_string(), SettingValue::String("power_saver".to_string()));
        battery.settings.insert("spatial.mesh_visualization".to_string(), SettingValue::Bool(false));
        battery.tags.push("power".to_string());
        self.profiles.insert("battery_saver".to_string(), battery);
        
        // Accessibility profile
        let mut access = UserProfile::system("accessibility", "Enhanced accessibility features");
        access.settings.insert("accessibility.high_contrast".to_string(), SettingValue::Bool(true));
        access.settings.insert("display.text_scale".to_string(), SettingValue::Float(1.5));
        access.settings.insert("audio.volume".to_string(), SettingValue::Float(0.9));
        access.settings.insert("accessibility.reduce_motion".to_string(), SettingValue::Bool(true));
        access.tags.push("accessibility".to_string());
        self.profiles.insert("accessibility".to_string(), access);
    }
    
    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&UserProfile> {
        self.profiles.get(name)
    }
    
    /// Get mutable profile
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut UserProfile> {
        self.profiles.get_mut(name)
    }
    
    /// Create new user profile
    pub fn create_profile(&mut self, name: &str) -> Result<&mut UserProfile, String> {
        if self.profiles.len() >= self.max_profiles {
            return Err("Maximum number of profiles reached".to_string());
        }
        
        if self.profiles.contains_key(name) {
            return Err(format!("Profile '{}' already exists", name));
        }
        
        let profile = UserProfile::new(name);
        self.profiles.insert(name.to_string(), profile);
        
        Ok(self.profiles.get_mut(name).unwrap())
    }
    
    /// Clone an existing profile
    pub fn clone_profile(&mut self, source: &str, new_name: &str) -> Result<&mut UserProfile, String> {
        if self.profiles.len() >= self.max_profiles {
            return Err("Maximum number of profiles reached".to_string());
        }
        
        if self.profiles.contains_key(new_name) {
            return Err(format!("Profile '{}' already exists", new_name));
        }
        
        let source_profile = self.profiles.get(source)
            .ok_or_else(|| format!("Source profile '{}' not found", source))?
            .clone();
        
        let mut new_profile = source_profile;
        new_profile.name = new_name.to_string();
        new_profile.is_system = false;
        new_profile.is_locked = false;
        new_profile.created_at = Instant::now();
        new_profile.modified_at = Instant::now();
        
        self.profiles.insert(new_name.to_string(), new_profile);
        
        Ok(self.profiles.get_mut(new_name).unwrap())
    }
    
    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) -> Result<(), String> {
        if let Some(profile) = self.profiles.get(name) {
            if profile.is_locked {
                return Err(format!("Profile '{}' is locked and cannot be deleted", name));
            }
            
            if Some(name) == self.active_profile.as_deref() {
                self.active_profile = Some(self.default_profile.clone());
            }
            
            self.profiles.remove(name);
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", name))
        }
    }
    
    /// Rename a profile
    pub fn rename_profile(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        if let Some(profile) = self.profiles.get(old_name) {
            if profile.is_system {
                return Err("Cannot rename system profiles".to_string());
            }
        }
        
        if self.profiles.contains_key(new_name) {
            return Err(format!("Profile '{}' already exists", new_name));
        }
        
        if let Some(mut profile) = self.profiles.remove(old_name) {
            profile.name = new_name.to_string();
            profile.modified_at = Instant::now();
            self.profiles.insert(new_name.to_string(), profile);
            
            if self.active_profile.as_deref() == Some(old_name) {
                self.active_profile = Some(new_name.to_string());
            }
            
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", old_name))
        }
    }
    
    /// Set active profile
    pub fn set_active(&mut self, name: &str) -> Result<(), String> {
        if self.profiles.contains_key(name) {
            self.active_profile = Some(name.to_string());
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", name))
        }
    }
    
    /// Get active profile
    pub fn active_profile(&self) -> Option<&UserProfile> {
        self.active_profile.as_ref()
            .and_then(|name| self.profiles.get(name))
    }
    
    /// Get active profile name
    pub fn active_profile_name(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }
    
    /// List all profiles
    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }
    
    /// List profiles with tag
    pub fn list_profiles_with_tag(&self, tag: &str) -> Vec<&str> {
        self.profiles
            .iter()
            .filter(|(_, p)| p.tags.contains(&tag.to_string()))
            .map(|(k, _)| k.as_str())
            .collect()
    }
    
    /// Get system profiles
    pub fn system_profiles(&self) -> Vec<&str> {
        self.profiles
            .iter()
            .filter(|(_, p)| p.is_system)
            .map(|(k, _)| k.as_str())
            .collect()
    }
    
    /// Get user profiles
    pub fn user_profiles(&self) -> Vec<&str> {
        self.profiles
            .iter()
            .filter(|(_, p)| !p.is_system)
            .map(|(k, _)| k.as_str())
            .collect()
    }
    
    /// Profile count
    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profile_manager_creation() {
        let manager = ProfileManager::new();
        
        assert!(manager.get_profile("default").is_some());
        assert!(manager.get_profile("night").is_some());
        assert!(manager.get_profile("outdoor").is_some());
    }
    
    #[test]
    fn test_create_profile() {
        let mut manager = ProfileManager::new();
        
        let result = manager.create_profile("custom");
        assert!(result.is_ok());
        
        assert!(manager.get_profile("custom").is_some());
    }
    
    #[test]
    fn test_clone_profile() {
        let mut manager = ProfileManager::new();
        
        let result = manager.clone_profile("default", "my_default");
        assert!(result.is_ok());
        
        let cloned = manager.get_profile("my_default").unwrap();
        assert!(!cloned.is_system);
    }
    
    #[test]
    fn test_delete_profile() {
        let mut manager = ProfileManager::new();
        
        // Create a user profile
        manager.create_profile("temp").unwrap();
        
        // Can delete user profile
        assert!(manager.delete_profile("temp").is_ok());
        
        // Cannot delete locked profile
        assert!(manager.delete_profile("default").is_err());
    }
    
    #[test]
    fn test_active_profile() {
        let mut manager = ProfileManager::new();
        
        manager.set_active("night").unwrap();
        assert_eq!(manager.active_profile_name(), Some("night"));
    }
    
    #[test]
    fn test_profile_tags() {
        let manager = ProfileManager::new();
        
        let lighting = manager.list_profiles_with_tag("lighting");
        assert!(lighting.contains(&"night"));
    }
    
    #[test]
    fn test_system_vs_user_profiles() {
        let mut manager = ProfileManager::new();
        
        manager.create_profile("user1").unwrap();
        
        let system = manager.system_profiles();
        let user = manager.user_profiles();
        
        assert!(system.contains(&"default"));
        assert!(user.contains(&"user1"));
    }
}
