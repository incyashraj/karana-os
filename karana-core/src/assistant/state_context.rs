// Kāraṇa OS - State Context System
// Tracks visible UI elements and resolves ambiguous voice references

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

/// Represents a UI element visible to the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub id: String,
    pub element_type: UIElementType,
    pub label: String,
    pub position: Position,
    pub is_interactive: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIElementType {
    Button,
    Input,
    Text,
    Image,
    List,
    ListItem,
    Card,
    Window,
    Menu,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub z_index: i32,
}

impl Position {
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn is_left_of(&self, other: &Position) -> bool {
        self.x < other.x
    }

    pub fn is_right_of(&self, other: &Position) -> bool {
        self.x > other.x
    }

    pub fn is_above(&self, other: &Position) -> bool {
        self.y < other.y
    }

    pub fn is_below(&self, other: &Position) -> bool {
        self.y > other.y
    }
}

/// Recent user action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentAction {
    pub action_type: String,
    pub target_id: Option<String>,
    pub description: String,
    pub timestamp: u64,
}

/// State context for voice command resolution
#[derive(Debug, Clone)]
pub struct StateContext {
    /// Currently visible UI elements
    visible_elements: Arc<RwLock<Vec<UIElement>>>,
    
    /// Currently active app/screen
    active_app: Arc<RwLock<Option<String>>>,
    
    /// Recent user actions
    recent_actions: Arc<RwLock<Vec<RecentAction>>>,
    
    /// Last mentioned element (for "that", "it" references)
    last_mentioned: Arc<RwLock<Option<String>>>,
    
    /// User's current focus area
    focus_position: Arc<RwLock<Option<Position>>>,
}

impl StateContext {
    pub fn new() -> Self {
        Self {
            visible_elements: Arc::new(RwLock::new(Vec::new())),
            active_app: Arc::new(RwLock::new(None)),
            recent_actions: Arc::new(RwLock::new(Vec::new())),
            last_mentioned: Arc::new(RwLock::new(None)),
            focus_position: Arc::new(RwLock::new(None)),
        }
    }

    /// Update visible elements
    pub async fn set_visible_elements(&self, elements: Vec<UIElement>) {
        let mut visible = self.visible_elements.write().await;
        *visible = elements;
    }

    /// Add a visible element
    pub async fn add_element(&self, element: UIElement) {
        let mut visible = self.visible_elements.write().await;
        visible.push(element);
    }

    /// Set active app
    pub async fn set_active_app(&self, app: Option<String>) {
        let mut active = self.active_app.write().await;
        *active = app;
    }

    /// Record a user action
    pub async fn record_action(&self, action: RecentAction) {
        let mut actions = self.recent_actions.write().await;
        actions.push(action);
        
        // Keep only last 20 actions
        if actions.len() > 20 {
            actions.remove(0);
        }
    }

    /// Set last mentioned element
    pub async fn set_last_mentioned(&self, element_id: Option<String>) {
        let mut mentioned = self.last_mentioned.write().await;
        *mentioned = element_id;
    }

    /// Set user's focus position
    pub async fn set_focus(&self, position: Option<Position>) {
        let mut focus = self.focus_position.write().await;
        *focus = position;
    }

    /// Resolve ambiguous reference in voice command
    pub async fn resolve_reference(&self, text: &str) -> Option<String> {
        let lower = text.to_lowercase();

        // Direct references
        if lower.contains("that") || lower.contains("it") || lower.contains("this") {
            return self.last_mentioned.read().await.clone();
        }

        // Ordinal references: "first", "second", "third", etc.
        if let Some(index) = self.extract_ordinal(&lower) {
            let elements = self.visible_elements.read().await;
            if index < elements.len() {
                return Some(elements[index].id.clone());
            }
        }

        // Positional references: "the one on the left", "top button", etc.
        if let Some(element_id) = self.resolve_positional(&lower).await {
            return Some(element_id);
        }

        // Label-based references
        if let Some(element_id) = self.resolve_by_label(&lower).await {
            return Some(element_id);
        }

        None
    }

    /// Extract ordinal number from text
    fn extract_ordinal(&self, text: &str) -> Option<usize> {
        if text.contains("first") || text.contains("1st") {
            Some(0)
        } else if text.contains("second") || text.contains("2nd") {
            Some(1)
        } else if text.contains("third") || text.contains("3rd") {
            Some(2)
        } else if text.contains("fourth") || text.contains("4th") {
            Some(3)
        } else if text.contains("fifth") || text.contains("5th") {
            Some(4)
        } else if text.contains("last") {
            // Will be handled separately
            None
        } else {
            None
        }
    }

