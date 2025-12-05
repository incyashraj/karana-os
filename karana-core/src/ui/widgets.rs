//! UI Widget System
//! 
//! Composable widget system for building AR interfaces

use super::*;
use std::any::Any;

/// Layout constraints for measuring widgets
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl LayoutConstraints {
    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self { min_width, max_width, min_height, max_height }
    }

    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }

    pub fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }
}

/// Base widget trait
pub trait Widget: Send + Sync {
    /// Get widget ID
    fn id(&self) -> u64;

    /// Get widget type name
    fn type_name(&self) -> &'static str;

    /// Measure widget size given constraints
    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError>;

    /// Set widget position (after layout)
    fn set_position(&mut self, position: Point);

    /// Get widget bounds
    fn bounds(&self) -> Rect;

    /// Update widget state
    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError>;

    /// Handle input event
    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool;

    /// Render widget
    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError>;

    /// Is widget visible
    fn is_visible(&self) -> bool { true }

    /// Is widget enabled
    fn is_enabled(&self) -> bool { true }

    /// Can receive focus
    fn can_focus(&self) -> bool { false }

    /// Get children (for container widgets)
    fn children(&self) -> &[Box<dyn Widget>] { &[] }

    /// Get mutable children
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut [] }

    /// Hit test
    fn hit_test(&self, point: &Point) -> bool {
        self.is_visible() && self.bounds().contains(point)
    }

    /// Get accessibility info
    fn accessibility_info(&self) -> Option<AccessibilityInfo> { None }

    /// As any (for downcasting)
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Widget style properties
#[derive(Debug, Clone, Default)]
pub struct WidgetStyle {
    pub background: Background,
    pub border: Border,
    pub shadow: Option<Shadow>,
    pub opacity: f32,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
}

impl WidgetStyle {
    pub fn new() -> Self {
        Self {
            opacity: 1.0,
            ..Default::default()
        }
    }

    pub fn with_background(mut self, bg: Background) -> Self {
        self.background = bg;
        self
    }

    pub fn with_border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    pub fn with_shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn with_padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }
}

// =============================================================================
// Container Widget
// =============================================================================

/// Container widget for grouping children
pub struct Container {
    id: u64,
    bounds: Rect,
    style: WidgetStyle,
    child: Option<Box<dyn Widget>>,
    visible: bool,
    enabled: bool,
}

impl Container {
    pub fn new() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            style: WidgetStyle::new(),
            child: None,
            visible: true,
            enabled: true,
        }
    }

    pub fn with_child(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }

    pub fn with_style(mut self, style: WidgetStyle) -> Self {
        self.style = style;
        self
    }

    pub fn set_child(&mut self, child: Box<dyn Widget>) {
        self.child = Some(child);
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Container {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Container" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let padding = &self.style.padding;
        let child_constraints = LayoutConstraints {
            min_width: (constraints.min_width - padding.horizontal()).max(0.0),
            max_width: (constraints.max_width - padding.horizontal()).max(0.0),
            min_height: (constraints.min_height - padding.vertical()).max(0.0),
            max_height: (constraints.max_height - padding.vertical()).max(0.0),
        };

        let child_size = if let Some(child) = &mut self.child {
            child.layout(&child_constraints)?
        } else {
            Size::zero()
        };

        let size = Size::new(
            child_size.width + padding.horizontal(),
            child_size.height + padding.vertical(),
        );
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
        if let Some(child) = &mut self.child {
            child.set_position(Point::new(
                position.x + self.style.padding.left,
                position.y + self.style.padding.top,
            ));
        }
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        if let Some(child) = &mut self.child {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        if let Some(child) = &mut self.child {
            if child.handle_event(event, ctx) {
                return true;
            }
        }
        false
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        // Render background, border, shadow would go here
        if let Some(child) = &self.child {
            child.render(ctx)?;
        }
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }
    fn is_enabled(&self) -> bool { self.enabled }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Text Widget
// =============================================================================

/// Text display widget
pub struct Text {
    id: u64,
    bounds: Rect,
    content: String,
    font: FontConfig,
    color: Color,
    align: TextAlign,
    overflow: TextOverflow,
    max_lines: Option<usize>,
    visible: bool,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            content: content.into(),
            font: FontConfig::default(),
            color: Color::WHITE,
            align: TextAlign::Left,
            overflow: TextOverflow::Ellipsis,
            max_lines: None,
            visible: true,
        }
    }

    pub fn with_font(mut self, font: FontConfig) -> Self {
        self.font = font;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.font.size = size;
        self
    }

    pub fn with_max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    fn measure_text(&self, max_width: f32) -> Size {
        // Simplified text measurement
        let char_width = self.font.size * 0.6;
        let line_height = self.font.size * self.font.line_height;
        
        let chars_per_line = (max_width / char_width).max(1.0) as usize;
        let total_chars = self.content.len();
        let lines = ((total_chars as f32 / chars_per_line as f32).ceil() as usize).max(1);
        let lines = self.max_lines.map(|m| lines.min(m)).unwrap_or(lines);
        
        Size::new(
            (total_chars.min(chars_per_line) as f32 * char_width).min(max_width),
            lines as f32 * line_height,
        )
    }
}

