// Access control system for Kāraṇa OS

use std::collections::{HashMap, HashSet};

/// Access controller for permission management
pub struct AccessController {
    users: HashMap<String, User>,
    roles: HashMap<String, Role>,
    permissions: HashMap<String, PermissionDefinition>,
    user_roles: HashMap<String, HashSet<String>>,
    resource_policies: HashMap<String, ResourcePolicy>,
}

/// User information
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub is_active: bool,
    pub is_admin: bool,
}

/// Role definition
#[derive(Debug, Clone)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashSet<Permission>,
    pub is_system_role: bool,
}

/// Permission definition
#[derive(Debug, Clone)]
pub struct PermissionDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PermissionCategory,
    pub risk_level: RiskLevel,
}

/// Permission identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    // System permissions
    SystemSettings,
    SystemUpdate,
    SystemDiagnostics,
    SystemShutdown,
    
    // Camera/Sensors
    CameraAccess,
    MicrophoneAccess,
    LocationAccess,
    SensorAccess,
    
    // Data permissions
    ReadContacts,
    WriteContacts,
    ReadCalendar,
    WriteCalendar,
    ReadStorage,
    WriteStorage,
    
    // Network
    NetworkAccess,
    BluetoothAccess,
    WifiManagement,
    
    // AR/Display
    AROverlay,
    DisplayControl,
    NotificationAccess,
    
    // App management
    InstallApps,
    UninstallApps,
    ManageApps,
    
    // Security
    SecuritySettings,
    BiometricEnroll,
    ViewAuditLog,
    
    // Custom permission
    Custom(String),
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SystemSettings => "system.settings",
            Self::SystemUpdate => "system.update",
            Self::SystemDiagnostics => "system.diagnostics",
            Self::SystemShutdown => "system.shutdown",
            Self::CameraAccess => "sensor.camera",
            Self::MicrophoneAccess => "sensor.microphone",
            Self::LocationAccess => "sensor.location",
            Self::SensorAccess => "sensor.generic",
            Self::ReadContacts => "data.contacts.read",
            Self::WriteContacts => "data.contacts.write",
            Self::ReadCalendar => "data.calendar.read",
            Self::WriteCalendar => "data.calendar.write",
            Self::ReadStorage => "storage.read",
            Self::WriteStorage => "storage.write",
            Self::NetworkAccess => "network.access",
            Self::BluetoothAccess => "network.bluetooth",
            Self::WifiManagement => "network.wifi",
            Self::AROverlay => "display.ar",
            Self::DisplayControl => "display.control",
            Self::NotificationAccess => "display.notifications",
            Self::InstallApps => "apps.install",
            Self::UninstallApps => "apps.uninstall",
            Self::ManageApps => "apps.manage",
            Self::SecuritySettings => "security.settings",
            Self::BiometricEnroll => "security.biometric",
            Self::ViewAuditLog => "security.audit",
            Self::Custom(s) => s,
        }
    }
}

/// Permission category
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermissionCategory {
    System,
    Sensors,
    Data,
    Network,
    Display,
    Apps,
    Security,
    Custom,
}

/// Risk level of permission
#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Policy for resource access
#[derive(Debug, Clone)]
pub struct ResourcePolicy {
    pub resource_id: String,
    pub resource_type: ResourceType,
    pub owner_id: String,
    pub access_rules: Vec<AccessRule>,
    pub default_action: AccessAction,
}

/// Type of resource
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    File,
    Directory,
    App,
    Setting,
    Sensor,
    API,
}

/// Access rule for resource
#[derive(Debug, Clone)]
pub struct AccessRule {
    pub subject: AccessSubject,
    pub actions: HashSet<AccessAction>,
    pub conditions: Vec<AccessCondition>,
}

/// Subject of access control
#[derive(Debug, Clone, PartialEq)]
pub enum AccessSubject {
    User(String),
    Role(String),
    App(String),
    Everyone,
}

/// Access action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessAction {
    Read,
    Write,
    Execute,
    Delete,
    Admin,
    Deny,
}

/// Condition for access
#[derive(Debug, Clone)]
pub enum AccessCondition {
    TimeRange { start: u64, end: u64 },
    Location { latitude: f64, longitude: f64, radius_m: f64 },
    DeviceState(DeviceStateCondition),
    Custom(String),
}

/// Device state condition
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceStateCondition {
    Locked,
    Unlocked,
    Charging,
    OnWifi,
    LowBattery,
}

impl AccessController {
    pub fn new() -> Self {
        let mut controller = Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            permissions: HashMap::new(),
            user_roles: HashMap::new(),
            resource_policies: HashMap::new(),
        };
        
        // Initialize default roles
        controller.create_default_roles();
        
