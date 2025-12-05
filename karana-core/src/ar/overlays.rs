// AR Overlays for Kāraṇa OS
// Handles 2D overlays, labels, UI panels in AR space

use super::*;
use std::collections::HashMap;

/// Text alignment for labels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

/// AR text label
#[derive(Debug, Clone)]
pub struct TextLabel {
    pub id: ContentId,
    pub text: String,
    pub position: Point3<f32>,
    pub font_size: f32,
    pub color: [f32; 4],
    pub background_color: Option<[f32; 4]>,
    pub align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub max_width: Option<f32>,
    pub billboard: bool,
    pub visibility: Visibility,
    pub layer: ContentLayer,
}

impl TextLabel {
    pub fn new(text: &str, position: Point3<f32>) -> Self {
        Self {
            id: next_content_id(),
            text: text.to_string(),
            position,
            font_size: 0.05,
            color: [1.0, 1.0, 1.0, 1.0],
            background_color: None,
            align: TextAlign::Center,
            vertical_align: VerticalAlign::Middle,
            max_width: None,
            billboard: true,
            visibility: Visibility::Visible,
            layer: ContentLayer::World,
        }
    }
    
    /// Set font size in world units
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size.max(0.001);
        self
    }
    
    /// Set text color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
    
    /// Set background color
    pub fn with_background(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.background_color = Some([r, g, b, a]);
        self
    }
    
    /// Set alignment
    pub fn with_alignment(mut self, horizontal: TextAlign, vertical: VerticalAlign) -> Self {
        self.align = horizontal;
        self.vertical_align = vertical;
        self
    }
    
    /// Set max width for text wrapping
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }
    
    /// Disable billboard (text won't face camera)
    pub fn with_fixed_orientation(mut self) -> Self {
        self.billboard = false;
        self
    }
    
    /// Update text content
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

/// AR UI panel for displaying information
#[derive(Debug, Clone)]
pub struct UIPanel {
    pub id: ContentId,
    pub name: String,
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub size: (f32, f32),
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub corner_radius: f32,
    pub opacity: f32,
    pub elements: Vec<UIElement>,
    pub visibility: Visibility,
    pub layer: ContentLayer,
    pub interactive: bool,
}

impl UIPanel {
    pub fn new(name: &str, position: Point3<f32>, size: (f32, f32)) -> Self {
        Self {
            id: next_content_id(),
            name: name.to_string(),
            position,
            rotation: UnitQuaternion::identity(),
            size,
            background_color: [0.1, 0.1, 0.1, 0.9],
            border_color: [0.3, 0.3, 0.3, 1.0],
            border_width: 0.002,
            corner_radius: 0.01,
            opacity: 1.0,
            elements: Vec::new(),
            visibility: Visibility::Visible,
            layer: ContentLayer::UI,
            interactive: true,
        }
    }
    
    /// Set background color
    pub fn with_background(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.background_color = [r, g, b, a];
        self
    }
    
    /// Set border
    pub fn with_border(mut self, width: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.border_width = width;
        self.border_color = [r, g, b, a];
        self
    }
    
    /// Add UI element
    pub fn add_element(&mut self, element: UIElement) {
        self.elements.push(element);
    }
    
    /// Get element by ID
    pub fn get_element(&self, id: ContentId) -> Option<&UIElement> {
        self.elements.iter().find(|e| e.id() == id)
    }
    
    /// Remove element by ID
    pub fn remove_element(&mut self, id: ContentId) {
        self.elements.retain(|e| e.id() != id);
    }
    
    /// Clear all elements
    pub fn clear_elements(&mut self) {
        self.elements.clear();
    }
    
    /// Look at a target point
    pub fn look_at(&mut self, target: &Point3<f32>) {
        let direction = (target - self.position).normalize();
        if direction.norm() > 0.0 {
            self.rotation = UnitQuaternion::face_towards(&(-direction), &Vector3::y());
        }
    }
}

