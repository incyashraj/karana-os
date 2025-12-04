//! App Sandbox for Kāraṇa OS AR Glasses
//!
//! Isolated execution environment for app security.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

/// Resource type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// CPU time
    CPU,
    /// Memory
    Memory,
    /// Disk storage
    Disk,
    /// Network bandwidth
    Network,
    /// GPU
    GPU,
    /// File handles
    FileHandles,
    /// Threads
    Threads,
}

/// Resource limit
#[derive(Debug, Clone)]
pub struct ResourceLimit {
    /// Resource type
    pub resource_type: ResourceType,
    /// Soft limit
    pub soft_limit: u64,
    /// Hard limit
    pub hard_limit: u64,
    /// Current usage
    pub current_usage: u64,
}

impl ResourceLimit {
    /// Create new limit
    pub fn new(resource_type: ResourceType, soft: u64, hard: u64) -> Self {
        Self {
            resource_type,
            soft_limit: soft,
            hard_limit: hard,
            current_usage: 0,
        }
    }
    
    /// Check if at soft limit
    pub fn at_soft_limit(&self) -> bool {
        self.current_usage >= self.soft_limit
    }
    
    /// Check if at hard limit
    pub fn at_hard_limit(&self) -> bool {
        self.current_usage >= self.hard_limit
    }
    
    /// Usage percentage
    pub fn usage_percent(&self) -> f32 {
        if self.hard_limit == 0 {
            0.0
        } else {
            (self.current_usage as f32 / self.hard_limit as f32) * 100.0
        }
    }
    
    /// Try allocate
    pub fn try_allocate(&mut self, amount: u64) -> bool {
        if self.current_usage + amount > self.hard_limit {
            false
        } else {
            self.current_usage += amount;
            true
        }
    }
    
    /// Release
    pub fn release(&mut self, amount: u64) {
        self.current_usage = self.current_usage.saturating_sub(amount);
    }
}

/// File system access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAccess {
    /// No access
    None,
    /// Read only
    ReadOnly,
    /// Write only
    WriteOnly,
    /// Read and write
    ReadWrite,
}

impl FileAccess {
    /// Can read
    pub fn can_read(&self) -> bool {
        matches!(self, Self::ReadOnly | Self::ReadWrite)
    }
    
    /// Can write
    pub fn can_write(&self) -> bool {
        matches!(self, Self::WriteOnly | Self::ReadWrite)
    }
}

/// File system rule
#[derive(Debug, Clone)]
pub struct FileSystemRule {
    /// Path pattern
    pub path: PathBuf,
    /// Access level
    pub access: FileAccess,
    /// Is recursive
    pub recursive: bool,
}

impl FileSystemRule {
    /// Create new rule
    pub fn new(path: PathBuf, access: FileAccess) -> Self {
        Self {
            path,
            access,
            recursive: false,
        }
    }
    
    /// With recursive
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }
    
    /// Check if path matches
    pub fn matches(&self, target: &PathBuf) -> bool {
        if self.recursive {
            target.starts_with(&self.path)
        } else {
            target == &self.path
        }
    }
}

/// Network rule
#[derive(Debug, Clone)]
pub struct NetworkRule {
    /// Host pattern
    pub host: String,
    /// Port range
    pub port_start: u16,
    pub port_end: u16,
    /// Allowed
    pub allowed: bool,
}

impl NetworkRule {
    /// Create new rule
    pub fn allow(host: String) -> Self {
        Self {
            host,
            port_start: 0,
            port_end: 65535,
            allowed: true,
        }
    }
    
    /// Deny rule
    pub fn deny(host: String) -> Self {
        Self {
            host,
            port_start: 0,
            port_end: 65535,
            allowed: false,
        }
    }
    
    /// With port range
    pub fn with_ports(mut self, start: u16, end: u16) -> Self {
        self.port_start = start;
        self.port_end = end;
        self
    }
    
    /// Check if matches
    pub fn matches(&self, host: &str, port: u16) -> bool {
        let host_match = if self.host == "*" {
            true
        } else if self.host.starts_with("*.") {
            host.ends_with(&self.host[1..])
        } else {
            host == self.host
        };
        
        host_match && port >= self.port_start && port <= self.port_end
    }
}

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// App ID
    pub app_id: String,
    /// CPU limit (percentage 0-100)
    pub cpu_limit: f32,
    /// Memory limit (bytes)
    pub memory_limit: u64,
    /// Disk limit (bytes)
    pub disk_limit: u64,
    /// Network bandwidth limit (bytes/sec)
    pub bandwidth_limit: u64,
    /// Max threads
    pub max_threads: u32,
    /// Max file handles
    pub max_file_handles: u32,
    /// Execution timeout
    pub timeout: Option<Duration>,
    /// File system rules
    pub fs_rules: Vec<FileSystemRule>,
    /// Network rules
    pub network_rules: Vec<NetworkRule>,
    /// Allowed syscalls
    pub allowed_syscalls: HashSet<String>,
}

