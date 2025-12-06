//! Privacy & Security System for Kāraṇa OS AR Glasses
//!
//! Manages privacy controls, data protection, and security features
//! essential for user trust in AR wearables with cameras and sensors.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub mod permissions;
pub mod encryption;
pub mod face_blur;
pub mod data_retention;
pub mod audit;

// Phase 50: Enhanced privacy management
pub mod retention;
pub mod dashboard;
pub mod ephemeral;

pub use permissions::{Permission, PermissionManager, PermissionState, PermissionScope};
pub use encryption::{EncryptionManager, EncryptionLevel, KeyInfo};
pub use face_blur::{FaceBlurEngine, BlurMode, FaceDetection, PrivacyZone};
pub use data_retention::{DataRetentionPolicy, RetentionManager, DataCategory};
pub use audit::{AuditLog, AuditEvent, AuditEventType, AuditManager};

/// Privacy mode levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivacyMode {
    /// Minimal privacy restrictions
    Open,
    /// Standard privacy protections
    Standard,
    /// Enhanced privacy (face blur, limited recording)
    Enhanced,
    /// Maximum privacy (no recording, minimal data)
    Maximum,
    /// Custom privacy settings
    Custom,
}

impl PrivacyMode {
    /// Get description
    pub fn description(&self) -> &str {
        match self {
            PrivacyMode::Open => "Minimal restrictions, full features enabled",
            PrivacyMode::Standard => "Balanced privacy with standard protections",
            PrivacyMode::Enhanced => "Enhanced privacy with face blur and limited recording",
            PrivacyMode::Maximum => "Maximum privacy, cameras and sensors restricted",
            PrivacyMode::Custom => "Custom privacy settings",
        }
    }
    
    /// Check if camera recording is allowed
    pub fn allows_recording(&self) -> bool {
        matches!(self, PrivacyMode::Open | PrivacyMode::Standard | PrivacyMode::Custom)
    }
    
    /// Check if face blur is required
    pub fn requires_face_blur(&self) -> bool {
        matches!(self, PrivacyMode::Enhanced | PrivacyMode::Maximum)
    }
}

/// Privacy indicator state (LED or display indicator)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyIndicator {
    /// No privacy-sensitive operation
    Off,
    /// Camera is active
    CameraActive,
    /// Microphone is active
    MicrophoneActive,
    /// Both camera and microphone active
    Recording,
    /// Location being accessed
    LocationActive,
    /// Data being transmitted
    DataTransmit,
}

impl PrivacyIndicator {
    /// Get LED color (RGB)
    pub fn led_color(&self) -> (u8, u8, u8) {
        match self {
            PrivacyIndicator::Off => (0, 0, 0),
            PrivacyIndicator::CameraActive => (255, 0, 0),      // Red
            PrivacyIndicator::MicrophoneActive => (255, 165, 0), // Orange
            PrivacyIndicator::Recording => (255, 0, 0),         // Bright red
            PrivacyIndicator::LocationActive => (0, 0, 255),    // Blue
            PrivacyIndicator::DataTransmit => (0, 255, 0),      // Green
        }
    }
    
    /// Get indicator priority (higher = shown first)
    pub fn priority(&self) -> u8 {
        match self {
            PrivacyIndicator::Off => 0,
            PrivacyIndicator::DataTransmit => 1,
            PrivacyIndicator::LocationActive => 2,
            PrivacyIndicator::MicrophoneActive => 3,
            PrivacyIndicator::CameraActive => 4,
            PrivacyIndicator::Recording => 5,
        }
    }
}

/// Consent type for data collection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsentType {
    /// Analytics data collection
    Analytics,
    /// Crash reports
    CrashReports,
    /// Usage statistics
    UsageStats,
    /// Location history
    LocationHistory,
    /// Voice data for improvement
    VoiceData,
    /// Camera data for ML improvement
    CameraData,
    /// Personalization
    Personalization,
    /// Third-party sharing
    ThirdPartySharing,
}