/// UI element types for panels
#[derive(Debug, Clone)]
pub enum UIElement {
    Text(UIText),
    Button(UIButton),
    Image(UIImage),
    ProgressBar(UIProgressBar),
    Slider(UISlider),
    Divider(UIDivider),
    Spacer(f32),
}

impl UIElement {
    pub fn id(&self) -> ContentId {
        match self {
            UIElement::Text(t) => t.id,
            UIElement::Button(b) => b.id,
            UIElement::Image(i) => i.id,
            UIElement::ProgressBar(p) => p.id,
            UIElement::Slider(s) => s.id,
            UIElement::Divider(d) => d.id,
            UIElement::Spacer(_) => 0,
        }
    }
}

/// UI text element
#[derive(Debug, Clone)]
pub struct UIText {
    pub id: ContentId,
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub align: TextAlign,
}

impl UIText {
    pub fn new(text: &str) -> Self {
        Self {
            id: next_content_id(),
            text: text.to_string(),
            font_size: 0.02,
            color: [1.0, 1.0, 1.0, 1.0],
            align: TextAlign::Left,
        }
    }
}

/// UI button element
#[derive(Debug, Clone)]
pub struct UIButton {
    pub id: ContentId,
    pub label: String,
    pub size: (f32, f32),
    pub color: [f32; 4],
    pub hover_color: [f32; 4],
    pub text_color: [f32; 4],
    pub enabled: bool,
}

impl UIButton {
    pub fn new(label: &str) -> Self {
        Self {
            id: next_content_id(),
            label: label.to_string(),
            size: (0.1, 0.03),
            color: [0.2, 0.4, 0.8, 1.0],
            hover_color: [0.3, 0.5, 0.9, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            enabled: true,
        }
    }
    
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }
}

/// UI image element
#[derive(Debug, Clone)]
pub struct UIImage {
    pub id: ContentId,
    pub texture_id: ContentId,
    pub size: (f32, f32),
    pub tint: [f32; 4],
}

impl UIImage {
    pub fn new(texture_id: ContentId, width: f32, height: f32) -> Self {
        Self {
            id: next_content_id(),
            texture_id,
            size: (width, height),
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// UI progress bar element
#[derive(Debug, Clone)]
pub struct UIProgressBar {
    pub id: ContentId,
    pub progress: f32,
    pub size: (f32, f32),
    pub background_color: [f32; 4],
    pub fill_color: [f32; 4],
}

impl UIProgressBar {
    pub fn new(progress: f32) -> Self {
        Self {
            id: next_content_id(),
            progress: progress.clamp(0.0, 1.0),
            size: (0.15, 0.015),
            background_color: [0.2, 0.2, 0.2, 1.0],
            fill_color: [0.2, 0.7, 0.3, 1.0],
        }
    }
    
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
}

/// UI slider element
#[derive(Debug, Clone)]
pub struct UISlider {
    pub id: ContentId,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub size: (f32, f32),
    pub track_color: [f32; 4],
    pub handle_color: [f32; 4],
}

impl UISlider {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        Self {
            id: next_content_id(),
            value: value.clamp(min, max),
            min,
            max,
            size: (0.15, 0.02),
            track_color: [0.3, 0.3, 0.3, 1.0],
            handle_color: [0.8, 0.8, 0.8, 1.0],
        }
    }
    
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }
    
    pub fn normalized_value(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }
}

/// UI divider element
#[derive(Debug, Clone)]
pub struct UIDivider {
    pub id: ContentId,
    pub color: [f32; 4],
    pub thickness: f32,
}

impl UIDivider {
    pub fn new() -> Self {
        Self {
            id: next_content_id(),
            color: [0.5, 0.5, 0.5, 1.0],
            thickness: 0.001,
        }
    }
}

impl Default for UIDivider {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification toast overlay
#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub id: ContentId,
    pub title: String,
    pub message: String,
    pub icon: Option<ContentId>,
    pub duration_ms: u64,
    pub created_at: std::time::Instant,
    pub position: ToastPosition,
    pub style: ToastStyle,
}

/// Toast position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastPosition {
    TopCenter,
    TopRight,
    BottomCenter,
    BottomRight,
}