impl SandboxConfig {
    /// Create default config for app
    pub fn default_for(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            cpu_limit: 50.0,
            memory_limit: 100 * 1024 * 1024, // 100MB
            disk_limit: 50 * 1024 * 1024, // 50MB
            bandwidth_limit: 1024 * 1024, // 1MB/s
            max_threads: 8,
            max_file_handles: 64,
            timeout: None,
            fs_rules: Vec::new(),
            network_rules: vec![
                NetworkRule::allow("*".to_string()),
            ],
            allowed_syscalls: HashSet::new(),
        }
    }
    
    /// Restrictive config
    pub fn restrictive_for(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            cpu_limit: 25.0,
            memory_limit: 50 * 1024 * 1024, // 50MB
            disk_limit: 10 * 1024 * 1024, // 10MB
            bandwidth_limit: 512 * 1024, // 512KB/s
            max_threads: 4,
            max_file_handles: 16,
            timeout: Some(Duration::from_secs(30)),
            fs_rules: Vec::new(),
            network_rules: Vec::new(),
            allowed_syscalls: HashSet::new(),
        }
    }
    
    /// Add file system rule
    pub fn add_fs_rule(&mut self, rule: FileSystemRule) {
        self.fs_rules.push(rule);
    }
    
    /// Add network rule
    pub fn add_network_rule(&mut self, rule: NetworkRule) {
        self.network_rules.push(rule);
    }
}

/// Sandbox violation
#[derive(Debug, Clone)]
pub struct SandboxViolation {
    /// Violation type
    pub violation_type: String,
    /// Details
    pub details: String,
    /// Severity
    pub severity: ViolationSeverity,
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationSeverity {
    /// Warning (logged but allowed)
    Warning,
    /// Error (blocked)
    Error,
    /// Critical (app terminated)
    Critical,
}

/// Sandbox state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    /// Not started
    Idle,
    /// Running
    Running,
    /// Suspended
    Suspended,
    /// Terminated
    Terminated,
}

/// App sandbox
#[derive(Debug)]
pub struct Sandbox {
    /// Configuration
    config: SandboxConfig,
    /// State
    state: SandboxState,
    /// Resource limits
    resources: Vec<ResourceLimit>,
    /// Violations
    violations: Vec<SandboxViolation>,
    /// Max violations before termination
    max_violations: usize,
}

impl Sandbox {
    /// Create new sandbox
    pub fn new(config: SandboxConfig) -> Self {
        let resources = vec![
            ResourceLimit::new(ResourceType::CPU, 
                (config.cpu_limit * 0.8) as u64, 
                config.cpu_limit as u64),
            ResourceLimit::new(ResourceType::Memory, 
                (config.memory_limit as f64 * 0.8) as u64, 
                config.memory_limit),
            ResourceLimit::new(ResourceType::Disk,
                (config.disk_limit as f64 * 0.8) as u64,
                config.disk_limit),
            ResourceLimit::new(ResourceType::FileHandles,
                (config.max_file_handles as f64 * 0.8) as u64,
                config.max_file_handles as u64),
            ResourceLimit::new(ResourceType::Threads,
                (config.max_threads as f64 * 0.8) as u64,
                config.max_threads as u64),
        ];
        
        Self {
            config,
            state: SandboxState::Idle,
            resources,
            violations: Vec::new(),
            max_violations: 10,
        }
    }
    
