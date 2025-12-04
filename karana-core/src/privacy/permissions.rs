//! Permission System for Kāraṇa OS AR Glasses
//!
//! Fine-grained permission control for apps accessing sensitive resources.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// System permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Access to camera
    Camera,
    /// Access to microphone
    Microphone,
    /// Access to fine location
    LocationFine,
    /// Access to coarse location
    LocationCoarse,
    /// Access to contacts
    Contacts,
    /// Access to calendar
    Calendar,
    /// Access to eye tracking
    EyeTracking,
    /// Access to gaze data
    GazeData,
    /// Access to hand tracking
    HandTracking,
    /// Access to body pose
    BodyPose,
    /// Access to biometric data
    Biometrics,
    /// Access to health data
    HealthData,
    /// Access to spatial mapping
    SpatialMapping,
    /// Access to world anchors
    WorldAnchors,
    /// Access to notifications
    Notifications,
    /// Access to internet
    Internet,
    /// Access to Bluetooth
    Bluetooth,
    /// Access to storage
    Storage,
    /// Run in background
    Background,
    /// Access system settings
    SystemSettings,
    /// Access to other apps
    CrossApp,
    /// Record video
    VideoRecording,
    /// Record audio
    AudioRecording,
}

impl Permission {
    /// Get permission description
    pub fn description(&self) -> &str {
        match self {
            Permission::Camera => "Access to camera for photos and video",
            Permission::Microphone => "Access to microphone for audio",
            Permission::LocationFine => "Access to precise location",
            Permission::LocationCoarse => "Access to approximate location",
            Permission::Contacts => "Access to contacts list",
            Permission::Calendar => "Access to calendar events",
            Permission::EyeTracking => "Track eye movement and gaze",
            Permission::GazeData => "Access gaze direction data",
            Permission::HandTracking => "Track hand gestures and position",
            Permission::BodyPose => "Track body pose and movement",
            Permission::Biometrics => "Access fingerprint or face ID",
            Permission::HealthData => "Access health and fitness data",
            Permission::SpatialMapping => "Map the environment around you",
            Permission::WorldAnchors => "Place persistent virtual objects",
            Permission::Notifications => "Send notifications",
            Permission::Internet => "Access the internet",
            Permission::Bluetooth => "Connect to Bluetooth devices",
            Permission::Storage => "Read and write files",
            Permission::Background => "Run in background",
            Permission::SystemSettings => "Access system settings",
            Permission::CrossApp => "Interact with other apps",
            Permission::VideoRecording => "Record video",
            Permission::AudioRecording => "Record audio",
        }
    }
    
    /// Get permission category
    pub fn category(&self) -> PermissionCategory {
        match self {
            Permission::Camera | Permission::Microphone | 
            Permission::VideoRecording | Permission::AudioRecording => PermissionCategory::Recording,
            
            Permission::LocationFine | Permission::LocationCoarse => PermissionCategory::Location,
            
            Permission::EyeTracking | Permission::GazeData |
            Permission::HandTracking | Permission::BodyPose => PermissionCategory::BodyTracking,
            
            Permission::Biometrics | Permission::HealthData => PermissionCategory::SensitiveData,
            
            Permission::Contacts | Permission::Calendar => PermissionCategory::PersonalData,
            
            Permission::SpatialMapping | Permission::WorldAnchors => PermissionCategory::Spatial,
            
            Permission::Internet | Permission::Bluetooth => PermissionCategory::Network,
            
            Permission::Notifications | Permission::Storage |
            Permission::Background | Permission::SystemSettings |
            Permission::CrossApp => PermissionCategory::System,
        }
    }
    
