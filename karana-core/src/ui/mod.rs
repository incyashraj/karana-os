//! KaranaOS UI Framework
//! 
//! A comprehensive UI framework for AR smart glasses with support for:
//! - Widget system with composable components
//! - Layout managers (flexbox, grid, stack)
//! - Theme engine with light/dark modes
//! - Animation system for smooth transitions
//! - Accessibility features
//! - Gesture-driven interactions

use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant};

// Legacy UI submodules (gui, input, render_proof, theme, compositor)
pub mod gui;
pub mod input;
pub mod render_proof;
pub mod compositor;
pub mod glasses_sim;

// Legacy UI module (preserves KaranaUI)
pub mod legacy_ui;

// New modular UI components
pub mod widgets;
pub mod layout;
pub mod dao_theme;  // DAO theme system
pub mod theming;    // New theme engine
pub mod animation;
pub mod accessibility;

// Re-export legacy types for compatibility
pub use legacy_ui::{KaranaUI, UiState, RenderIntent};

// Note: Don't re-export new theme module to avoid conflict with legacy theme submodule
pub use widgets::*;
pub use layout::*;
pub use theming::*;
pub use animation::*;
pub use accessibility::*;

/// UI Error types
#[derive(Debug, Clone)]
pub enum UiError {
    /// Widget not found
    WidgetNotFound(String),
    /// Invalid layout
    InvalidLayout(String),
    /// Render error
    RenderError(String),
    /// Theme error
    ThemeError(String),
    /// Animation error
    AnimationError(String),
    /// Resource not found
    ResourceNotFound(String),
    /// Out of memory
    OutOfMemory,
    /// Invalid state
    InvalidState(String),
}

impl std::fmt::Display for UiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WidgetNotFound(id) => write!(f, "Widget not found: {}", id),
            Self::InvalidLayout(msg) => write!(f, "Invalid layout: {}", msg),
            Self::RenderError(msg) => write!(f, "Render error: {}", msg),
            Self::ThemeError(msg) => write!(f, "Theme error: {}", msg),
            Self::AnimationError(msg) => write!(f, "Animation error: {}", msg),
            Self::ResourceNotFound(id) => write!(f, "Resource not found: {}", id),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for UiError {}

/// Widget ID generator
static WIDGET_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate unique widget ID
pub fn generate_widget_id() -> u64 {
    WIDGET_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 2D Point
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn distance(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Size
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn zero() -> Self {
        Self { width: 0.0, height: 0.0 }
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

/// Rectangle
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub fn zero() -> Self {
        Self {
            origin: Point::zero(),
            size: Size::zero(),
        }
    }

    pub fn x(&self) -> f32 { self.origin.x }
    pub fn y(&self) -> f32 { self.origin.y }
    pub fn width(&self) -> f32 { self.size.width }
    pub fn height(&self) -> f32 { self.size.height }

    pub fn left(&self) -> f32 { self.origin.x }
    pub fn top(&self) -> f32 { self.origin.y }
    pub fn right(&self) -> f32 { self.origin.x + self.size.width }
    pub fn bottom(&self) -> f32 { self.origin.y + self.size.height }

    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.left() && point.x <= self.right() &&
        point.y >= self.top() && point.y <= self.bottom()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right() && self.right() > other.left() &&
        self.top() < other.bottom() && self.bottom() > other.top()
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }
        let x = self.left().max(other.left());
        let y = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Some(Rect::new(x, y, right - x, bottom - y))
    }

    pub fn inset(&self, amount: f32) -> Self {
        Self::new(
            self.origin.x + amount,
            self.origin.y + amount,
            self.size.width - 2.0 * amount,
            self.size.height - 2.0 * amount,
        )
    }
}

/// Edge insets (padding/margin)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    pub fn all(value: f32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }

    pub fn horizontal(&self) -> f32 { self.left + self.right }
    pub fn vertical(&self) -> f32 { self.top + self.bottom }
}

/// Color (RGBA)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255, a: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    pub fn from_hex_with_alpha(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    pub fn with_alpha(&self, a: u8) -> Self {
        Self { r: self.r, g: self.g, b: self.b, a }
    }

    pub fn blend(&self, other: &Color, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        let inv = 1.0 - factor;
        Self {
            r: (self.r as f32 * inv + other.r as f32 * factor) as u8,
            g: (self.g as f32 * inv + other.g as f32 * factor) as u8,
            b: (self.b as f32 * inv + other.b as f32 * factor) as u8,
            a: (self.a as f32 * inv + other.a as f32 * factor) as u8,
        }
    }

    pub fn to_rgba_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    pub fn to_argb_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    Black,
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

/// Vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

/// Font configuration
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "System".into(),
            size: 16.0,
            weight: FontWeight::Regular,
            style: FontStyle::Normal,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

/// Text overflow behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextOverflow {
    Clip,
    #[default]
    Ellipsis,
    Fade,
    Visible,
}

/// Border style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
}

/// Border configuration
#[derive(Debug, Clone, Copy, Default)]
pub struct Border {
    pub width: f32,
    pub color: Color,
    pub style: BorderStyle,
    pub radius: f32,
}

impl Border {
    pub fn solid(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Solid,
            radius: 0.0,
        }
    }

    pub fn rounded(width: f32, color: Color, radius: f32) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Solid,
            radius,
        }
    }
}

/// Shadow configuration
#[derive(Debug, Clone, Copy, Default)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Shadow {
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            spread: 0.0,
            color,
        }
    }

    pub fn drop(blur: f32, color: Color) -> Self {
        Self::new(0.0, blur / 2.0, blur, color)
    }
}

/// Background type
#[derive(Debug, Clone)]
pub enum Background {
    /// Solid color
    Color(Color),
    /// Linear gradient
    LinearGradient {
        start: Point,
        end: Point,
        stops: Vec<(f32, Color)>,
    },
    /// Radial gradient
    RadialGradient {
        center: Point,
        radius: f32,
        stops: Vec<(f32, Color)>,
    },
    /// Image
    Image {
        url: String,
        fit: ImageFit,
    },
    /// Blur (for glass effect)
    Blur {
        radius: f32,
        tint: Color,
    },
}

impl Default for Background {
    fn default() -> Self {
        Self::Color(Color::TRANSPARENT)
    }
}

/// Image fit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFit {
    #[default]
    Contain,
    Cover,
    Fill,
    ScaleDown,
    None,
}

/// Cursor type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cursor {
    #[default]
    Default,
    Pointer,
    Text,
    Move,
    Crosshair,
    NotAllowed,
    Wait,
    Grab,
    Grabbing,
    ResizeNS,
    ResizeEW,
    ResizeNESW,
    ResizeNWSE,
}

/// Input event
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Pointer down
    PointerDown { id: u32, position: Point, button: u8 },
    /// Pointer up
    PointerUp { id: u32, position: Point, button: u8 },
    /// Pointer move
    PointerMove { id: u32, position: Point },
    /// Pointer enter widget
    PointerEnter { id: u32 },
    /// Pointer leave widget
    PointerLeave { id: u32 },
    /// Scroll
    Scroll { delta_x: f32, delta_y: f32 },
    /// Key down
    KeyDown { key: String, modifiers: Modifiers },
    /// Key up
    KeyUp { key: String, modifiers: Modifiers },
    /// Character input
    Character { char: char },
    /// Focus gained
    Focus,
    /// Focus lost
    Blur,
    /// Gesture
    Gesture(GestureEvent),
}

/// Gesture event
#[derive(Debug, Clone)]
pub enum GestureEvent {
    Tap { position: Point },
    DoubleTap { position: Point },
    LongPress { position: Point },
    Pan { delta: Point, velocity: Point },
    Pinch { scale: f32, center: Point },
    Rotate { angle: f32, center: Point },
    Swipe { direction: SwipeDirection, velocity: f32 },
}

/// Swipe direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Render context
pub struct RenderContext {
    /// Current transform
    pub transform: Transform2D,
    /// Current clip rect
    pub clip_rect: Option<Rect>,
    /// Current opacity
    pub opacity: f32,
    /// Theme
    pub theme: Arc<Theme>,
    /// Viewport size
    pub viewport: Size,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self {
            transform: Transform2D::identity(),
            clip_rect: None,
            opacity: 1.0,
            theme: Arc::new(Theme::default()),
            viewport: Size::new(1920.0, 1080.0),
        }
    }
}

/// 2D Transform
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    /// Matrix elements [a, b, c, d, tx, ty]
    pub matrix: [f32; 6],
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform2D {
    pub fn identity() -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }

    pub fn translation(tx: f32, ty: f32) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, tx, ty],
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            matrix: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    pub fn rotation(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            matrix: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let [a1, b1, c1, d1, tx1, ty1] = self.matrix;
        let [a2, b2, c2, d2, tx2, ty2] = other.matrix;
        Self {
            matrix: [
                a1 * a2 + c1 * b2,
                b1 * a2 + d1 * b2,
                a1 * c2 + c1 * d2,
                b1 * c2 + d1 * d2,
                a1 * tx2 + c1 * ty2 + tx1,
                b1 * tx2 + d1 * ty2 + ty1,
            ],
        }
    }

    pub fn transform_point(&self, point: &Point) -> Point {
        let [a, b, c, d, tx, ty] = self.matrix;
        Point {
            x: a * point.x + c * point.y + tx,
            y: b * point.x + d * point.y + ty,
        }
    }
}