    /// Get config
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }
    
    /// Get state
    pub fn state(&self) -> SandboxState {
        self.state
    }
    
    /// Start sandbox
    pub fn start(&mut self) {
        self.state = SandboxState::Running;
    }
    
    /// Suspend sandbox
    pub fn suspend(&mut self) {
        if self.state == SandboxState::Running {
            self.state = SandboxState::Suspended;
        }
    }
    
    /// Resume sandbox
    pub fn resume(&mut self) {
        if self.state == SandboxState::Suspended {
            self.state = SandboxState::Running;
        }
    }
    
    /// Terminate sandbox
    pub fn terminate(&mut self) {
        self.state = SandboxState::Terminated;
    }
    
    /// Check file access
    pub fn check_file_access(&self, path: &PathBuf, write: bool) -> bool {
        for rule in &self.config.fs_rules {
            if rule.matches(path) {
                if write {
                    return rule.access.can_write();
                } else {
                    return rule.access.can_read();
                }
            }
        }
        false // Deny by default
    }
    
    /// Check network access
    pub fn check_network(&self, host: &str, port: u16) -> bool {
        // Check deny rules first
        for rule in &self.config.network_rules {
            if !rule.allowed && rule.matches(host, port) {
                return false;
            }
        }
        
        // Check allow rules
        for rule in &self.config.network_rules {
            if rule.allowed && rule.matches(host, port) {
                return true;
            }
        }
        
        false // Deny by default if no rules match
    }
    
    /// Try allocate resource
    pub fn try_allocate(&mut self, resource: ResourceType, amount: u64) -> bool {
        for limit in &mut self.resources {
            if limit.resource_type == resource {
                return limit.try_allocate(amount);
            }
        }
        false
    }
    
    /// Release resource
    pub fn release(&mut self, resource: ResourceType, amount: u64) {
        for limit in &mut self.resources {
            if limit.resource_type == resource {
                limit.release(amount);
                return;
            }
        }
    }
    
    /// Get resource usage
    pub fn resource_usage(&self, resource: ResourceType) -> Option<(u64, u64)> {
        for limit in &self.resources {
            if limit.resource_type == resource {
                return Some((limit.current_usage, limit.hard_limit));
            }
        }
        None
    }
    
    /// Record violation
    pub fn record_violation(&mut self, violation: SandboxViolation) {
        let should_terminate = violation.severity == ViolationSeverity::Critical ||
            self.violations.len() >= self.max_violations;
        
        self.violations.push(violation);
        
        if should_terminate {
            self.terminate();
        }
    }
    
    /// Get violations
    pub fn violations(&self) -> &[SandboxViolation] {
        &self.violations
    }
    
    /// Clear violations
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_limit() {
        let mut limit = ResourceLimit::new(ResourceType::Memory, 80, 100);
        
        assert!(!limit.at_soft_limit());
        assert!(limit.try_allocate(50));
        assert!(!limit.at_soft_limit());
        
        assert!(limit.try_allocate(40));
        assert!(limit.at_soft_limit());
        
        assert!(!limit.try_allocate(20)); // Would exceed hard limit
    }
    
    #[test]
    fn test_file_system_rule() {
        let rule = FileSystemRule::new(
            PathBuf::from("/app/data"),
            FileAccess::ReadWrite,
        ).recursive();
        
        assert!(rule.matches(&PathBuf::from("/app/data/file.txt")));
        assert!(!rule.matches(&PathBuf::from("/other/file.txt")));
    }
    
    #[test]
    fn test_network_rule() {
        let rule = NetworkRule::allow("*.example.com".to_string())
            .with_ports(80, 443);
        
        assert!(rule.matches("api.example.com", 443));
        assert!(!rule.matches("api.example.com", 8080));
        assert!(!rule.matches("other.com", 443));
    }
    
    #[test]
    fn test_sandbox_config() {
        let config = SandboxConfig::default_for("com.test.app");
        
        assert_eq!(config.cpu_limit, 50.0);
        assert!(config.memory_limit > 0);
    }
    
    #[test]
    fn test_sandbox_creation() {
        let config = SandboxConfig::default_for("com.test.app");
        let sandbox = Sandbox::new(config);
        
        assert_eq!(sandbox.state(), SandboxState::Idle);
    }
    
    #[test]
    fn test_sandbox_lifecycle() {
        let config = SandboxConfig::default_for("com.test.app");
        let mut sandbox = Sandbox::new(config);
        
        sandbox.start();
        assert_eq!(sandbox.state(), SandboxState::Running);
        
        sandbox.suspend();
        assert_eq!(sandbox.state(), SandboxState::Suspended);
        
        sandbox.resume();
        assert_eq!(sandbox.state(), SandboxState::Running);
        
        sandbox.terminate();
        assert_eq!(sandbox.state(), SandboxState::Terminated);
    }
    
    #[test]
    fn test_sandbox_resource_allocation() {
        let config = SandboxConfig::default_for("com.test.app");
        let mut sandbox = Sandbox::new(config);
        
        assert!(sandbox.try_allocate(ResourceType::Memory, 1024));
        
        let (used, _) = sandbox.resource_usage(ResourceType::Memory).unwrap();
        assert_eq!(used, 1024);
        
        sandbox.release(ResourceType::Memory, 1024);
        let (used, _) = sandbox.resource_usage(ResourceType::Memory).unwrap();
        assert_eq!(used, 0);
    }
    
    #[test]
    fn test_sandbox_file_access() {
        let mut config = SandboxConfig::default_for("com.test.app");
        config.add_fs_rule(FileSystemRule::new(
            PathBuf::from("/app/data"),
            FileAccess::ReadOnly,
        ).recursive());
        
        let sandbox = Sandbox::new(config);
        
        assert!(sandbox.check_file_access(&PathBuf::from("/app/data/file.txt"), false));
        assert!(!sandbox.check_file_access(&PathBuf::from("/app/data/file.txt"), true));
    }
    
    #[test]
    fn test_violation_termination() {
        let config = SandboxConfig::default_for("com.test.app");
        let mut sandbox = Sandbox::new(config);
        sandbox.start();
        
        sandbox.record_violation(SandboxViolation {
            violation_type: "test".to_string(),
            details: "test".to_string(),
            severity: ViolationSeverity::Critical,
        });
        
        assert_eq!(sandbox.state(), SandboxState::Terminated);
    }
}