impl Widget for Text {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Text" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let size = self.measure_text(constraints.max_width);
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }

    fn handle_event(&mut self, _event: &UiEvent, _ctx: &UiContext) -> bool { false }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        // Text rendering would be implemented here
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn accessibility_info(&self) -> Option<AccessibilityInfo> {
        Some(AccessibilityInfo {
            role: AccessibilityRole::Text,
            label: self.content.clone(),
            value: None,
            hint: None,
            focusable: false,
            actions: vec![],
        })
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Button Widget
// =============================================================================

/// Callback type for button press
pub type ButtonCallback = Box<dyn Fn() + Send + Sync>;

/// Interactive button widget
pub struct Button {
    id: u64,
    bounds: Rect,
    label: String,
    style: ButtonStyle,
    state: ButtonState,
    on_press: Option<ButtonCallback>,
    enabled: bool,
    visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub background: Color,
    pub background_hovered: Color,
    pub background_pressed: Color,
    pub background_disabled: Color,
    pub text_color: Color,
    pub text_color_disabled: Color,
    pub border: Border,
    pub padding: EdgeInsets,
    pub min_width: f32,
    pub min_height: f32,
    pub font: FontConfig,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            background: Color::from_hex(0x2196F3),
            background_hovered: Color::from_hex(0x1976D2),
            background_pressed: Color::from_hex(0x0D47A1),
            background_disabled: Color::GRAY,
            text_color: Color::WHITE,
            text_color_disabled: Color::from_hex(0x999999),
            border: Border::rounded(0.0, Color::TRANSPARENT, 8.0),
            padding: EdgeInsets::symmetric(16.0, 12.0),
            min_width: 64.0,
            min_height: 40.0,
            font: FontConfig {
                size: 14.0,
                weight: FontWeight::Medium,
                ..Default::default()
            },
        }
    }
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            label: label.into(),
            style: ButtonStyle::default(),
            state: ButtonState::Normal,
            on_press: None,
            enabled: true,
            visible: true,
        }
    }

    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_on_press<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_press = Some(Box::new(callback));
        self
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.state = if enabled { ButtonState::Normal } else { ButtonState::Disabled };
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    fn current_background(&self) -> Color {
        match self.state {
            ButtonState::Normal => self.style.background,
            ButtonState::Hovered | ButtonState::Focused => self.style.background_hovered,
            ButtonState::Pressed => self.style.background_pressed,
            ButtonState::Disabled => self.style.background_disabled,
        }
    }
}

impl Widget for Button {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Button" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let char_width = self.style.font.size * 0.6;
        let text_width = self.label.len() as f32 * char_width;
        let text_height = self.style.font.size;