        controller
    }
    
    /// Create default system roles
    fn create_default_roles(&mut self) {
        // Admin role
        let mut admin_perms = HashSet::new();
        admin_perms.insert(Permission::SystemSettings);
        admin_perms.insert(Permission::SystemUpdate);
        admin_perms.insert(Permission::SystemDiagnostics);
        admin_perms.insert(Permission::SystemShutdown);
        admin_perms.insert(Permission::SecuritySettings);
        admin_perms.insert(Permission::ViewAuditLog);
        admin_perms.insert(Permission::ManageApps);
        
        self.roles.insert("admin".to_string(), Role {
            id: "admin".to_string(),
            name: "Administrator".to_string(),
            description: "Full system access".to_string(),
            permissions: admin_perms,
            is_system_role: true,
        });
        
        // User role
        let mut user_perms = HashSet::new();
        user_perms.insert(Permission::CameraAccess);
        user_perms.insert(Permission::MicrophoneAccess);
        user_perms.insert(Permission::ReadStorage);
        user_perms.insert(Permission::NetworkAccess);
        user_perms.insert(Permission::AROverlay);
        
        self.roles.insert("user".to_string(), Role {
            id: "user".to_string(),
            name: "Standard User".to_string(),
            description: "Standard user access".to_string(),
            permissions: user_perms,
            is_system_role: true,
        });
        
        // Guest role
        let guest_perms = HashSet::new();
        
        self.roles.insert("guest".to_string(), Role {
            id: "guest".to_string(),
            name: "Guest".to_string(),
            description: "Limited guest access".to_string(),
            permissions: guest_perms,
            is_system_role: true,
        });
    }
    
    /// Add a user
    pub fn add_user(&mut self, user: User) {
        self.user_roles.insert(user.id.clone(), HashSet::new());
        self.users.insert(user.id.clone(), user);
    }
    
    /// Remove a user
    pub fn remove_user(&mut self, user_id: &str) -> bool {
        self.user_roles.remove(user_id);
        self.users.remove(user_id).is_some()
    }
    
    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> Option<&User> {
        self.users.get(user_id)
    }
    
    /// Assign role to user
    pub fn assign_role(&mut self, user_id: &str, role_id: &str) -> bool {
        if !self.roles.contains_key(role_id) {
            return false;
        }
        
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.insert(role_id.to_string());
            true
        } else {
            false
        }
    }
    
    /// Remove role from user
    pub fn revoke_role(&mut self, user_id: &str, role_id: &str) -> bool {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.remove(role_id)
        } else {
            false
        }
    }
    
    /// Get user's roles
    pub fn get_user_roles(&self, user_id: &str) -> Vec<&Role> {
        self.user_roles.get(user_id)
            .map(|role_ids| {
                role_ids.iter()
                    .filter_map(|id| self.roles.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get user's permissions (aggregated from all roles)
    pub fn get_user_permissions(&self, user_id: &str) -> Vec<Permission> {
        let mut permissions = HashSet::new();
        
        // Check if admin
        if let Some(user) = self.users.get(user_id) {
            if user.is_admin {
                // Admin gets all permissions
                permissions.insert(Permission::SystemSettings);
                permissions.insert(Permission::SystemUpdate);
                permissions.insert(Permission::SystemDiagnostics);
                permissions.insert(Permission::SystemShutdown);
                permissions.insert(Permission::SecuritySettings);
                permissions.insert(Permission::ViewAuditLog);
            }
        }
        
        // Get permissions from roles
        for role in self.get_user_roles(user_id) {
            permissions.extend(role.permissions.iter().cloned());
        }
        
        permissions.into_iter().collect()
    }
    
    /// Check if user has permission
    pub fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        // Admin check
        if let Some(user) = self.users.get(user_id) {
            if user.is_admin {
                return true;
            }
        }
        
        // Check roles
        for role in self.get_user_roles(user_id) {
            if role.permissions.contains(permission) {
                return true;
            }
        }
        
        false
    }
    
    /// Create a custom role
    pub fn create_role(&mut self, role: Role) {
        self.roles.insert(role.id.clone(), role);
    }
    
    /// Delete a role (only non-system roles)
    pub fn delete_role(&mut self, role_id: &str) -> bool {
        if let Some(role) = self.roles.get(role_id) {
            if role.is_system_role {
                return false;
            }
        }
        
        // Remove role from all users
        for user_roles in self.user_roles.values_mut() {
            user_roles.remove(role_id);
        }
        
        self.roles.remove(role_id).is_some()
    }
    
    /// Add permission to role
    pub fn add_permission_to_role(&mut self, role_id: &str, permission: Permission) -> bool {
        if let Some(role) = self.roles.get_mut(role_id) {
            role.permissions.insert(permission);
            true
        } else {
            false
        }
    }
    
    /// Remove permission from role
    pub fn remove_permission_from_role(&mut self, role_id: &str, permission: &Permission) -> bool {
        if let Some(role) = self.roles.get_mut(role_id) {
            role.permissions.remove(permission)
        } else {
            false
        }
    }
    
    /// Set resource policy
    pub fn set_resource_policy(&mut self, policy: ResourcePolicy) {
        self.resource_policies.insert(policy.resource_id.clone(), policy);
    }
    
    /// Check resource access
    pub fn check_resource_access(
        &self, 
        user_id: &str, 
        resource_id: &str, 
        action: AccessAction
    ) -> bool {
        let policy = match self.resource_policies.get(resource_id) {
            Some(p) => p,
            None => return false, // No policy = no access
        };
        
        // Check owner
        if policy.owner_id == user_id {
            return true;
        }
        
        // Check rules
        for rule in &policy.access_rules {
            let subject_matches = match &rule.subject {
                AccessSubject::User(id) => id == user_id,
                AccessSubject::Role(role_id) => {
                    self.user_roles.get(user_id)
                        .map(|roles| roles.contains(role_id))
                        .unwrap_or(false)
                }
                AccessSubject::Everyone => true,
                AccessSubject::App(_) => false, // Would check app context
            };
            
            if subject_matches {
                if rule.actions.contains(&AccessAction::Deny) {
                    return false;
                }
                if rule.actions.contains(&action) || rule.actions.contains(&AccessAction::Admin) {
                    return true;
                }
            }
        }
        
        // Default action
        policy.default_action == action
    }
    
    /// Get all roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }
    
    /// Get all users
    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}

impl Default for AccessController {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission request for runtime permission prompts
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub app_id: String,
    pub permission: Permission,
    pub reason: String,
    pub requested_at: u64,
    pub granted: Option<bool>,
}

/// Permission grant record
#[derive(Debug, Clone)]
pub struct PermissionGrant {
    pub app_id: String,
    pub permission: Permission,
    pub granted_at: u64,
    pub granted_by: String,
    pub expires_at: Option<u64>,
    pub is_revoked: bool,
}

/// App permission manager
pub struct AppPermissionManager {
    grants: HashMap<String, Vec<PermissionGrant>>,
    pending_requests: Vec<PermissionRequest>,
}

impl AppPermissionManager {
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
            pending_requests: Vec::new(),
        }
    }
    
    /// Request permission for app
    pub fn request_permission(&mut self, app_id: &str, permission: Permission, reason: &str) {
        self.pending_requests.push(PermissionRequest {
            app_id: app_id.to_string(),
            permission,
            reason: reason.to_string(),
            requested_at: 0,
            granted: None,
        });
    }
    
    /// Grant permission to app
    pub fn grant_permission(&mut self, app_id: &str, permission: Permission, granted_by: &str) {
        let grant = PermissionGrant {
            app_id: app_id.to_string(),
            permission,
            granted_at: 0,
            granted_by: granted_by.to_string(),
            expires_at: None,
            is_revoked: false,
        };
        
        self.grants
            .entry(app_id.to_string())
            .or_insert_with(Vec::new)
            .push(grant);
    }
    
    /// Revoke permission from app
    pub fn revoke_permission(&mut self, app_id: &str, permission: &Permission) -> bool {
        if let Some(grants) = self.grants.get_mut(app_id) {
            for grant in grants.iter_mut() {
                if &grant.permission == permission && !grant.is_revoked {
                    grant.is_revoked = true;
                    return true;
                }
            }
        }
        false
    }
    
    /// Check if app has permission
    pub fn has_permission(&self, app_id: &str, permission: &Permission) -> bool {
        self.grants.get(app_id)
            .map(|grants| {
                grants.iter().any(|g| {
                    &g.permission == permission && 
                    !g.is_revoked &&
                    g.expires_at.map(|exp| exp > 0).unwrap_or(true)
                })
            })
            .unwrap_or(false)
    }
    
    /// Get app's permissions
    pub fn get_app_permissions(&self, app_id: &str) -> Vec<&Permission> {
        self.grants.get(app_id)
            .map(|grants| {
                grants.iter()
                    .filter(|g| !g.is_revoked)
                    .map(|g| &g.permission)
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get pending requests
    pub fn pending_requests(&self) -> &[PermissionRequest] {
        &self.pending_requests
    }
    
    /// Clear pending requests for app
    pub fn clear_pending(&mut self, app_id: &str) {
        self.pending_requests.retain(|r| r.app_id != app_id);
    }
}

impl Default for AppPermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_access_controller_creation() {
        let controller = AccessController::new();
        
        // Should have default roles
        assert!(controller.roles.contains_key("admin"));
        assert!(controller.roles.contains_key("user"));
        assert!(controller.roles.contains_key("guest"));
    }
    
    #[test]
    fn test_add_user() {
        let mut controller = AccessController::new();
        
        controller.add_user(User {
            id: "user1".to_string(),
            name: "Test User".to_string(),
            created_at: 0,
            last_login: None,
            is_active: true,
            is_admin: false,
        });
        
        assert!(controller.get_user("user1").is_some());
    }
    
    #[test]
    fn test_assign_role() {
        let mut controller = AccessController::new();
        
        controller.add_user(User {
            id: "user1".to_string(),
            name: "Test User".to_string(),
            created_at: 0,
            last_login: None,
            is_active: true,
            is_admin: false,
        });
        
        assert!(controller.assign_role("user1", "user"));
        
        let roles = controller.get_user_roles("user1");
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].id, "user");
    }
    
    #[test]
    fn test_has_permission() {
        let mut controller = AccessController::new();
        
        controller.add_user(User {
            id: "user1".to_string(),
            name: "Test User".to_string(),
            created_at: 0,
            last_login: None,
            is_active: true,
            is_admin: false,
        });
        
        controller.assign_role("user1", "user");
        
        assert!(controller.has_permission("user1", &Permission::CameraAccess));
        assert!(!controller.has_permission("user1", &Permission::SystemSettings));
    }
    
    #[test]
    fn test_admin_has_all_permissions() {
        let mut controller = AccessController::new();
        
        controller.add_user(User {
            id: "admin1".to_string(),
            name: "Admin User".to_string(),
            created_at: 0,
            last_login: None,
            is_active: true,
            is_admin: true,
        });
        
        assert!(controller.has_permission("admin1", &Permission::SystemSettings));
        assert!(controller.has_permission("admin1", &Permission::SecuritySettings));
    }
    
    #[test]
    fn test_resource_policy() {
        let mut controller = AccessController::new();
        
        controller.add_user(User {
            id: "user1".to_string(),
            name: "Test User".to_string(),
            created_at: 0,
            last_login: None,
            is_active: true,
            is_admin: false,
        });
        
        let mut read_actions = HashSet::new();
        read_actions.insert(AccessAction::Read);
        
        controller.set_resource_policy(ResourcePolicy {
            resource_id: "file1".to_string(),
            resource_type: ResourceType::File,
            owner_id: "owner".to_string(),
            access_rules: vec![
                AccessRule {
                    subject: AccessSubject::User("user1".to_string()),
                    actions: read_actions,
                    conditions: vec![],
                },
            ],
            default_action: AccessAction::Deny,
        });
        
        assert!(controller.check_resource_access("user1", "file1", AccessAction::Read));
        assert!(!controller.check_resource_access("user1", "file1", AccessAction::Write));
    }
    
    #[test]
    fn test_owner_access() {
        let mut controller = AccessController::new();
        
        controller.add_user(User {
            id: "owner1".to_string(),
            name: "Owner".to_string(),
            created_at: 0,
            last_login: None,
            is_active: true,
            is_admin: false,
        });
        
        controller.set_resource_policy(ResourcePolicy {
            resource_id: "file1".to_string(),
            resource_type: ResourceType::File,
            owner_id: "owner1".to_string(),
            access_rules: vec![],
            default_action: AccessAction::Deny,
        });
        
        // Owner should have access
        assert!(controller.check_resource_access("owner1", "file1", AccessAction::Read));
        assert!(controller.check_resource_access("owner1", "file1", AccessAction::Write));
    }
    
    #[test]
    fn test_permission_as_str() {
        assert_eq!(Permission::CameraAccess.as_str(), "sensor.camera");
        assert_eq!(Permission::SystemSettings.as_str(), "system.settings");
    }
    
    #[test]
    fn test_app_permission_manager() {
        let mut manager = AppPermissionManager::new();
        
        manager.grant_permission("app1", Permission::CameraAccess, "user1");
        
        assert!(manager.has_permission("app1", &Permission::CameraAccess));
        assert!(!manager.has_permission("app1", &Permission::MicrophoneAccess));
    }
    
    #[test]
    fn test_revoke_permission() {
        let mut manager = AppPermissionManager::new();
        
        manager.grant_permission("app1", Permission::CameraAccess, "user1");
        assert!(manager.has_permission("app1", &Permission::CameraAccess));
        
        assert!(manager.revoke_permission("app1", &Permission::CameraAccess));
        assert!(!manager.has_permission("app1", &Permission::CameraAccess));
    }
    
    #[test]
    fn test_delete_role() {
        let mut controller = AccessController::new();
        
        // Can't delete system role
        assert!(!controller.delete_role("admin"));
        
        // Can delete custom role
        controller.create_role(Role {
            id: "custom".to_string(),
            name: "Custom".to_string(),
            description: "Custom role".to_string(),
            permissions: HashSet::new(),
            is_system_role: false,
        });
        
        assert!(controller.delete_role("custom"));
    }
}