/// UI Context for widget building
pub struct UiContext {
    /// Theme
    pub theme: Arc<Theme>,
    /// Animations
    pub animations: AnimationController,
    /// Focus stack
    pub focus_stack: Vec<u64>,
    /// Accessibility state
    pub accessibility: AccessibilityState,
    /// Current time
    pub now: Instant,
    /// Delta time since last frame
    pub delta_time: Duration,
    /// Viewport size
    pub viewport: Size,
    /// Scale factor
    pub scale_factor: f32,
}

impl Default for UiContext {
    fn default() -> Self {
        Self {
            theme: Arc::new(Theme::default()),
            animations: AnimationController::new(),
            focus_stack: Vec::new(),
            accessibility: AccessibilityState::default(),
            now: Instant::now(),
            delta_time: Duration::from_millis(16),
            viewport: Size::new(1920.0, 1080.0),
            scale_factor: 1.0,
        }
    }
}

/// UI Manager
pub struct UiManager {
    /// Root widget tree
    root: Option<Box<dyn Widget>>,
    /// Widget registry
    widgets: HashMap<u64, WidgetState>,
    /// Theme
    theme: Arc<Theme>,
    /// Animation controller
    animations: AnimationController,
    /// Focus manager
    focus: FocusManager,
    /// Accessibility
    accessibility: AccessibilityState,
    /// Layout cache
    layout_cache: HashMap<u64, Rect>,
    /// Render cache enabled
    cache_enabled: bool,
    /// Last frame time
    last_frame: Instant,
    /// Frame count
    frame_count: u64,
    /// Viewport size
    viewport: Size,
    /// Scale factor
    scale_factor: f32,
}

impl Default for UiManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UiManager {
    pub fn new() -> Self {
        Self {
            root: None,
            widgets: HashMap::new(),
            theme: Arc::new(Theme::default()),
            animations: AnimationController::new(),
            focus: FocusManager::new(),
            accessibility: AccessibilityState::default(),
            layout_cache: HashMap::new(),
            cache_enabled: true,
            last_frame: Instant::now(),
            frame_count: 0,
            viewport: Size::new(1920.0, 1080.0),
            scale_factor: 1.0,
        }
    }

    /// Set root widget
    pub fn set_root(&mut self, widget: Box<dyn Widget>) {
        self.root = Some(widget);
        self.invalidate_layout();
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = Arc::new(theme);
        self.invalidate_layout();
    }

    /// Get current theme
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Set viewport size
    pub fn set_viewport(&mut self, size: Size) {
        if self.viewport != size {
            self.viewport = size;
            self.invalidate_layout();
        }
    }

    /// Set scale factor
    pub fn set_scale_factor(&mut self, factor: f32) {
        if self.scale_factor != factor {
            self.scale_factor = factor;
            self.invalidate_layout();
        }
    }

    /// Invalidate layout cache
    pub fn invalidate_layout(&mut self) {
        self.layout_cache.clear();
    }

    /// Update UI (call each frame)
    pub fn update(&mut self, delta: Duration) -> Result<(), UiError> {
        let now = Instant::now();
        
        // Update animations
        self.animations.update(delta);
        
        // Update widgets
        let ctx = UiContext {
            theme: Arc::clone(&self.theme),
            animations: self.animations.clone(),
            focus_stack: self.focus.stack.clone(),
            accessibility: self.accessibility.clone(),
            now,
            delta_time: delta,
            viewport: self.viewport,
            scale_factor: self.scale_factor,
        };

        if let Some(root) = &mut self.root {
            root.update(&ctx)?;
        }

        self.last_frame = now;
        self.frame_count += 1;
        Ok(())
    }

    /// Handle UI event
    pub fn handle_event(&mut self, event: &UiEvent) -> bool {
        if let Some(root) = &mut self.root {
            let ctx = UiContext {
                theme: Arc::clone(&self.theme),
                animations: self.animations.clone(),
                focus_stack: self.focus.stack.clone(),
                accessibility: self.accessibility.clone(),
                now: Instant::now(),
                delta_time: Duration::from_millis(16),
                viewport: self.viewport,
                scale_factor: self.scale_factor,
            };
            return root.handle_event(event, &ctx);
        }
        false
    }

