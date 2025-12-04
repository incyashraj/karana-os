//! App Manifest for Kāraṇa OS AR Glasses
//!
//! Define app metadata, capabilities, and requirements.

use std::collections::HashSet;

/// App type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppType {
    /// Standard AR app
    Standard,
    /// Always-on overlay
    Overlay,
    /// Background service
    Service,
    /// Widget for HUD
    Widget,
    /// AR game
    Game,
    /// System app
    System,
}

impl AppType {
    /// Is background capable
    pub fn is_background(&self) -> bool {
        matches!(self, Self::Service | Self::Overlay)
    }
    
    /// Needs AR rendering
    pub fn needs_ar(&self) -> bool {
        matches!(self, Self::Standard | Self::Overlay | Self::Game)
    }
}

impl Default for AppType {
    fn default() -> Self {
        Self::Standard
    }
}

/// App capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppCapability {
    /// Camera access
    Camera,
    /// Microphone access
    Microphone,
    /// Location access
    Location,
    /// Storage access
    Storage,
    /// Network access
    Network,
    /// Contacts access
    Contacts,
    /// Notifications
    Notifications,
    /// Background execution
    Background,
    /// AR rendering
    ARRendering,
    /// Hand tracking
    HandTracking,
    /// Eye tracking
    EyeTracking,
    /// Spatial anchors
    SpatialAnchors,
    /// Bluetooth
    Bluetooth,
    /// System settings
    SystemSettings,
}

impl AppCapability {
    /// Is sensitive (requires explicit permission)
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self,
            Self::Camera | Self::Microphone | Self::Location |
            Self::Contacts | Self::EyeTracking | Self::SystemSettings
        )
    }
    
    /// Display name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Camera => "Camera",
            Self::Microphone => "Microphone",
            Self::Location => "Location",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Contacts => "Contacts",
            Self::Notifications => "Notifications",
            Self::Background => "Background",
            Self::ARRendering => "AR Rendering",
            Self::HandTracking => "Hand Tracking",
            Self::EyeTracking => "Eye Tracking",
            Self::SpatialAnchors => "Spatial Anchors",
            Self::Bluetooth => "Bluetooth",
            Self::SystemSettings => "System Settings",
        }
    }
    
    /// Description
    pub fn description(&self) -> &str {
        match self {
            Self::Camera => "Access the camera for photos and video",
            Self::Microphone => "Record audio from the microphone",
            Self::Location => "Access your current location",
            Self::Storage => "Read and write files",
            Self::Network => "Connect to the internet",
            Self::Contacts => "Access your contacts list",
            Self::Notifications => "Show notifications",
            Self::Background => "Run in the background",
            Self::ARRendering => "Display AR content",
            Self::HandTracking => "Track hand gestures",
            Self::EyeTracking => "Track eye gaze position",
            Self::SpatialAnchors => "Place persistent AR content",
            Self::Bluetooth => "Connect to Bluetooth devices",
            Self::SystemSettings => "Modify system settings",
        }
    }
}

/// Hardware requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HardwareRequirement {
    /// AR display
    ARDisplay,
    /// Camera
    Camera,
    /// Depth sensor
    DepthSensor,
    /// Hand tracking
    HandTracking,
    /// Eye tracking
    EyeTracking,
    /// GPS
    GPS,
    /// IMU
    IMU,
    /// Speaker
    Speaker,
    /// Microphone
    Microphone,
}

/// OS version requirement
#[derive(Debug, Clone)]
pub struct OSRequirement {
    /// Minimum version
    pub min_version: super::AppVersion,
    /// Maximum version (if any)
    pub max_version: Option<super::AppVersion>,
}

impl OSRequirement {
    /// Create requirement
    pub fn min(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            min_version: super::AppVersion::new(major, minor, patch),
            max_version: None,
        }
    }
}

impl Default for OSRequirement {
    fn default() -> Self {
        Self::min(1, 0, 0)
    }
}

