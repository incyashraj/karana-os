// Ephemeral Modes - Temporary privacy modes with no data persistence
// Phase 50: Private browsing equivalent for AR glasses

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use super::retention::DataCategory;

/// Ephemeral mode type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EphemeralMode {
    /// No data persistence at all
    FullyPrivate,
    
    /// Only essential data persisted
    MinimalTrace,
    
    /// Session-only persistence (deleted on exit)
    SessionOnly,
    
    /// Incognito mode (like browser private mode)
    Incognito,
}

impl EphemeralMode {
    /// Get categories that are disabled in this mode
    pub fn disabled_categories(&self) -> Vec<DataCategory> {
        match self {
            EphemeralMode::FullyPrivate => vec![
                DataCategory::SensorData,
                DataCategory::VoiceRecordings,
                DataCategory::VisionCaptures,
                DataCategory::Conversations,
                DataCategory::LocationHistory,
                DataCategory::UsageData,
                DataCategory::KnowledgeEntries,
                DataCategory::Diagnostics,
                DataCategory::TemporaryData,
            ],
            EphemeralMode::MinimalTrace => vec![
                DataCategory::VoiceRecordings,
                DataCategory::VisionCaptures,
                DataCategory::Conversations,
                DataCategory::LocationHistory,
            ],
            EphemeralMode::SessionOnly => vec![
                DataCategory::Conversations,
                DataCategory::LocationHistory,
                DataCategory::TemporaryData,
            ],
            EphemeralMode::Incognito => vec![
                DataCategory::Conversations,
                DataCategory::UsageData,
                DataCategory::TemporaryData,
            ],
        }
    }
    
    /// Check if category is allowed in this mode
    pub fn allows_category(&self, category: &DataCategory) -> bool {
        !self.disabled_categories().contains(category)
    }
    
    /// Get mode description
    pub fn description(&self) -> &'static str {
        match self {
            EphemeralMode::FullyPrivate => "Complete privacy - no data saved",
            EphemeralMode::MinimalTrace => "Minimal trace - only essential data",
            EphemeralMode::SessionOnly => "Session only - data deleted on exit",
            EphemeralMode::Incognito => "Incognito - private browsing mode",
        }
    }
}

/// Ephemeral session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralSession {
    pub session_id: String,
    pub mode: EphemeralMode,
    pub started_at: SystemTime,
    pub ended_at: Option<SystemTime>,
    pub data_captured: HashSet<DataCategory>,
    pub items_pending_deletion: Vec<String>,
}

impl EphemeralSession {
    /// Create new ephemeral session
    pub fn new(mode: EphemeralMode) -> Self {
        Self {
            session_id: format!("ephemeral-{}", SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()),
            mode,
            started_at: SystemTime::now(),
            ended_at: None,
            data_captured: HashSet::new(),
            items_pending_deletion: Vec::new(),
        }
    }
    
    /// Get session duration
    pub fn duration(&self) -> Duration {
        let end = self.ended_at.unwrap_or_else(SystemTime::now);
        end.duration_since(self.started_at).unwrap_or_default()
    }
    
    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }
}

/// Ephemeral mode statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EphemeralStats {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub total_session_time: Duration,
    pub items_blocked: u64,
    pub items_deleted_on_exit: u64,
}

/// Ephemeral mode manager
pub struct EphemeralModeManager {
    current_session: Arc<RwLock<Option<EphemeralSession>>>,
    session_history: Arc<RwLock<Vec<EphemeralSession>>>,
    stats: Arc<RwLock<EphemeralStats>>,
    max_history: usize,
}

impl EphemeralModeManager {
    /// Create new ephemeral mode manager
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(RwLock::new(None)),
            session_history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(EphemeralStats::default())),
            max_history: 100,
        }
    }
    
    /// Start ephemeral mode
    pub async fn start_ephemeral(&self, mode: EphemeralMode) -> Result<String> {
        let mut current = self.current_session.write().await;
        
        if current.is_some() {
            return Err(anyhow::anyhow!("Ephemeral mode already active"));
        }
        
        let session = EphemeralSession::new(mode);
        let session_id = session.session_id.clone();
        
        *current = Some(session);
        
        let mut stats = self.stats.write().await;
        stats.total_sessions += 1;
        stats.active_sessions += 1;
        
        Ok(session_id)
    }
    
    /// Stop ephemeral mode
    pub async fn stop_ephemeral(&self) -> Result<Vec<String>> {
        let mut current = self.current_session.write().await;
        
        let mut session = current.take().ok_or_else(|| {
            anyhow::anyhow!("No active ephemeral session")
        })?;
        
        session.ended_at = Some(SystemTime::now());
        
        // Get items to delete
        let items_to_delete = session.items_pending_deletion.clone();
        let delete_count = items_to_delete.len();
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.active_sessions = stats.active_sessions.saturating_sub(1);
        stats.total_session_time += session.duration();
        stats.items_deleted_on_exit += delete_count as u64;
        drop(stats);
        
        // Archive session
        let mut history = self.session_history.write().await;
        history.push(session);
        
        // Keep only recent history
        if history.len() > self.max_history {
            let drain_end = history.len() - self.max_history;
            history.drain(0..drain_end);
        }
        
        Ok(items_to_delete)
    }
    
    /// Check if ephemeral mode is active
    pub async fn is_active(&self) -> bool {
        self.current_session.read().await.is_some()
    }
    
    /// Get current ephemeral mode
    pub async fn get_current_mode(&self) -> Option<EphemeralMode> {
        self.current_session.read().await.as_ref().map(|s| s.mode)
    }
    
    /// Check if data capture is allowed
    pub async fn allows_data_capture(&self, category: &DataCategory) -> bool {
        if let Some(session) = self.current_session.read().await.as_ref() {
            session.mode.allows_category(category)
        } else {
            true // Not in ephemeral mode, allow all
        }
    }
    
    /// Record data capture attempt
    pub async fn record_capture_attempt(&self, category: DataCategory, item_id: String) -> bool {
        let mut current = self.current_session.write().await;
        
        if let Some(session) = current.as_mut() {
            if session.mode.allows_category(&category) {
                session.data_captured.insert(category);
                session.items_pending_deletion.push(item_id);
                true
            } else {
                // Block capture
                let mut stats = self.stats.write().await;
                stats.items_blocked += 1;
                false
            }
        } else {
            true // Not in ephemeral mode
        }
    }
    
    /// Get current session info
    pub async fn get_current_session(&self) -> Option<EphemeralSession> {
        self.current_session.read().await.clone()
    }
    
    /// Get session history
    pub async fn get_session_history(&self) -> Vec<EphemeralSession> {
        self.session_history.read().await.clone()
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> EphemeralStats {
        self.stats.read().await.clone()
    }
    
    /// Clear session history
    pub async fn clear_history(&self) {
        self.session_history.write().await.clear();
    }
    
    /// Get items pending deletion
    pub async fn get_pending_deletions(&self) -> Vec<String> {
        self.current_session
            .read()
            .await
            .as_ref()
            .map(|s| s.items_pending_deletion.clone())
            .unwrap_or_default()
    }
}

