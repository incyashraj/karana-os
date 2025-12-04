//! Notification Display System for Kāraṇa OS AR Glasses
//!
//! Controls how notifications are rendered in AR space.

use nalgebra::Vector3;
use std::time::Duration;

/// Display position for notifications
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayPosition {
    /// Top left of field of view
    TopLeft,
    /// Top center
    TopCenter,
    /// Top right
    TopRight,
    /// Center left (peripheral)
    CenterLeft,
    /// Center (interrupting)
    Center,
    /// Center right (peripheral)
    CenterRight,
    /// Bottom left
    BottomLeft,
    /// Bottom center
    BottomCenter,
    /// Bottom right
    BottomRight,
    /// World-anchored position
    WorldAnchored(Vector3<f32>),
    /// Follow gaze with offset
    GazeFollowing,
}

impl DisplayPosition {
    /// Get position offset in normalized coordinates (-1 to 1)
    pub fn normalized_offset(&self) -> (f32, f32) {
        match self {
            DisplayPosition::TopLeft => (-0.8, 0.8),
            DisplayPosition::TopCenter => (0.0, 0.8),
            DisplayPosition::TopRight => (0.8, 0.8),
            DisplayPosition::CenterLeft => (-0.9, 0.0),
            DisplayPosition::Center => (0.0, 0.0),
            DisplayPosition::CenterRight => (0.9, 0.0),
            DisplayPosition::BottomLeft => (-0.8, -0.8),
            DisplayPosition::BottomCenter => (0.0, -0.8),
            DisplayPosition::BottomRight => (0.8, -0.8),
            DisplayPosition::WorldAnchored(_) => (0.0, 0.0),
            DisplayPosition::GazeFollowing => (0.0, 0.0),
        }
    }
    
    /// Is this position in peripheral vision?
    pub fn is_peripheral(&self) -> bool {
        matches!(
            self,
            DisplayPosition::CenterLeft | DisplayPosition::CenterRight |
            DisplayPosition::TopLeft | DisplayPosition::TopRight |
            DisplayPosition::BottomLeft | DisplayPosition::BottomRight
        )
    }
}

/// Display style for notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    /// Compact single line
    Compact,
    /// Standard notification card
    Standard,
    /// Expanded with full content
    Expanded,
    /// Minimal icon-only indicator
    IconOnly,
    /// Banner across top
    Banner,
    /// Full screen takeover
    FullScreen,
    /// Floating bubble
    Bubble,
    /// Heads-up display style
    HeadsUp,
}

impl DisplayStyle {
    /// Max lines of body text
    pub fn max_body_lines(&self) -> usize {
        match self {
            DisplayStyle::Compact => 1,
            DisplayStyle::Standard => 3,
            DisplayStyle::Expanded => 10,
            DisplayStyle::IconOnly => 0,
            DisplayStyle::Banner => 1,
            DisplayStyle::FullScreen => 20,
            DisplayStyle::Bubble => 2,
            DisplayStyle::HeadsUp => 2,
        }
    }
    
    /// Shows title?
    pub fn shows_title(&self) -> bool {
        !matches!(self, DisplayStyle::IconOnly)
    }
    
    /// Shows actions?
    pub fn shows_actions(&self) -> bool {
        matches!(
            self,
            DisplayStyle::Standard | DisplayStyle::Expanded | 
            DisplayStyle::FullScreen | DisplayStyle::HeadsUp
        )
    }
}

/// Animation type for notification appearance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationAnimation {
    /// No animation
    None,
    /// Fade in/out
    Fade,
    /// Slide from side
    Slide,
    /// Scale from zero
    Scale,
    /// Bounce in
    Bounce,
    /// Materialize (AR-specific)
    Materialize,
}

/// Notification display settings
#[derive(Debug, Clone)]
pub struct NotificationDisplay {
    /// Default position
    default_position: DisplayPosition,
    /// Default style
    default_style: DisplayStyle,
    /// Animation type
    animation: NotificationAnimation,
    /// Animation duration
    animation_duration: Duration,
    /// Max simultaneous notifications
    max_visible: usize,
    /// Stack offset between notifications
    stack_offset: f32,
    /// Opacity (0-1)
    opacity: f32,
    /// Scale factor
    scale: f32,
    /// Distance from user (meters)
    distance: f32,
    /// Enable depth occlusion
    depth_occlusion: bool,
    /// Group similar notifications
    group_similar: bool,
    /// Position overrides by category
    position_overrides: Vec<(super::NotificationCategory, DisplayPosition)>,
    /// Style overrides by priority
    style_overrides: Vec<(super::NotificationPriority, DisplayStyle)>,
}

impl NotificationDisplay {
    /// Create new notification display
    pub fn new() -> Self {
        Self {
            default_position: DisplayPosition::TopRight,
            default_style: DisplayStyle::Standard,
            animation: NotificationAnimation::Slide,
            animation_duration: Duration::from_millis(300),
            max_visible: 3,
            stack_offset: 0.15,
            opacity: 0.95,
            scale: 1.0,
            distance: 2.0,
            depth_occlusion: true,
            group_similar: true,
            position_overrides: Vec::new(),
            style_overrides: Vec::new(),
        }
    }
    