/// App manifest
#[derive(Debug, Clone)]
pub struct AppManifest {
    /// App ID (reverse domain)
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Version
    pub version: super::AppVersion,
    /// Author
    pub author: String,
    /// App type
    pub app_type: AppType,
    /// Icon URL
    pub icon: Option<String>,
    /// Capabilities needed
    pub capabilities: HashSet<AppCapability>,
    /// Hardware requirements
    pub hardware: HashSet<HardwareRequirement>,
    /// OS requirement
    pub os_requirement: OSRequirement,
    /// Entry point
    pub entry_point: String,
    /// Size in bytes
    pub size: u64,
    /// Screenshots
    pub screenshots: Vec<String>,
    /// Category
    pub category: String,
    /// Tags
    pub tags: Vec<String>,
    /// Is paid
    pub is_paid: bool,
    /// Price (if paid)
    pub price: Option<f64>,
}

impl AppManifest {
    /// Create new manifest
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            version: super::AppVersion::default(),
            author: String::new(),
            app_type: AppType::Standard,
            icon: None,
            capabilities: HashSet::new(),
            hardware: HashSet::new(),
            os_requirement: OSRequirement::default(),
            entry_point: "main".to_string(),
            size: 0,
            screenshots: Vec::new(),
            category: "Utilities".to_string(),
            tags: Vec::new(),
            is_paid: false,
            price: None,
        }
    }
    
    /// With description
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }
    
    /// With version
    pub fn with_version(mut self, version: super::AppVersion) -> Self {
        self.version = version;
        self
    }
    
    /// With author
    pub fn with_author(mut self, author: String) -> Self {
        self.author = author;
        self
    }
    
    /// With app type
    pub fn with_type(mut self, app_type: AppType) -> Self {
        self.app_type = app_type;
        self
    }
    
    /// Add capability
    pub fn add_capability(&mut self, cap: AppCapability) {
        self.capabilities.insert(cap);
    }
    
    /// With capabilities
    pub fn with_capabilities(mut self, caps: Vec<AppCapability>) -> Self {
        for cap in caps {
            self.capabilities.insert(cap);
        }
        self
    }
    
    /// Add hardware requirement
    pub fn add_hardware(&mut self, hw: HardwareRequirement) {
        self.hardware.insert(hw);
    }
    
    /// With hardware requirements
    pub fn with_hardware(mut self, hw: Vec<HardwareRequirement>) -> Self {
        for h in hw {
            self.hardware.insert(h);
        }
        self
    }
    
    /// Has sensitive capabilities
    pub fn has_sensitive_capabilities(&self) -> bool {
        self.capabilities.iter().any(|c| c.is_sensitive())
    }
    
    /// Get sensitive capabilities
    pub fn sensitive_capabilities(&self) -> Vec<AppCapability> {
        self.capabilities.iter()
            .filter(|c| c.is_sensitive())
            .copied()
            .collect()
    }
    
    /// Validate manifest
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.id.is_empty() {
            errors.push("App ID is required".to_string());
        }
        
        if !self.id.contains('.') {
            errors.push("App ID should be in reverse domain format".to_string());
        }
        
        if self.name.is_empty() {
            errors.push("App name is required".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_type() {
        assert!(AppType::Service.is_background());
        assert!(!AppType::Standard.is_background());
        
        assert!(AppType::Standard.needs_ar());
        assert!(!AppType::Service.needs_ar());
    }
    
    #[test]
    fn test_capability_sensitive() {
        assert!(AppCapability::Camera.is_sensitive());
        assert!(AppCapability::Location.is_sensitive());
        assert!(!AppCapability::Network.is_sensitive());
    }
    
    #[test]
    fn test_manifest_creation() {
        let manifest = AppManifest::new(
            "com.test.app".to_string(),
            "Test App".to_string(),
        ).with_author("Test".to_string());
        
        assert_eq!(manifest.id, "com.test.app");
        assert_eq!(manifest.author, "Test");
    }
    
    #[test]
    fn test_manifest_capabilities() {
        let mut manifest = AppManifest::new(
            "com.test.app".to_string(),
            "Test App".to_string(),
        );
        
        manifest.add_capability(AppCapability::Camera);
        manifest.add_capability(AppCapability::Network);
        
        assert!(manifest.has_sensitive_capabilities());
        assert!(manifest.capabilities.contains(&AppCapability::Camera));
    }
    
    #[test]
    fn test_manifest_validation() {
        let valid = AppManifest::new(
            "com.test.app".to_string(),
            "Test App".to_string(),
        );
        assert!(valid.validate().is_ok());
        
        let invalid = AppManifest::new(
            "".to_string(),
            "".to_string(),
        );
        assert!(invalid.validate().is_err());
    }
}
