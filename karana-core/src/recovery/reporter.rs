// Kāraṇa OS - Crash Reporter
// Report crashes to telemetry service

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use super::error_log::SystemError;

/// Crash reporter
pub struct CrashReporter {
    /// Report queue
    queue: VecDeque<CrashReport>,
    /// Max queue size
    max_queue: usize,
    /// Reporting endpoint
    endpoint: Option<String>,
    /// Sent reports
    sent_count: u64,
    /// Failed reports
    failed_count: u64,
    /// Is enabled
    enabled: bool,
    /// Anonymize data
    anonymize: bool,
}

/// Crash report
#[derive(Debug, Clone)]
pub struct CrashReport {
    /// Error details
    pub error: SystemError,
    /// Associated dump ID
    pub dump_id: Option<String>,
    /// Device info
    pub device_info: DeviceInfo,
    /// Report timestamp
    pub timestamp: Instant,
}

/// Device information for reports
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device ID (anonymized)
    pub device_id: String,
    /// OS version
    pub os_version: String,
    /// Device model
    pub device_model: String,
    /// Firmware version
    pub firmware_version: String,
    /// Total memory
    pub total_memory_mb: u32,
    /// Available memory at crash
    pub available_memory_mb: u32,
    /// Battery level
    pub battery_level: u8,
    /// Was charging
    pub charging: bool,
    /// Device locale
    pub locale: String,
    /// Timezone
    pub timezone: String,
}

impl DeviceInfo {
    /// Collect current device info
    pub fn collect() -> Self {
        Self {
            device_id: Self::generate_anonymous_id(),
            os_version: "Karana OS 0.1.0".to_string(),
            device_model: "Smart Glasses Gen1".to_string(),
            firmware_version: "1.0.0".to_string(),
            total_memory_mb: 2048,
            available_memory_mb: 512,
            battery_level: 85,
            charging: false,
            locale: "en_US".to_string(),
            timezone: "UTC".to_string(),
        }
    }

    fn generate_anonymous_id() -> String {
        use std::time::SystemTime;
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("anon_{:x}", secs % 1_000_000)
    }

    /// Anonymize sensitive data
    pub fn anonymize(&mut self) {
        self.device_id = "REDACTED".to_string();
    }
}

impl CrashReporter {
    /// Create new reporter
    pub fn new(endpoint: Option<String>) -> Self {
        Self {
            queue: VecDeque::new(),
            max_queue: 100,
            endpoint,
            sent_count: 0,
            failed_count: 0,
            enabled: true,
            anonymize: true,
        }
    }

    /// Queue a report
    pub fn queue_report(&mut self, mut report: CrashReport) {
        if !self.enabled {
            return;
        }

        // Anonymize if configured
        if self.anonymize {
            report.device_info.anonymize();
        }

        self.queue.push_back(report);

        // Trim queue if over limit
        while self.queue.len() > self.max_queue {
            self.queue.pop_front();
        }
    }

    /// Send queued reports
    pub fn send_reports(&mut self) -> ReportResult {
        if !self.enabled {
            return ReportResult {
                sent: 0,
                failed: 0,
                remaining: 0,
            };
        }

        let endpoint = match &self.endpoint {
            Some(e) => e.clone(),
            None => return ReportResult {
                sent: 0,
                failed: self.queue.len(),
                remaining: self.queue.len(),
            },
        };

        let mut sent = 0;
        let mut failed = 0;

        while let Some(report) = self.queue.pop_front() {
            // Simulate sending (in real implementation, would make HTTP request)
            if self.simulate_send(&endpoint, &report) {
                sent += 1;
                self.sent_count += 1;
            } else {
                failed += 1;
                self.failed_count += 1;
                // Re-queue failed report
                self.queue.push_back(report);
                break; // Stop on first failure
            }
        }

        ReportResult {
            sent,
            failed,
            remaining: self.queue.len(),
        }
    }

    fn simulate_send(&self, _endpoint: &str, _report: &CrashReport) -> bool {
        // In real implementation, would make HTTP POST request
        // For simulation, always succeed
        true
    }

