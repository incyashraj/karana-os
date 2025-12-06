// Privacy Dashboard - User-facing privacy controls and transparency
// Phase 50: Visibility and control over personal data

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

use super::retention::{DataCategory, DataItem, RetentionPolicy};

/// Privacy consent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentStatus {
    Granted,
    Denied,
    Pending,
    Revoked,
}

/// Privacy consent for a specific purpose
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConsent {
    pub purpose: String,
    pub status: ConsentStatus,
    pub granted_at: Option<SystemTime>,
    pub revoked_at: Option<SystemTime>,
    pub required: bool,
}

/// Data access log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessLog {
    pub timestamp: SystemTime,
    pub accessor: String,
    pub data_category: DataCategory,
    pub action: AccessAction,
    pub item_count: usize,
}

/// Type of data access action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessAction {
    Read,
    Write,
    Delete,
    Export,
    Share,
}

/// Privacy summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySummary {
    pub total_data_items: u64,
    pub total_size_mb: f64,
    pub categories: HashMap<DataCategory, CategorySummary>,
    pub recent_accesses: Vec<DataAccessLog>,
    pub consent_summary: ConsentSummary,
    pub retention_summary: RetentionSummary,
}

/// Summary for a data category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub item_count: u64,
    pub size_mb: f64,
    pub oldest_item_age_days: u32,
    pub newest_item_age_days: u32,
}

/// Summary of consents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentSummary {
    pub total_consents: usize,
    pub granted: usize,
    pub denied: usize,
    pub pending: usize,
    pub revoked: usize,
}

/// Summary of retention policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionSummary {
    pub total_policies: usize,
    pub auto_delete_enabled: usize,
    pub items_to_expire_soon: u64,
    pub next_cleanup: Option<SystemTime>,
}

/// Privacy dashboard manager
pub struct PrivacyDashboard {
    consents: Arc<RwLock<HashMap<String, PrivacyConsent>>>,
    access_logs: Arc<RwLock<Vec<DataAccessLog>>>,
    max_logs: usize,
}

impl PrivacyDashboard {
    /// Create new privacy dashboard
    pub fn new() -> Self {
        Self {
            consents: Arc::new(RwLock::new(HashMap::new())),
            access_logs: Arc::new(RwLock::new(Vec::new())),
            max_logs: 1000,
        }
    }
    
    /// Initialize with default consents
    pub async fn initialize_defaults(&self) {
        let default_consents = vec![
            ("analytics", "Usage analytics for product improvement", false),
            ("telemetry", "Technical telemetry for debugging", false),
            ("personalization", "Personalized experience based on usage", true),
            ("location_tracking", "Location-based features", true),
            ("voice_processing", "Voice command processing", true),
            ("vision_processing", "Visual scene understanding", true),
            ("data_sharing", "Share data with trusted partners", false),
            ("marketing", "Marketing communications", false),
        ];
        
        let mut consents = self.consents.write().await;
        for (purpose, description, required) in default_consents {
            consents.insert(
                purpose.to_string(),
                PrivacyConsent {
                    purpose: description.to_string(),
                    status: if required {
                        ConsentStatus::Granted
                    } else {
                        ConsentStatus::Pending
                    },
                    granted_at: if required {
                        Some(SystemTime::now())
                    } else {
                        None
                    },
                    revoked_at: None,
                    required,
                },
            );
        }
    }
    
    /// Grant consent for a purpose
    pub async fn grant_consent(&self, purpose: &str) -> Result<()> {
        let mut consents = self.consents.write().await;
        
        if let Some(consent) = consents.get_mut(purpose) {
            consent.status = ConsentStatus::Granted;
            consent.granted_at = Some(SystemTime::now());
            consent.revoked_at = None;
        }
        
        Ok(())
    }
    
    /// Deny consent for a purpose
    pub async fn deny_consent(&self, purpose: &str) -> Result<()> {
        let mut consents = self.consents.write().await;
        
        if let Some(consent) = consents.get_mut(purpose) {
            if consent.required {
                return Err(anyhow::anyhow!("Cannot deny required consent: {}", purpose));
            }
            
            consent.status = ConsentStatus::Denied;
            consent.granted_at = None;
        }
        
        Ok(())
    }
    
