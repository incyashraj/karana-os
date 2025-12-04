//! Permission System for Kāraṇa OS AR Glasses
//!
//! Manage app permissions and user consent.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::manifest::AppCapability;

/// Permission level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionLevel {
    /// Not requested
    NotRequested,
    /// Denied by user
    Denied,
    /// Granted for this session
    GrantedSession,
    /// Granted permanently
    GrantedPermanent,
    /// Ask each time
    AskEachTime,
}

impl PermissionLevel {
    /// Is granted (session or permanent)
    pub fn is_granted(&self) -> bool {
        matches!(self, Self::GrantedSession | Self::GrantedPermanent)
    }
    
    /// Needs prompt
    pub fn needs_prompt(&self) -> bool {
        matches!(self, Self::NotRequested | Self::AskEachTime)
    }
}

impl Default for PermissionLevel {
    fn default() -> Self {
        Self::NotRequested
    }
}

/// Permission record
#[derive(Debug, Clone)]
pub struct Permission {
    /// Capability
    pub capability: AppCapability,
    /// Level
    pub level: PermissionLevel,
    /// Last requested
    pub last_requested: Option<Instant>,
    /// Last granted
    pub last_granted: Option<Instant>,
    /// Request count
    pub request_count: u32,
}

impl Permission {
    /// Create new permission
    pub fn new(capability: AppCapability) -> Self {
        Self {
            capability,
            level: PermissionLevel::NotRequested,
            last_requested: None,
            last_granted: None,
            request_count: 0,
        }
    }
    
    /// Grant permission
    pub fn grant(&mut self, permanent: bool) {
        self.level = if permanent {
            PermissionLevel::GrantedPermanent
        } else {
            PermissionLevel::GrantedSession
        };
        self.last_granted = Some(Instant::now());
    }
    
    /// Deny permission
    pub fn deny(&mut self) {
        self.level = PermissionLevel::Denied;
    }
    
    /// Set ask each time
    pub fn ask_each_time(&mut self) {
        self.level = PermissionLevel::AskEachTime;
    }
    
    /// Record request
    pub fn record_request(&mut self) {
        self.last_requested = Some(Instant::now());
        self.request_count += 1;
    }
    
    /// Reset to default
    pub fn reset(&mut self) {
        self.level = PermissionLevel::NotRequested;
    }
}

/// Permission request
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    /// App ID
    pub app_id: String,
    /// App name
    pub app_name: String,
    /// Capability
    pub capability: AppCapability,
    /// Reason provided by app
    pub reason: Option<String>,
    /// Request ID
    pub request_id: u64,
    /// Created at
    pub created: Instant,
}

impl PermissionRequest {
    /// Create new request
    pub fn new(app_id: String, app_name: String, capability: AppCapability, request_id: u64) -> Self {
        Self {
            app_id,
            app_name,
            capability,
            reason: None,
            request_id,
            created: Instant::now(),
        }
    }
    
    /// With reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

/// App permission set
#[derive(Debug, Clone, Default)]
pub struct AppPermissions {
    /// App ID
    pub app_id: String,
    /// Permissions
    permissions: HashMap<AppCapability, Permission>,
    /// Registered capabilities
    registered: HashSet<AppCapability>,
}

impl AppPermissions {
    /// Create new
    pub fn new(app_id: String) -> Self {
        Self {
            app_id,
            permissions: HashMap::new(),
            registered: HashSet::new(),
        }
    }
    
    /// Register capability
    pub fn register(&mut self, cap: AppCapability) {
        self.registered.insert(cap);
        if !self.permissions.contains_key(&cap) {
            self.permissions.insert(cap, Permission::new(cap));
        }
    }
    
    /// Get permission
    pub fn get(&self, cap: &AppCapability) -> Option<&Permission> {
        self.permissions.get(cap)
    }
    
    /// Get mutable permission
    pub fn get_mut(&mut self, cap: &AppCapability) -> Option<&mut Permission> {
        self.permissions.get_mut(cap)
    }
    
    /// Check if capability is granted
    pub fn is_granted(&self, cap: &AppCapability) -> bool {
        self.permissions.get(cap)
            .map(|p| p.level.is_granted())
            .unwrap_or(false)
    }
    
    /// Check if needs prompt
    pub fn needs_prompt(&self, cap: &AppCapability) -> bool {
        self.permissions.get(cap)
            .map(|p| p.level.needs_prompt())
            .unwrap_or(true)
    }
    
