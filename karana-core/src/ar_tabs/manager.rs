//! # Tab Manager
//!
//! Manages multiple AR tabs - creation, pinning, focus, lifecycle.
//! Think of it as the "window manager" for spatial AR.

use std::collections::{HashMap, VecDeque};
use crate::spatial::{SpatialAnchor, WorldPosition, AnchorId};
use super::tab::{ARTab, TabId, TabContent, TabSize, TabState, TabStyle, TabMetadata};

/// Manages all AR tabs in the system
#[derive(Debug)]
pub struct TabManager {
    /// All open tabs
    tabs: HashMap<TabId, ARTab>,
    /// Currently focused tab
    focused: Option<TabId>,
    /// Tab focus history (most recent first)
    focus_history: VecDeque<TabId>,
    /// Tabs visible in current FOV
    visible: Vec<TabId>,
    /// Tab layout engine
    layout: TabLayout,
    /// Maximum number of tabs
    max_tabs: usize,
    /// Suspended tabs (saved state, freed memory)
    suspended_tabs: HashMap<TabId, TabSnapshot>,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            focused: None,
            focus_history: VecDeque::with_capacity(100),
            visible: Vec::new(),
            layout: TabLayout::default(),
            max_tabs: 50,
            suspended_tabs: HashMap::new(),
        }
    }
    
    /// Create with custom max tabs
    pub fn with_max_tabs(max_tabs: usize) -> Self {
        Self {
            max_tabs,
            ..Self::new()
        }
    }
    
    /// Pin a new tab at the given anchor
    pub fn pin_tab(
        &mut self,
        content: TabContent,
        size: TabSize,
        anchor: SpatialAnchor,
        location_hint: Option<&str>,
    ) -> Result<TabId, TabError> {
        // Check tab limit
        if self.tabs.len() >= self.max_tabs {
            // Try to suspend oldest unused tab
            if !self.suspend_oldest_unused() {
                return Err(TabError::TooManyTabs(self.max_tabs));
            }
        }
        
        // Create tab
        let mut tab = ARTab::new(content, anchor).with_size(size);
        
        // Set location hint if provided
        if let Some(hint) = location_hint {
            tab.metadata = tab.metadata.with_location(hint);
        }
        
        let id = tab.id;
        self.tabs.insert(id, tab);
        
        // Auto-focus new tab
        self.focus(id);
        
        Ok(id)
    }
    
    /// Get a tab by ID
    pub fn get_tab(&self, id: TabId) -> Option<&ARTab> {
        self.tabs.get(&id)
    }
    
    /// Get a mutable tab by ID
    pub fn get_tab_mut(&mut self, id: TabId) -> Option<&mut ARTab> {
        self.tabs.get_mut(&id)
    }
    
    /// Get the currently focused tab
    pub fn get_focused(&self) -> Option<&ARTab> {
        self.focused.and_then(|id| self.tabs.get(&id))
    }
    
    /// Get the currently focused tab mutably
    pub fn get_focused_mut(&mut self) -> Option<&mut ARTab> {
        if let Some(id) = self.focused {
            self.tabs.get_mut(&id)
        } else {
            None
        }
    }
    
    /// Focus a tab (brings to front, enables interaction)
    pub fn focus(&mut self, id: TabId) -> bool {
        // Check if tab exists
        if !self.tabs.contains_key(&id) {
            return false;
        }
        
        // Unfocus previous tab first
        if let Some(prev_id) = self.focused {
            if prev_id != id {
                if let Some(prev) = self.tabs.get_mut(&prev_id) {
                    prev.set_state(TabState::Background);
                    prev.style = TabStyle::default();
                }
            }
        }
        
        // Now focus the new tab
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.set_state(TabState::Active);
            tab.style = TabStyle::focused();
            self.focused = Some(id);
            
            // Update history
            self.focus_history.retain(|&x| x != id);
            self.focus_history.push_front(id);
            if self.focus_history.len() > 100 {
                self.focus_history.pop_back();
            }
            
            true
        } else {
            false
        }
    }
    
    /// Focus the previously focused tab
    pub fn focus_previous(&mut self) -> bool {
        if self.focus_history.len() > 1 {
            // Skip current focused
            let prev_id = self.focus_history.get(1).copied();
            if let Some(id) = prev_id {
                return self.focus(id);
            }
        }
        false
    }
    
    /// Minimize a tab (shrinks to icon)
    pub fn minimize(&mut self, id: TabId) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.set_state(TabState::Minimized);
            tab.style = TabStyle::minimized();
            
            // If this was focused, focus previous
            if self.focused == Some(id) {
                self.focused = None;
                self.focus_previous();
            }
            
            true
        } else {
            false
        }
    }
    
    /// Restore a minimized tab
    pub fn restore(&mut self, id: TabId) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            if tab.state == TabState::Minimized {
                self.focus(id);
                return true;
            }
        }
        false
    }
    
    /// Close a tab
    pub fn close(&mut self, id: TabId) -> Option<ARTab> {
        let tab = self.tabs.remove(&id)?;
        
        // Update focus if needed
        if self.focused == Some(id) {
            self.focused = None;
            self.focus_previous();
        }
        
        // Remove from history
        self.focus_history.retain(|&x| x != id);
        self.visible.retain(|&x| x != id);
        
        Some(tab)
    }
    
    /// Close all tabs
    pub fn close_all(&mut self) {
        self.tabs.clear();
        self.focused = None;
        self.focus_history.clear();
        self.visible.clear();
        self.suspended_tabs.clear();
    }
    
    /// Get all tabs
    pub fn all_tabs(&self) -> impl Iterator<Item = &ARTab> {
        self.tabs.values()
    }
    
    /// Get all tab IDs
    pub fn all_tab_ids(&self) -> impl Iterator<Item = TabId> + '_ {
        self.tabs.keys().copied()
    }
    
    /// Get number of open tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
    
    /// Get tabs at a specific location
    pub fn get_tabs_at(&self, position: &WorldPosition, radius: f32) -> Vec<&ARTab> {
        self.tabs.values()
            .filter(|tab| tab.position().distance_to(position) <= radius)
            .collect()
    }
    
    /// Get tabs in the current field of view
    pub fn get_visible_tabs(&self, viewer_position: &WorldPosition, fov_direction: &[f32; 3]) -> Vec<&ARTab> {
        // Simple frustum culling - check if tab is in front and within FOV
        self.tabs.values()
            .filter(|tab| {
                let to_tab = [
                    tab.position().local.x - viewer_position.local.x,
                    tab.position().local.y - viewer_position.local.y,
                    tab.position().local.z - viewer_position.local.z,
                ];
                
                // Dot product with view direction
                let dot = to_tab[0] * fov_direction[0] + 
                          to_tab[1] * fov_direction[1] + 
                          to_tab[2] * fov_direction[2];
                
                // Tab is in front if dot > 0 and within reasonable distance
                let dist = (to_tab[0].powi(2) + to_tab[1].powi(2) + to_tab[2].powi(2)).sqrt();
                
                dot > 0.0 && dist < 20.0 && tab.is_visible()
            })
            .collect()
    }
    
    /// Update visible tabs list based on viewer position
    pub fn update_visibility(&mut self, viewer_position: &WorldPosition, fov_direction: &[f32; 3]) {
        self.visible = self.tabs.iter()
            .filter_map(|(id, tab)| {
                let to_tab = [
                    tab.position().local.x - viewer_position.local.x,
                    tab.position().local.y - viewer_position.local.y,
                    tab.position().local.z - viewer_position.local.z,
                ];
                
                let dot = to_tab[0] * fov_direction[0] + 
                          to_tab[1] * fov_direction[1] + 
                          to_tab[2] * fov_direction[2];
                
                let dist = (to_tab[0].powi(2) + to_tab[1].powi(2) + to_tab[2].powi(2)).sqrt();
                
                if dot > 0.0 && dist < 20.0 && tab.is_visible() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
    }
    
    /// Get tabs by tag
    pub fn get_tabs_by_tag(&self, tag: &str) -> Vec<&ARTab> {
        self.tabs.values()
            .filter(|tab| tab.metadata.tags.contains(&tag.to_string()))
            .collect()
    }
    
    /// Get tabs by content type
    pub fn get_browser_tabs(&self) -> Vec<&ARTab> {
        self.tabs.values()
            .filter(|tab| matches!(tab.content, TabContent::Browser(_)))
            .collect()
    }
    
    /// Get tabs by content type
    pub fn get_video_tabs(&self) -> Vec<&ARTab> {
        self.tabs.values()
            .filter(|tab| matches!(tab.content, TabContent::VideoPlayer(_)))
            .collect()
    }
    
    /// Get tabs by content type
    pub fn get_game_tabs(&self) -> Vec<&ARTab> {
        self.tabs.values()
            .filter(|tab| matches!(tab.content, TabContent::Game(_)))
            .collect()
    }
    
    /// Suspend a tab (save state, free memory)
    pub fn suspend(&mut self, id: TabId) -> bool {
        if let Some(tab) = self.tabs.get_mut(&id) {
            // Create snapshot
            let snapshot = TabSnapshot {
                id,
                content_type: tab.content.clone(),
                anchor: tab.anchor.clone(),
                size: tab.size.clone(),
                style: tab.style.clone(),
                metadata: tab.metadata.clone(),
                suspended_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            self.suspended_tabs.insert(id, snapshot);
            tab.set_state(TabState::Suspended);
            
            // If focused, move focus
            if self.focused == Some(id) {
                self.focused = None;
                self.focus_previous();
            }
            
            true
        } else {
            false
        }
    }
    
    /// Restore a suspended tab
    pub fn restore_suspended(&mut self, id: TabId) -> bool {
        if let Some(snapshot) = self.suspended_tabs.remove(&id) {
            if let Some(tab) = self.tabs.get_mut(&id) {
                // Restore from snapshot
                tab.set_state(TabState::Loading);
                // Note: actual content restoration would happen asynchronously
                return true;
            }
        }
        false
    }
    
    /// Suspend oldest unused tab to make room
    fn suspend_oldest_unused(&mut self) -> bool {
        // Find oldest non-focused, visible tab
        let oldest = self.tabs.iter()
            .filter(|(id, tab)| {
                Some(**id) != self.focused && 
                tab.state != TabState::Suspended &&
                tab.state != TabState::Minimized
            })
            .min_by_key(|(_, tab)| tab.metadata.last_accessed)
            .map(|(id, _)| *id);
        
        if let Some(id) = oldest {
            self.suspend(id)
        } else {
            false
        }
    }
    
    /// Update tabs after relocalization (when returning to a location)
    pub fn on_relocalize(&mut self, anchor_updates: &[(AnchorId, SpatialAnchor)]) {
        for (anchor_id, new_anchor) in anchor_updates {
            // Find tabs with this anchor
            for tab in self.tabs.values_mut() {
                if tab.anchor.id == *anchor_id {
                    // Update anchor with new tracking data
                    tab.anchor.position = new_anchor.position.clone();
                    tab.anchor.confidence = new_anchor.confidence;
                    
                    // Update interaction zone
                    let position = tab.anchor.position.clone();
                    tab.interaction_zone = super::tab::InteractionZone::from_size_and_position(
                        &tab.size, 
                        &position
                    );
                }
            }
        }
    }
    
    /// Apply layout to arrange tabs
    pub fn apply_layout(&mut self, mode: LayoutMode, viewer_position: &WorldPosition) {
        self.layout.mode = mode;
        
        let visible: Vec<TabId> = self.visible.clone();
        match mode {
            LayoutMode::Free => {
                // Tabs stay where they are
            }
            LayoutMode::Grid => {
                self.arrange_grid(&visible, viewer_position);
            }
            LayoutMode::Stack => {
                self.arrange_stack(&visible, viewer_position);
            }
            LayoutMode::Carousel => {
                self.arrange_carousel(&visible, viewer_position);
            }
            LayoutMode::Dock => {
                self.arrange_dock(&visible, viewer_position);
            }
        }
    }
    
    fn arrange_grid(&mut self, tab_ids: &[TabId], viewer_pos: &WorldPosition) {
        let cols = 3;
        let spacing = 0.9; // 90cm between centers
        let distance = 2.0; // 2m in front
        
        for (i, &id) in tab_ids.iter().enumerate() {
            if let Some(tab) = self.tabs.get_mut(&id) {
                let row = i / cols;
                let col = i % cols;
                
                let x = (col as f32 - 1.0) * spacing + viewer_pos.local.x;
                let y = (1.0 - row as f32) * spacing * 0.6 + viewer_pos.local.y;
                let z = viewer_pos.local.z + distance;
                
                tab.anchor.position.local.x = x;
                tab.anchor.position.local.y = y;
                tab.anchor.position.local.z = z;
            }
        }
    }
    
    fn arrange_stack(&mut self, tab_ids: &[TabId], viewer_pos: &WorldPosition) {
        let offset = 0.02; // 2cm depth offset
        let distance = 1.5;
        
        for (i, &id) in tab_ids.iter().enumerate() {
            if let Some(tab) = self.tabs.get_mut(&id) {
                tab.anchor.position.local.x = viewer_pos.local.x;
                tab.anchor.position.local.y = viewer_pos.local.y;
                tab.anchor.position.local.z = viewer_pos.local.z + distance + (i as f32 * offset);
            }
        }
    }
    
    fn arrange_carousel(&mut self, tab_ids: &[TabId], viewer_pos: &WorldPosition) {
        let radius = 2.5; // 2.5m radius
        let angle_step = std::f32::consts::PI * 2.0 / tab_ids.len().max(1) as f32;
        
        for (i, &id) in tab_ids.iter().enumerate() {
            if let Some(tab) = self.tabs.get_mut(&id) {
                let angle = angle_step * i as f32 - std::f32::consts::PI / 2.0;
                
                tab.anchor.position.local.x = viewer_pos.local.x + angle.sin() * radius;
                tab.anchor.position.local.y = viewer_pos.local.y;
                tab.anchor.position.local.z = viewer_pos.local.z + angle.cos() * radius;
            }
        }
    }
    
    fn arrange_dock(&mut self, tab_ids: &[TabId], viewer_pos: &WorldPosition) {
        // Dock at bottom of FOV
        let dock_y = viewer_pos.local.y - 0.5;
        let dock_z = viewer_pos.local.z + 1.5;
        let spacing = 0.15; // 15cm between minimized icons
        
        let start_x = viewer_pos.local.x - (tab_ids.len() as f32 * spacing / 2.0);
        
        for (i, &id) in tab_ids.iter().enumerate() {
            if let Some(tab) = self.tabs.get_mut(&id) {
                // Minimize for dock
                tab.size = TabSize::small();
                tab.anchor.position.local.x = start_x + (i as f32 * spacing);
                tab.anchor.position.local.y = dock_y;
                tab.anchor.position.local.z = dock_z;
            }
        }
    }
    
    /// Get layout info
    pub fn layout(&self) -> &TabLayout {
        &self.layout
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab layout configuration
#[derive(Debug, Clone)]
pub struct TabLayout {
    /// Layout mode
    pub mode: LayoutMode,
    /// Grid columns
    pub grid_columns: usize,
    /// Spacing between tabs (meters)
    pub spacing: f32,
    /// Default distance from viewer
    pub default_distance: f32,
}

impl Default for TabLayout {
    fn default() -> Self {
        Self {
            mode: LayoutMode::Free,
            grid_columns: 3,
            spacing: 0.9,
            default_distance: 2.0,
        }
    }
}

/// Layout modes for tab arrangement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    /// Tabs stay where pinned
    Free,
    /// Arrange in a grid
    Grid,
    /// Stack tabs with slight offset
    Stack,
    /// Arrange in a carousel around user
    Carousel,
    /// Minimize to dock at bottom
    Dock,
}

/// Snapshot of a suspended tab
#[derive(Debug, Clone)]
pub struct TabSnapshot {
    /// Tab ID
    pub id: TabId,
    /// Content type (not full content)
    pub content_type: TabContent,
    /// Spatial anchor
    pub anchor: SpatialAnchor,
    /// Tab size
    pub size: TabSize,
    /// Visual style
    pub style: TabStyle,
    /// Metadata
    pub metadata: TabMetadata,
    /// When suspended
    pub suspended_at: u64,
}

/// Tab manager errors
#[derive(Debug, Clone, PartialEq)]
pub enum TabError {
    /// Too many tabs open
    TooManyTabs(usize),
    /// Tab not found
    NotFound(TabId),
    /// Invalid tab state for operation
    InvalidState(String),
    /// Anchor error
    AnchorError(String),
}

impl std::fmt::Display for TabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabError::TooManyTabs(max) => write!(f, "Too many tabs open (max: {})", max),
            TabError::NotFound(id) => write!(f, "Tab not found: {}", id),
            TabError::InvalidState(msg) => write!(f, "Invalid tab state: {}", msg),
            TabError::AnchorError(msg) => write!(f, "Anchor error: {}", msg),
        }
    }
}