    /// Is this a dangerous permission requiring explicit consent?
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            Permission::Camera | Permission::Microphone |
            Permission::LocationFine | Permission::EyeTracking |
            Permission::GazeData | Permission::Biometrics |
            Permission::HealthData | Permission::VideoRecording |
            Permission::AudioRecording | Permission::SpatialMapping
        )
    }
    
    /// Privacy risk level (0-10)
    pub fn privacy_risk(&self) -> u8 {
        match self {
            Permission::VideoRecording => 10,
            Permission::Camera => 9,
            Permission::AudioRecording => 9,
            Permission::Microphone => 8,
            Permission::EyeTracking => 8,
            Permission::GazeData => 7,
            Permission::LocationFine => 7,
            Permission::Biometrics => 7,
            Permission::HealthData => 6,
            Permission::SpatialMapping => 6,
            Permission::BodyPose => 5,
            Permission::HandTracking => 5,
            Permission::Contacts => 5,
            Permission::LocationCoarse => 4,
            Permission::Calendar => 4,
            Permission::WorldAnchors => 3,
            Permission::Storage => 3,
            Permission::Background => 2,
            Permission::Bluetooth => 2,
            Permission::Internet => 2,
            Permission::Notifications => 1,
            Permission::SystemSettings => 1,
            Permission::CrossApp => 1,
        }
    }
}

/// Permission categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionCategory {
    /// Recording (camera, mic)
    Recording,
    /// Location services
    Location,
    /// Body tracking (eye, hand, pose)
    BodyTracking,
    /// Sensitive data (health, biometrics)
    SensitiveData,
    /// Personal data (contacts, calendar)
    PersonalData,
    /// Spatial features
    Spatial,
    /// Network access
    Network,
    /// System features
    System,
}

/// Permission state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    /// Not yet requested
    NotRequested,
    /// Granted by user
    Granted,
    /// Denied by user
    Denied,
    /// Granted only while app is in use
    GrantedWhileInUse,
    /// Temporarily granted (time-limited)
    TemporaryGrant,
    /// Requires additional authentication
    RequiresAuth,
}

impl PermissionState {
    /// Check if effectively granted
    pub fn is_granted(&self) -> bool {
        matches!(
            self,
            PermissionState::Granted | 
            PermissionState::GrantedWhileInUse |
            PermissionState::TemporaryGrant
        )
    }
}

/// Permission scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionScope {
    /// Full access
    Full,
    /// Limited access
    Limited,
    /// One-time access
    OneTime,
    /// Access while app is in foreground
    WhileInUse,
}

/// Permission request
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    /// Permission requested
    pub permission: Permission,
    /// Requesting app ID
    pub app_id: String,
    /// Reason for request
    pub reason: String,
    /// Request time
    pub requested_at: Instant,
    /// Desired scope
    pub scope: PermissionScope,
}

/// Permission grant record
#[derive(Debug, Clone)]
pub struct PermissionGrant {
    /// Permission
    pub permission: Permission,
    /// App ID
    pub app_id: String,
    /// State
    pub state: PermissionState,
    /// Scope
    pub scope: PermissionScope,
    /// When granted
    pub granted_at: Option<Instant>,
    /// Expiration (for temporary grants)
    pub expires_at: Option<Instant>,
    /// Use count
    pub use_count: u64,
    /// Last used
    pub last_used: Option<Instant>,
}

/// Permission manager
#[derive(Debug)]
pub struct PermissionManager {
    /// Permission grants by app and permission
    grants: HashMap<(String, Permission), PermissionGrant>,
    /// Global permission overrides
    global_overrides: HashMap<Permission, PermissionState>,
    /// Pending requests
    pending_requests: Vec<PermissionRequest>,
    /// Default state for new requests
    default_state: PermissionState,
}