/// Toast style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastStyle {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastStyle {
    pub fn color(&self) -> [f32; 4] {
        match self {
            ToastStyle::Info => [0.2, 0.4, 0.8, 0.95],
            ToastStyle::Success => [0.2, 0.7, 0.3, 0.95],
            ToastStyle::Warning => [0.9, 0.7, 0.1, 0.95],
            ToastStyle::Error => [0.8, 0.2, 0.2, 0.95],
        }
    }
}

impl ToastNotification {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            id: next_content_id(),
            title: title.to_string(),
            message: message.to_string(),
            icon: None,
            duration_ms: 3000,
            created_at: std::time::Instant::now(),
            position: ToastPosition::TopCenter,
            style: ToastStyle::Info,
        }
    }
    
    pub fn with_style(mut self, style: ToastStyle) -> Self {
        self.style = style;
        self
    }
    
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
    
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_millis() as u64 >= self.duration_ms
    }
    
    pub fn remaining_time(&self) -> u64 {
        let elapsed = self.created_at.elapsed().as_millis() as u64;
        self.duration_ms.saturating_sub(elapsed)
    }
}

/// Highlight overlay for pointing out objects
#[derive(Debug, Clone)]
pub struct HighlightOverlay {
    pub id: ContentId,
    pub target_position: Point3<f32>,
    pub radius: f32,
    pub color: [f32; 4],
    pub pulse: bool,
    pub pulse_speed: f32,
    pub visibility: Visibility,
}

impl HighlightOverlay {
    pub fn new(position: Point3<f32>, radius: f32) -> Self {
        Self {
            id: next_content_id(),
            target_position: position,
            radius,
            color: [1.0, 0.8, 0.0, 0.5],
            pulse: true,
            pulse_speed: 2.0,
            visibility: Visibility::Visible,
        }
    }
    
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }
    
    /// Calculate current scale based on pulsing
    pub fn current_scale(&self, time: f32) -> f32 {
        if self.pulse {
            1.0 + 0.2 * (time * self.pulse_speed).sin()
        } else {
            1.0
        }
    }
}

/// Distance indicator overlay
#[derive(Debug, Clone)]
pub struct DistanceIndicator {
    pub id: ContentId,
    pub from: Point3<f32>,
    pub to: Point3<f32>,
    pub show_line: bool,
    pub show_label: bool,
    pub color: [f32; 4],
    pub unit: DistanceUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceUnit {
    Meters,
    Feet,
    Auto,
}

impl DistanceIndicator {
    pub fn new(from: Point3<f32>, to: Point3<f32>) -> Self {
        Self {
            id: next_content_id(),
            from,
            to,
            show_line: true,
            show_label: true,
            color: [1.0, 1.0, 1.0, 0.8],
            unit: DistanceUnit::Auto,
        }
    }
    
    /// Calculate distance
    pub fn distance(&self) -> f32 {
        (self.to - self.from).norm()
    }
    
    /// Get formatted distance string
    pub fn formatted_distance(&self) -> String {
        let dist = self.distance();
        match self.unit {
            DistanceUnit::Meters => {
                if dist < 1.0 {
                    format!("{:.0} cm", dist * 100.0)
                } else {
                    format!("{:.1} m", dist)
                }
            }
            DistanceUnit::Feet => {
                format!("{:.1} ft", dist * 3.28084)
            }
            DistanceUnit::Auto => {
                if dist < 1.0 {
                    format!("{:.0} cm", dist * 100.0)
                } else if dist < 100.0 {
                    format!("{:.1} m", dist)
                } else {
                    format!("{:.0} m", dist)
                }
            }
        }
    }
    
