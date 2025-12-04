//! Audit System for Kāraṇa OS AR Glasses
//!
//! Comprehensive audit logging for privacy and security events
//! to support compliance and user transparency.

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

/// Audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditEventType {
    // Privacy events
    /// Privacy mode changed
    PrivacyModeChanged,
    /// Consent changed
    ConsentChanged,
    /// Privacy zone entered/exited
    PrivacyZoneEvent,
    /// Privacy zone configured
    ZoneConfigured,
    
    // Permission events
    /// Permission requested
    PermissionRequested,
    /// Permission granted
    PermissionGranted,
    /// Permission denied
    PermissionDenied,
    /// Permission revoked
    PermissionRevoked,
    
    // Sensor access events
    /// Camera accessed
    CameraAccess,
    /// Microphone accessed
    MicrophoneAccess,
    /// Location accessed
    LocationAccess,
    /// Eye tracking accessed
    EyeTrackingAccess,
    /// Generic sensor access
    SensorAccess,
    
    // Data events
    /// Data exported
    DataExported,
    /// Data deleted
    DataDeleted,
    /// Data shared
    DataShared,
    /// Backup created
    BackupCreated,
    
    // Security events
    /// Authentication success
    AuthSuccess,
    /// Authentication failure
    AuthFailure,
    /// Device locked
    DeviceLocked,
    /// Device unlocked
    DeviceUnlocked,
    /// Failed login attempt
    FailedLogin,
    /// Encryption key rotated
    KeyRotation,
    
    // App events
    /// App installed
    AppInstalled,
    /// App uninstalled
    AppUninstalled,
    /// App data cleared
    AppDataCleared,
    
    // System events
    /// System boot
    SystemBoot,
    /// System shutdown
    SystemShutdown,
    /// Settings changed
    SettingsChanged,
    /// Firmware updated
    FirmwareUpdate,
    
    // Network events
    /// Network connected
    NetworkConnected,
    /// Network disconnected
    NetworkDisconnected,
    /// Data sync
    DataSync,
    /// Cloud access
    CloudAccess,
}

impl AuditEventType {
    /// Get category
    pub fn category(&self) -> AuditCategory {
        match self {
            AuditEventType::PrivacyModeChanged |
            AuditEventType::ConsentChanged |
            AuditEventType::PrivacyZoneEvent |
            AuditEventType::ZoneConfigured => AuditCategory::Privacy,
            
            AuditEventType::PermissionRequested |
            AuditEventType::PermissionGranted |
            AuditEventType::PermissionDenied |
            AuditEventType::PermissionRevoked => AuditCategory::Permission,
            
            AuditEventType::CameraAccess |
            AuditEventType::MicrophoneAccess |
            AuditEventType::LocationAccess |
            AuditEventType::EyeTrackingAccess |
            AuditEventType::SensorAccess => AuditCategory::SensorAccess,
            
            AuditEventType::DataExported |
            AuditEventType::DataDeleted |
            AuditEventType::DataShared |
            AuditEventType::BackupCreated => AuditCategory::Data,
            
            AuditEventType::AuthSuccess |
            AuditEventType::AuthFailure |
            AuditEventType::DeviceLocked |
            AuditEventType::DeviceUnlocked |
            AuditEventType::FailedLogin |
            AuditEventType::KeyRotation => AuditCategory::Security,
            
            AuditEventType::AppInstalled |
            AuditEventType::AppUninstalled |
            AuditEventType::AppDataCleared => AuditCategory::App,
            
            AuditEventType::SystemBoot |
            AuditEventType::SystemShutdown |
            AuditEventType::SettingsChanged |
            AuditEventType::FirmwareUpdate => AuditCategory::System,
            
            AuditEventType::NetworkConnected |
            AuditEventType::NetworkDisconnected |
            AuditEventType::DataSync |
            AuditEventType::CloudAccess => AuditCategory::Network,
        }
    }
    
    /// Is this a security-critical event?
    pub fn is_security_critical(&self) -> bool {
        matches!(
            self,
            AuditEventType::AuthFailure |
            AuditEventType::FailedLogin |
            AuditEventType::PermissionDenied |
            AuditEventType::DataExported |
            AuditEventType::DataShared
        )
    }
    
