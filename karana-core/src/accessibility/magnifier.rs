//! Magnifier system for low-vision users

use std::time::{Duration, Instant};

/// Magnifier display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnifierMode {
    /// Full screen magnification
    FullScreen,
    /// Lens mode (magnified area follows gaze)
    Lens,
    /// Docked mode (portion of screen magnified)
    Docked,
    /// Picture-in-picture magnified view
    PictureInPicture,
}

/// Lens shape for lens mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensShape {
    /// Circular lens
    Circle,
    /// Rectangular lens
    Rectangle,
    /// Oval lens
    Oval,
}

/// Docked position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockedPosition {
    /// Top of screen
    Top,
    /// Bottom of screen
    Bottom,
    /// Left of screen
    Left,
    /// Right of screen
    Right,
}

/// Color filter for magnifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFilter {
    /// No filter
    None,
    /// Inverted colors
    Inverted,
    /// Grayscale inverted
    GrayscaleInverted,
    /// Yellow on blue
    YellowOnBlue,
    /// White on black
    WhiteOnBlack,
    /// Green on black
    GreenOnBlack,
    /// Red on black
    RedOnBlack,
}

/// Magnifier system
#[derive(Debug)]
pub struct Magnifier {
    /// Whether magnifier is enabled
    enabled: bool,
    /// Magnifier mode
    mode: MagnifierMode,
    /// Zoom level (1.0 - 20.0)
    zoom: f32,
    /// Lens shape
    lens_shape: LensShape,
    /// Lens size (diameter in pixels)
    lens_size: u32,
    /// Docked position
    docked_position: DockedPosition,
    /// Docked size (percentage of screen)
    docked_size: f32,
    /// Color filter
    color_filter: ColorFilter,
    /// Focus position (normalized 0.0 - 1.0)
    focus_position: (f32, f32),
    /// Follow cursor/gaze
    follow_focus: bool,
    /// Smooth follow (lerp factor)
    smooth_follow: f32,
    /// Current view position
    view_position: (f32, f32),
    /// Edge detection enabled
    edge_detection: bool,
    /// Edge detection strength
    edge_strength: f32,
    /// Anti-aliasing
    anti_aliasing: bool,
    /// Last zoom change
    last_zoom_change: Instant,
    /// Zoom animation in progress
    animating: bool,
    /// Target zoom for animation
    target_zoom: f32,
}