    /// Get midpoint for label placement
    pub fn midpoint(&self) -> Point3<f32> {
        Point3::from((self.from.coords + self.to.coords) * 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_label() {
        let label = TextLabel::new("Hello", Point3::new(0.0, 1.0, 0.0))
            .with_font_size(0.1)
            .with_color(1.0, 0.0, 0.0, 1.0);
        
        assert_eq!(label.text, "Hello");
        assert_eq!(label.font_size, 0.1);
        assert!(label.billboard);
    }
    
    #[test]
    fn test_text_label_update() {
        let mut label = TextLabel::new("Initial", Point3::origin());
        label.set_text("Updated");
        
        assert_eq!(label.text, "Updated");
    }
    
    #[test]
    fn test_ui_panel() {
        let panel = UIPanel::new("test", Point3::new(0.0, 1.5, -1.0), (0.3, 0.2));
        
        assert_eq!(panel.name, "test");
        assert_eq!(panel.size, (0.3, 0.2));
        assert!(panel.interactive);
    }
    
    #[test]
    fn test_ui_panel_elements() {
        let mut panel = UIPanel::new("test", Point3::origin(), (0.3, 0.2));
        
        let text = UIText::new("Title");
        let button = UIButton::new("Click");
        
        panel.add_element(UIElement::Text(text.clone()));
        panel.add_element(UIElement::Button(button));
        
        assert_eq!(panel.elements.len(), 2);
        
        panel.remove_element(text.id);
        assert_eq!(panel.elements.len(), 1);
    }
    
    #[test]
    fn test_ui_progress_bar() {
        let mut bar = UIProgressBar::new(0.5);
        
        assert_eq!(bar.progress, 0.5);
        
        bar.set_progress(1.5); // Should clamp
        assert_eq!(bar.progress, 1.0);
        
        bar.set_progress(-0.5); // Should clamp
        assert_eq!(bar.progress, 0.0);
    }
    
    #[test]
    fn test_ui_slider() {
        let mut slider = UISlider::new(0.0, 100.0, 50.0);
        
        assert_eq!(slider.normalized_value(), 0.5);
        
        slider.set_value(75.0);
        assert_eq!(slider.normalized_value(), 0.75);
    }
    
    #[test]
    fn test_toast_notification() {
        let toast = ToastNotification::new("Test", "Message")
            .with_style(ToastStyle::Success)
            .with_duration(5000);
        
        assert_eq!(toast.title, "Test");
        assert_eq!(toast.duration_ms, 5000);
        assert!(!toast.is_expired());
    }
    
    #[test]
    fn test_toast_style_colors() {
        assert_eq!(ToastStyle::Error.color()[0], 0.8);
        assert_eq!(ToastStyle::Success.color()[1], 0.7);
    }
    
    #[test]
    fn test_highlight_overlay() {
        let highlight = HighlightOverlay::new(Point3::new(1.0, 0.0, 0.0), 0.5);
        
        assert!(highlight.pulse);
        
        let scale_at_0 = highlight.current_scale(0.0);
        let scale_at_quarter = highlight.current_scale(std::f32::consts::PI / (2.0 * highlight.pulse_speed));
        
        assert!(scale_at_quarter > scale_at_0);
    }
    
    #[test]
    fn test_distance_indicator() {
        let indicator = DistanceIndicator::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 4.0, 0.0),
        );
        
        assert!((indicator.distance() - 5.0).abs() < 0.001);
        assert!(indicator.formatted_distance().contains("5.0 m"));
    }
    
    #[test]
    fn test_distance_indicator_small() {
        let indicator = DistanceIndicator::new(
            Point3::origin(),
            Point3::new(0.5, 0.0, 0.0),
        );
        
        assert!(indicator.formatted_distance().contains("cm"));
    }
    
    #[test]
    fn test_distance_indicator_midpoint() {
        let indicator = DistanceIndicator::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
        );
        
        let mid = indicator.midpoint();
        assert!((mid.x - 5.0).abs() < 0.001);
    }
}