/// Privacy manager
#[derive(Debug)]
pub struct PrivacyManager {
    /// Current privacy mode
    mode: PrivacyMode,
    /// Permission manager
    permissions: PermissionManager,
    /// Current privacy indicator
    indicator: PrivacyIndicator,
    /// Active indicators
    active_indicators: HashSet<PrivacyIndicator>,
    /// User consents
    consents: HashMap<ConsentType, bool>,
    /// Privacy zones (locations with specific rules)
    privacy_zones: Vec<PrivacyZoneConfig>,
    /// Audit manager
    audit: AuditManager,
    /// Face blur engine enabled
    face_blur_enabled: bool,
    /// Bystander protection enabled
    bystander_protection: bool,
    /// Last indicator update
    last_indicator_update: Instant,
}

/// Privacy zone configuration
#[derive(Debug, Clone)]
pub struct PrivacyZoneConfig {
    /// Zone name
    pub name: String,
    /// Zone type
    pub zone_type: PrivacyZoneType,
    /// Location (lat, lon, radius_m)
    pub location: Option<(f64, f64, f32)>,
    /// WiFi SSID trigger
    pub wifi_ssid: Option<String>,
    /// Bluetooth device trigger
    pub bluetooth_trigger: Option<String>,
    /// Privacy mode to apply
    pub mode: PrivacyMode,
    /// Active
    pub active: bool,
}

/// Privacy zone types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyZoneType {
    /// Home zone
    Home,
    /// Work zone
    Work,
    /// Public space
    Public,
    /// Healthcare facility
    Healthcare,
    /// School/educational
    Education,
    /// Government facility
    Government,
    /// Custom zone
    Custom,
}

impl PrivacyManager {
    /// Create new privacy manager
    pub fn new() -> Self {
        Self {
            mode: PrivacyMode::Standard,
            permissions: PermissionManager::new(),
            indicator: PrivacyIndicator::Off,
            active_indicators: HashSet::new(),
            consents: Self::default_consents(),
            privacy_zones: Vec::new(),
            audit: AuditManager::new(),
            face_blur_enabled: true,
            bystander_protection: true,
            last_indicator_update: Instant::now(),
        }
    }
    
    /// Default consent states
    fn default_consents() -> HashMap<ConsentType, bool> {
        let mut consents = HashMap::new();
        consents.insert(ConsentType::CrashReports, true);
        consents.insert(ConsentType::Analytics, false);
        consents.insert(ConsentType::UsageStats, false);
        consents.insert(ConsentType::LocationHistory, false);
        consents.insert(ConsentType::VoiceData, false);
        consents.insert(ConsentType::CameraData, false);
        consents.insert(ConsentType::Personalization, true);
        consents.insert(ConsentType::ThirdPartySharing, false);
        consents
    }
    
    /// Set privacy mode
    pub fn set_mode(&mut self, mode: PrivacyMode) {
        let old_mode = self.mode;
        self.mode = mode;
        
        // Log the change
        self.audit.log_event(AuditEvent::new(
            AuditEventType::PrivacyModeChanged,
            format!("Privacy mode changed from {:?} to {:?}", old_mode, mode),
        ));
        
        // Apply mode-specific settings
        self.apply_mode_settings();
    }
    
    /// Get current mode
    pub fn mode(&self) -> PrivacyMode {
        self.mode
    }
    
    /// Apply mode-specific settings
    fn apply_mode_settings(&mut self) {
        match self.mode {
            PrivacyMode::Maximum => {
                self.face_blur_enabled = true;
                self.bystander_protection = true;
            }
            PrivacyMode::Enhanced => {
                self.face_blur_enabled = true;
                self.bystander_protection = true;
            }
            PrivacyMode::Standard => {
                self.face_blur_enabled = false;
                self.bystander_protection = true;
            }
            PrivacyMode::Open => {
                self.face_blur_enabled = false;
                self.bystander_protection = false;
            }
            PrivacyMode::Custom => {
                // Don't change settings
            }
        }
    }
    
