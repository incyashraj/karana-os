// Kāraṇa OS - System Watchdog
// Deadlock detection and system hang recovery

use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// System watchdog for deadlock detection
pub struct SystemWatchdog {
    /// Last pet time
    last_pet: Arc<Mutex<Instant>>,
    /// Timeout duration
    timeout_ms: u64,
    /// Is enabled
    enabled: AtomicBool,
    /// Pet count
    pet_count: AtomicU64,
    /// Timeout callbacks
    timeout_callbacks: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
    /// Component heartbeats
    heartbeats: Arc<Mutex<HashMap<String, ComponentHeartbeat>>>,
    /// Recovery actions
    recovery_actions: Arc<Mutex<Vec<RecoveryAction>>>,
}

/// Component heartbeat tracking
#[derive(Debug, Clone)]
pub struct ComponentHeartbeat {
    /// Component name
    pub name: String,
    /// Last heartbeat time
    pub last_beat: Instant,
    /// Expected interval (ms)
    pub expected_interval_ms: u64,
    /// Miss count
    pub miss_count: u32,
    /// Max allowed misses before alert
    pub max_misses: u32,
    /// Is healthy
    pub healthy: bool,
}

/// Recovery action
#[derive(Debug, Clone)]
pub struct RecoveryAction {
    /// Action name
    pub name: String,
    /// Priority (lower = earlier)
    pub priority: u8,
    /// Description
    pub description: String,
    /// Was executed
    pub executed: bool,
    /// Execution time
    pub executed_at: Option<Instant>,
}

impl SystemWatchdog {
    /// Create new watchdog
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            last_pet: Arc::new(Mutex::new(Instant::now())),
            timeout_ms,
            enabled: AtomicBool::new(true),
            pet_count: AtomicU64::new(0),
            timeout_callbacks: Arc::new(Mutex::new(Vec::new())),
            heartbeats: Arc::new(Mutex::new(HashMap::new())),
            recovery_actions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Pet the watchdog (reset timeout)
    pub fn pet(&self) {
        if !self.enabled.load(Ordering::SeqCst) {
            return;
        }

        if let Ok(mut last) = self.last_pet.lock() {
            *last = Instant::now();
        }
        self.pet_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Check if watchdog has timed out
    pub fn is_timed_out(&self) -> bool {
        if !self.enabled.load(Ordering::SeqCst) {
            return false;
        }

        if let Ok(last) = self.last_pet.lock() {
            last.elapsed().as_millis() as u64 > self.timeout_ms
        } else {
            false
        }
    }

    /// Get time since last pet
    pub fn time_since_pet(&self) -> Duration {
        if let Ok(last) = self.last_pet.lock() {
            last.elapsed()
        } else {
            Duration::ZERO
        }
    }

    /// Register timeout callback
    pub fn on_timeout<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.timeout_callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    /// Check and trigger callbacks if timed out
    pub fn check_and_trigger(&self) {
        if self.is_timed_out() {
            if let Ok(callbacks) = self.timeout_callbacks.lock() {
                for callback in callbacks.iter() {
                    callback();
                }
            }
        }
    }

    /// Enable/disable watchdog
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
        if enabled {
            self.pet(); // Reset timer when re-enabled
        }
    }

    /// Is watchdog enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Get pet count
    pub fn pet_count(&self) -> u64 {
        self.pet_count.load(Ordering::SeqCst)
    }

    // Component heartbeat management

    /// Register a component for heartbeat monitoring
    pub fn register_component(&self, name: &str, expected_interval_ms: u64, max_misses: u32) {
        if let Ok(mut heartbeats) = self.heartbeats.lock() {
            heartbeats.insert(name.to_string(), ComponentHeartbeat {
                name: name.to_string(),
                last_beat: Instant::now(),
                expected_interval_ms,
                miss_count: 0,
                max_misses,
                healthy: true,
            });
        }
    }