impl Default for EphemeralModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ephemeral_manager_creation() {
        let manager = EphemeralModeManager::new();
        assert!(!manager.is_active().await);
    }
    
    #[tokio::test]
    async fn test_start_ephemeral() {
        let manager = EphemeralModeManager::new();
        
        let session_id = manager.start_ephemeral(EphemeralMode::Incognito).await.unwrap();
        assert!(!session_id.is_empty());
        assert!(manager.is_active().await);
        
        let mode = manager.get_current_mode().await;
        assert_eq!(mode, Some(EphemeralMode::Incognito));
    }
    
    #[tokio::test]
    async fn test_stop_ephemeral() {
        let manager = EphemeralModeManager::new();
        
        manager.start_ephemeral(EphemeralMode::SessionOnly).await.unwrap();
        
        let items = manager.stop_ephemeral().await.unwrap();
        assert!(!manager.is_active().await);
        assert!(items.is_empty());
    }
    
    #[tokio::test]
    async fn test_double_start_fails() {
        let manager = EphemeralModeManager::new();
        
        manager.start_ephemeral(EphemeralMode::Incognito).await.unwrap();
        
        let result = manager.start_ephemeral(EphemeralMode::FullyPrivate).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_allows_data_capture() {
        let manager = EphemeralModeManager::new();
        
        // Not in ephemeral mode - allow all
        assert!(manager.allows_data_capture(&DataCategory::Conversations).await);
        
        // Start fully private mode
        manager.start_ephemeral(EphemeralMode::FullyPrivate).await.unwrap();
        
        // Block most categories
        assert!(!manager.allows_data_capture(&DataCategory::Conversations).await);
        assert!(!manager.allows_data_capture(&DataCategory::VoiceRecordings).await);
        
        // Allow essential categories
        assert!(manager.allows_data_capture(&DataCategory::Preferences).await);
    }
    
    #[tokio::test]
    async fn test_record_capture_attempt() {
        let manager = EphemeralModeManager::new();
        
        manager.start_ephemeral(EphemeralMode::Incognito).await.unwrap();
        
        // Allowed category
        let allowed = manager
            .record_capture_attempt(DataCategory::Preferences, "item-1".to_string())
            .await;
        assert!(allowed);
        
        // Blocked category
        let blocked = manager
            .record_capture_attempt(DataCategory::Conversations, "item-2".to_string())
            .await;
        assert!(!blocked);
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.items_blocked, 1);
    }
    
    #[tokio::test]
    async fn test_pending_deletions() {
        let manager = EphemeralModeManager::new();
        
        manager.start_ephemeral(EphemeralMode::SessionOnly).await.unwrap();
        
        manager
            .record_capture_attempt(DataCategory::SensorData, "item-1".to_string())
            .await;
        manager
            .record_capture_attempt(DataCategory::UsageData, "item-2".to_string())
            .await;
        
        let pending = manager.get_pending_deletions().await;
        assert_eq!(pending.len(), 2);
        
        let items = manager.stop_ephemeral().await.unwrap();
        assert_eq!(items.len(), 2);
    }
    
    #[tokio::test]
    async fn test_session_history() {
        let manager = EphemeralModeManager::new();
        
        manager.start_ephemeral(EphemeralMode::Incognito).await.unwrap();
        manager.stop_ephemeral().await.unwrap();
        
        manager.start_ephemeral(EphemeralMode::FullyPrivate).await.unwrap();
        manager.stop_ephemeral().await.unwrap();
        
        let history = manager.get_session_history().await;
        assert_eq!(history.len(), 2);
    }
    
    #[tokio::test]
    async fn test_ephemeral_mode_categories() {
        let fully_private = EphemeralMode::FullyPrivate;
        let disabled = fully_private.disabled_categories();
        
        assert!(disabled.contains(&DataCategory::VoiceRecordings));
        assert!(disabled.contains(&DataCategory::Conversations));
        
        assert!(fully_private.allows_category(&DataCategory::Preferences));
        assert!(!fully_private.allows_category(&DataCategory::VoiceRecordings));
    }
    
    #[tokio::test]
    async fn test_statistics() {
        let manager = EphemeralModeManager::new();
        
        manager.start_ephemeral(EphemeralMode::Incognito).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        manager.stop_ephemeral().await.unwrap();
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.active_sessions, 0);
        assert!(stats.total_session_time.as_millis() >= 100);
    }
}
