//! App & Plugin Framework for Kāraṇa OS AR Glasses
//!
//! Extensible application system for third-party apps and plugins.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub mod manifest;
pub mod runtime;
pub mod permissions;
pub mod sandbox;

pub use manifest::{AppManifest, AppType, AppCapability};
pub use runtime::{AppRuntime, AppInstance, AppState};
pub use permissions::{Permission, PermissionLevel, PermissionManager};
pub use sandbox::{Sandbox, SandboxConfig, ResourceLimit};

/// App identifier
pub type AppId = String;

/// App version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl AppVersion {
    /// Create new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
    
    /// Parse from string (e.g., "1.2.3")
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        
        Some(Self {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }
    
    /// To string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
    
    /// Is compatible with (same major version)
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major
    }
    
    /// Is newer than
    pub fn is_newer_than(&self, other: &Self) -> bool {
        (self.major, self.minor, self.patch) > (other.major, other.minor, other.patch)
    }
}

impl Default for AppVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

/// App info
#[derive(Debug, Clone)]
pub struct AppInfo {
    /// App ID
    pub id: AppId,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Version
    pub version: AppVersion,
    /// Author
    pub author: String,
    /// Icon URL
    pub icon: Option<String>,
    /// App type
    pub app_type: AppType,
    /// Size in bytes
    pub size: u64,
    /// Installed timestamp
    pub installed: Option<Instant>,
    /// Last updated
    pub updated: Option<Instant>,
    /// Is system app
    pub is_system: bool,
    /// Is enabled
    pub is_enabled: bool,
}

impl AppInfo {
    /// Create new app info
    pub fn new(id: AppId, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            version: AppVersion::default(),
            author: String::new(),
            icon: None,
            app_type: AppType::Standard,
            size: 0,
            installed: None,
            updated: None,
            is_system: false,
            is_enabled: true,
        }
    }
    
    /// With description
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }
    
    /// With version
    pub fn with_version(mut self, version: AppVersion) -> Self {
        self.version = version;
        self
    }
    
    /// With author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }
    
    /// As system app
    pub fn as_system_app(mut self) -> Self {
        self.is_system = true;
        self
    }
}

/// App event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEventType {
    /// App installed
    Installed,
    /// App uninstalled
    Uninstalled,
    /// App updated
    Updated,
    /// App launched
    Launched,
    /// App closed
    Closed,
    /// App suspended
    Suspended,
    /// App resumed
    Resumed,
    /// App crashed
    Crashed,
    /// Permission granted
    PermissionGranted,
    /// Permission denied
    PermissionDenied,
}

/// App event
#[derive(Debug, Clone)]
pub struct AppEvent {
    /// Event type
    pub event_type: AppEventType,
    /// App ID
    pub app_id: AppId,
    /// Timestamp
    pub timestamp: Instant,
    /// Additional data
    pub data: Option<String>,
}

impl AppEvent {
    /// Create new event
    pub fn new(event_type: AppEventType, app_id: AppId) -> Self {
        Self {
            event_type,
            app_id,
            timestamp: Instant::now(),
            data: None,
        }
    }
    
    /// With data
    pub fn with_data(mut self, data: String) -> Self {
        self.data = Some(data);
        self
    }
}

/// App manager configuration
#[derive(Debug, Clone)]
pub struct AppManagerConfig {
    /// Max installed apps
    pub max_apps: usize,
    /// Max running apps
    pub max_running: usize,
    /// Auto-suspend inactive apps
    pub auto_suspend: bool,
    /// Suspend timeout
    pub suspend_timeout: Duration,
    /// Allow untrusted sources
    pub allow_untrusted: bool,
}

impl Default for AppManagerConfig {
    fn default() -> Self {
        Self {
            max_apps: 100,
            max_running: 5,
            auto_suspend: true,
            suspend_timeout: Duration::from_secs(300),
            allow_untrusted: false,
        }
    }
}

/// App manager
#[derive(Debug)]
pub struct AppManager {
    /// Installed apps
    apps: HashMap<AppId, AppInfo>,
    /// Running app instances
    running: HashMap<AppId, AppInstance>,
    /// App order (for launcher)
    app_order: Vec<AppId>,
    /// Recently used apps
    recent: VecDeque<AppId>,
    /// Max recent
    max_recent: usize,
    /// Permission manager
    permissions: PermissionManager,
    /// App runtime
    runtime: AppRuntime,
    /// Event history
    events: VecDeque<AppEvent>,
    /// Max events
    max_events: usize,
    /// Configuration
    config: AppManagerConfig,
    /// Foreground app
    foreground_app: Option<AppId>,
}