impl PermissionManager {
    /// Create new permission manager
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
            global_overrides: HashMap::new(),
            pending_requests: Vec::new(),
            default_state: PermissionState::NotRequested,
        }
    }
    
    /// Check if permission is granted (system-level check)
    pub fn check(&self, permission: Permission) -> bool {
        // Check global override first
        if let Some(state) = self.global_overrides.get(&permission) {
            return state.is_granted();
        }
        
        // Default to granted for non-dangerous permissions
        !permission.is_dangerous()
    }
    
    /// Check permission for specific app
    pub fn check_for_app(&self, permission: Permission, app_id: &str) -> bool {
        // Check global override first
        if let Some(state) = self.global_overrides.get(&permission) {
            if !state.is_granted() {
                return false;
            }
        }
        
        // Check app-specific grant
        if let Some(grant) = self.grants.get(&(app_id.to_string(), permission)) {
            // Check expiration
            if let Some(expires) = grant.expires_at {
                if Instant::now() >= expires {
                    return false;
                }
            }
            
            return grant.state.is_granted();
        }
        
        // Not requested yet
        false
    }
    
    /// Request permission
    pub fn request(&mut self, request: PermissionRequest) -> PermissionState {
        let key = (request.app_id.clone(), request.permission);
        
        // Check if already granted
        if let Some(grant) = self.grants.get(&key) {
            return grant.state;
        }
        
        // Add to pending
        self.pending_requests.push(request);
        
        PermissionState::NotRequested
    }
    
    /// Grant permission
    pub fn grant(
        &mut self,
        permission: Permission,
        app_id: &str,
        scope: PermissionScope,
        duration: Option<Duration>,
    ) {
        let now = Instant::now();
        let expires_at = duration.map(|d| now + d);
        
        let state = match scope {
            PermissionScope::OneTime => PermissionState::TemporaryGrant,
            PermissionScope::WhileInUse => PermissionState::GrantedWhileInUse,
            _ => PermissionState::Granted,
        };
        
        let grant = PermissionGrant {
            permission,
            app_id: app_id.to_string(),
            state,
            scope,
            granted_at: Some(now),
            expires_at,
            use_count: 0,
            last_used: None,
        };
        
        self.grants.insert((app_id.to_string(), permission), grant);
    }
    
    /// Deny permission
    pub fn deny(&mut self, permission: Permission, app_id: &str) {
        let grant = PermissionGrant {
            permission,
            app_id: app_id.to_string(),
            state: PermissionState::Denied,
            scope: PermissionScope::Full,
            granted_at: None,
            expires_at: None,
            use_count: 0,
            last_used: None,
        };
        
        self.grants.insert((app_id.to_string(), permission), grant);
    }
    
    /// Revoke permission
    pub fn revoke(&mut self, permission: Permission, app_id: &str) {
        self.grants.remove(&(app_id.to_string(), permission));
    }
    
    /// Revoke all permissions for app
    pub fn revoke_all_for_app(&mut self, app_id: &str) {
        self.grants.retain(|(id, _), _| id != app_id);
    }
    
    /// Set global override
    pub fn set_global_override(&mut self, permission: Permission, state: PermissionState) {
        self.global_overrides.insert(permission, state);
    }
    
    /// Clear global override
    pub fn clear_global_override(&mut self, permission: Permission) {
        self.global_overrides.remove(&permission);
    }
    
    /// Record permission use
    pub fn record_use(&mut self, permission: Permission, app_id: &str) {
        if let Some(grant) = self.grants.get_mut(&(app_id.to_string(), permission)) {
            grant.use_count += 1;
            grant.last_used = Some(Instant::now());
        }
    }
    
    /// Get grants for app
    pub fn grants_for_app(&self, app_id: &str) -> Vec<&PermissionGrant> {
        self.grants
            .iter()
            .filter(|((id, _), _)| id == app_id)
            .map(|(_, grant)| grant)
            .collect()
    }
    
    /// Get all apps with specific permission
    pub fn apps_with_permission(&self, permission: Permission) -> Vec<String> {
        self.grants
            .iter()
            .filter(|((_, perm), grant)| *perm == permission && grant.state.is_granted())
            .map(|((app_id, _), _)| app_id.clone())
            .collect()
    }
    
    /// Count granted permissions
    pub fn granted_count(&self) -> usize {
        self.grants.values().filter(|g| g.state.is_granted()).count()
    }
    
    /// Count denied permissions
    pub fn denied_count(&self) -> usize {
        self.grants.values().filter(|g| g.state == PermissionState::Denied).count()
    }
    
    /// Get pending requests
    pub fn pending_requests(&self) -> &[PermissionRequest] {
        &self.pending_requests
    }
    
    /// Clear pending request
    pub fn clear_pending(&mut self, app_id: &str, permission: Permission) {
        self.pending_requests.retain(|r| !(r.app_id == app_id && r.permission == permission));
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_creation() {
        let pm = PermissionManager::new();
        assert_eq!(pm.granted_count(), 0);
    }
    
    #[test]
    fn test_permission_grant() {
        let mut pm = PermissionManager::new();
        
        pm.grant(Permission::Camera, "app1", PermissionScope::Full, None);
        
        assert!(pm.check_for_app(Permission::Camera, "app1"));
        assert!(!pm.check_for_app(Permission::Camera, "app2"));
    }
    
    #[test]
    fn test_permission_deny() {
        let mut pm = PermissionManager::new();
        
        pm.deny(Permission::Microphone, "app1");
        
        assert!(!pm.check_for_app(Permission::Microphone, "app1"));
        assert_eq!(pm.denied_count(), 1);
    }
    
    #[test]
    fn test_permission_revoke() {
        let mut pm = PermissionManager::new();
        
        pm.grant(Permission::LocationFine, "app1", PermissionScope::Full, None);
        assert!(pm.check_for_app(Permission::LocationFine, "app1"));
        
        pm.revoke(Permission::LocationFine, "app1");
        assert!(!pm.check_for_app(Permission::LocationFine, "app1"));
    }
    
    #[test]
    fn test_global_override() {
        let mut pm = PermissionManager::new();
        
        // Grant to app
        pm.grant(Permission::Camera, "app1", PermissionScope::Full, None);
        assert!(pm.check_for_app(Permission::Camera, "app1"));
        
        // Global deny
        pm.set_global_override(Permission::Camera, PermissionState::Denied);
        assert!(!pm.check_for_app(Permission::Camera, "app1"));
    }
    
    #[test]
    fn test_permission_categories() {
        assert_eq!(Permission::Camera.category(), PermissionCategory::Recording);
        assert_eq!(Permission::EyeTracking.category(), PermissionCategory::BodyTracking);
        assert_eq!(Permission::HealthData.category(), PermissionCategory::SensitiveData);
    }
    
    #[test]
    fn test_dangerous_permissions() {
        assert!(Permission::Camera.is_dangerous());
        assert!(Permission::EyeTracking.is_dangerous());
        assert!(!Permission::Notifications.is_dangerous());
        assert!(!Permission::Internet.is_dangerous());
    }
    
    #[test]
    fn test_privacy_risk() {
        assert!(Permission::VideoRecording.privacy_risk() > Permission::Notifications.privacy_risk());
        assert!(Permission::Camera.privacy_risk() > Permission::Storage.privacy_risk());
    }
    
    #[test]
    fn test_apps_with_permission() {
        let mut pm = PermissionManager::new();
        
        pm.grant(Permission::Camera, "app1", PermissionScope::Full, None);
        pm.grant(Permission::Camera, "app2", PermissionScope::Full, None);
        pm.grant(Permission::Microphone, "app1", PermissionScope::Full, None);
        
        let apps = pm.apps_with_permission(Permission::Camera);
        assert_eq!(apps.len(), 2);
    }
    
    #[test]
    fn test_use_recording() {
        let mut pm = PermissionManager::new();
        
        pm.grant(Permission::Camera, "app1", PermissionScope::Full, None);
        pm.record_use(Permission::Camera, "app1");
        pm.record_use(Permission::Camera, "app1");
        
        let grants = pm.grants_for_app("app1");
        assert_eq!(grants[0].use_count, 2);
    }
}