    /// Set consent
    pub fn set_consent(&mut self, consent_type: ConsentType, granted: bool) {
        self.consents.insert(consent_type, granted);
        
        self.audit.log_event(AuditEvent::new(
            AuditEventType::ConsentChanged,
            format!("Consent {:?} set to {}", consent_type, granted),
        ));
    }
    
    /// Check consent
    pub fn has_consent(&self, consent_type: ConsentType) -> bool {
        self.consents.get(&consent_type).copied().unwrap_or(false)
    }
    
    /// Activate privacy indicator
    pub fn activate_indicator(&mut self, indicator: PrivacyIndicator) {
        self.active_indicators.insert(indicator);
        self.update_indicator();
        
        self.audit.log_event(AuditEvent::new(
            AuditEventType::SensorAccess,
            format!("Privacy indicator activated: {:?}", indicator),
        ));
    }
    
    /// Deactivate privacy indicator
    pub fn deactivate_indicator(&mut self, indicator: PrivacyIndicator) {
        self.active_indicators.remove(&indicator);
        self.update_indicator();
    }
    
    /// Update current indicator to highest priority
    fn update_indicator(&mut self) {
        self.indicator = self.active_indicators
            .iter()
            .max_by_key(|i| i.priority())
            .copied()
            .unwrap_or(PrivacyIndicator::Off);
        self.last_indicator_update = Instant::now();
    }
    
    /// Get current indicator
    pub fn current_indicator(&self) -> PrivacyIndicator {
        self.indicator
    }
    
    /// Add privacy zone
    pub fn add_privacy_zone(&mut self, zone: PrivacyZoneConfig) {
        self.audit.log_event(AuditEvent::new(
            AuditEventType::ZoneConfigured,
            format!("Privacy zone added: {}", zone.name),
        ));
        self.privacy_zones.push(zone);
    }
    
    /// Check if in privacy zone
    pub fn check_privacy_zone(&self, lat: f64, lon: f64) -> Option<&PrivacyZoneConfig> {
        for zone in &self.privacy_zones {
            if !zone.active {
                continue;
            }
            
            if let Some((zone_lat, zone_lon, radius)) = zone.location {
                let distance = Self::haversine_distance(lat, lon, zone_lat, zone_lon);
                if distance <= radius as f64 {
                    return Some(zone);
                }
            }
        }
        None
    }
    
    /// Haversine distance calculation
    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const R: f64 = 6371000.0; // Earth radius in meters
        
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        
        R * c
    }
    
    /// Request permission check
    pub fn check_permission(&self, permission: Permission, requester: &str) -> bool {
        let granted = self.permissions.check(permission);
        
        // Audit the check
        // Note: We don't log every check to avoid spam, but could log denials
        if !granted {
            // Would log denial in production
        }
        
        granted
    }
    
    /// Get permission manager
    pub fn permissions(&self) -> &PermissionManager {
        &self.permissions
    }
    
    /// Get mutable permission manager
    pub fn permissions_mut(&mut self) -> &mut PermissionManager {
        &mut self.permissions
    }
    
    /// Is face blur enabled
    pub fn is_face_blur_enabled(&self) -> bool {
        self.face_blur_enabled
    }
    
    /// Set face blur
    pub fn set_face_blur(&mut self, enabled: bool) {
        self.face_blur_enabled = enabled;
    }
    
    /// Is bystander protection enabled
    pub fn is_bystander_protection_enabled(&self) -> bool {
        self.bystander_protection
    }
    
    /// Get audit log
    pub fn audit_log(&self) -> &AuditManager {
        &self.audit
    }
    
    /// Export privacy report
    pub fn export_privacy_report(&self) -> PrivacyReport {
        PrivacyReport {
            mode: self.mode,
            consents: self.consents.clone(),
            permissions_granted: self.permissions.granted_count(),
            permissions_denied: self.permissions.denied_count(),
            privacy_zones: self.privacy_zones.len(),
            face_blur_enabled: self.face_blur_enabled,
            bystander_protection: self.bystander_protection,
            audit_entries: self.audit.entry_count(),
        }
    }
}