    /// Severity level (1-5, 5 being most severe)
    pub fn severity(&self) -> u8 {
        match self {
            AuditEventType::AuthFailure | AuditEventType::FailedLogin => 5,
            AuditEventType::DataExported | AuditEventType::DataShared => 4,
            AuditEventType::PermissionDenied | AuditEventType::PermissionRevoked => 3,
            AuditEventType::CameraAccess | AuditEventType::MicrophoneAccess => 3,
            AuditEventType::PrivacyModeChanged | AuditEventType::ConsentChanged => 2,
            _ => 1,
        }
    }
}

/// Audit category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditCategory {
    /// Privacy-related events
    Privacy,
    /// Permission-related events
    Permission,
    /// Sensor access events
    SensorAccess,
    /// Data handling events
    Data,
    /// Security events
    Security,
    /// App lifecycle events
    App,
    /// System events
    System,
    /// Network events
    Network,
}

/// Audit event
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// Event ID
    pub id: u64,
    /// Event type
    pub event_type: AuditEventType,
    /// Event description
    pub description: String,
    /// Timestamp (system time for persistence)
    pub timestamp: SystemTime,
    /// Instant for runtime calculations
    pub instant: Instant,
    /// Associated app ID (if any)
    pub app_id: Option<String>,
    /// User action (vs system action)
    pub user_initiated: bool,
    /// Additional metadata
    pub metadata: Vec<(String, String)>,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(event_type: AuditEventType, description: String) -> Self {
        Self {
            id: 0, // Set by manager
            event_type,
            description,
            timestamp: SystemTime::now(),
            instant: Instant::now(),
            app_id: None,
            user_initiated: false,
            metadata: Vec::new(),
        }
    }
    
    /// Set app ID
    pub fn with_app(mut self, app_id: &str) -> Self {
        self.app_id = Some(app_id.to_string());
        self
    }
    
    /// Mark as user initiated
    pub fn user_initiated(mut self) -> Self {
        self.user_initiated = true;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.push((key.to_string(), value.to_string()));
        self
    }
    
    /// Get age of event
    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.instant)
    }
}

/// Audit log (in-memory buffer)
#[derive(Debug)]
pub struct AuditLog {
    /// Events (circular buffer)
    events: VecDeque<AuditEvent>,
    /// Maximum events to keep
    max_events: usize,
    /// Next event ID
    next_id: u64,
}

impl AuditLog {
    /// Create new audit log
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
            next_id: 1,
        }
    }
    
    /// Add event
    pub fn push(&mut self, mut event: AuditEvent) {
        event.id = self.next_id;
        self.next_id += 1;
        
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        
        self.events.push_back(event);
    }
    
    /// Get all events
    pub fn events(&self) -> impl Iterator<Item = &AuditEvent> {
        self.events.iter()
    }
    
    /// Get events by type
    pub fn events_by_type(&self, event_type: AuditEventType) -> Vec<&AuditEvent> {
        self.events.iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }
    
    /// Get events by category
    pub fn events_by_category(&self, category: AuditCategory) -> Vec<&AuditEvent> {
        self.events.iter()
            .filter(|e| e.event_type.category() == category)
            .collect()
    }
    
    /// Get events for app
    pub fn events_for_app(&self, app_id: &str) -> Vec<&AuditEvent> {
        self.events.iter()
            .filter(|e| e.app_id.as_deref() == Some(app_id))
            .collect()
    }
    
    /// Get recent events
    pub fn recent(&self, count: usize) -> Vec<&AuditEvent> {
        self.events.iter().rev().take(count).collect()
    }
    
    /// Get events in time range
    pub fn in_range(&self, since: Duration) -> Vec<&AuditEvent> {
        let cutoff = Instant::now() - since;
        self.events.iter()
            .filter(|e| e.instant >= cutoff)
            .collect()
    }
    
    /// Get security events
    pub fn security_events(&self) -> Vec<&AuditEvent> {
        self.events.iter()
            .filter(|e| e.event_type.is_security_critical())
            .collect()
    }
    
    /// Event count
    pub fn len(&self) -> usize {
        self.events.len()
    }
    
    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    
    /// Clear log
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Audit manager
#[derive(Debug)]
pub struct AuditManager {
    /// In-memory audit log
    log: AuditLog,
    /// Security alerts enabled
    security_alerts: bool,
    /// Alert threshold (events per minute)
    alert_threshold: u32,
    /// Recent security events (for rate detection)
    recent_security: VecDeque<Instant>,
    /// Categories to track
    tracked_categories: Vec<AuditCategory>,
    /// Export format
    export_format: ExportFormat,
}