    /// Resolve positional reference (left, right, top, bottom)
    async fn resolve_positional(&self, text: &str) -> Option<String> {
        let elements = self.visible_elements.read().await;
        
        if elements.is_empty() {
            return None;
        }

        // Get reference position (focus or screen center)
        let reference_pos = if let Some(focus) = self.focus_position.read().await.clone() {
            focus.center()
        } else {
            (0.5, 0.5) // Screen center
        };

        let mut candidates: Vec<&UIElement> = elements.iter().collect();

        // Filter by position
        if text.contains("left") {
            candidates.sort_by(|a, b| {
                a.position.x.partial_cmp(&b.position.x).unwrap()
            });
        } else if text.contains("right") {
            candidates.sort_by(|a, b| {
                b.position.x.partial_cmp(&a.position.x).unwrap()
            });
        } else if text.contains("top") || text.contains("above") {
            candidates.sort_by(|a, b| {
                a.position.y.partial_cmp(&b.position.y).unwrap()
            });
        } else if text.contains("bottom") || text.contains("below") {
            candidates.sort_by(|a, b| {
                b.position.y.partial_cmp(&a.position.y).unwrap()
            });
        }

        // Filter by type if specified
        if text.contains("button") {
            candidates.retain(|e| matches!(e.element_type, UIElementType::Button));
        } else if text.contains("input") || text.contains("field") {
            candidates.retain(|e| matches!(e.element_type, UIElementType::Input));
        }

        candidates.first().map(|e| e.id.clone())
    }

    /// Resolve by label text
    async fn resolve_by_label(&self, text: &str) -> Option<String> {
        let elements = self.visible_elements.read().await;

        // Find element with matching label
        for element in elements.iter() {
            let label_lower = element.label.to_lowercase();
            if text.contains(&label_lower) || label_lower.contains(text) {
                return Some(element.id.clone());
            }
        }

        None
    }

    /// Get element by ID
    pub async fn get_element(&self, id: &str) -> Option<UIElement> {
        let elements = self.visible_elements.read().await;
        elements.iter().find(|e| e.id == id).cloned()
    }

    /// Get all visible elements
    pub async fn get_visible_elements(&self) -> Vec<UIElement> {
        self.visible_elements.read().await.clone()
    }

    /// Get active app
    pub async fn get_active_app(&self) -> Option<String> {
        self.active_app.read().await.clone()
    }

    /// Get recent actions
    pub async fn get_recent_actions(&self, limit: usize) -> Vec<RecentAction> {
        let actions = self.recent_actions.read().await;
        actions.iter().rev().take(limit).cloned().collect()
    }

    /// Parse voice command with context
    pub async fn parse_with_context(&self, command: &str) -> Result<ParsedCommand> {
        let resolved_target = self.resolve_reference(command).await;
        
        Ok(ParsedCommand {
            original: command.to_string(),
            resolved_target,
            context: CommandContext {
                active_app: self.get_active_app().await,
                visible_elements: self.get_visible_elements().await.len(),
                has_focus: self.focus_position.read().await.is_some(),
            },
        })
    }
}

impl Default for StateContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed voice command with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub original: String,
    pub resolved_target: Option<String>,
    pub context: CommandContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    pub active_app: Option<String>,
    pub visible_elements: usize,
    pub has_focus: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_context() {
        let context = StateContext::new();
        
        // Add element
        let element = UIElement {
            id: "btn_1".to_string(),
            element_type: UIElementType::Button,
            label: "Submit".to_string(),
            position: Position {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 40.0,
                z_index: 1,
            },
            is_interactive: true,
            metadata: HashMap::new(),
        };
        
        context.add_element(element).await;
        
        let elements = context.get_visible_elements().await;
        assert_eq!(elements.len(), 1);
    }

    #[tokio::test]
    async fn test_resolve_reference() {
        let context = StateContext::new();
        
        // Set last mentioned
        context.set_last_mentioned(Some("btn_1".to_string())).await;
        
        // Should resolve "that"
        let resolved = context.resolve_reference("click that").await;
        assert_eq!(resolved, Some("btn_1".to_string()));
    }

    #[tokio::test]
    async fn test_ordinal_extraction() {
        let context = StateContext::new();
        
        assert_eq!(context.extract_ordinal("the first button"), Some(0));
        assert_eq!(context.extract_ordinal("click the third item"), Some(2));
        assert_eq!(context.extract_ordinal("the 2nd option"), Some(1));
    }

    #[tokio::test]
    async fn test_positional_reference() {
        let context = StateContext::new();
        
        // Add elements at different positions
        for i in 0..3 {
            let element = UIElement {
                id: format!("btn_{}", i),
                element_type: UIElementType::Button,
                label: format!("Button {}", i),
                position: Position {
                    x: i as f32 * 100.0,
                    y: 100.0,
                    width: 80.0,
                    height: 40.0,
                    z_index: 1,
                },
                is_interactive: true,
                metadata: HashMap::new(),
            };
            context.add_element(element).await;
        }
        
        // Resolve "the left button"
        let resolved = context.resolve_positional("the left button").await;
        assert_eq!(resolved, Some("btn_0".to_string()));
        
        // Resolve "the right button"
        let resolved = context.resolve_positional("the right button").await;
        assert_eq!(resolved, Some("btn_2".to_string()));
    }
}