impl AppManager {
    /// Create new app manager
    pub fn new() -> Self {
        Self::with_config(AppManagerConfig::default())
    }
    
    /// Create with configuration
    pub fn with_config(config: AppManagerConfig) -> Self {
        let mut manager = Self {
            apps: HashMap::new(),
            running: HashMap::new(),
            app_order: Vec::new(),
            recent: VecDeque::new(),
            max_recent: 10,
            permissions: PermissionManager::new(),
            runtime: AppRuntime::new(),
            events: VecDeque::new(),
            max_events: 100,
            config,
            foreground_app: None,
        };
        
        // Register system apps
        manager.register_system_apps();
        
        manager
    }
    
    /// Register system apps
    fn register_system_apps(&mut self) {
        // Settings app
        let settings = AppInfo::new(
            "com.karana.settings".to_string(),
            "Settings".to_string(),
        )
        .with_description("System settings".to_string())
        .as_system_app();
        self.apps.insert(settings.id.clone(), settings);
        
        // Launcher
        let launcher = AppInfo::new(
            "com.karana.launcher".to_string(),
            "Launcher".to_string(),
        )
        .with_description("App launcher".to_string())
        .as_system_app();
        self.apps.insert(launcher.id.clone(), launcher);
        
        // Camera
        let camera = AppInfo::new(
            "com.karana.camera".to_string(),
            "Camera".to_string(),
        )
        .with_description("Camera app".to_string())
        .as_system_app();
        self.apps.insert(camera.id.clone(), camera);
        
        // Assistant
        let assistant = AppInfo::new(
            "com.karana.assistant".to_string(),
            "Assistant".to_string(),
        )
        .with_description("AI Assistant".to_string())
        .as_system_app();
        self.apps.insert(assistant.id.clone(), assistant);
    }
    
    /// Install app
    pub fn install(&mut self, manifest: AppManifest) -> Result<AppId, String> {
        if self.apps.len() >= self.config.max_apps {
            return Err("Maximum apps installed".to_string());
        }
        
        if self.apps.contains_key(&manifest.id) {
            return Err("App already installed".to_string());
        }
        
        let app_info = AppInfo {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            description: manifest.description.clone(),
            version: manifest.version.clone(),
            author: manifest.author.clone(),
            icon: manifest.icon.clone(),
            app_type: manifest.app_type,
            size: manifest.size,
            installed: Some(Instant::now()),
            updated: None,
            is_system: false,
            is_enabled: true,
        };
        
        let app_id = app_info.id.clone();
        self.apps.insert(app_id.clone(), app_info);
        self.app_order.push(app_id.clone());
        
        // Register permissions
        for cap in &manifest.capabilities {
            self.permissions.register_capability(&app_id, cap);
        }
        
        self.add_event(AppEvent::new(AppEventType::Installed, app_id.clone()));
        
        Ok(app_id)
    }
    
    /// Uninstall app
    pub fn uninstall(&mut self, app_id: &str) -> Result<(), String> {
        let app = self.apps.get(app_id).ok_or("App not found")?;
        
        if app.is_system {
            return Err("Cannot uninstall system app".to_string());
        }
        
        // Stop if running
        self.stop(app_id);
        
        self.apps.remove(app_id);
        self.app_order.retain(|id| id != app_id);
        self.recent.retain(|id| id != app_id);
        self.permissions.remove_app(app_id);
        
        self.add_event(AppEvent::new(AppEventType::Uninstalled, app_id.to_string()));
        
        Ok(())
    }
    
    /// Update app
    pub fn update(&mut self, manifest: AppManifest) -> Result<(), String> {
        let app = self.apps.get_mut(&manifest.id).ok_or("App not found")?;
        
        if !manifest.version.is_newer_than(&app.version) {
            return Err("Not a newer version".to_string());
        }
        
        app.version = manifest.version;
        app.name = manifest.name;
        app.description = manifest.description;
        app.author = manifest.author;
        app.icon = manifest.icon;
        app.size = manifest.size;
        app.updated = Some(Instant::now());
        
        self.add_event(AppEvent::new(AppEventType::Updated, manifest.id));
        
        Ok(())
    }
    