/// Export format for audit logs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Plain text
    Text,
}

impl AuditManager {
    /// Create new audit manager
    pub fn new() -> Self {
        Self {
            log: AuditLog::default(),
            security_alerts: true,
            alert_threshold: 10,
            recent_security: VecDeque::new(),
            tracked_categories: vec![
                AuditCategory::Privacy,
                AuditCategory::Permission,
                AuditCategory::Security,
                AuditCategory::Data,
                AuditCategory::System,
                AuditCategory::SensorAccess,
            ],
            export_format: ExportFormat::Json,
        }
    }
    
    /// Log an event
    pub fn log_event(&mut self, event: AuditEvent) {
        // Check for security alert
        if self.security_alerts && event.event_type.is_security_critical() {
            self.check_security_alert();
            self.recent_security.push_back(Instant::now());
        }
        
        // Only log tracked categories (or all if empty)
        if self.tracked_categories.is_empty() || 
           self.tracked_categories.contains(&event.event_type.category()) {
            self.log.push(event);
        }
    }
    
    /// Check for security alert conditions
    fn check_security_alert(&mut self) {
        // Clean old entries
        let cutoff = Instant::now() - Duration::from_secs(60);
        self.recent_security.retain(|t| *t >= cutoff);
        
        // Check threshold
        if self.recent_security.len() >= self.alert_threshold as usize {
            // Would trigger alert in production
        }
    }
    
    /// Get audit log
    pub fn audit_log(&self) -> &AuditLog {
        &self.log
    }
    
    /// Entry count
    pub fn entry_count(&self) -> usize {
        self.log.len()
    }
    
    /// Export audit log
    pub fn export(&self) -> String {
        match self.export_format {
            ExportFormat::Json => self.export_json(),
            ExportFormat::Csv => self.export_csv(),
            ExportFormat::Text => self.export_text(),
        }
    }
    
    /// Export as JSON
    fn export_json(&self) -> String {
        let mut output = String::from("[\n");
        
        for (i, event) in self.log.events().enumerate() {
            if i > 0 {
                output.push_str(",\n");
            }
            output.push_str(&format!(
                "  {{\"id\": {}, \"type\": \"{:?}\", \"description\": \"{}\"}}",
                event.id, event.event_type, event.description
            ));
        }
        
        output.push_str("\n]");
        output
    }
    
    /// Export as CSV
    fn export_csv(&self) -> String {
        let mut output = String::from("id,type,description,app_id,user_initiated\n");
        
        for event in self.log.events() {
            output.push_str(&format!(
                "{},{:?},{},{},{}\n",
                event.id,
                event.event_type,
                event.description,
                event.app_id.as_deref().unwrap_or(""),
                event.user_initiated
            ));
        }
        
        output
    }
    
    /// Export as text
    fn export_text(&self) -> String {
        let mut output = String::from("=== Audit Log ===\n\n");
        
        for event in self.log.events() {
            output.push_str(&format!(
                "[{}] {:?}: {}\n",
                event.id, event.event_type, event.description
            ));
        }
        
        output
    }
    
    /// Set export format
    pub fn set_export_format(&mut self, format: ExportFormat) {
        self.export_format = format;
    }
    
    /// Enable/disable security alerts
    pub fn set_security_alerts(&mut self, enabled: bool) {
        self.security_alerts = enabled;
    }
    
    /// Set tracked categories
    pub fn set_tracked_categories(&mut self, categories: Vec<AuditCategory>) {
        self.tracked_categories = categories;
    }
    
    /// Get statistics
    pub fn stats(&self) -> AuditStats {
        let by_category: std::collections::HashMap<AuditCategory, usize> = 
            [AuditCategory::Privacy, AuditCategory::Permission, 
             AuditCategory::Security, AuditCategory::Data,
             AuditCategory::SensorAccess, AuditCategory::App,
             AuditCategory::System, AuditCategory::Network]
            .iter()
            .map(|cat| (*cat, self.log.events_by_category(*cat).len()))
            .collect();
        
        AuditStats {
            total_events: self.log.len(),
            security_events: self.log.security_events().len(),
            events_by_category: by_category,
            recent_security_count: self.recent_security.len(),
        }
    }
    