    /// Set default position
    pub fn set_default_position(&mut self, position: DisplayPosition) {
        self.default_position = position;
    }
    
    /// Get default position
    pub fn default_position(&self) -> DisplayPosition {
        self.default_position
    }
    
    /// Set default style
    pub fn set_default_style(&mut self, style: DisplayStyle) {
        self.default_style = style;
    }
    
    /// Get default style
    pub fn default_style(&self) -> DisplayStyle {
        self.default_style
    }
    
    /// Set animation
    pub fn set_animation(&mut self, animation: NotificationAnimation, duration: Duration) {
        self.animation = animation;
        self.animation_duration = duration;
    }
    
    /// Set max visible
    pub fn set_max_visible(&mut self, max: usize) {
        self.max_visible = max;
    }
    
    /// Get max visible
    pub fn max_visible(&self) -> usize {
        self.max_visible
    }
    
    /// Set opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
    
    /// Get opacity
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    
    /// Set scale
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.clamp(0.5, 2.0);
    }
    
    /// Get scale
    pub fn scale(&self) -> f32 {
        self.scale
    }
    
    /// Set distance
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance.clamp(0.5, 10.0);
    }
    
    /// Get distance
    pub fn distance(&self) -> f32 {
        self.distance
    }
    
    /// Set depth occlusion
    pub fn set_depth_occlusion(&mut self, enabled: bool) {
        self.depth_occlusion = enabled;
    }
    
    /// Add position override for category
    pub fn add_position_override(&mut self, category: super::NotificationCategory, position: DisplayPosition) {
        self.position_overrides.push((category, position));
    }
    
    /// Add style override for priority
    pub fn add_style_override(&mut self, priority: super::NotificationPriority, style: DisplayStyle) {
        self.style_overrides.push((priority, style));
    }
    
    /// Get position for notification
    pub fn get_position(&self, notification: &super::Notification) -> DisplayPosition {
        for (category, position) in &self.position_overrides {
            if notification.category == *category {
                return *position;
            }
        }
        self.default_position
    }
    
    /// Get style for notification
    pub fn get_style(&self, notification: &super::Notification) -> DisplayStyle {
        for (priority, style) in &self.style_overrides {
            if notification.priority == *priority {
                return *style;
            }
        }
        self.default_style
    }
    
    /// Calculate stack position for notification index
    pub fn calculate_stack_position(&self, index: usize) -> Vector3<f32> {
        let base_offset = self.default_position.normalized_offset();
        
        // Convert to 3D position
        let x = base_offset.0 * self.distance * 0.5;
        let y = base_offset.1 * self.distance * 0.3 - (index as f32 * self.stack_offset);
        let z = -self.distance;
        
        Vector3::new(x, y, z)
    }
}

impl Default for NotificationDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// Rendered notification info
#[derive(Debug, Clone)]
pub struct RenderedNotification {
    /// Notification ID
    pub id: u64,
    /// Position in 3D space
    pub position: Vector3<f32>,
    /// Current style
    pub style: DisplayStyle,
    /// Current opacity
    pub opacity: f32,
    /// Scale
    pub scale: f32,
    /// Is animating
    pub is_animating: bool,
    /// Time visible
    pub visible_duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_position() {
        let pos = DisplayPosition::TopRight;
        let offset = pos.normalized_offset();
        assert!(offset.0 > 0.0); // Right
        assert!(offset.1 > 0.0); // Top
    }
    
    #[test]
    fn test_peripheral_detection() {
        assert!(DisplayPosition::CenterLeft.is_peripheral());
        assert!(!DisplayPosition::Center.is_peripheral());
    }
    
    #[test]
    fn test_display_style() {
        assert!(DisplayStyle::Compact.max_body_lines() < DisplayStyle::Expanded.max_body_lines());
        assert!(DisplayStyle::Standard.shows_actions());
        assert!(!DisplayStyle::IconOnly.shows_title());
    }
    
    #[test]
    fn test_notification_display() {
        let display = NotificationDisplay::new();
        assert_eq!(display.max_visible(), 3);
        assert!(display.opacity() > 0.0);
    }
    
    #[test]
    fn test_opacity_clamping() {
        let mut display = NotificationDisplay::new();
        
        display.set_opacity(1.5);
        assert_eq!(display.opacity(), 1.0);
        
        display.set_opacity(-0.5);
        assert_eq!(display.opacity(), 0.0);
    }
    
    #[test]
    fn test_scale_clamping() {
        let mut display = NotificationDisplay::new();
        
        display.set_scale(5.0);
        assert_eq!(display.scale(), 2.0);
        
        display.set_scale(0.1);
        assert_eq!(display.scale(), 0.5);
    }
    
    #[test]
    fn test_stack_position() {
        let display = NotificationDisplay::new();
        
        let pos0 = display.calculate_stack_position(0);
        let pos1 = display.calculate_stack_position(1);
        
        // Second notification should be lower
        assert!(pos1.y < pos0.y);
    }
}