    /// Record heartbeat from component
    pub fn heartbeat(&self, name: &str) {
        if let Ok(mut heartbeats) = self.heartbeats.lock() {
            if let Some(hb) = heartbeats.get_mut(name) {
                hb.last_beat = Instant::now();
                hb.miss_count = 0;
                hb.healthy = true;
            }
        }
    }

    /// Check all component heartbeats
    pub fn check_heartbeats(&self) -> Vec<String> {
        let mut unhealthy = Vec::new();

        if let Ok(mut heartbeats) = self.heartbeats.lock() {
            for (name, hb) in heartbeats.iter_mut() {
                let elapsed = hb.last_beat.elapsed().as_millis() as u64;
                
                if elapsed > hb.expected_interval_ms {
                    hb.miss_count += 1;
                    
                    if hb.miss_count >= hb.max_misses {
                        hb.healthy = false;
                        unhealthy.push(name.clone());
                    }
                }
            }
        }

        unhealthy
    }

    /// Get component health status
    pub fn get_component_health(&self, name: &str) -> Option<bool> {
        if let Ok(heartbeats) = self.heartbeats.lock() {
            heartbeats.get(name).map(|hb| hb.healthy)
        } else {
            None
        }
    }

    /// Get all unhealthy components
    pub fn unhealthy_components(&self) -> Vec<String> {
        if let Ok(heartbeats) = self.heartbeats.lock() {
            heartbeats.iter()
                .filter(|(_, hb)| !hb.healthy)
                .map(|(name, _)| name.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    // Recovery actions

    /// Register recovery action
    pub fn register_recovery_action(&self, name: &str, priority: u8, description: &str) {
        if let Ok(mut actions) = self.recovery_actions.lock() {
            actions.push(RecoveryAction {
                name: name.to_string(),
                priority,
                description: description.to_string(),
                executed: false,
                executed_at: None,
            });
            actions.sort_by_key(|a| a.priority);
        }
    }

    /// Mark recovery action as executed
    pub fn mark_recovery_executed(&self, name: &str) {
        if let Ok(mut actions) = self.recovery_actions.lock() {
            if let Some(action) = actions.iter_mut().find(|a| a.name == name) {
                action.executed = true;
                action.executed_at = Some(Instant::now());
            }
        }
    }

    /// Get next recovery action to execute
    pub fn next_recovery_action(&self) -> Option<RecoveryAction> {
        if let Ok(actions) = self.recovery_actions.lock() {
            actions.iter()
                .find(|a| !a.executed)
                .cloned()
        } else {
            None
        }
    }

    /// Reset all recovery actions
    pub fn reset_recovery_actions(&self) {
        if let Ok(mut actions) = self.recovery_actions.lock() {
            for action in actions.iter_mut() {
                action.executed = false;
                action.executed_at = None;
            }
        }
    }
}

impl Default for SystemWatchdog {
    fn default() -> Self {
        Self::new(5000) // 5 second default timeout
    }
}

/// Multi-component watchdog coordinator
pub struct WatchdogCoordinator {
    /// Main system watchdog
    system_watchdog: SystemWatchdog,
    /// Component-specific watchdogs
    component_watchdogs: HashMap<String, SystemWatchdog>,
    /// Alert handlers
    alert_handlers: Vec<Box<dyn Fn(&str, &str) + Send + Sync>>,
}

impl WatchdogCoordinator {
    /// Create new coordinator
    pub fn new(system_timeout_ms: u64) -> Self {
        Self {
            system_watchdog: SystemWatchdog::new(system_timeout_ms),
            component_watchdogs: HashMap::new(),
            alert_handlers: Vec::new(),
        }
    }

    /// Add component watchdog
    pub fn add_component(&mut self, name: &str, timeout_ms: u64) {
        self.component_watchdogs.insert(name.to_string(), SystemWatchdog::new(timeout_ms));
    }

    /// Pet system watchdog
    pub fn pet_system(&self) {
        self.system_watchdog.pet();
    }

    /// Pet component watchdog
    pub fn pet_component(&self, name: &str) {
        if let Some(wd) = self.component_watchdogs.get(name) {
            wd.pet();
        }
    }

    /// Check all watchdogs
    pub fn check_all(&self) -> WatchdogStatus {
        let mut status = WatchdogStatus {
            system_ok: !self.system_watchdog.is_timed_out(),
            component_status: HashMap::new(),
        };

        for (name, wd) in &self.component_watchdogs {
            status.component_status.insert(name.clone(), !wd.is_timed_out());
        }

        status
    }

    /// Register alert handler
    pub fn on_alert<F>(&mut self, handler: F)
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.alert_handlers.push(Box::new(handler));
    }
}

/// Watchdog status
#[derive(Debug, Clone)]
pub struct WatchdogStatus {
    /// System watchdog OK
    pub system_ok: bool,
    /// Component watchdog status
    pub component_status: HashMap<String, bool>,
}

impl WatchdogStatus {
    /// Are all watchdogs OK
    pub fn all_ok(&self) -> bool {
        self.system_ok && self.component_status.values().all(|ok| *ok)
    }

    /// Get failed components
    pub fn failed_components(&self) -> Vec<String> {
        self.component_status.iter()
            .filter(|(_, ok)| !*ok)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_watchdog_creation() {
        let wd = SystemWatchdog::new(1000);
        assert!(wd.is_enabled());
        assert!(!wd.is_timed_out());
    }

    #[test]
    fn test_pet() {
        let wd = SystemWatchdog::new(1000);
        wd.pet();
        assert_eq!(wd.pet_count(), 1);
        wd.pet();
        assert_eq!(wd.pet_count(), 2);
    }

    #[test]
    fn test_timeout() {
        let wd = SystemWatchdog::new(50); // 50ms timeout
        wd.pet();
        
        assert!(!wd.is_timed_out());
        
        thread::sleep(Duration::from_millis(100));
        assert!(wd.is_timed_out());
        
        wd.pet();
        assert!(!wd.is_timed_out());
    }

    #[test]
    fn test_enable_disable() {
        let wd = SystemWatchdog::new(1000);
        
        assert!(wd.is_enabled());
        wd.set_enabled(false);
        assert!(!wd.is_enabled());
        
        // Should not timeout when disabled
        assert!(!wd.is_timed_out());
    }

    #[test]
    fn test_component_heartbeat() {
        let wd = SystemWatchdog::new(1000);
        
        wd.register_component("renderer", 100, 3);
        wd.heartbeat("renderer");
        
        assert_eq!(wd.get_component_health("renderer"), Some(true));
    }

    #[test]
    fn test_recovery_actions() {
        let wd = SystemWatchdog::new(1000);
        
        wd.register_recovery_action("restart_renderer", 1, "Restart the rendering system");
        wd.register_recovery_action("clear_cache", 0, "Clear all caches");
        
        // Should get lowest priority first
        let next = wd.next_recovery_action().unwrap();
        assert_eq!(next.name, "clear_cache");
        
        wd.mark_recovery_executed("clear_cache");
        
        let next = wd.next_recovery_action().unwrap();
        assert_eq!(next.name, "restart_renderer");
    }

    #[test]
    fn test_coordinator() {
        let mut coord = WatchdogCoordinator::new(1000);
        coord.add_component("audio", 500);
        coord.add_component("video", 500);
        
        coord.pet_system();
        coord.pet_component("audio");
        coord.pet_component("video");
        
        let status = coord.check_all();
        assert!(status.all_ok());
    }

    #[test]
    fn test_unhealthy_components() {
        let wd = SystemWatchdog::new(1000);
        
        wd.register_component("test", 10, 1);
        
        // Wait for timeout
        thread::sleep(Duration::from_millis(50));
        
        let unhealthy = wd.check_heartbeats();
        assert!(unhealthy.contains(&"test".to_string()));
    }

    #[test]
    fn test_watchdog_status() {
        let status = WatchdogStatus {
            system_ok: true,
            component_status: HashMap::from([
                ("a".to_string(), true),
                ("b".to_string(), false),
            ]),
        };

        assert!(!status.all_ok());
        assert_eq!(status.failed_components(), vec!["b".to_string()]);
    }
}