    /// Grant permission
    pub fn grant(&mut self, cap: AppCapability, permanent: bool) {
        if let Some(perm) = self.permissions.get_mut(&cap) {
            perm.grant(permanent);
        } else {
            let mut perm = Permission::new(cap);
            perm.grant(permanent);
            self.permissions.insert(cap, perm);
        }
    }
    
    /// Deny permission
    pub fn deny(&mut self, cap: AppCapability) {
        if let Some(perm) = self.permissions.get_mut(&cap) {
            perm.deny();
        } else {
            let mut perm = Permission::new(cap);
            perm.deny();
            self.permissions.insert(cap, perm);
        }
    }
    
    /// Get all registered capabilities
    pub fn registered_capabilities(&self) -> Vec<AppCapability> {
        self.registered.iter().copied().collect()
    }
    
    /// Get granted capabilities
    pub fn granted_capabilities(&self) -> Vec<AppCapability> {
        self.permissions.iter()
            .filter(|(_, p)| p.level.is_granted())
            .map(|(c, _)| *c)
            .collect()
    }
    
    /// Get denied capabilities
    pub fn denied_capabilities(&self) -> Vec<AppCapability> {
        self.permissions.iter()
            .filter(|(_, p)| p.level == PermissionLevel::Denied)
            .map(|(c, _)| *c)
            .collect()
    }
    
    /// Reset session permissions
    pub fn reset_session(&mut self) {
        for perm in self.permissions.values_mut() {
            if perm.level == PermissionLevel::GrantedSession {
                perm.level = PermissionLevel::NotRequested;
            }
        }
    }
    