impl std::error::Error for TabError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::{WorldPosition, SpatialAnchor, AnchorContent, AnchorState, Quaternion};

    fn create_test_anchor(x: f32, y: f32, z: f32) -> SpatialAnchor {
        SpatialAnchor {
            id: 1,
            position: WorldPosition::from_local(x, y, z),
            orientation: Quaternion::identity(),
            visual_signature: [0u8; 32],
            content_hash: [0u8; 32],
            content: AnchorContent::Text { text: "test".to_string() },
            state: AnchorState::Active,
            confidence: 1.0,
            created_at: 0,
            updated_at: 0,
            owner_did: None,
            label: None,
        }
    }

    #[test]
    fn test_manager_creation() {
        let manager = TabManager::new();
        assert_eq!(manager.tab_count(), 0);
        assert!(manager.get_focused().is_none());
    }

    #[test]
    fn test_pin_tab() {
        let mut manager = TabManager::new();
        let anchor = create_test_anchor(0.0, 1.5, 2.0);
        let content = TabContent::browser("https://test.com");
        
        let result = manager.pin_tab(content, TabSize::default(), anchor, Some("desk"));
        assert!(result.is_ok());
        
        let id = result.unwrap();
        assert_eq!(manager.tab_count(), 1);
        
        let tab = manager.get_tab(id).unwrap();
        assert_eq!(tab.metadata.location_hint, "desk");
        assert!(manager.get_focused().is_some());
    }

    #[test]
    fn test_focus_management() {
        let mut manager = TabManager::new();
        
        // Create two tabs
        let anchor1 = create_test_anchor(0.0, 1.5, 2.0);
        let anchor2 = create_test_anchor(1.0, 1.5, 2.0);
        
        let id1 = manager.pin_tab(
            TabContent::browser("https://test1.com"), 
            TabSize::default(), 
            anchor1, 
            None
        ).unwrap();
        
        let id2 = manager.pin_tab(
            TabContent::browser("https://test2.com"), 
            TabSize::default(), 
            anchor2, 
            None
        ).unwrap();
        
        // Second tab should be focused
        assert_eq!(manager.focused, Some(id2));
        
        // Focus first tab
        manager.focus(id1);
        assert_eq!(manager.focused, Some(id1));
        
        // Tab 1 should be active, tab 2 background
        assert_eq!(manager.get_tab(id1).unwrap().state, TabState::Active);
        assert_eq!(manager.get_tab(id2).unwrap().state, TabState::Background);
    }

    #[test]
    fn test_focus_previous() {
        let mut manager = TabManager::new();
        
        let anchor1 = create_test_anchor(0.0, 1.5, 2.0);
        let anchor2 = create_test_anchor(1.0, 1.5, 2.0);
        
        let id1 = manager.pin_tab(
            TabContent::browser("https://test1.com"), 
            TabSize::default(), 
            anchor1, 
            None
        ).unwrap();
        
        let id2 = manager.pin_tab(
            TabContent::browser("https://test2.com"), 
            TabSize::default(), 
            anchor2, 
            None
        ).unwrap();
        
        // Focus first tab
        manager.focus(id1);
        assert_eq!(manager.focused, Some(id1));
        
        // Focus previous should go back to id2
        manager.focus_previous();
        assert_eq!(manager.focused, Some(id2));
    }

    #[test]
    fn test_minimize_restore() {
        let mut manager = TabManager::new();
        let anchor = create_test_anchor(0.0, 1.5, 2.0);
        
        let id = manager.pin_tab(
            TabContent::browser("https://test.com"), 
            TabSize::default(), 
            anchor, 
            None
        ).unwrap();
        
        // Minimize
        assert!(manager.minimize(id));
        assert_eq!(manager.get_tab(id).unwrap().state, TabState::Minimized);
        assert!(manager.focused.is_none());
        
        // Restore
        assert!(manager.restore(id));
        assert_eq!(manager.get_tab(id).unwrap().state, TabState::Active);
        assert_eq!(manager.focused, Some(id));
    }

    #[test]
    fn test_close_tab() {
        let mut manager = TabManager::new();
        let anchor = create_test_anchor(0.0, 1.5, 2.0);
        
        let id = manager.pin_tab(
            TabContent::browser("https://test.com"), 
            TabSize::default(), 
            anchor, 
            None
        ).unwrap();
        
        let closed = manager.close(id);
        assert!(closed.is_some());
        assert_eq!(manager.tab_count(), 0);
        assert!(manager.get_tab(id).is_none());
    }

    #[test]
    fn test_max_tabs_limit() {
        let mut manager = TabManager::with_max_tabs(2);
        
        let anchor1 = create_test_anchor(0.0, 1.5, 2.0);
        let anchor2 = create_test_anchor(1.0, 1.5, 2.0);
        let anchor3 = create_test_anchor(2.0, 1.5, 2.0);
        
        manager.pin_tab(TabContent::browser("https://1.com"), TabSize::default(), anchor1, None).unwrap();
        manager.pin_tab(TabContent::browser("https://2.com"), TabSize::default(), anchor2, None).unwrap();
        
        // Third should trigger suspension of oldest
        let result = manager.pin_tab(TabContent::browser("https://3.com"), TabSize::default(), anchor3, None);
        assert!(result.is_ok());
        
        // One tab should be suspended
        assert!(manager.tabs.values().any(|t| t.state == TabState::Suspended));
    }

    #[test]
    fn test_get_tabs_at_location() {
        let mut manager = TabManager::new();
        
        let anchor1 = create_test_anchor(0.0, 1.5, 2.0);
        let anchor2 = create_test_anchor(5.0, 1.5, 2.0);
        
        manager.pin_tab(TabContent::browser("https://1.com"), TabSize::default(), anchor1, None).unwrap();
        manager.pin_tab(TabContent::browser("https://2.com"), TabSize::default(), anchor2, None).unwrap();
        
        let pos = WorldPosition::from_local(0.0, 1.5, 2.0);
        let nearby = manager.get_tabs_at(&pos, 1.0);
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn test_get_visible_tabs() {
        let mut manager = TabManager::new();
        
        // Tab in front
        let anchor1 = create_test_anchor(0.0, 1.5, 5.0);
        // Tab behind
        let anchor2 = create_test_anchor(0.0, 1.5, -5.0);
        
        manager.pin_tab(TabContent::browser("https://front.com"), TabSize::default(), anchor1, None).unwrap();
        manager.pin_tab(TabContent::browser("https://back.com"), TabSize::default(), anchor2, None).unwrap();
        
        let viewer_pos = WorldPosition::from_local(0.0, 1.5, 0.0);
        let view_dir = [0.0, 0.0, 1.0]; // Looking forward
        
        let visible = manager.get_visible_tabs(&viewer_pos, &view_dir);
        assert_eq!(visible.len(), 1); // Only front tab visible
    }

    #[test]
    fn test_get_tabs_by_type() {
        let mut manager = TabManager::new();
        
        let anchor1 = create_test_anchor(0.0, 1.5, 2.0);
        let anchor2 = create_test_anchor(1.0, 1.5, 2.0);
        let anchor3 = create_test_anchor(2.0, 1.5, 2.0);
        
        manager.pin_tab(TabContent::browser("https://1.com"), TabSize::default(), anchor1, None).unwrap();
        manager.pin_tab(TabContent::video("https://yt.com", "Video"), TabSize::default(), anchor2, None).unwrap();
        manager.pin_tab(TabContent::browser("https://2.com"), TabSize::default(), anchor3, None).unwrap();
        
        assert_eq!(manager.get_browser_tabs().len(), 2);
        assert_eq!(manager.get_video_tabs().len(), 1);
        assert_eq!(manager.get_game_tabs().len(), 0);
    }

    #[test]
    fn test_suspend_and_restore() {
        let mut manager = TabManager::new();
        let anchor = create_test_anchor(0.0, 1.5, 2.0);
        
        let id = manager.pin_tab(
            TabContent::browser("https://test.com"), 
            TabSize::default(), 
            anchor, 
            None
        ).unwrap();
        
        // Suspend
        assert!(manager.suspend(id));
        assert_eq!(manager.get_tab(id).unwrap().state, TabState::Suspended);
        assert!(manager.suspended_tabs.contains_key(&id));
        
        // Restore
        assert!(manager.restore_suspended(id));
        assert_eq!(manager.get_tab(id).unwrap().state, TabState::Loading);
    }

    #[test]
    fn test_layout_modes() {
        let mut manager = TabManager::new();
        
        for i in 0..6 {
            let anchor = create_test_anchor(0.0, 1.5, 2.0 + i as f32);
            manager.pin_tab(
                TabContent::browser(&format!("https://{}.com", i)), 
                TabSize::default(), 
                anchor, 
                None
            ).unwrap();
        }
        
        // Update visibility first
        let viewer_pos = WorldPosition::from_local(0.0, 1.5, 0.0);
        let view_dir = [0.0, 0.0, 1.0];
        manager.update_visibility(&viewer_pos, &view_dir);
        
        // Test grid layout
        manager.apply_layout(LayoutMode::Grid, &viewer_pos);
        assert_eq!(manager.layout().mode, LayoutMode::Grid);
        
        // Test carousel layout
        manager.apply_layout(LayoutMode::Carousel, &viewer_pos);
        assert_eq!(manager.layout().mode, LayoutMode::Carousel);
    }

    #[test]
    fn test_relocalize_updates_tabs() {
        let mut manager = TabManager::new();
        let anchor = create_test_anchor(0.0, 1.5, 2.0);
        let anchor_id = anchor.id;
        
        manager.pin_tab(
            TabContent::browser("https://test.com"), 
            TabSize::default(), 
            anchor.clone(), 
            None
        ).unwrap();
        
        // Create updated anchor with new position
        let mut updated_anchor = anchor.clone();
        updated_anchor.position.local.x = 1.0;
        updated_anchor.confidence = 0.99;
        
        // Simulate relocalization
        manager.on_relocalize(&[(anchor_id, updated_anchor)]);
        
        // Check tab position was updated
        let tab = manager.all_tabs().next().unwrap();
        assert!((tab.position().local.x - 1.0).abs() < 0.001);
        assert!((tab.anchor.confidence - 0.99).abs() < 0.001);
    }

    #[test]
    fn test_close_all() {
        let mut manager = TabManager::new();
        
        for i in 0..5 {
            let anchor = create_test_anchor(i as f32, 1.5, 2.0);
            manager.pin_tab(
                TabContent::browser(&format!("https://{}.com", i)), 
                TabSize::default(), 
                anchor, 
                None
            ).unwrap();
        }
        
        assert_eq!(manager.tab_count(), 5);
        
        manager.close_all();
        assert_eq!(manager.tab_count(), 0);
        assert!(manager.focused.is_none());
        assert!(manager.focus_history.is_empty());
    }
}