        let size = Size::new(
            (text_width + self.style.padding.horizontal()).max(self.style.min_width),
            (text_height + self.style.padding.vertical()).max(self.style.min_height),
        );
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &UiContext) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            UiEvent::PointerDown { position, .. } if self.bounds.contains(position) => {
                self.state = ButtonState::Pressed;
                true
            }
            UiEvent::PointerUp { position, .. } if self.state == ButtonState::Pressed => {
                if self.bounds.contains(position) {
                    if let Some(callback) = &self.on_press {
                        callback();
                    }
                    self.state = ButtonState::Hovered;
                } else {
                    self.state = ButtonState::Normal;
                }
                true
            }
            UiEvent::PointerMove { position, .. } => {
                let new_state = if self.bounds.contains(position) {
                    if self.state == ButtonState::Pressed {
                        ButtonState::Pressed
                    } else {
                        ButtonState::Hovered
                    }
                } else {
                    ButtonState::Normal
                };
                if new_state != self.state {
                    self.state = new_state;
                    return true;
                }
                false
            }
            UiEvent::Gesture(GestureEvent::Tap { position }) if self.bounds.contains(position) => {
                if let Some(callback) = &self.on_press {
                    callback();
                }
                true
            }
            _ => false,
        }
    }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        // Button rendering would be implemented here
        // Would draw: background, border, text
        let _bg = self.current_background();
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }
    fn is_enabled(&self) -> bool { self.enabled }
    fn can_focus(&self) -> bool { self.enabled }

    fn accessibility_info(&self) -> Option<AccessibilityInfo> {
        Some(AccessibilityInfo {
            role: AccessibilityRole::Button,
            label: self.label.clone(),
            value: None,
            hint: Some("Tap to activate".into()),
            focusable: true,
            actions: vec![AccessibilityAction::Press],
        })
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Image Widget
// =============================================================================

/// Image display widget
pub struct Image {
    id: u64,
    bounds: Rect,
    source: ImageSource,
    fit: ImageFit,
    tint: Option<Color>,
    visible: bool,
    aspect_ratio: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    Url(String),
    Asset(String),
    Data(Vec<u8>),
    Placeholder,
}

impl Image {
    pub fn new(source: ImageSource) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            source,
            fit: ImageFit::Contain,
            tint: None,
            visible: true,
            aspect_ratio: None,
        }
    }

    pub fn with_fit(mut self, fit: ImageFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = Some(tint);
        self
    }

    pub fn with_aspect_ratio(mut self, ratio: f32) -> Self {
        self.aspect_ratio = Some(ratio);
        self
    }
}

impl Widget for Image {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Image" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let size = if let Some(ratio) = self.aspect_ratio {
            let width = constraints.max_width.min(constraints.max_height * ratio);
            Size::new(width, width / ratio)
        } else {
            Size::new(constraints.max_width, constraints.max_height)
        };
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }
    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }
    fn handle_event(&mut self, _event: &UiEvent, _ctx: &UiContext) -> bool { false }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        // Image rendering would be implemented here
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn accessibility_info(&self) -> Option<AccessibilityInfo> {
        Some(AccessibilityInfo {
            role: AccessibilityRole::Image,
            label: match &self.source {
                ImageSource::Url(url) => format!("Image: {}", url),
                ImageSource::Asset(name) => format!("Image: {}", name),
                _ => "Image".into(),
            },
            value: None,
            hint: None,
            focusable: false,
            actions: vec![],
        })
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Icon Widget
// =============================================================================

/// Icon from icon set
pub struct Icon {
    id: u64,
    bounds: Rect,
    name: String,
    size: f32,
    color: Color,
    visible: bool,
}

impl Icon {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            name: name.into(),
            size: 24.0,
            color: Color::WHITE,
            visible: true,
        }
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Widget for Icon {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Icon" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let size = Size::new(self.size, self.size);
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }
    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }
    fn handle_event(&mut self, _event: &UiEvent, _ctx: &UiContext) -> bool { false }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        // Icon rendering would be implemented here
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// TextField Widget
// =============================================================================

/// Text input field
pub struct TextField {
    id: u64,
    bounds: Rect,
    value: String,
    placeholder: String,
    style: TextFieldStyle,
    focused: bool,
    cursor_position: usize,
    selection: Option<(usize, usize)>,
    enabled: bool,
    visible: bool,
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct TextFieldStyle {
    pub background: Color,
    pub background_focused: Color,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub border: Border,
    pub border_focused: Border,
    pub padding: EdgeInsets,
    pub font: FontConfig,
}

impl Default for TextFieldStyle {
    fn default() -> Self {
        Self {
            background: Color::from_hex(0x1E1E1E),
            background_focused: Color::from_hex(0x2A2A2A),
            text_color: Color::WHITE,
            placeholder_color: Color::from_hex(0x666666),
            border: Border::rounded(1.0, Color::from_hex(0x444444), 4.0),
            border_focused: Border::rounded(2.0, Color::from_hex(0x2196F3), 4.0),
            padding: EdgeInsets::symmetric(12.0, 8.0),
            font: FontConfig::default(),
        }
    }
}

impl TextField {
    pub fn new() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            value: String::new(),
            placeholder: String::new(),
            style: TextFieldStyle::default(),
            focused: false,
            cursor_position: 0,
            selection: None,
            enabled: true,
            visible: true,
            on_change: None,
            on_submit: None,
        }
    }

    pub fn with_placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self
    }

    pub fn with_on_change<F: Fn(&str) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn with_on_submit<F: Fn(&str) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_submit = Some(Box::new(callback));
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor_position = self.cursor_position.min(self.value.len());
    }

    fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
        if let Some(callback) = &self.on_change {
            callback(&self.value);
        }
    }

    fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
            if let Some(callback) = &self.on_change {
                callback(&self.value);
            }
        }
    }
}