    /// Perform layout
    pub fn layout(&mut self) -> Result<(), UiError> {
        if let Some(root) = &mut self.root {
            let constraints = LayoutConstraints {
                min_width: 0.0,
                max_width: self.viewport.width,
                min_height: 0.0,
                max_height: self.viewport.height,
            };
            let _ = root.layout(&constraints)?;
            root.set_position(Point::zero());
        }
        Ok(())
    }

    /// Render UI
    pub fn render(&self, ctx: &mut RenderContext) -> Result<(), UiError> {
        if let Some(root) = &self.root {
            root.render(ctx)?;
        }
        Ok(())
    }

    /// Get focused widget ID
    pub fn focused_widget(&self) -> Option<u64> {
        self.focus.current()
    }

    /// Focus widget by ID
    pub fn focus_widget(&mut self, id: u64) {
        self.focus.set_focus(id);
    }

    /// Get animations controller
    pub fn animations(&mut self) -> &mut AnimationController {
        &mut self.animations
    }
    
    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

/// Widget state storage
#[derive(Debug, Clone)]
pub struct WidgetState {
    pub id: u64,
    pub visible: bool,
    pub enabled: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub bounds: Rect,
    pub dirty: bool,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            id: generate_widget_id(),
            visible: true,
            enabled: true,
            focused: false,
            hovered: false,
            pressed: false,
            bounds: Rect::zero(),
            dirty: true,
        }
    }
}

/// Focus manager
#[derive(Debug, Clone)]
pub struct FocusManager {
    /// Current focus
    current_focus: Option<u64>,
    /// Focus stack
    stack: Vec<u64>,
    /// Tab order
    tab_order: Vec<u64>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            current_focus: None,
            stack: Vec::new(),
            tab_order: Vec::new(),
        }
    }

    pub fn current(&self) -> Option<u64> {
        self.current_focus
    }

    pub fn set_focus(&mut self, id: u64) {
        self.current_focus = Some(id);
        self.stack.push(id);
    }

    pub fn clear_focus(&mut self) {
        self.current_focus = None;
    }

    pub fn focus_next(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }
        let current_idx = self.current_focus
            .and_then(|id| self.tab_order.iter().position(|&x| x == id))
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.tab_order.len();
        self.current_focus = Some(self.tab_order[next_idx]);
    }

    pub fn focus_previous(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }
        let current_idx = self.current_focus
            .and_then(|id| self.tab_order.iter().position(|&x| x == id))
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 { self.tab_order.len() - 1 } else { current_idx - 1 };
        self.current_focus = Some(self.tab_order[prev_idx]);
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains(&Point::new(50.0, 30.0)));
        assert!(!rect.contains(&Point::new(5.0, 5.0)));
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let r3 = Rect::new(200.0, 200.0, 50.0, 50.0);
        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_color_blend() {
        let white = Color::WHITE;
        let black = Color::BLACK;
        let gray = white.blend(&black, 0.5);
        assert!(gray.r > 120 && gray.r < 135);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::from_hex(0xFF5500);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 85);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform2D::identity();
        let p = Point::new(10.0, 20.0);
        let result = t.transform_point(&p);
        assert!((result.x - 10.0).abs() < 0.001);
        assert!((result.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_translation() {
        let t = Transform2D::translation(5.0, 10.0);
        let p = Point::new(10.0, 20.0);
        let result = t.transform_point(&p);
        assert!((result.x - 15.0).abs() < 0.001);
        assert!((result.y - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_ui_manager_create() {
        let manager = UiManager::new();
        assert!(manager.root.is_none());
        assert_eq!(manager.frame_count(), 0);
    }

    #[test]
    fn test_focus_manager() {
        let mut focus = FocusManager::new();
        assert!(focus.current().is_none());
        focus.set_focus(42);
        assert_eq!(focus.current(), Some(42));
        focus.clear_focus();
        assert!(focus.current().is_none());
    }

    #[test]
    fn test_widget_id_generator() {
        let id1 = generate_widget_id();
        let id2 = generate_widget_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_edge_insets() {
        let insets = EdgeInsets::all(10.0);
        assert_eq!(insets.horizontal(), 20.0);
        assert_eq!(insets.vertical(), 20.0);
    }

    #[test]
    fn test_rect_inset() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let inset = rect.inset(10.0);
        assert_eq!(inset.x(), 10.0);
        assert_eq!(inset.y(), 10.0);
        assert_eq!(inset.width(), 80.0);
        assert_eq!(inset.height(), 80.0);
    }
}