    /// Launch app
    pub fn launch(&mut self, app_id: &str) -> Result<(), String> {
        if !self.apps.contains_key(app_id) {
            return Err("App not found".to_string());
        }
        
        // Get app info we need before any mutable borrows
        let (is_enabled, app_type) = {
            let app = self.apps.get(app_id).unwrap();
            (app.is_enabled, app.app_type)
        };
        
        if !is_enabled {
            return Err("App is disabled".to_string());
        }
        
        if self.running.contains_key(app_id) {
            // Already running, bring to foreground
            self.foreground_app = Some(app_id.to_string());
            return Ok(());
        }
        
        if self.running.len() >= self.config.max_running {
            // Suspend least recent app
            if let Some(oldest) = self.find_oldest_suspended() {
                self.stop(&oldest);
            } else {
                return Err("Too many running apps".to_string());
            }
        }
        
        let instance = self.runtime.create_instance(app_id, &app_type);
        self.running.insert(app_id.to_string(), instance);
        self.foreground_app = Some(app_id.to_string());
        
        // Update recent
        self.recent.retain(|id| id != app_id);
        if self.recent.len() >= self.max_recent {
            self.recent.pop_back();
        }
        self.recent.push_front(app_id.to_string());
        
        self.add_event(AppEvent::new(AppEventType::Launched, app_id.to_string()));
        
        Ok(())
    }
    
    /// Stop app
    pub fn stop(&mut self, app_id: &str) {
        if let Some(mut instance) = self.running.remove(app_id) {
            instance.stop();
            
            if self.foreground_app.as_ref().map(|s| s.as_str()) == Some(app_id) {
                self.foreground_app = None;
            }
            
            self.add_event(AppEvent::new(AppEventType::Closed, app_id.to_string()));
        }
    }
    
    /// Suspend app
    pub fn suspend(&mut self, app_id: &str) {
        if let Some(instance) = self.running.get_mut(app_id) {
            instance.suspend();
            
            if self.foreground_app.as_ref().map(|s| s.as_str()) == Some(app_id) {
                self.foreground_app = None;
            }
            
            self.add_event(AppEvent::new(AppEventType::Suspended, app_id.to_string()));
        }
    }
    
    /// Resume app
    pub fn resume(&mut self, app_id: &str) {
        if let Some(instance) = self.running.get_mut(app_id) {
            instance.resume();
            self.foreground_app = Some(app_id.to_string());
            
            self.add_event(AppEvent::new(AppEventType::Resumed, app_id.to_string()));
        }
    }
    
    /// Find oldest suspended app
    fn find_oldest_suspended(&self) -> Option<String> {
        self.running.iter()
            .filter(|(_, inst)| inst.state() == AppState::Suspended)
            .min_by_key(|(_, inst)| inst.last_active())
            .map(|(id, _)| id.clone())
    }
    
    /// Get app info
    pub fn get_app(&self, app_id: &str) -> Option<&AppInfo> {
        self.apps.get(app_id)
    }
    
    /// Get all apps
    pub fn all_apps(&self) -> Vec<&AppInfo> {
        self.app_order.iter()
            .filter_map(|id| self.apps.get(id))
            .collect()
    }
    
    /// Get installed app count
    pub fn app_count(&self) -> usize {
        self.apps.len()
    }
    
    /// Get running apps
    pub fn running_apps(&self) -> Vec<&AppId> {
        self.running.keys().collect()
    }
    
    /// Get running app count
    pub fn running_count(&self) -> usize {
        self.running.len()
    }
    
    /// Get recent apps
    pub fn recent_apps(&self) -> Vec<&AppInfo> {
        self.recent.iter()
            .filter_map(|id| self.apps.get(id))
            .collect()
    }
    
    /// Get foreground app
    pub fn foreground_app(&self) -> Option<&str> {
        self.foreground_app.as_deref()
    }
    
    /// Is app running
    pub fn is_running(&self, app_id: &str) -> bool {
        self.running.contains_key(app_id)
    }
    
    /// Enable/disable app
    pub fn set_enabled(&mut self, app_id: &str, enabled: bool) -> Result<(), String> {
        let app = self.apps.get_mut(app_id).ok_or("App not found")?;
        app.is_enabled = enabled;
        
        if !enabled {
            self.stop(app_id);
        }
        
        Ok(())
    }
    
    /// Get permission manager
    pub fn permissions(&self) -> &PermissionManager {
        &self.permissions
    }
    
    /// Get mutable permission manager
    pub fn permissions_mut(&mut self) -> &mut PermissionManager {
        &mut self.permissions
    }
    
