//! Content Sharing for Kāraṇa OS AR Glasses
//!
//! Share content (photos, locations, contacts) with others.

use std::collections::HashMap;
use std::time::Instant;

/// Content type for sharing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareContentType {
    /// Text
    Text,
    /// URL
    URL,
    /// Image
    Image,
    /// Video
    Video,
    /// Location
    Location,
    /// Contact
    Contact,
    /// File
    File,
}

/// Shareable content
#[derive(Debug, Clone)]
pub enum ShareContent {
    /// Text content
    Text(String),
    /// URL
    URL { url: String, title: Option<String> },
    /// Image
    Image { path: String, caption: Option<String> },
    /// Video
    Video { path: String, caption: Option<String> },
    /// Location
    Location { lat: f64, lon: f64, name: Option<String> },
    /// Contact
    Contact { id: String, name: String },
    /// File
    File { path: String, filename: String, mime_type: String },
}

impl ShareContent {
    /// Get content type
    pub fn content_type(&self) -> ShareContentType {
        match self {
            Self::Text(_) => ShareContentType::Text,
            Self::URL { .. } => ShareContentType::URL,
            Self::Image { .. } => ShareContentType::Image,
            Self::Video { .. } => ShareContentType::Video,
            Self::Location { .. } => ShareContentType::Location,
            Self::Contact { .. } => ShareContentType::Contact,
            Self::File { .. } => ShareContentType::File,
        }
    }
    
    /// Get preview text
    pub fn preview(&self) -> String {
        match self {
            Self::Text(s) => {
                if s.len() > 50 {
                    format!("{}...", &s[..50])
                } else {
                    s.clone()
                }
            }
            Self::URL { url, title } => title.clone().unwrap_or_else(|| url.clone()),
            Self::Image { caption, .. } => caption.clone().unwrap_or_else(|| "Image".to_string()),
            Self::Video { caption, .. } => caption.clone().unwrap_or_else(|| "Video".to_string()),
            Self::Location { name, .. } => name.clone().unwrap_or_else(|| "Location".to_string()),
            Self::Contact { name, .. } => format!("Contact: {}", name),
            Self::File { filename, .. } => filename.clone(),
        }
    }
}

/// Share target type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShareTargetType {
    /// Contact
    Contact,
    /// Group
    Group,
    /// App
    App,
    /// Social media
    Social,
    /// Nearby device
    Nearby,
    /// Clipboard
    Clipboard,
    /// Save to device
    Save,
}

/// Share target
#[derive(Debug, Clone)]
pub struct ShareTarget {
    /// Target ID
    pub id: String,
    /// Target name
    pub name: String,
    /// Target type
    pub target_type: ShareTargetType,
    /// Icon/photo URL
    pub icon: Option<String>,
    /// Supported content types
    pub supported_types: Vec<ShareContentType>,
}

impl ShareTarget {
    /// Create new share target
    pub fn new(id: String, name: String, target_type: ShareTargetType) -> Self {
        Self {
            id,
            name,
            target_type,
            icon: None,
            supported_types: vec![
                ShareContentType::Text,
                ShareContentType::URL,
                ShareContentType::Image,
            ],
        }
    }
    
    /// With icon
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }
    
    /// With supported types
    pub fn with_supported_types(mut self, types: Vec<ShareContentType>) -> Self {
        self.supported_types = types;
        self
    }
    
    /// Can handle content type
    pub fn can_handle(&self, content_type: &ShareContentType) -> bool {
        self.supported_types.contains(content_type)
    }
}

/// Share status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareStatus {
    /// Pending
    Pending,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

/// Share operation
#[derive(Debug, Clone)]
pub struct ShareOperation {
    /// Operation ID
    pub id: String,
    /// Content being shared
    pub content: ShareContent,
    /// Target
    pub target: ShareTarget,
    /// Status
    pub status: ShareStatus,
    /// Started at
    pub started: Instant,
    /// Progress (0-100)
    pub progress: u8,
    /// Error message
    pub error: Option<String>,
}