impl Default for TextField {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextField {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "TextField" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let height = self.style.font.size + self.style.padding.vertical();
        let size = Size::new(constraints.max_width, height.max(40.0));
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &UiContext) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            UiEvent::PointerDown { position, .. } if self.bounds.contains(position) => {
                self.focused = true;
                // Calculate cursor position from click
                true
            }
            UiEvent::Character { char } if self.focused => {
                self.insert_char(*char);
                true
            }
            UiEvent::KeyDown { key, .. } if self.focused => {
                match key.as_str() {
                    "Backspace" => {
                        self.delete_char();
                        true
                    }
                    "Enter" => {
                        if let Some(callback) = &self.on_submit {
                            callback(&self.value);
                        }
                        true
                    }
                    "ArrowLeft" if self.cursor_position > 0 => {
                        self.cursor_position -= 1;
                        true
                    }
                    "ArrowRight" if self.cursor_position < self.value.len() => {
                        self.cursor_position += 1;
                        true
                    }
                    _ => false,
                }
            }
            UiEvent::Blur => {
                self.focused = false;
                true
            }
            _ => false,
        }
    }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        // TextField rendering would be implemented here
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }
    fn is_enabled(&self) -> bool { self.enabled }
    fn can_focus(&self) -> bool { self.enabled }

    fn accessibility_info(&self) -> Option<AccessibilityInfo> {
        Some(AccessibilityInfo {
            role: AccessibilityRole::TextField,
            label: if self.value.is_empty() { self.placeholder.clone() } else { "Text input".into() },
            value: Some(self.value.clone()),
            hint: Some("Enter text".into()),
            focusable: true,
            actions: vec![AccessibilityAction::SetValue],
        })
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Switch Widget
// =============================================================================

/// Toggle switch widget
pub struct Switch {
    id: u64,
    bounds: Rect,
    value: bool,
    style: SwitchStyle,
    enabled: bool,
    visible: bool,
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct SwitchStyle {
    pub track_off: Color,
    pub track_on: Color,
    pub thumb_off: Color,
    pub thumb_on: Color,
    pub track_width: f32,
    pub track_height: f32,
    pub thumb_size: f32,
}

impl Default for SwitchStyle {
    fn default() -> Self {
        Self {
            track_off: Color::from_hex(0x555555),
            track_on: Color::from_hex(0x2196F3),
            thumb_off: Color::from_hex(0xCCCCCC),
            thumb_on: Color::WHITE,
            track_width: 48.0,
            track_height: 24.0,
            thumb_size: 20.0,
        }
    }
}

impl Switch {
    pub fn new(value: bool) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            value,
            style: SwitchStyle::default(),
            enabled: true,
            visible: true,
            on_change: None,
        }
    }

    pub fn with_on_change<F: Fn(bool) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn value(&self) -> bool {
        self.value
    }

    pub fn set_value(&mut self, value: bool) {
        self.value = value;
    }

    fn toggle(&mut self) {
        self.value = !self.value;
        if let Some(callback) = &self.on_change {
            callback(self.value);
        }
    }
}