    /// Reset all permissions
    pub fn reset_all(&mut self) {
        for perm in self.permissions.values_mut() {
            perm.reset();
        }
    }
}

/// Permission manager
#[derive(Debug)]
pub struct PermissionManager {
    /// Per-app permissions
    apps: HashMap<String, AppPermissions>,
    /// Pending requests
    pending: Vec<PermissionRequest>,
    /// Next request ID
    next_request_id: u64,
    /// Auto-grant non-sensitive
    auto_grant_non_sensitive: bool,
}

impl PermissionManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
            pending: Vec::new(),
            next_request_id: 1,
            auto_grant_non_sensitive: true,
        }
    }
    
    /// Register app capability
    pub fn register_capability(&mut self, app_id: &str, cap: &AppCapability) {
        let perms = self.apps.entry(app_id.to_string())
            .or_insert_with(|| AppPermissions::new(app_id.to_string()));
        perms.register(*cap);
    }
    
    /// Remove app
    pub fn remove_app(&mut self, app_id: &str) {
        self.apps.remove(app_id);
        self.pending.retain(|r| r.app_id != app_id);
    }
    
    /// Check permission
    pub fn check(&self, app_id: &str, cap: &AppCapability) -> bool {
        self.apps.get(app_id)
            .map(|p| p.is_granted(cap))
            .unwrap_or(false)
    }
    
    /// Request permission
    pub fn request(&mut self, app_id: &str, app_name: &str, cap: AppCapability, reason: Option<String>) -> Option<u64> {
        // Record request
        if let Some(perms) = self.apps.get_mut(app_id) {
            if let Some(perm) = perms.get_mut(&cap) {
                perm.record_request();
            }
        }
        
        // Check if already granted
        if self.check(app_id, &cap) {
            return None; // No prompt needed
        }
        
        // Auto-grant non-sensitive
        if self.auto_grant_non_sensitive && !cap.is_sensitive() {
            self.grant(app_id, cap, true);
            return None;
        }
        
        // Create request
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        
        let mut request = PermissionRequest::new(
            app_id.to_string(),
            app_name.to_string(),
            cap,
            request_id,
        );
        
        if let Some(r) = reason {
            request = request.with_reason(r);
        }
        
        self.pending.push(request);
        
        Some(request_id)
    }
    
    /// Get pending requests
    pub fn pending_requests(&self) -> &[PermissionRequest] {
        &self.pending
    }
    
    /// Has pending requests
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
    
    /// Respond to request
    pub fn respond(&mut self, request_id: u64, granted: bool, permanent: bool) {
        if let Some(pos) = self.pending.iter().position(|r| r.request_id == request_id) {
            let request = self.pending.remove(pos);
            
            if granted {
                self.grant(&request.app_id, request.capability, permanent);
            } else {
                self.deny(&request.app_id, request.capability);
            }
        }
    }
    
    /// Grant permission
    pub fn grant(&mut self, app_id: &str, cap: AppCapability, permanent: bool) {
        if let Some(perms) = self.apps.get_mut(app_id) {
            perms.grant(cap, permanent);
        } else {
            let mut perms = AppPermissions::new(app_id.to_string());
            perms.grant(cap, permanent);
            self.apps.insert(app_id.to_string(), perms);
        }
    }
    
    /// Deny permission
    pub fn deny(&mut self, app_id: &str, cap: AppCapability) {
        if let Some(perms) = self.apps.get_mut(app_id) {
            perms.deny(cap);
        }
    }
    
    /// Revoke permission
    pub fn revoke(&mut self, app_id: &str, cap: AppCapability) {
        if let Some(perms) = self.apps.get_mut(app_id) {
            if let Some(perm) = perms.get_mut(&cap) {
                perm.reset();
            }
        }
    }
    
    /// Get app permissions
    pub fn get_app_permissions(&self, app_id: &str) -> Option<&AppPermissions> {
        self.apps.get(app_id)
    }
    
    /// Reset session permissions for all apps
    pub fn reset_session(&mut self) {
        for perms in self.apps.values_mut() {
            perms.reset_session();
        }
    }
    
    /// Set auto-grant non-sensitive
    pub fn set_auto_grant(&mut self, enabled: bool) {
        self.auto_grant_non_sensitive = enabled;
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
    fn test_permission_level() {
        assert!(PermissionLevel::GrantedPermanent.is_granted());
        assert!(PermissionLevel::GrantedSession.is_granted());
        assert!(!PermissionLevel::Denied.is_granted());
        
        assert!(PermissionLevel::NotRequested.needs_prompt());
        assert!(!PermissionLevel::Denied.needs_prompt());
    }
    
    #[test]
    fn test_permission() {
        let mut perm = Permission::new(AppCapability::Camera);
        
        assert!(!perm.level.is_granted());
        
        perm.grant(true);
        assert!(perm.level.is_granted());
        assert_eq!(perm.level, PermissionLevel::GrantedPermanent);
    }
    
    #[test]
    fn test_app_permissions() {
        let mut perms = AppPermissions::new("com.test.app".to_string());
        
        perms.register(AppCapability::Camera);
        perms.register(AppCapability::Network);
        
        assert!(!perms.is_granted(&AppCapability::Camera));
        
        perms.grant(AppCapability::Camera, true);
        assert!(perms.is_granted(&AppCapability::Camera));
    }
    
    #[test]
    fn test_permission_manager() {
        let mut manager = PermissionManager::new();
        
        manager.register_capability("com.test.app", &AppCapability::Camera);
        
        assert!(!manager.check("com.test.app", &AppCapability::Camera));
        
        manager.grant("com.test.app", AppCapability::Camera, true);
        assert!(manager.check("com.test.app", &AppCapability::Camera));
    }
    
    #[test]
    fn test_permission_request() {
        let mut manager = PermissionManager::new();
        manager.set_auto_grant(false);
        
        manager.register_capability("com.test.app", &AppCapability::Camera);
        
        let req_id = manager.request(
            "com.test.app",
            "Test App",
            AppCapability::Camera,
            None,
        );
        
        assert!(req_id.is_some());
        assert!(manager.has_pending());
        
        manager.respond(req_id.unwrap(), true, true);
        assert!(!manager.has_pending());
        assert!(manager.check("com.test.app", &AppCapability::Camera));
    }
    
    #[test]
    fn test_auto_grant_non_sensitive() {
        let mut manager = PermissionManager::new();
        manager.set_auto_grant(true);
        
        let req_id = manager.request(
            "com.test.app",
            "Test App",
            AppCapability::Network, // Non-sensitive
            None,
        );
        
        // Should auto-grant
        assert!(req_id.is_none());
        assert!(manager.check("com.test.app", &AppCapability::Network));
    }
    
    #[test]
    fn test_revoke_permission() {
        let mut manager = PermissionManager::new();
        
        manager.grant("com.test.app", AppCapability::Camera, true);
        assert!(manager.check("com.test.app", &AppCapability::Camera));
        
        manager.revoke("com.test.app", AppCapability::Camera);
        assert!(!manager.check("com.test.app", &AppCapability::Camera));
    }
}