    /// Add event
    fn add_event(&mut self, event: AppEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_back();
        }
        self.events.push_front(event);
    }
    
    /// Get recent events
    pub fn events(&self) -> &VecDeque<AppEvent> {
        &self.events
    }
    
    /// Check and auto-suspend inactive apps
    pub fn check_auto_suspend(&mut self) {
        if !self.config.auto_suspend {
            return;
        }
        
        let to_suspend: Vec<String> = self.running.iter()
            .filter(|(id, inst)| {
                Some(id.as_str()) != self.foreground_app.as_deref() &&
                inst.state() == AppState::Running &&
                inst.last_active().elapsed() > self.config.suspend_timeout
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for app_id in to_suspend {
            self.suspend(&app_id);
        }
    }
}

impl Default for AppManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_version() {
        let v1 = AppVersion::new(1, 2, 3);
        assert_eq!(v1.to_string(), "1.2.3");
        
        let v2 = AppVersion::parse("2.0.0").unwrap();
        assert!(v2.is_newer_than(&v1));
        
        let v3 = AppVersion::new(1, 3, 0);
        assert!(v3.is_compatible(&v1));
    }
    
    #[test]
    fn test_app_info() {
        let app = AppInfo::new(
            "com.test.app".to_string(),
            "Test App".to_string(),
        ).with_author("Test".to_string());
        
        assert_eq!(app.author, "Test");
        assert!(!app.is_system);
    }
    
    #[test]
    fn test_app_manager_creation() {
        let manager = AppManager::new();
        
        // Should have system apps
        assert!(manager.get_app("com.karana.settings").is_some());
        assert!(manager.get_app("com.karana.launcher").is_some());
        assert!(manager.app_count() >= 4);
    }
    
    #[test]
    fn test_install_app() {
        let mut manager = AppManager::new();
        
        let manifest = AppManifest::new(
            "com.test.myapp".to_string(),
            "My App".to_string(),
        );
        
        let result = manager.install(manifest);
        assert!(result.is_ok());
        assert!(manager.get_app("com.test.myapp").is_some());
    }
    
    #[test]
    fn test_uninstall_app() {
        let mut manager = AppManager::new();
        
        let manifest = AppManifest::new(
            "com.test.uninstall".to_string(),
            "Uninstall Me".to_string(),
        );
        manager.install(manifest).unwrap();
        
        let result = manager.uninstall("com.test.uninstall");
        assert!(result.is_ok());
        assert!(manager.get_app("com.test.uninstall").is_none());
    }
    
    #[test]
    fn test_cannot_uninstall_system() {
        let mut manager = AppManager::new();
        
        let result = manager.uninstall("com.karana.settings");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_launch_app() {
        let mut manager = AppManager::new();
        
        let manifest = AppManifest::new(
            "com.test.launch".to_string(),
            "Launch Me".to_string(),
        );
        manager.install(manifest).unwrap();
        
        let result = manager.launch("com.test.launch");
        assert!(result.is_ok());
        assert!(manager.is_running("com.test.launch"));
    }
    
    #[test]
    fn test_stop_app() {
        let mut manager = AppManager::new();
        
        let manifest = AppManifest::new(
            "com.test.stop".to_string(),
            "Stop Me".to_string(),
        );
        manager.install(manifest).unwrap();
        manager.launch("com.test.stop").unwrap();
        
        manager.stop("com.test.stop");
        assert!(!manager.is_running("com.test.stop"));
    }
    
    #[test]
    fn test_foreground_app() {
        let mut manager = AppManager::new();
        
        let manifest = AppManifest::new(
            "com.test.fg".to_string(),
            "Foreground".to_string(),
        );
        manager.install(manifest).unwrap();
        manager.launch("com.test.fg").unwrap();
        
        assert_eq!(manager.foreground_app(), Some("com.test.fg"));
    }
    
    #[test]
    fn test_recent_apps() {
        let mut manager = AppManager::new();
        
        let m1 = AppManifest::new("com.test.r1".to_string(), "R1".to_string());
        let m2 = AppManifest::new("com.test.r2".to_string(), "R2".to_string());
        
        manager.install(m1).unwrap();
        manager.install(m2).unwrap();
        
        manager.launch("com.test.r1").unwrap();
        manager.launch("com.test.r2").unwrap();
        
        let recent = manager.recent_apps();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, "com.test.r2");
    }
}