    /// Revoke previously granted consent
    pub async fn revoke_consent(&self, purpose: &str) -> Result<()> {
        let mut consents = self.consents.write().await;
        
        if let Some(consent) = consents.get_mut(purpose) {
            if consent.required {
                return Err(anyhow::anyhow!("Cannot revoke required consent: {}", purpose));
            }
            
            if consent.status == ConsentStatus::Granted {
                consent.status = ConsentStatus::Revoked;
                consent.revoked_at = Some(SystemTime::now());
            }
        }
        
        Ok(())
    }
    
    /// Check if consent is granted
    pub async fn has_consent(&self, purpose: &str) -> bool {
        self.consents
            .read()
            .await
            .get(purpose)
            .map(|c| c.status == ConsentStatus::Granted)
            .unwrap_or(false)
    }
    
    /// Get all consents
    pub async fn get_all_consents(&self) -> HashMap<String, PrivacyConsent> {
        self.consents.read().await.clone()
    }
    
    /// Log data access
    pub async fn log_access(
        &self,
        accessor: String,
        category: DataCategory,
        action: AccessAction,
        item_count: usize,
    ) {
        let log = DataAccessLog {
            timestamp: SystemTime::now(),
            accessor,
            data_category: category,
            action,
            item_count,
        };
        
        let mut logs = self.access_logs.write().await;
        logs.push(log);
        
        // Keep only recent logs
        if logs.len() > self.max_logs {
            let drain_end = logs.len() - self.max_logs;
            logs.drain(0..drain_end);
        }
    }
    
    /// Get recent access logs
    pub async fn get_recent_accesses(&self, limit: usize) -> Vec<DataAccessLog> {
        let logs = self.access_logs.read().await;
        let start = logs.len().saturating_sub(limit);
        logs[start..].to_vec()
    }
    
    /// Get access logs for a category
    pub async fn get_accesses_for_category(&self, category: &DataCategory) -> Vec<DataAccessLog> {
        self.access_logs
            .read()
            .await
            .iter()
            .filter(|log| &log.data_category == category)
            .cloned()
            .collect()
    }
    