impl Widget for Switch {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Switch" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let size = Size::new(self.style.track_width, self.style.track_height);
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }
    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &UiContext) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            UiEvent::PointerUp { position, .. } if self.bounds.contains(position) => {
                self.toggle();
                true
            }
            UiEvent::Gesture(GestureEvent::Tap { position }) if self.bounds.contains(position) => {
                self.toggle();
                true
            }
            _ => false,
        }
    }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        // Switch rendering would be implemented here
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }
    fn is_enabled(&self) -> bool { self.enabled }
    fn can_focus(&self) -> bool { self.enabled }

    fn accessibility_info(&self) -> Option<AccessibilityInfo> {
        Some(AccessibilityInfo {
            role: AccessibilityRole::Switch,
            label: "Toggle switch".into(),
            value: Some(if self.value { "On" } else { "Off" }.into()),
            hint: Some("Tap to toggle".into()),
            focusable: true,
            actions: vec![AccessibilityAction::Toggle],
        })
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Slider Widget
// =============================================================================

/// Slider for selecting numeric values
pub struct Slider {
    id: u64,
    bounds: Rect,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    style: SliderStyle,
    dragging: bool,
    enabled: bool,
    visible: bool,
    on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct SliderStyle {
    pub track_color: Color,
    pub track_active_color: Color,
    pub thumb_color: Color,
    pub thumb_color_active: Color,
    pub track_height: f32,
    pub thumb_size: f32,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_color: Color::from_hex(0x444444),
            track_active_color: Color::from_hex(0x2196F3),
            thumb_color: Color::WHITE,
            thumb_color_active: Color::from_hex(0xE3F2FD),
            track_height: 4.0,
            thumb_size: 20.0,
        }
    }
}

impl Slider {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            value: value.clamp(min, max),
            min,
            max,
            step: None,
            style: SliderStyle::default(),
            dragging: false,
            enabled: true,
            visible: true,
            on_change: None,
        }
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn with_on_change<F: Fn(f32) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
        if let Some(step) = self.step {
            self.value = (self.value / step).round() * step;
        }
    }

    fn update_from_position(&mut self, x: f32) {
        let track_start = self.bounds.x() + self.style.thumb_size / 2.0;
        let track_width = self.bounds.width() - self.style.thumb_size;
        let ratio = ((x - track_start) / track_width).clamp(0.0, 1.0);
        let new_value = self.min + ratio * (self.max - self.min);
        self.set_value(new_value);
        if let Some(callback) = &self.on_change {
            callback(self.value);
        }
    }

    fn normalized_value(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }
}

impl Widget for Slider {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Slider" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let size = Size::new(
            constraints.max_width.min(200.0),
            self.style.thumb_size.max(self.style.track_height),
        );
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }
    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }

    fn handle_event(&mut self, event: &UiEvent, _ctx: &UiContext) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            UiEvent::PointerDown { position, .. } if self.bounds.contains(position) => {
                self.dragging = true;
                self.update_from_position(position.x);
                true
            }
            UiEvent::PointerMove { position, .. } if self.dragging => {
                self.update_from_position(position.x);
                true
            }
            UiEvent::PointerUp { .. } if self.dragging => {
                self.dragging = false;
                true
            }
            _ => false,
        }
    }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        let _ratio = self.normalized_value();
        // Slider rendering would be implemented here
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }
    fn is_enabled(&self) -> bool { self.enabled }
    fn can_focus(&self) -> bool { self.enabled }

    fn accessibility_info(&self) -> Option<AccessibilityInfo> {
        Some(AccessibilityInfo {
            role: AccessibilityRole::Slider,
            label: "Slider".into(),
            value: Some(format!("{:.1}", self.value)),
            hint: Some(format!("Range: {:.1} to {:.1}", self.min, self.max)),
            focusable: true,
            actions: vec![AccessibilityAction::Increment, AccessibilityAction::Decrement],
        })
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// Progress Widget
// =============================================================================

/// Progress bar/indicator
pub struct Progress {
    id: u64,
    bounds: Rect,
    value: f32,
    style: ProgressStyle,
    indeterminate: bool,
    visible: bool,
}

#[derive(Debug, Clone)]
pub struct ProgressStyle {
    pub track_color: Color,
    pub progress_color: Color,
    pub height: f32,
    pub border_radius: f32,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            track_color: Color::from_hex(0x333333),
            progress_color: Color::from_hex(0x2196F3),
            height: 4.0,
            border_radius: 2.0,
        }
    }
}

impl Progress {
    pub fn new(value: f32) -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            value: value.clamp(0.0, 1.0),
            style: ProgressStyle::default(),
            indeterminate: false,
            visible: true,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            value: 0.0,
            style: ProgressStyle::default(),
            indeterminate: true,
            visible: true,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Widget for Progress {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "Progress" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        let size = Size::new(constraints.max_width, self.style.height);
        self.bounds.size = constraints.constrain(size);
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
    }

