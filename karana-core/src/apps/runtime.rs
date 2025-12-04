//! App Runtime for Kāraṇa OS AR Glasses
//!
//! Execution environment for apps.

use std::collections::HashMap;
use std::time::Instant;

use super::manifest::AppType;

/// App state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Starting up
    Starting,
    /// Running
    Running,
    /// Suspended (paused)
    Suspended,
    /// Stopping
    Stopping,
    /// Stopped
    Stopped,
    /// Crashed
    Crashed,
}

impl AppState {
    /// Is active (running or suspended)
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Suspended)
    }
    
    /// Can transition to
    pub fn can_transition_to(&self, target: AppState) -> bool {
        match (self, target) {
            (Self::Starting, Self::Running) => true,
            (Self::Starting, Self::Crashed) => true,
            (Self::Running, Self::Suspended) => true,
            (Self::Running, Self::Stopping) => true,
            (Self::Running, Self::Crashed) => true,
            (Self::Suspended, Self::Running) => true,
            (Self::Suspended, Self::Stopping) => true,
            (Self::Stopping, Self::Stopped) => true,
            _ => false,
        }
    }
}

/// App instance
#[derive(Debug)]
pub struct AppInstance {
    /// App ID
    app_id: String,
    /// App type
    app_type: AppType,
    /// Current state
    state: AppState,
    /// Started at
    started: Instant,
    /// Last active
    last_active: Instant,
    /// CPU usage (0-100)
    cpu_usage: f32,
    /// Memory usage (bytes)
    memory_usage: u64,
    /// Process ID
    pid: Option<u32>,
    /// Error message (if crashed)
    error: Option<String>,
}

impl AppInstance {
    /// Create new instance
    pub fn new(app_id: String, app_type: AppType) -> Self {
        let now = Instant::now();
        Self {
            app_id,
            app_type,
            state: AppState::Starting,
            started: now,
            last_active: now,
            cpu_usage: 0.0,
            memory_usage: 0,
            pid: None,
            error: None,
        }
    }
    
    /// Get app ID
    pub fn app_id(&self) -> &str {
        &self.app_id
    }
    
    /// Get state
    pub fn state(&self) -> AppState {
        self.state
    }
    
    /// Get last active time
    pub fn last_active(&self) -> Instant {
        self.last_active
    }
    
    /// Start the instance
    pub fn start(&mut self) {
        if self.state.can_transition_to(AppState::Running) {
            self.state = AppState::Running;
            self.last_active = Instant::now();
        }
    }
    
    /// Suspend the instance
    pub fn suspend(&mut self) {
        if self.state.can_transition_to(AppState::Suspended) {
            self.state = AppState::Suspended;
        }
    }
    
    /// Resume the instance
    pub fn resume(&mut self) {
        if self.state.can_transition_to(AppState::Running) {
            self.state = AppState::Running;
            self.last_active = Instant::now();
        }
    }
    
    /// Stop the instance
    pub fn stop(&mut self) {
        if self.state.can_transition_to(AppState::Stopping) {
            self.state = AppState::Stopping;
        }
        self.state = AppState::Stopped;
    }
    
    /// Crash the instance
    pub fn crash(&mut self, error: String) {
        self.state = AppState::Crashed;
        self.error = Some(error);
    }
    
    /// Update activity
    pub fn update_activity(&mut self) {
        if self.state == AppState::Running {
            self.last_active = Instant::now();
        }
    }
    
    /// Update resource usage
    pub fn update_usage(&mut self, cpu: f32, memory: u64) {
        self.cpu_usage = cpu;
        self.memory_usage = memory;
    }
    
    /// Get CPU usage
    pub fn cpu_usage(&self) -> f32 {
        self.cpu_usage
    }
    
    /// Get memory usage
    pub fn memory_usage(&self) -> u64 {
        self.memory_usage
    }
    
    /// Get runtime duration
    pub fn runtime(&self) -> std::time::Duration {
        self.started.elapsed()
    }
    