    /// Generate privacy summary
    pub async fn generate_summary(
        &self,
        items: &[DataItem],
        policies: &HashMap<DataCategory, RetentionPolicy>,
    ) -> PrivacySummary {
        // Calculate category summaries
        let mut categories = HashMap::new();
        let now = SystemTime::now();
        
        for item in items {
            let entry = categories
                .entry(item.category.clone())
                .or_insert_with(|| CategorySummary {
                    item_count: 0,
                    size_mb: 0.0,
                    oldest_item_age_days: 0,
                    newest_item_age_days: u32::MAX,
                });
            
            entry.item_count += 1;
            entry.size_mb += item.size_bytes as f64 / (1024.0 * 1024.0);
            
            if let Ok(age) = now.duration_since(item.created_at) {
                let age_days = (age.as_secs() / 86400) as u32;
                entry.oldest_item_age_days = entry.oldest_item_age_days.max(age_days);
                entry.newest_item_age_days = entry.newest_item_age_days.min(age_days);
            }
        }
        
        // Calculate consent summary
        let consents = self.consents.read().await;
        let consent_summary = ConsentSummary {
            total_consents: consents.len(),
            granted: consents.values().filter(|c| c.status == ConsentStatus::Granted).count(),
            denied: consents.values().filter(|c| c.status == ConsentStatus::Denied).count(),
            pending: consents.values().filter(|c| c.status == ConsentStatus::Pending).count(),
            revoked: consents.values().filter(|c| c.status == ConsentStatus::Revoked).count(),
        };
        
        // Calculate retention summary
        let retention_summary = RetentionSummary {
            total_policies: policies.len(),
            auto_delete_enabled: policies.values().filter(|p| p.auto_delete).count(),
            items_to_expire_soon: items
                .iter()
                .filter(|item| {
                    if let Some(policy) = policies.get(&item.category) {
                        if let Ok(age) = now.duration_since(item.created_at) {
                            let remaining = policy.retention_period.saturating_sub(age);
                            remaining.as_secs() < 7 * 24 * 3600 // Expires within 7 days
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                })
                .count() as u64,
            next_cleanup: policies
                .values()
                .filter_map(|p| p.last_cleanup)
                .max()
                .map(|last| last + std::time::Duration::from_secs(24 * 3600)), // Daily cleanup
        };
        
        PrivacySummary {
            total_data_items: items.len() as u64,
            total_size_mb: items.iter().map(|i| i.size_bytes as f64 / (1024.0 * 1024.0)).sum(),
            categories,
            recent_accesses: self.get_recent_accesses(10).await,
            consent_summary,
            retention_summary,
        }
    }
    
    /// Export user data (GDPR right to data portability)
    pub async fn export_user_data(&self, items: &[DataItem]) -> Result<Vec<u8>> {
        // In real implementation, this would create a comprehensive data export
        // in a standard format (JSON, CSV, etc.)
        
        let data = serde_json::to_vec_pretty(&items)?;
        
        // Log the export
        self.log_access(
            "user".to_string(),
            DataCategory::Preferences,
            AccessAction::Export,
            items.len(),
        )
        .await;
        
        Ok(data)
    }
    
    /// Clear all access logs
    pub async fn clear_access_logs(&self) {
        self.access_logs.write().await.clear();
    }
}

impl Default for PrivacyDashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_privacy_dashboard_creation() {
        let dashboard = PrivacyDashboard::new();
        dashboard.initialize_defaults().await;
        
        let consents = dashboard.get_all_consents().await;
        assert_eq!(consents.len(), 8);
    }
    
    #[tokio::test]
    async fn test_grant_consent() {
        let dashboard = PrivacyDashboard::new();
        dashboard.initialize_defaults().await;
        
        dashboard.grant_consent("analytics").await.unwrap();
        assert!(dashboard.has_consent("analytics").await);
    }
    
    #[tokio::test]
    async fn test_deny_consent() {
        let dashboard = PrivacyDashboard::new();
        dashboard.initialize_defaults().await;
        
        dashboard.deny_consent("marketing").await.unwrap();
        assert!(!dashboard.has_consent("marketing").await);
    }
    
    #[tokio::test]
    async fn test_cannot_deny_required() {
        let dashboard = PrivacyDashboard::new();
        dashboard.initialize_defaults().await;
        
        let result = dashboard.deny_consent("voice_processing").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_revoke_consent() {
        let dashboard = PrivacyDashboard::new();
        dashboard.initialize_defaults().await;
        
        dashboard.grant_consent("analytics").await.unwrap();
        assert!(dashboard.has_consent("analytics").await);
        
        dashboard.revoke_consent("analytics").await.unwrap();
        assert!(!dashboard.has_consent("analytics").await);
    }
    
    #[tokio::test]
    async fn test_access_logging() {
        let dashboard = PrivacyDashboard::new();
        
        dashboard
            .log_access(
                "system".to_string(),
                DataCategory::SensorData,
                AccessAction::Read,
                10,
            )
            .await;
        
        let logs = dashboard.get_recent_accesses(10).await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].item_count, 10);
    }
    
    #[tokio::test]
    async fn test_category_access_logs() {
        let dashboard = PrivacyDashboard::new();
        
        dashboard
            .log_access(
                "system".to_string(),
                DataCategory::SensorData,
                AccessAction::Read,
                5,
            )
            .await;
        
        dashboard
            .log_access(
                "system".to_string(),
                DataCategory::VoiceRecordings,
                AccessAction::Write,
                3,
            )
            .await;
        
        let logs = dashboard.get_accesses_for_category(&DataCategory::SensorData).await;
        assert_eq!(logs.len(), 1);
    }
    
    #[tokio::test]
    async fn test_generate_summary() {
        let dashboard = PrivacyDashboard::new();
        dashboard.initialize_defaults().await;
        
        let items = vec![
            DataItem {
                id: "1".to_string(),
                category: DataCategory::SensorData,
                created_at: SystemTime::now(),
                size_bytes: 1024 * 1024, // 1 MB
                metadata: HashMap::new(),
            },
            DataItem {
                id: "2".to_string(),
                category: DataCategory::SensorData,
                created_at: SystemTime::now(),
                size_bytes: 2 * 1024 * 1024, // 2 MB
                metadata: HashMap::new(),
            },
        ];
        
        let policies = HashMap::new();
        let summary = dashboard.generate_summary(&items, &policies).await;
        
        assert_eq!(summary.total_data_items, 2);
        assert!((summary.total_size_mb - 3.0).abs() < 0.1);
        assert_eq!(summary.categories.len(), 1);
    }
}