impl ShareOperation {
    /// Create new share operation
    pub fn new(id: String, content: ShareContent, target: ShareTarget) -> Self {
        Self {
            id,
            content,
            target,
            status: ShareStatus::Pending,
            started: Instant::now(),
            progress: 0,
            error: None,
        }
    }
    
    /// Start operation
    pub fn start(&mut self) {
        self.status = ShareStatus::InProgress;
    }
    
    /// Update progress
    pub fn update_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }
    
    /// Complete operation
    pub fn complete(&mut self) {
        self.status = ShareStatus::Completed;
        self.progress = 100;
    }
    
    /// Fail operation
    pub fn fail(&mut self, error: String) {
        self.status = ShareStatus::Failed;
        self.error = Some(error);
    }
    
    /// Cancel operation
    pub fn cancel(&mut self) {
        self.status = ShareStatus::Cancelled;
    }
}

/// Share manager
#[derive(Debug)]
pub struct ShareManager {
    /// Available targets
    targets: HashMap<String, ShareTarget>,
    /// Recent targets
    recent_targets: Vec<String>,
    /// Max recent targets
    max_recent: usize,
    /// Active operations
    active_operations: HashMap<String, ShareOperation>,
    /// Share history
    history: Vec<ShareOperation>,
    /// Max history
    max_history: usize,
    /// Last operation ID
    last_operation_id: u64,
}

impl ShareManager {
    /// Create new share manager
    pub fn new() -> Self {
        let mut manager = Self {
            targets: HashMap::new(),
            recent_targets: Vec::new(),
            max_recent: 10,
            active_operations: HashMap::new(),
            history: Vec::new(),
            max_history: 50,
            last_operation_id: 0,
        };
        
        // Add built-in targets
        manager.add_target(ShareTarget::new(
            "clipboard".to_string(),
            "Copy to Clipboard".to_string(),
            ShareTargetType::Clipboard,
        ).with_supported_types(vec![ShareContentType::Text, ShareContentType::URL]));
        
        manager.add_target(ShareTarget::new(
            "save".to_string(),
            "Save to Device".to_string(),
            ShareTargetType::Save,
        ).with_supported_types(vec![
            ShareContentType::Image,
            ShareContentType::Video,
            ShareContentType::File,
        ]));
        
        manager
    }
    
    /// Add share target
    pub fn add_target(&mut self, target: ShareTarget) {
        self.targets.insert(target.id.clone(), target);
    }
    
    /// Remove target
    pub fn remove_target(&mut self, id: &str) {
        self.targets.remove(id);
        self.recent_targets.retain(|t| t != id);
    }
    
    /// Get target by ID
    pub fn get_target(&self, id: &str) -> Option<&ShareTarget> {
        self.targets.get(id)
    }
    
    /// Get all targets
    pub fn all_targets(&self) -> Vec<&ShareTarget> {
        self.targets.values().collect()
    }
    
    /// Get targets that can handle content type
    pub fn targets_for_content(&self, content: &ShareContent) -> Vec<&ShareTarget> {
        let content_type = content.content_type();
        self.targets.values()
            .filter(|t| t.can_handle(&content_type))
            .collect()
    }
    
    /// Get recent targets
    pub fn recent_targets(&self) -> Vec<&ShareTarget> {
        self.recent_targets.iter()
            .filter_map(|id| self.targets.get(id))
            .collect()
    }
    
    /// Share content to target
    pub fn share(&mut self, content: ShareContent, target_id: &str) -> Option<String> {
        let target = self.targets.get(target_id)?;
        
        if !target.can_handle(&content.content_type()) {
            return None;
        }
        
        self.last_operation_id += 1;
        let op_id = format!("share-{}", self.last_operation_id);
        
        let mut operation = ShareOperation::new(
            op_id.clone(),
            content,
            target.clone(),
        );
        operation.start();
        
        self.active_operations.insert(op_id.clone(), operation);
        
        // Update recent targets
        self.recent_targets.retain(|t| t != target_id);
        if self.recent_targets.len() >= self.max_recent {
            self.recent_targets.remove(0);
        }
        self.recent_targets.push(target_id.to_string());
        
        Some(op_id)
    }
    