    fn bounds(&self) -> Rect { self.bounds }
    fn update(&mut self, _ctx: &UiContext) -> Result<(), UiError> { Ok(()) }
    fn handle_event(&mut self, _event: &UiEvent, _ctx: &UiContext) -> bool { false }

    fn render(&self, _ctx: &mut RenderContext) -> Result<(), UiError> {
        // Progress rendering would be implemented here
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn accessibility_info(&self) -> Option<AccessibilityInfo> {
        Some(AccessibilityInfo {
            role: AccessibilityRole::ProgressBar,
            label: "Progress".into(),
            value: if self.indeterminate { 
                Some("Loading".into()) 
            } else { 
                Some(format!("{}%", (self.value * 100.0) as u32))
            },
            hint: None,
            focusable: false,
            actions: vec![],
        })
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// =============================================================================
// ScrollView Widget
// =============================================================================

/// Scrollable container
pub struct ScrollView {
    id: u64,
    bounds: Rect,
    content_size: Size,
    scroll_offset: Point,
    child: Option<Box<dyn Widget>>,
    horizontal: bool,
    vertical: bool,
    show_scrollbars: bool,
    visible: bool,
    velocity: Point,
    last_drag: Option<Point>,
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            id: generate_widget_id(),
            bounds: Rect::zero(),
            content_size: Size::zero(),
            scroll_offset: Point::zero(),
            child: None,
            horizontal: false,
            vertical: true,
            show_scrollbars: true,
            visible: true,
            velocity: Point::zero(),
            last_drag: None,
        }
    }

    pub fn with_child(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }

    pub fn with_horizontal(mut self, enabled: bool) -> Self {
        self.horizontal = enabled;
        self
    }

    pub fn with_vertical(mut self, enabled: bool) -> Self {
        self.vertical = enabled;
        self
    }

    pub fn scroll_to(&mut self, offset: Point) {
        self.scroll_offset = self.clamp_offset(offset);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset.y = 0.0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset.y = (self.content_size.height - self.bounds.height()).max(0.0);
    }

    fn clamp_offset(&self, offset: Point) -> Point {
        Point::new(
            if self.horizontal {
                offset.x.clamp(0.0, (self.content_size.width - self.bounds.width()).max(0.0))
            } else {
                0.0
            },
            if self.vertical {
                offset.y.clamp(0.0, (self.content_size.height - self.bounds.height()).max(0.0))
            } else {
                0.0
            },
        )
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ScrollView {
    fn id(&self) -> u64 { self.id }
    fn type_name(&self) -> &'static str { "ScrollView" }

    fn layout(&mut self, constraints: &LayoutConstraints) -> Result<Size, UiError> {
        // Measure content with unbounded constraints in scroll direction
        let content_constraints = LayoutConstraints {
            min_width: if self.horizontal { 0.0 } else { constraints.min_width },
            max_width: if self.horizontal { f32::INFINITY } else { constraints.max_width },
            min_height: if self.vertical { 0.0 } else { constraints.min_height },
            max_height: if self.vertical { f32::INFINITY } else { constraints.max_height },
        };

        if let Some(child) = &mut self.child {
            self.content_size = child.layout(&content_constraints)?;
        }

        self.bounds.size = constraints.constrain(Size::new(
            constraints.max_width,
            constraints.max_height,
        ));
        Ok(self.bounds.size)
    }

    fn set_position(&mut self, position: Point) {
        self.bounds.origin = position;
        if let Some(child) = &mut self.child {
            child.set_position(Point::new(
                position.x - self.scroll_offset.x,
                position.y - self.scroll_offset.y,
            ));
        }
    }

    fn bounds(&self) -> Rect { self.bounds }

    fn update(&mut self, ctx: &UiContext) -> Result<(), UiError> {
        // Apply velocity for smooth scrolling
        if self.velocity.x.abs() > 0.1 || self.velocity.y.abs() > 0.1 {
            let delta = ctx.delta_time.as_secs_f32();
            self.scroll_offset.x += self.velocity.x * delta;
            self.scroll_offset.y += self.velocity.y * delta;
            self.scroll_offset = self.clamp_offset(self.scroll_offset);
            
            // Friction
            self.velocity.x *= 0.95;
            self.velocity.y *= 0.95;
        }

        if let Some(child) = &mut self.child {
            child.update(ctx)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &UiEvent, ctx: &UiContext) -> bool {
        match event {
            UiEvent::Scroll { delta_x, delta_y } if self.bounds.contains(&ctx.viewport.into()) => {
                if self.horizontal {
                    self.scroll_offset.x += delta_x;
                }
                if self.vertical {
                    self.scroll_offset.y += delta_y;
                }
                self.scroll_offset = self.clamp_offset(self.scroll_offset);
                true
            }
            UiEvent::PointerDown { position, .. } if self.bounds.contains(position) => {
                self.last_drag = Some(*position);
                self.velocity = Point::zero();
                true
            }
            UiEvent::PointerMove { position, .. } if self.last_drag.is_some() => {
                if let Some(last) = self.last_drag {
                    let delta = Point::new(last.x - position.x, last.y - position.y);
                    if self.horizontal {
                        self.scroll_offset.x += delta.x;
                    }
                    if self.vertical {
                        self.scroll_offset.y += delta.y;
                    }
                    self.scroll_offset = self.clamp_offset(self.scroll_offset);
                    self.velocity = Point::new(delta.x * 10.0, delta.y * 10.0);
                    self.last_drag = Some(*position);
                }
                true
            }
            UiEvent::PointerUp { .. } if self.last_drag.is_some() => {
                self.last_drag = None;
                true
            }
            UiEvent::Gesture(GestureEvent::Pan { delta, velocity }) => {
                if self.horizontal {
                    self.scroll_offset.x -= delta.x;
                }
                if self.vertical {
                    self.scroll_offset.y -= delta.y;
                }
                self.scroll_offset = self.clamp_offset(self.scroll_offset);
                self.velocity = *velocity;
                true
            }
            _ => {
                if let Some(child) = &mut self.child {
                    child.handle_event(event, ctx)
                } else {
                    false
                }
            }
        }
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        // Set clip rect
        let old_clip = ctx.clip_rect;
        ctx.clip_rect = Some(self.bounds);

        if let Some(child) = &self.child {
            child.render(ctx)?;
        }

        // Restore clip
        ctx.clip_rect = old_clip;
        Ok(())
    }

    fn is_visible(&self) -> bool { self.visible }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl From<Size> for Point {
    fn from(size: Size) -> Self {
        Point::new(size.width / 2.0, size.height / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constraints() {
        let tight = LayoutConstraints::tight(Size::new(100.0, 50.0));
        assert_eq!(tight.min_width, 100.0);
        assert_eq!(tight.max_width, 100.0);

        let loose = LayoutConstraints::loose(Size::new(100.0, 50.0));
        assert_eq!(loose.min_width, 0.0);
        assert_eq!(loose.max_width, 100.0);
    }

    #[test]
    fn test_container_widget() {
        let container = Container::new();
        assert_eq!(container.type_name(), "Container");
    }

    #[test]
    fn test_text_widget() {
        let text = Text::new("Hello");
        assert_eq!(text.content(), "Hello");
    }

    #[test]
    fn test_button_widget() {
        let button = Button::new("Click Me");
        assert_eq!(button.type_name(), "Button");
        assert!(button.is_enabled());
    }

    #[test]
    fn test_switch_toggle() {
        let mut switch = Switch::new(false);
        assert!(!switch.value());
        switch.set_value(true);
        assert!(switch.value());
    }

    #[test]
    fn test_slider_range() {
        let mut slider = Slider::new(0.0, 100.0, 50.0);
        assert!((slider.value() - 50.0).abs() < 0.001);
        slider.set_value(150.0); // Beyond max
        assert!((slider.value() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_progress_value() {
        let mut progress = Progress::new(0.5);
        assert!((progress.value - 0.5).abs() < 0.001);
        progress.set_value(1.5); // Beyond 1.0
        assert!((progress.value - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_scrollview_clamp() {
        let mut scroll = ScrollView::new();
        scroll.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        scroll.content_size = Size::new(100.0, 300.0);
        
        scroll.scroll_to(Point::new(0.0, 500.0));
        assert!((scroll.scroll_offset.y - 200.0).abs() < 0.001);
    }
}