    /// Enable/disable reporting
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Is reporting enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set endpoint
    pub fn set_endpoint(&mut self, endpoint: Option<String>) {
        self.endpoint = endpoint;
    }

    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.queue.len()
    }

    /// Get sent count
    pub fn sent_count(&self) -> u64 {
        self.sent_count
    }

    /// Get failed count
    pub fn failed_count(&self) -> u64 {
        self.failed_count
    }

    /// Set anonymization
    pub fn set_anonymize(&mut self, anonymize: bool) {
        self.anonymize = anonymize;
    }

    /// Clear queue
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    /// Get statistics
    pub fn statistics(&self) -> ReporterStats {
        ReporterStats {
            queued: self.queue.len(),
            sent: self.sent_count,
            failed: self.failed_count,
            enabled: self.enabled,
            has_endpoint: self.endpoint.is_some(),
        }
    }
}

/// Result of sending reports
#[derive(Debug, Clone)]
pub struct ReportResult {
    /// Number sent successfully
    pub sent: usize,
    /// Number failed
    pub failed: usize,
    /// Remaining in queue
    pub remaining: usize,
}

/// Reporter statistics
#[derive(Debug, Clone)]
pub struct ReporterStats {
    /// Reports in queue
    pub queued: usize,
    /// Total sent
    pub sent: u64,
    /// Total failed
    pub failed: u64,
    /// Is enabled
    pub enabled: bool,
    /// Has endpoint configured
    pub has_endpoint: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::error_log::ErrorSeverity;

    fn mock_report() -> CrashReport {
        CrashReport {
            error: SystemError {
                id: 1,
                timestamp: Instant::now(),
                severity: ErrorSeverity::Fatal,
                component: "test".to_string(),
                message: "Test crash".to_string(),
                backtrace: None,
                context: HashMap::new(),
            },
            dump_id: Some("dump_001".to_string()),
            device_info: DeviceInfo::collect(),
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn test_reporter_creation() {
        let reporter = CrashReporter::new(None);
        assert!(reporter.is_enabled());
        assert_eq!(reporter.queue_length(), 0);
    }

    #[test]
    fn test_queue_report() {
        let mut reporter = CrashReporter::new(None);
        let report = mock_report();
        
        reporter.queue_report(report);
        
        assert_eq!(reporter.queue_length(), 1);
    }

    #[test]
    fn test_queue_limit() {
        let mut reporter = CrashReporter::new(None);
        reporter.max_queue = 5;
        
        for _ in 0..10 {
            reporter.queue_report(mock_report());
        }
        
        assert!(reporter.queue_length() <= 5);
    }

    #[test]
    fn test_send_reports() {
        let mut reporter = CrashReporter::new(Some("https://api.example.com/crash".to_string()));
        
        reporter.queue_report(mock_report());
        reporter.queue_report(mock_report());
        
        let result = reporter.send_reports();
        
        assert_eq!(result.sent, 2);
        assert_eq!(result.remaining, 0);
    }

    #[test]
    fn test_disabled_reporter() {
        let mut reporter = CrashReporter::new(None);
        reporter.set_enabled(false);
        
        reporter.queue_report(mock_report());
        
        assert_eq!(reporter.queue_length(), 0);
    }

    #[test]
    fn test_device_info_collect() {
        let info = DeviceInfo::collect();
        assert!(!info.device_id.is_empty());
        assert!(!info.os_version.is_empty());
    }

    #[test]
    fn test_device_info_anonymize() {
        let mut info = DeviceInfo::collect();
        info.anonymize();
        assert_eq!(info.device_id, "REDACTED");
    }

    #[test]
    fn test_statistics() {
        let mut reporter = CrashReporter::new(Some("https://example.com".to_string()));
        reporter.queue_report(mock_report());
        
        let stats = reporter.statistics();
        assert_eq!(stats.queued, 1);
        assert!(stats.has_endpoint);
    }

    #[test]
    fn test_clear_queue() {
        let mut reporter = CrashReporter::new(None);
        reporter.queue_report(mock_report());
        reporter.queue_report(mock_report());
        
        assert_eq!(reporter.queue_length(), 2);
        
        reporter.clear_queue();
        assert_eq!(reporter.queue_length(), 0);
    }
}
