// Kāraṇa OS - Error Logger
// Structured error logging and history

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// System error entry
#[derive(Debug, Clone)]
pub struct SystemError {
    /// Unique error ID
    pub id: u64,
    /// Timestamp
    pub timestamp: Instant,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Component that raised the error
    pub component: String,
    /// Error message
    pub message: String,
    /// Optional backtrace
    pub backtrace: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    /// Debug level - verbose info
    Debug,
    /// Info - normal operation notes
    Info,
    /// Warning - potential issues
    Warning,
    /// Error - recoverable failures
    Error,
    /// Critical - system degradation
    Critical,
    /// Fatal - system failure
    Fatal,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Debug => "DEBUG",
            ErrorSeverity::Info => "INFO",
            ErrorSeverity::Warning => "WARN",
            ErrorSeverity::Error => "ERROR",
            ErrorSeverity::Critical => "CRITICAL",
            ErrorSeverity::Fatal => "FATAL",
        }
    }
}

/// Error logger
pub struct ErrorLogger {
    /// Error log queue
    log: VecDeque<SystemError>,
    /// Max entries
    max_entries: usize,
    /// ID counter
    id_counter: AtomicU64,
    /// Error counts by severity
    counts: HashMap<ErrorSeverity, u64>,
    /// Error counts by component
    component_counts: HashMap<String, u64>,
    /// Log listeners
    listeners: Vec<Box<dyn Fn(&SystemError) + Send + Sync>>,
}

