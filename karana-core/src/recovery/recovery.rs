// Kāraṇa OS - Recovery Engine
// Automatic recovery strategies

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::error_log::{SystemError, ErrorSeverity};

/// Recovery engine
pub struct RecoveryEngine {
    /// Recovery strategies
    strategies: Vec<Box<dyn RecoveryStrategy + Send + Sync>>,
    /// Recovery history
    history: Vec<RecoveryAttempt>,
    /// Max attempts per error
    max_attempts: u32,
    /// Current attempt count per component
    attempt_counts: HashMap<String, u32>,
    /// Cooldown between attempts (ms)
    cooldown_ms: u64,
    /// Last attempt times
    last_attempts: HashMap<String, Instant>,
}

/// Recovery strategy trait
pub trait RecoveryStrategy: Send + Sync {
    /// Strategy name
    fn name(&self) -> &str;
    
    /// Can this strategy handle the error?
    fn can_handle(&self, error: &SystemError) -> bool;
    
    /// Priority (lower = try first)
    fn priority(&self) -> u8;
    
    /// Execute recovery
    fn execute(&self, error: &SystemError) -> RecoveryResult;
    
    /// Is this strategy destructive (loses data)?
    fn is_destructive(&self) -> bool { false }
}

/// Recovery attempt record
#[derive(Debug, Clone)]
pub struct RecoveryAttempt {
    /// Error ID
    pub error_id: u64,
    /// Strategy used
    pub strategy: String,
    /// Result
    pub result: RecoveryResult,
    /// Timestamp
    pub timestamp: Instant,
    /// Duration
    pub duration: Duration,
}

/// Recovery result
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// Was recovery successful
    pub success: bool,
    /// Strategy name
    pub strategy: String,
    /// Message
    pub message: String,
    /// Actions taken
    pub actions: Vec<RecoveryAction>,
    /// Requires restart
    pub requires_restart: bool,
    /// Data loss occurred
    pub data_loss: bool,
}

/// Recovery action taken
#[derive(Debug, Clone)]
pub struct RecoveryAction {
    /// Action name
    pub name: String,
    /// Description
    pub description: String,
    /// Was successful
    pub success: bool,
}

impl RecoveryEngine {
    /// Create new recovery engine
    pub fn new(max_attempts: u32) -> Self {
        let mut engine = Self {
            strategies: Vec::new(),
            history: Vec::new(),
            max_attempts,
            attempt_counts: HashMap::new(),
            cooldown_ms: 1000,
            last_attempts: HashMap::new(),
        };

        // Register default strategies
        engine.register_default_strategies();
        engine
    }

    fn register_default_strategies(&mut self) {
        self.strategies.push(Box::new(RestartComponentStrategy));
        self.strategies.push(Box::new(ClearCacheStrategy));
        self.strategies.push(Box::new(ResetStateStrategy));
        self.strategies.push(Box::new(GracefulDegradationStrategy));
        self.strategies.push(Box::new(FullRestartStrategy));

        // Sort by priority
        self.strategies.sort_by_key(|s| s.priority());
    }

    /// Attempt recovery
    pub fn attempt_recovery(&mut self, error: &SystemError) -> RecoveryResult {
        let component = &error.component;

        // Check cooldown
        if let Some(last) = self.last_attempts.get(component) {
            if (last.elapsed().as_millis() as u64) < self.cooldown_ms {
                return RecoveryResult {
                    success: false,
                    strategy: "cooldown".to_string(),
                    message: "In cooldown period".to_string(),
                    actions: vec![],
                    requires_restart: false,
                    data_loss: false,
                };
            }
        }

        // Check attempt limit
        let attempts = self.attempt_counts.entry(component.clone()).or_insert(0);
        if *attempts >= self.max_attempts {
            return RecoveryResult {
                success: false,
                strategy: "limit".to_string(),
                message: "Max recovery attempts exceeded".to_string(),
                actions: vec![],
                requires_restart: true,
                data_loss: false,
            };
        }

        *attempts += 1;
        self.last_attempts.insert(component.clone(), Instant::now());

        // Try strategies in priority order
        let start = Instant::now();
        for strategy in &self.strategies {
            if strategy.can_handle(error) {
                let result = strategy.execute(error);
                
                self.history.push(RecoveryAttempt {
                    error_id: error.id,
                    strategy: strategy.name().to_string(),
                    result: result.clone(),
                    timestamp: Instant::now(),
                    duration: start.elapsed(),
                });

                if result.success {
                    // Reset attempt count on success
                    self.attempt_counts.insert(component.clone(), 0);
                    return result;
                }
            }
        }

        RecoveryResult {
            success: false,
            strategy: "none".to_string(),
            message: "No recovery strategy succeeded".to_string(),
            actions: vec![],
            requires_restart: true,
            data_loss: false,
        }
    }