impl Magnifier {
    /// Create new magnifier
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: MagnifierMode::FullScreen,
            zoom: 2.0,
            lens_shape: LensShape::Circle,
            lens_size: 300,
            docked_position: DockedPosition::Top,
            docked_size: 0.3,
            color_filter: ColorFilter::None,
            focus_position: (0.5, 0.5),
            follow_focus: true,
            smooth_follow: 0.2,
            view_position: (0.5, 0.5),
            edge_detection: false,
            edge_strength: 0.5,
            anti_aliasing: true,
            last_zoom_change: Instant::now(),
            animating: false,
            target_zoom: 2.0,
        }
    }
    
    /// Enable magnifier
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable magnifier
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Toggle magnifier
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Set magnifier mode
    pub fn set_mode(&mut self, mode: MagnifierMode) {
        self.mode = mode;
    }
    
    /// Get magnifier mode
    pub fn mode(&self) -> MagnifierMode {
        self.mode
    }
    
    /// Set zoom level (1.0 - 20.0)
    pub fn set_zoom(&mut self, zoom: f32) {
        let new_zoom = zoom.clamp(1.0, 20.0);
        self.target_zoom = new_zoom;
        
        // Animate if significant change
        if (new_zoom - self.zoom).abs() > 0.1 {
            self.animating = true;
        } else {
            self.zoom = new_zoom;
        }
        
        self.last_zoom_change = Instant::now();
    }
    
    /// Get current zoom level
    pub fn zoom(&self) -> f32 {
        self.zoom
    }
    
    /// Zoom in by step
    pub fn zoom_in(&mut self, step: f32) {
        self.set_zoom(self.zoom + step);
    }
    
    /// Zoom out by step
    pub fn zoom_out(&mut self, step: f32) {
        self.set_zoom(self.zoom - step);
    }
    
    /// Reset zoom to default
    pub fn reset_zoom(&mut self) {
        self.set_zoom(2.0);
    }
    
    /// Set lens shape
    pub fn set_lens_shape(&mut self, shape: LensShape) {
        self.lens_shape = shape;
    }
    
    /// Get lens shape
    pub fn lens_shape(&self) -> LensShape {
        self.lens_shape
    }
    
    /// Set lens size
    pub fn set_lens_size(&mut self, size: u32) {
        self.lens_size = size.clamp(100, 800);
    }
    
    /// Get lens size
    pub fn lens_size(&self) -> u32 {
        self.lens_size
    }
    
    /// Set docked position
    pub fn set_docked_position(&mut self, position: DockedPosition) {
        self.docked_position = position;
    }
    
    /// Get docked position
    pub fn docked_position(&self) -> DockedPosition {
        self.docked_position
    }
    
    /// Set docked size (0.1 - 0.5)
    pub fn set_docked_size(&mut self, size: f32) {
        self.docked_size = size.clamp(0.1, 0.5);
    }
    
    /// Get docked size
    pub fn docked_size(&self) -> f32 {
        self.docked_size
    }
    
    /// Set color filter
    pub fn set_color_filter(&mut self, filter: ColorFilter) {
        self.color_filter = filter;
    }
    
    /// Get color filter
    pub fn color_filter(&self) -> ColorFilter {
        self.color_filter
    }
    
    /// Update focus position (e.g., from gaze)
    pub fn update_focus(&mut self, x: f32, y: f32) {
        self.focus_position = (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
    }
    
    /// Get focus position
    pub fn focus_position(&self) -> (f32, f32) {
        self.focus_position
    }
    
    /// Get current view position (may be smoothed)
    pub fn view_position(&self) -> (f32, f32) {
        self.view_position
    }
    
    /// Set follow focus mode
    pub fn set_follow_focus(&mut self, follow: bool) {
        self.follow_focus = follow;
    }
    
    /// Check if following focus
    pub fn is_following_focus(&self) -> bool {
        self.follow_focus
    }
    
    /// Set smooth follow factor (0.0 = instant, 1.0 = very smooth)
    pub fn set_smooth_follow(&mut self, factor: f32) {
        self.smooth_follow = factor.clamp(0.0, 0.95);
    }
    
    /// Enable edge detection
    pub fn enable_edge_detection(&mut self, strength: f32) {
        self.edge_detection = true;
        self.edge_strength = strength.clamp(0.0, 1.0);
    }
    
    /// Disable edge detection
    pub fn disable_edge_detection(&mut self) {
        self.edge_detection = false;
    }
    
    /// Check if edge detection enabled
    pub fn is_edge_detection(&self) -> bool {
        self.edge_detection
    }
    
    /// Get viewport bounds (in normalized coordinates)
    pub fn viewport_bounds(&self) -> (f32, f32, f32, f32) {
        // Calculate size of visible area based on zoom
        let half_width = 0.5 / self.zoom;
        let half_height = 0.5 / self.zoom;
        
        let (cx, cy) = self.view_position;
        
        (
            (cx - half_width).max(0.0),
            (cy - half_height).max(0.0),
            (cx + half_width).min(1.0),
            (cy + half_height).min(1.0),
        )
    }
    
    /// Update magnifier state
    pub fn update(&mut self, _delta: Duration) {
        // Update view position to follow focus
        if self.follow_focus {
            let (fx, fy) = self.focus_position;
            let (vx, vy) = self.view_position;
            
            // Lerp toward focus
            let lerp_factor = 1.0 - self.smooth_follow;
            self.view_position = (
                vx + (fx - vx) * lerp_factor,
                vy + (fy - vy) * lerp_factor,
            );
        }
        
        // Animate zoom
        if self.animating {
            let lerp_factor = 0.15;
            self.zoom = self.zoom + (self.target_zoom - self.zoom) * lerp_factor;
            
            if (self.zoom - self.target_zoom).abs() < 0.01 {
                self.zoom = self.target_zoom;
                self.animating = false;
            }
        }
    }
    
    /// Get color transformation matrix for filter
    pub fn color_matrix(&self) -> [[f32; 4]; 4] {
        match self.color_filter {
            ColorFilter::None => [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorFilter::Inverted => [
                [-1.0, 0.0, 0.0, 1.0],
                [0.0, -1.0, 0.0, 1.0],
                [0.0, 0.0, -1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorFilter::GrayscaleInverted => [
                [-0.299, -0.587, -0.114, 1.0],
                [-0.299, -0.587, -0.114, 1.0],
                [-0.299, -0.587, -0.114, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorFilter::YellowOnBlue => [
                [0.0, 0.0, 0.0, 1.0],  // R -> Yellow
                [0.0, 0.0, 0.0, 1.0],  // G -> Yellow
                [0.0, 0.0, 1.0, 0.0],  // B stays
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorFilter::WhiteOnBlack => [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorFilter::GreenOnBlack => [
                [0.0, 0.0, 0.0, 0.0],
                [0.299, 0.587, 0.114, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorFilter::RedOnBlack => [
                [0.299, 0.587, 0.114, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl Default for Magnifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_magnifier_creation() {
        let mag = Magnifier::new();
        assert!(!mag.is_enabled());
        assert_eq!(mag.zoom(), 2.0);
        assert_eq!(mag.mode(), MagnifierMode::FullScreen);
    }
    
    #[test]
    fn test_enable_toggle() {
        let mut mag = Magnifier::new();
        
        mag.enable();
        assert!(mag.is_enabled());
        
        mag.toggle();
        assert!(!mag.is_enabled());
    }
    
    #[test]
    fn test_zoom() {
        let mut mag = Magnifier::new();
        
        mag.set_zoom(4.0);
        // Due to animation, need to wait or update
        mag.update(Duration::from_millis(1000));
        
        // Zoom should be approaching target
        assert!(mag.zoom() > 2.0);
        
        // Test clamping
        mag.set_zoom(50.0);
        assert!(mag.target_zoom <= 20.0);
    }
    
    #[test]
    fn test_zoom_in_out() {
        let mut mag = Magnifier::new();
        
        let initial = mag.zoom();
        mag.zoom_in(1.0);
        
        // Target should increase
        assert!(mag.target_zoom > initial);
    }
    
    #[test]
    fn test_lens_settings() {
        let mut mag = Magnifier::new();
        
        mag.set_lens_shape(LensShape::Rectangle);
        assert_eq!(mag.lens_shape(), LensShape::Rectangle);
        
        mag.set_lens_size(500);
        assert_eq!(mag.lens_size(), 500);
    }
    
    #[test]
    fn test_docked_settings() {
        let mut mag = Magnifier::new();
        
        mag.set_mode(MagnifierMode::Docked);
        mag.set_docked_position(DockedPosition::Bottom);
        mag.set_docked_size(0.4);
        
        assert_eq!(mag.mode(), MagnifierMode::Docked);
        assert_eq!(mag.docked_position(), DockedPosition::Bottom);
        assert!((mag.docked_size() - 0.4).abs() < 0.01);
    }
    
    #[test]
    fn test_color_filter() {
        let mut mag = Magnifier::new();
        
        mag.set_color_filter(ColorFilter::YellowOnBlue);
        assert_eq!(mag.color_filter(), ColorFilter::YellowOnBlue);
    }
    
    #[test]
    fn test_focus_tracking() {
        let mut mag = Magnifier::new();
        mag.enable();
        mag.set_follow_focus(true);
        mag.set_smooth_follow(0.0); // Instant follow
        
        mag.update_focus(0.8, 0.2);
        mag.update(Duration::from_millis(16));
        
        let (vx, vy) = mag.view_position();
        assert!((vx - 0.8).abs() < 0.1);
        assert!((vy - 0.2).abs() < 0.1);
    }
    
    #[test]
    fn test_viewport_bounds() {
        let mut mag = Magnifier::new();
        mag.set_zoom(2.0);
        
        // At 2x zoom, should see 50% of screen
        let (x1, y1, x2, y2) = mag.viewport_bounds();
        
        let width = x2 - x1;
        let height = y2 - y1;
        
        assert!((width - 0.5).abs() < 0.1);
        assert!((height - 0.5).abs() < 0.1);
    }
}