impl Default for PrivacyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Privacy report for user review
#[derive(Debug, Clone)]
pub struct PrivacyReport {
    /// Current privacy mode
    pub mode: PrivacyMode,
    /// Consent states
    pub consents: HashMap<ConsentType, bool>,
    /// Number of granted permissions
    pub permissions_granted: usize,
    /// Number of denied permissions
    pub permissions_denied: usize,
    /// Number of privacy zones
    pub privacy_zones: usize,
    /// Face blur enabled
    pub face_blur_enabled: bool,
    /// Bystander protection enabled
    pub bystander_protection: bool,
    /// Total audit entries
    pub audit_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_privacy_manager_creation() {
        let pm = PrivacyManager::new();
        assert_eq!(pm.mode(), PrivacyMode::Standard);
        assert_eq!(pm.current_indicator(), PrivacyIndicator::Off);
    }
    
    #[test]
    fn test_privacy_mode_setting() {
        let mut pm = PrivacyManager::new();
        
        pm.set_mode(PrivacyMode::Enhanced);
        assert_eq!(pm.mode(), PrivacyMode::Enhanced);
        assert!(pm.is_face_blur_enabled());
    }
    
    #[test]
    fn test_privacy_indicator() {
        let mut pm = PrivacyManager::new();
        
        pm.activate_indicator(PrivacyIndicator::CameraActive);
        assert_eq!(pm.current_indicator(), PrivacyIndicator::CameraActive);
        
        pm.deactivate_indicator(PrivacyIndicator::CameraActive);
        assert_eq!(pm.current_indicator(), PrivacyIndicator::Off);
    }
    
    #[test]
    fn test_indicator_priority() {
        let mut pm = PrivacyManager::new();
        
        pm.activate_indicator(PrivacyIndicator::LocationActive);
        pm.activate_indicator(PrivacyIndicator::Recording);
        
        // Recording has higher priority
        assert_eq!(pm.current_indicator(), PrivacyIndicator::Recording);
    }
    
    #[test]
    fn test_consent_management() {
        let mut pm = PrivacyManager::new();
        
        assert!(!pm.has_consent(ConsentType::Analytics));
        
        pm.set_consent(ConsentType::Analytics, true);
        assert!(pm.has_consent(ConsentType::Analytics));
    }
    
    #[test]
    fn test_privacy_zone() {
        let mut pm = PrivacyManager::new();
        
        pm.add_privacy_zone(PrivacyZoneConfig {
            name: "Home".to_string(),
            zone_type: PrivacyZoneType::Home,
            location: Some((37.7749, -122.4194, 100.0)),
            wifi_ssid: None,
            bluetooth_trigger: None,
            mode: PrivacyMode::Open,
            active: true,
        });
        
        // Inside zone
        let zone = pm.check_privacy_zone(37.7749, -122.4194);
        assert!(zone.is_some());
        
        // Outside zone
        let zone = pm.check_privacy_zone(40.7128, -74.0060);
        assert!(zone.is_none());
    }
    
    #[test]
    fn test_privacy_report() {
        let pm = PrivacyManager::new();
        let report = pm.export_privacy_report();
        
        assert_eq!(report.mode, PrivacyMode::Standard);
        assert!(!report.consents.is_empty());
    }
    
    #[test]
    fn test_maximum_privacy_mode() {
        let mut pm = PrivacyManager::new();
        pm.set_mode(PrivacyMode::Maximum);
        
        assert!(pm.is_face_blur_enabled());
        assert!(pm.is_bystander_protection_enabled());
        assert!(!pm.mode().allows_recording());
    }
}