    /// Register custom strategy
    pub fn register_strategy<S: RecoveryStrategy + 'static>(&mut self, strategy: S) {
        self.strategies.push(Box::new(strategy));
        self.strategies.sort_by_key(|s| s.priority());
    }

    /// Get recovery history
    pub fn history(&self) -> &[RecoveryAttempt] {
        &self.history
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Reset attempt counts
    pub fn reset_counts(&mut self) {
        self.attempt_counts.clear();
    }

    /// Get attempt count for component
    pub fn get_attempts(&self, component: &str) -> u32 {
        *self.attempt_counts.get(component).unwrap_or(&0)
    }
}

// Default recovery strategies

struct RestartComponentStrategy;
impl RecoveryStrategy for RestartComponentStrategy {
    fn name(&self) -> &str { "restart_component" }
    
    fn can_handle(&self, error: &SystemError) -> bool {
        error.severity <= ErrorSeverity::Error
    }
    
    fn priority(&self) -> u8 { 1 }
    
    fn execute(&self, error: &SystemError) -> RecoveryResult {
        // Simulate component restart
        RecoveryResult {
            success: true,
            strategy: self.name().to_string(),
            message: format!("Restarted component: {}", error.component),
            actions: vec![
                RecoveryAction {
                    name: "stop_component".to_string(),
                    description: format!("Stopped {}", error.component),
                    success: true,
                },
                RecoveryAction {
                    name: "start_component".to_string(),
                    description: format!("Started {}", error.component),
                    success: true,
                },
            ],
            requires_restart: false,
            data_loss: false,
        }
    }
}

struct ClearCacheStrategy;
impl RecoveryStrategy for ClearCacheStrategy {
    fn name(&self) -> &str { "clear_cache" }
    
    fn can_handle(&self, _error: &SystemError) -> bool {
        true // Can always try clearing cache
    }
    
    fn priority(&self) -> u8 { 2 }
    
    fn execute(&self, error: &SystemError) -> RecoveryResult {
        RecoveryResult {
            success: true,
            strategy: self.name().to_string(),
            message: format!("Cleared cache for: {}", error.component),
            actions: vec![
                RecoveryAction {
                    name: "clear_cache".to_string(),
                    description: "Cleared component cache".to_string(),
                    success: true,
                },
            ],
            requires_restart: false,
            data_loss: false,
        }
    }
}

struct ResetStateStrategy;
impl RecoveryStrategy for ResetStateStrategy {
    fn name(&self) -> &str { "reset_state" }
    
    fn can_handle(&self, error: &SystemError) -> bool {
        error.severity >= ErrorSeverity::Error
    }
    
    fn priority(&self) -> u8 { 3 }
    
    fn execute(&self, error: &SystemError) -> RecoveryResult {
        RecoveryResult {
            success: true,
            strategy: self.name().to_string(),
            message: format!("Reset state for: {}", error.component),
            actions: vec![
                RecoveryAction {
                    name: "backup_state".to_string(),
                    description: "Backed up current state".to_string(),
                    success: true,
                },
                RecoveryAction {
                    name: "reset_state".to_string(),
                    description: "Reset to default state".to_string(),
                    success: true,
                },
            ],
            requires_restart: false,
            data_loss: true,
        }
    }
    
    fn is_destructive(&self) -> bool { true }
}

struct GracefulDegradationStrategy;
impl RecoveryStrategy for GracefulDegradationStrategy {
    fn name(&self) -> &str { "graceful_degradation" }
    
    fn can_handle(&self, error: &SystemError) -> bool {
        error.severity >= ErrorSeverity::Critical
    }
    
    fn priority(&self) -> u8 { 4 }
    