    /// Get active operation
    pub fn get_operation(&self, id: &str) -> Option<&ShareOperation> {
        self.active_operations.get(id)
    }
    
    /// Update operation progress
    pub fn update_operation(&mut self, id: &str, progress: u8) {
        if let Some(op) = self.active_operations.get_mut(id) {
            op.update_progress(progress);
        }
    }
    
    /// Complete operation
    pub fn complete_operation(&mut self, id: &str) {
        if let Some(mut op) = self.active_operations.remove(id) {
            op.complete();
            self.add_to_history(op);
        }
    }
    
    /// Fail operation
    pub fn fail_operation(&mut self, id: &str, error: String) {
        if let Some(mut op) = self.active_operations.remove(id) {
            op.fail(error);
            self.add_to_history(op);
        }
    }
    
    /// Cancel operation
    pub fn cancel_operation(&mut self, id: &str) {
        if let Some(mut op) = self.active_operations.remove(id) {
            op.cancel();
            self.add_to_history(op);
        }
    }
    
    /// Add to history
    fn add_to_history(&mut self, op: ShareOperation) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(op);
    }
    
    /// Get share history
    pub fn history(&self) -> &[ShareOperation] {
        &self.history
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /// Has active operations
    pub fn has_active_operations(&self) -> bool {
        !self.active_operations.is_empty()
    }
    
    /// Get active operation count
    pub fn active_count(&self) -> usize {
        self.active_operations.len()
    }
}

impl Default for ShareManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_share_content_type() {
        let content = ShareContent::Text("Hello".to_string());
        assert_eq!(content.content_type(), ShareContentType::Text);
        
        let url = ShareContent::URL {
            url: "https://example.com".to_string(),
            title: None,
        };
        assert_eq!(url.content_type(), ShareContentType::URL);
    }
    
    #[test]
    fn test_share_content_preview() {
        let content = ShareContent::Text("Hello world".to_string());
        assert_eq!(content.preview(), "Hello world");
        
        let contact = ShareContent::Contact {
            id: "1".to_string(),
            name: "John".to_string(),
        };
        assert!(contact.preview().contains("John"));
    }
    
    #[test]
    fn test_share_target() {
        let target = ShareTarget::new(
            "1".to_string(),
            "Alice".to_string(),
            ShareTargetType::Contact,
        );
        
        assert!(target.can_handle(&ShareContentType::Text));
    }
    
    #[test]
    fn test_share_manager_creation() {
        let manager = ShareManager::new();
        
        // Should have built-in targets
        assert!(manager.get_target("clipboard").is_some());
        assert!(manager.get_target("save").is_some());
    }
    
    #[test]
    fn test_add_target() {
        let mut manager = ShareManager::new();
        
        let target = ShareTarget::new(
            "alice".to_string(),
            "Alice".to_string(),
            ShareTargetType::Contact,
        );
        manager.add_target(target);
        
        assert!(manager.get_target("alice").is_some());
    }
    
    #[test]
    fn test_share_operation() {
        let mut manager = ShareManager::new();
        
        let content = ShareContent::Text("Hello".to_string());
        let op_id = manager.share(content, "clipboard");
        
        assert!(op_id.is_some());
        
        let op_id = op_id.unwrap();
        assert!(manager.get_operation(&op_id).is_some());
        
        manager.complete_operation(&op_id);
        assert!(manager.get_operation(&op_id).is_none());
        assert_eq!(manager.history().len(), 1);
    }
    
    #[test]
    fn test_targets_for_content() {
        let manager = ShareManager::new();
        
        let text = ShareContent::Text("Hello".to_string());
        let targets = manager.targets_for_content(&text);
        
        assert!(targets.iter().any(|t| t.id == "clipboard"));
    }
    
    #[test]
    fn test_recent_targets() {
        let mut manager = ShareManager::new();
        
        let content = ShareContent::Text("Hello".to_string());
        manager.share(content, "clipboard");
        
        let recent = manager.recent_targets();
        assert_eq!(recent.len(), 1);
    }
}
