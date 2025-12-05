// Kāraṇa OS - Crash Recovery System
// Error logging, crash dumps, and automatic recovery

pub mod error_log;
pub mod crash_dump;
pub mod recovery;
pub mod reporter;

pub use error_log::*;
pub use crash_dump::*;
pub use recovery::*;
pub use reporter::*;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Crash recovery system coordinator
pub struct CrashRecovery {
    /// Error logger
    error_log: ErrorLogger,
    /// Crash dump manager
    dump_manager: CrashDumpManager,
    /// Recovery engine
    recovery_engine: RecoveryEngine,
    /// Crash reporter
    reporter: CrashReporter,
    /// System state
    state: RecoveryState,
    /// Configuration
    config: CrashRecoveryConfig,
}

/// Recovery state
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryState {
    /// Normal operation
    Normal,
    /// Error detected, logging
    ErrorDetected,
    /// Creating crash dump
    DumpingState,
    /// Attempting recovery
    Recovering,
    /// Recovery succeeded
    Recovered,
    /// Recovery failed, needs manual intervention
    Failed,
}

/// Configuration
#[derive(Debug, Clone)]
pub struct CrashRecoveryConfig {
    /// Max error log entries
    pub max_log_entries: usize,
    /// Enable crash dumps
    pub dumps_enabled: bool,
    /// Max crash dumps to keep
    pub max_dumps: usize,
    /// Dump directory
    pub dump_dir: String,
    /// Auto-recovery enabled
    pub auto_recovery: bool,
    /// Max recovery attempts
    pub max_recovery_attempts: u32,
    /// Enable crash reporting
    pub reporting_enabled: bool,
    /// Report endpoint
    pub report_endpoint: Option<String>,
}

impl Default for CrashRecoveryConfig {
    fn default() -> Self {
        Self {
            max_log_entries: 10000,
            dumps_enabled: true,
            max_dumps: 10,
            dump_dir: "/data/crashes".to_string(),
            auto_recovery: true,
            max_recovery_attempts: 3,
            reporting_enabled: false,
            report_endpoint: None,
        }
    }
}

impl CrashRecovery {
    /// Create new crash recovery system
    pub fn new() -> Self {
        Self::with_config(CrashRecoveryConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: CrashRecoveryConfig) -> Self {
        Self {
            error_log: ErrorLogger::new(config.max_log_entries),
            dump_manager: CrashDumpManager::new(&config.dump_dir, config.max_dumps),
            recovery_engine: RecoveryEngine::new(config.max_recovery_attempts),
            reporter: CrashReporter::new(config.report_endpoint.clone()),
            state: RecoveryState::Normal,
            config,
        }
    }

    /// Log an error
    pub fn log_error(&mut self, error: SystemError) {
        self.error_log.log(error.clone());

        // Check if critical error requiring recovery
        if error.severity == ErrorSeverity::Critical || error.severity == ErrorSeverity::Fatal {
            self.state = RecoveryState::ErrorDetected;
            self.initiate_recovery(&error);
        }
    }

    /// Handle a panic/crash
    pub fn handle_crash(&mut self, panic_info: &str, backtrace: Option<&str>) {
        self.state = RecoveryState::ErrorDetected;

        // Log the crash
        let error = SystemError {
            id: self.error_log.next_id(),
            timestamp: Instant::now(),
            severity: ErrorSeverity::Fatal,
            component: "system".to_string(),
            message: panic_info.to_string(),
            backtrace: backtrace.map(|s| s.to_string()),
            context: std::collections::HashMap::new(),
        };
        self.error_log.log(error.clone());

        // Create crash dump if enabled
        if self.config.dumps_enabled {
            self.state = RecoveryState::DumpingState;
            if let Some(dump) = self.dump_manager.create_dump(&error) {
                // Report crash if enabled
                if self.config.reporting_enabled {
                    self.reporter.queue_report(CrashReport {
                        error: error.clone(),
                        dump_id: Some(dump.info.id.clone()),
                        device_info: DeviceInfo::collect(),
                        timestamp: Instant::now(),
                    });
                }
            }
        }

        // Attempt recovery
        if self.config.auto_recovery {
            self.initiate_recovery(&error);
        } else {
            self.state = RecoveryState::Failed;
        }
    }

    fn initiate_recovery(&mut self, error: &SystemError) {
        self.state = RecoveryState::Recovering;

        let result = self.recovery_engine.attempt_recovery(error);

        self.state = if result.success {
            RecoveryState::Recovered
        } else {
            RecoveryState::Failed
        };
    }

    /// Get current state
    pub fn state(&self) -> &RecoveryState {
        &self.state
    }

    /// Get recent errors
    pub fn recent_errors(&self, count: usize) -> Vec<&SystemError> {
        self.error_log.recent(count)
    }

    /// Get errors by severity
    pub fn errors_by_severity(&self, severity: ErrorSeverity) -> Vec<&SystemError> {
        self.error_log.by_severity(severity)
    }

    /// Clear error log
    pub fn clear_log(&mut self) {
        self.error_log.clear();
    }

    /// Reset state to normal
    pub fn reset_state(&mut self) {
        self.state = RecoveryState::Normal;
    }

    /// Get crash dumps
    pub fn list_dumps(&self) -> Vec<CrashDumpInfo> {
        self.dump_manager.list_dumps()
    }

    /// Delete old dumps
    pub fn cleanup_dumps(&mut self) {
        self.dump_manager.cleanup();
    }
}

impl Default for CrashRecovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_recovery_creation() {
        let cr = CrashRecovery::new();
        assert_eq!(*cr.state(), RecoveryState::Normal);
    }

    #[test]
    fn test_log_error() {
        let mut cr = CrashRecovery::new();
        
        cr.log_error(SystemError {
            id: 1,
            timestamp: Instant::now(),
            severity: ErrorSeverity::Warning,
            component: "test".to_string(),
            message: "Test warning".to_string(),
            backtrace: None,
            context: std::collections::HashMap::new(),
        });

        assert_eq!(cr.recent_errors(10).len(), 1);
    }

    #[test]
    fn test_critical_error_triggers_recovery() {
        let mut cr = CrashRecovery::new();
        
        cr.log_error(SystemError {
            id: 1,
            timestamp: Instant::now(),
            severity: ErrorSeverity::Critical,
            component: "test".to_string(),
            message: "Critical error".to_string(),
            backtrace: None,
            context: std::collections::HashMap::new(),
        });

        // Should have attempted recovery
        assert!(matches!(cr.state(), RecoveryState::Recovered | RecoveryState::Failed));
    }

    #[test]
    fn test_default_config() {
        let config = CrashRecoveryConfig::default();
        assert!(config.auto_recovery);
        assert!(config.dumps_enabled);
    }
}