    fn execute(&self, error: &SystemError) -> RecoveryResult {
        RecoveryResult {
            success: true,
            strategy: self.name().to_string(),
            message: format!("Degraded mode for: {}", error.component),
            actions: vec![
                RecoveryAction {
                    name: "disable_features".to_string(),
                    description: "Disabled non-essential features".to_string(),
                    success: true,
                },
                RecoveryAction {
                    name: "enable_fallback".to_string(),
                    description: "Enabled fallback mode".to_string(),
                    success: true,
                },
            ],
            requires_restart: false,
            data_loss: false,
        }
    }
}

struct FullRestartStrategy;
impl RecoveryStrategy for FullRestartStrategy {
    fn name(&self) -> &str { "full_restart" }
    
    fn can_handle(&self, error: &SystemError) -> bool {
        error.severity == ErrorSeverity::Fatal
    }
    
    fn priority(&self) -> u8 { 10 } // Last resort
    
    fn execute(&self, _error: &SystemError) -> RecoveryResult {
        RecoveryResult {
            success: true,
            strategy: self.name().to_string(),
            message: "Scheduled full system restart".to_string(),
            actions: vec![
                RecoveryAction {
                    name: "save_state".to_string(),
                    description: "Saved system state".to_string(),
                    success: true,
                },
                RecoveryAction {
                    name: "schedule_restart".to_string(),
                    description: "Scheduled system restart".to_string(),
                    success: true,
                },
            ],
            requires_restart: true,
            data_loss: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_error(severity: ErrorSeverity) -> SystemError {
        SystemError {
            id: 1,
            timestamp: Instant::now(),
            severity,
            component: "test".to_string(),
            message: "Test error".to_string(),
            backtrace: None,
            context: HashMap::new(),
        }
    }

    #[test]
    fn test_recovery_engine_creation() {
        let engine = RecoveryEngine::new(3);
        assert!(engine.history().is_empty());
    }

    #[test]
    fn test_attempt_recovery() {
        let mut engine = RecoveryEngine::new(3);
        let error = mock_error(ErrorSeverity::Warning);
        
        let result = engine.attempt_recovery(&error);
        assert!(result.success);
    }

    #[test]
    fn test_recovery_history() {
        let mut engine = RecoveryEngine::new(3);
        let error = mock_error(ErrorSeverity::Warning);
        
        engine.attempt_recovery(&error);
        
        assert!(!engine.history().is_empty());
    }

    #[test]
    fn test_max_attempts() {
        // Create engine with no default strategies to test the limit logic
        let mut engine = RecoveryEngine {
            strategies: Vec::new(), // No strategies
            history: Vec::new(),
            max_attempts: 2,
            attempt_counts: HashMap::new(),
            cooldown_ms: 0, // Disable cooldown for test
            last_attempts: HashMap::new(),
        };
        
        let error = mock_error(ErrorSeverity::Fatal);
        
        // First two attempts - no strategy succeeds
        let r1 = engine.attempt_recovery(&error);
        assert!(!r1.success);
        assert_eq!(r1.strategy, "none");
        
        let r2 = engine.attempt_recovery(&error);
        assert!(!r2.success);
        
        // Third should fail due to limit (2 attempts exceeded)
        let result = engine.attempt_recovery(&error);
        assert!(!result.success);
        assert_eq!(result.strategy, "limit");
    }

    #[test]
    fn test_get_attempts() {
        let mut engine = RecoveryEngine::new(10);
        let error = mock_error(ErrorSeverity::Warning);
        
        assert_eq!(engine.get_attempts("test"), 0);
        
        engine.attempt_recovery(&error);
        
        assert_eq!(engine.get_attempts("test"), 0); // Reset on success
    }

    #[test]
    fn test_reset_counts() {
        let mut engine = RecoveryEngine::new(10);
        let error = mock_error(ErrorSeverity::Fatal);
        
        engine.attempt_recovery(&error);
        engine.reset_counts();
        
        assert_eq!(engine.get_attempts("test"), 0);
    }

    #[test]
    fn test_strategy_priority() {
        let restart = RestartComponentStrategy;
        let cache = ClearCacheStrategy;
        
        assert!(restart.priority() < cache.priority());
    }

    #[test]
    fn test_destructive_strategy() {
        let reset = ResetStateStrategy;
        assert!(reset.is_destructive());
        
        let restart = RestartComponentStrategy;
        assert!(!restart.is_destructive());
    }
}