    /// Clear audit log
    pub fn clear(&mut self) {
        self.log.clear();
        self.recent_security.clear();
    }
}

impl Default for AuditManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit statistics
#[derive(Debug, Clone)]
pub struct AuditStats {
    /// Total events logged
    pub total_events: usize,
    /// Security-critical events
    pub security_events: usize,
    /// Events by category
    pub events_by_category: std::collections::HashMap<AuditCategory, usize>,
    /// Recent security events (last minute)
    pub recent_security_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_manager_creation() {
        let am = AuditManager::new();
        assert_eq!(am.entry_count(), 0);
    }
    
    #[test]
    fn test_log_event() {
        let mut am = AuditManager::new();
        
        am.log_event(AuditEvent::new(
            AuditEventType::PrivacyModeChanged,
            "Mode changed to Enhanced".to_string(),
        ));
        
        assert_eq!(am.entry_count(), 1);
    }
    
    #[test]
    fn test_event_categories() {
        assert_eq!(
            AuditEventType::PrivacyModeChanged.category(),
            AuditCategory::Privacy
        );
        assert_eq!(
            AuditEventType::AuthFailure.category(),
            AuditCategory::Security
        );
    }
    
    #[test]
    fn test_security_critical() {
        assert!(AuditEventType::AuthFailure.is_security_critical());
        assert!(!AuditEventType::SystemBoot.is_security_critical());
    }
    
    #[test]
    fn test_audit_log_circular() {
        let mut log = AuditLog::new(3);
        
        for i in 0..5 {
            log.push(AuditEvent::new(
                AuditEventType::SystemBoot,
                format!("Event {}", i),
            ));
        }
        
        assert_eq!(log.len(), 3);
        // Oldest events should be removed
        let events: Vec<_> = log.events().collect();
        assert!(events[0].description.contains("2"));
    }
    
    #[test]
    fn test_filter_by_type() {
        let mut am = AuditManager::new();
        
        am.log_event(AuditEvent::new(AuditEventType::SystemBoot, "Boot".to_string()));
        am.log_event(AuditEvent::new(AuditEventType::AuthSuccess, "Auth".to_string()));
        am.log_event(AuditEvent::new(AuditEventType::SystemBoot, "Boot2".to_string()));
        
        let boot_events = am.audit_log().events_by_type(AuditEventType::SystemBoot);
        assert_eq!(boot_events.len(), 2);
    }
    
    #[test]
    fn test_filter_by_app() {
        let mut am = AuditManager::new();
        
        am.log_event(AuditEvent::new(AuditEventType::CameraAccess, "Access".to_string())
            .with_app("app1"));
        am.log_event(AuditEvent::new(AuditEventType::CameraAccess, "Access".to_string())
            .with_app("app2"));
        
        let app1_events = am.audit_log().events_for_app("app1");
        assert_eq!(app1_events.len(), 1);
    }
    
    #[test]
    fn test_export_json() {
        let mut am = AuditManager::new();
        am.set_export_format(ExportFormat::Json);
        
        am.log_event(AuditEvent::new(AuditEventType::SystemBoot, "Boot".to_string()));
        
        let export = am.export();
        assert!(export.contains("SystemBoot"));
        assert!(export.starts_with("["));
    }
    
    #[test]
    fn test_export_csv() {
        let mut am = AuditManager::new();
        am.set_export_format(ExportFormat::Csv);
        
        am.log_event(AuditEvent::new(AuditEventType::SystemBoot, "Boot".to_string()));
        
        let export = am.export();
        assert!(export.contains("id,type,description"));
    }
    
    #[test]
    fn test_event_with_metadata() {
        let event = AuditEvent::new(AuditEventType::DataExported, "Export".to_string())
            .with_metadata("format", "json")
            .with_metadata("size", "1024");
        
        assert_eq!(event.metadata.len(), 2);
    }
    
    #[test]
    fn test_audit_stats() {
        let mut am = AuditManager::new();
        
        am.log_event(AuditEvent::new(AuditEventType::AuthFailure, "Failed".to_string()));
        am.log_event(AuditEvent::new(AuditEventType::SystemBoot, "Boot".to_string()));
        
        let stats = am.stats();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.security_events, 1);
    }
    
    #[test]
    fn test_severity_levels() {
        assert!(AuditEventType::AuthFailure.severity() > AuditEventType::SystemBoot.severity());
    }
}