impl ErrorLogger {
    /// Create new logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            log: VecDeque::with_capacity(max_entries),
            max_entries,
            id_counter: AtomicU64::new(1),
            counts: HashMap::new(),
            component_counts: HashMap::new(),
            listeners: Vec::new(),
        }
    }

    /// Get next error ID
    pub fn next_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Log an error
    pub fn log(&mut self, mut error: SystemError) {
        // Assign ID if not set
        if error.id == 0 {
            error.id = self.next_id();
        }

        // Update counts
        *self.counts.entry(error.severity).or_insert(0) += 1;
        *self.component_counts.entry(error.component.clone()).or_insert(0) += 1;

        // Notify listeners
        for listener in &self.listeners {
            listener(&error);
        }

        // Add to log
        self.log.push_back(error);

        // Trim if over limit
        while self.log.len() > self.max_entries {
            self.log.pop_front();
        }
    }

    /// Log a simple error
    pub fn log_simple(&mut self, severity: ErrorSeverity, component: &str, message: &str) {
        self.log(SystemError {
            id: 0,
            timestamp: Instant::now(),
            severity,
            component: component.to_string(),
            message: message.to_string(),
            backtrace: None,
            context: HashMap::new(),
        });
    }

    /// Get recent errors
    pub fn recent(&self, count: usize) -> Vec<&SystemError> {
        self.log.iter().rev().take(count).collect()
    }

    /// Get errors by severity
    pub fn by_severity(&self, severity: ErrorSeverity) -> Vec<&SystemError> {
        self.log.iter()
            .filter(|e| e.severity == severity)
            .collect()
    }

    /// Get errors by component
    pub fn by_component(&self, component: &str) -> Vec<&SystemError> {
        self.log.iter()
            .filter(|e| e.component == component)
            .collect()
    }

    /// Get errors since timestamp
    pub fn since(&self, since: Instant) -> Vec<&SystemError> {
        self.log.iter()
            .filter(|e| e.timestamp >= since)
            .collect()
    }

    /// Get error count by severity
    pub fn count_by_severity(&self, severity: ErrorSeverity) -> u64 {
        *self.counts.get(&severity).unwrap_or(&0)
    }

    /// Get error count by component
    pub fn count_by_component(&self, component: &str) -> u64 {
        *self.component_counts.get(component).unwrap_or(&0)
    }

    /// Total error count
    pub fn total_count(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.log.clear();
        self.counts.clear();
        self.component_counts.clear();
    }

    /// Register listener for new errors
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: Fn(&SystemError) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// Search errors by message
    pub fn search(&self, query: &str) -> Vec<&SystemError> {
        let query_lower = query.to_lowercase();
        self.log.iter()
            .filter(|e| e.message.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get error statistics
    pub fn statistics(&self) -> LogStatistics {
        LogStatistics {
            total_entries: self.log.len(),
            by_severity: self.counts.clone(),
            by_component: self.component_counts.clone(),
            oldest_entry: self.log.front().map(|e| e.timestamp),
            newest_entry: self.log.back().map(|e| e.timestamp),
        }
    }
}

/// Log statistics
#[derive(Debug, Clone)]
pub struct LogStatistics {
    /// Total entries in log
    pub total_entries: usize,
    /// Counts by severity
    pub by_severity: HashMap<ErrorSeverity, u64>,
    /// Counts by component
    pub by_component: HashMap<String, u64>,
    /// Oldest entry timestamp
    pub oldest_entry: Option<Instant>,
    /// Newest entry timestamp
    pub newest_entry: Option<Instant>,
}

/// Error builder for fluent API
pub struct ErrorBuilder {
    severity: ErrorSeverity,
    component: String,
    message: String,
    backtrace: Option<String>,
    context: HashMap<String, String>,
}

impl ErrorBuilder {
    pub fn new(severity: ErrorSeverity, component: &str, message: &str) -> Self {
        Self {
            severity,
            component: component.to_string(),
            message: message.to_string(),
            backtrace: None,
            context: HashMap::new(),
        }
    }

    pub fn with_backtrace(mut self, bt: &str) -> Self {
        self.backtrace = Some(bt.to_string());
        self
    }

    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(self) -> SystemError {
        SystemError {
            id: 0,
            timestamp: Instant::now(),
            severity: self.severity,
            component: self.component,
            message: self.message,
            backtrace: self.backtrace,
            context: self.context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = ErrorLogger::new(100);
        assert_eq!(logger.total_count(), 0);
    }

    #[test]
    fn test_log_error() {
        let mut logger = ErrorLogger::new(100);
        
        logger.log_simple(ErrorSeverity::Warning, "test", "Test message");
        
        assert_eq!(logger.total_count(), 1);
        assert_eq!(logger.count_by_severity(ErrorSeverity::Warning), 1);
    }

    #[test]
    fn test_recent_errors() {
        let mut logger = ErrorLogger::new(100);
        
        for i in 0..5 {
            logger.log_simple(ErrorSeverity::Info, "test", &format!("Message {}", i));
        }
        
        let recent = logger.recent(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_by_severity() {
        let mut logger = ErrorLogger::new(100);
        
        logger.log_simple(ErrorSeverity::Warning, "test", "Warning 1");
        logger.log_simple(ErrorSeverity::Error, "test", "Error 1");
        logger.log_simple(ErrorSeverity::Warning, "test", "Warning 2");
        
        let warnings = logger.by_severity(ErrorSeverity::Warning);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_by_component() {
        let mut logger = ErrorLogger::new(100);
        
        logger.log_simple(ErrorSeverity::Info, "audio", "Audio log");
        logger.log_simple(ErrorSeverity::Info, "video", "Video log");
        logger.log_simple(ErrorSeverity::Info, "audio", "Audio log 2");
        
        let audio_logs = logger.by_component("audio");
        assert_eq!(audio_logs.len(), 2);
    }

    #[test]
    fn test_max_entries() {
        let mut logger = ErrorLogger::new(5);
        
        for i in 0..10 {
            logger.log_simple(ErrorSeverity::Info, "test", &format!("Message {}", i));
        }
        
        assert!(logger.log.len() <= 5);
    }

    #[test]
    fn test_error_builder() {
        let error = ErrorBuilder::new(ErrorSeverity::Error, "test", "Test error")
            .with_context("key", "value")
            .with_backtrace("fake backtrace")
            .build();

        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(error.backtrace.is_some());
        assert!(error.context.contains_key("key"));
    }

    #[test]
    fn test_search() {
        let mut logger = ErrorLogger::new(100);
        
        logger.log_simple(ErrorSeverity::Info, "test", "Hello world");
        logger.log_simple(ErrorSeverity::Info, "test", "Goodbye world");
        logger.log_simple(ErrorSeverity::Info, "test", "Hello again");
        
        let results = logger.search("hello");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut logger = ErrorLogger::new(100);
        
        logger.log_simple(ErrorSeverity::Warning, "audio", "Warn 1");
        logger.log_simple(ErrorSeverity::Error, "video", "Error 1");
        
        let stats = logger.statistics();
        assert_eq!(stats.total_entries, 2);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ErrorSeverity::Debug < ErrorSeverity::Info);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Critical < ErrorSeverity::Fatal);
    }

    #[test]
    fn test_clear() {
        let mut logger = ErrorLogger::new(100);
        
        logger.log_simple(ErrorSeverity::Info, "test", "Test");
        assert!(!logger.log.is_empty());
        
        logger.clear();
        assert!(logger.log.is_empty());
        assert_eq!(logger.total_count(), 0);
    }
}