    /// Get error
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

/// App runtime manager
#[derive(Debug)]
pub struct AppRuntime {
    /// Next process ID
    next_pid: u32,
    /// Memory limit per app
    memory_limit: u64,
    /// CPU limit per app (0-100)
    cpu_limit: f32,
    /// Total memory available
    total_memory: u64,
    /// Used memory
    used_memory: u64,
}

impl AppRuntime {
    /// Create new runtime
    pub fn new() -> Self {
        Self {
            next_pid: 1000,
            memory_limit: 100 * 1024 * 1024, // 100MB
            cpu_limit: 50.0, // 50%
            total_memory: 1024 * 1024 * 1024, // 1GB
            used_memory: 0,
        }
    }
    
    /// Create app instance
    pub fn create_instance(&mut self, app_id: &str, app_type: &AppType) -> AppInstance {
        let mut instance = AppInstance::new(app_id.to_string(), *app_type);
        instance.pid = Some(self.next_pid);
        self.next_pid += 1;
        instance.start();
        instance
    }
    
    /// Get memory limit
    pub fn memory_limit(&self) -> u64 {
        self.memory_limit
    }
    
    /// Set memory limit
    pub fn set_memory_limit(&mut self, limit: u64) {
        self.memory_limit = limit;
    }
    
    /// Get CPU limit
    pub fn cpu_limit(&self) -> f32 {
        self.cpu_limit
    }
    
    /// Set CPU limit
    pub fn set_cpu_limit(&mut self, limit: f32) {
        self.cpu_limit = limit.clamp(1.0, 100.0);
    }
    
    /// Get available memory
    pub fn available_memory(&self) -> u64 {
        self.total_memory.saturating_sub(self.used_memory)
    }
    
    /// Can allocate memory
    pub fn can_allocate(&self, amount: u64) -> bool {
        amount <= self.available_memory()
    }
    
    /// Allocate memory
    pub fn allocate(&mut self, amount: u64) -> bool {
        if self.can_allocate(amount) {
            self.used_memory += amount;
            true
        } else {
            false
        }
    }
    
    /// Free memory
    pub fn free(&mut self, amount: u64) {
        self.used_memory = self.used_memory.saturating_sub(amount);
    }
}

impl Default for AppRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_state_transitions() {
        assert!(AppState::Starting.can_transition_to(AppState::Running));
        assert!(AppState::Running.can_transition_to(AppState::Suspended));
        assert!(!AppState::Stopped.can_transition_to(AppState::Running));
    }
    
    #[test]
    fn test_app_instance_creation() {
        let instance = AppInstance::new(
            "com.test.app".to_string(),
            AppType::Standard,
        );
        
        assert_eq!(instance.state(), AppState::Starting);
    }
    
    #[test]
    fn test_app_instance_lifecycle() {
        let mut instance = AppInstance::new(
            "com.test.app".to_string(),
            AppType::Standard,
        );
        
        instance.start();
        assert_eq!(instance.state(), AppState::Running);
        
        instance.suspend();
        assert_eq!(instance.state(), AppState::Suspended);
        
        instance.resume();
        assert_eq!(instance.state(), AppState::Running);
        
        instance.stop();
        assert_eq!(instance.state(), AppState::Stopped);
    }
    
    #[test]
    fn test_app_runtime_creation() {
        let runtime = AppRuntime::new();
        
        assert!(runtime.memory_limit() > 0);
        assert!(runtime.available_memory() > 0);
    }
    
    #[test]
    fn test_create_instance() {
        let mut runtime = AppRuntime::new();
        
        let instance = runtime.create_instance("com.test.app", &AppType::Standard);
        
        assert_eq!(instance.state(), AppState::Running);
        assert!(instance.pid.is_some());
    }
    
    #[test]
    fn test_memory_allocation() {
        let mut runtime = AppRuntime::new();
        
        let amount = 10 * 1024 * 1024; // 10MB
        assert!(runtime.allocate(amount));
        
        let available = runtime.available_memory();
        runtime.free(amount);
        
        assert!(runtime.available_memory() > available);
    }
}
