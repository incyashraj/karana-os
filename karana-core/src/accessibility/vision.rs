//! Vision assistance for low-vision and blind users

use std::time::Instant;

/// Color vision modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Normal color vision
    Normal,
    /// Protanopia (red-blind)
    Protanopia,
    /// Deuteranopia (green-blind)
    Deuteranopia,
    /// Tritanopia (blue-blind)
    Tritanopia,
    /// Grayscale
    Grayscale,
    /// Inverted colors
    Inverted,
    /// High contrast (dark)
    HighContrastDark,
    /// High contrast (light)
    HighContrastLight,
}

impl ColorMode {
    /// Get color transformation matrix (4x4 for RGBA)
    pub fn transform_matrix(&self) -> [[f32; 4]; 4] {
        match self {
            ColorMode::Normal => [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorMode::Protanopia => [
                [0.567, 0.433, 0.0, 0.0],
                [0.558, 0.442, 0.0, 0.0],
                [0.0, 0.242, 0.758, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorMode::Deuteranopia => [
                [0.625, 0.375, 0.0, 0.0],
                [0.7, 0.3, 0.0, 0.0],
                [0.0, 0.3, 0.7, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorMode::Tritanopia => [
                [0.95, 0.05, 0.0, 0.0],
                [0.0, 0.433, 0.567, 0.0],
                [0.0, 0.475, 0.525, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorMode::Grayscale => [
                [0.299, 0.587, 0.114, 0.0],
                [0.299, 0.587, 0.114, 0.0],
                [0.299, 0.587, 0.114, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorMode::Inverted => [
                [-1.0, 0.0, 0.0, 1.0],
                [0.0, -1.0, 0.0, 1.0],
                [0.0, 0.0, -1.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            ColorMode::HighContrastDark | ColorMode::HighContrastLight => [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

/// Contrast modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContrastMode {
    /// Normal contrast
    Normal,
    /// Increased contrast
    Increased,
    /// High contrast
    High,
    /// Maximum contrast
    Maximum,
}

impl ContrastMode {
    /// Get contrast multiplier
    pub fn multiplier(&self) -> f32 {
        match self {
            ContrastMode::Normal => 1.0,
            ContrastMode::Increased => 1.3,
            ContrastMode::High => 1.6,
            ContrastMode::Maximum => 2.0,
        }
    }
}

/// Vision assistance system
#[derive(Debug)]
pub struct VisionAssist {
    /// Color mode
    color_mode: ColorMode,
    /// Contrast mode
    contrast_mode: ContrastMode,
    /// Text scale factor
    text_scale: f32,
    /// Bold text enabled
    bold_text: bool,
    /// Cursor/pointer size multiplier
    cursor_scale: f32,
    /// Focus indicator thickness
    focus_thickness: f32,
    /// Reduce transparency
    reduce_transparency: bool,
    /// Edge enhancement enabled
    edge_enhancement: bool,
    /// Edge enhancement strength
    edge_strength: f32,
    /// Color inversion enabled
    inverted: bool,
    /// High contrast enabled
    high_contrast: bool,
    /// Last mode change
    last_change: Instant,
}

impl VisionAssist {
    /// Create new vision assist
    pub fn new() -> Self {
        Self {
            color_mode: ColorMode::Normal,
            contrast_mode: ContrastMode::Normal,
            text_scale: 1.0,
            bold_text: false,
            cursor_scale: 1.0,
            focus_thickness: 2.0,
            reduce_transparency: false,
            edge_enhancement: false,
            edge_strength: 0.5,
            inverted: false,
            high_contrast: false,
            last_change: Instant::now(),
        }
    }
    
    /// Set color mode
    pub fn set_color_mode(&mut self, mode: ColorMode) {
        self.color_mode = mode;
        self.last_change = Instant::now();
    }
    
    /// Get color mode
    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }
    
    /// Set contrast mode
    pub fn set_contrast_mode(&mut self, mode: ContrastMode) {
        self.contrast_mode = mode;
        self.last_change = Instant::now();
    }
    
    /// Get contrast mode
    pub fn contrast_mode(&self) -> ContrastMode {
        self.contrast_mode
    }
    
    /// Set text scale (0.5 - 3.0)
    pub fn set_text_scale(&mut self, scale: f32) {
        self.text_scale = scale.clamp(0.5, 3.0);
        self.last_change = Instant::now();
    }
    
    /// Get text scale
    pub fn text_scale(&self) -> f32 {
        self.text_scale
    }
    
    /// Enable/disable bold text
    pub fn set_bold_text(&mut self, enabled: bool) {
        self.bold_text = enabled;
    }
    
    /// Check if bold text is enabled
    pub fn is_bold_text(&self) -> bool {
        self.bold_text
    }
    
    /// Set cursor scale
    pub fn set_cursor_scale(&mut self, scale: f32) {
        self.cursor_scale = scale.clamp(1.0, 4.0);
    }
    
    /// Get cursor scale
    pub fn cursor_scale(&self) -> f32 {
        self.cursor_scale
    }
    
    /// Set focus indicator thickness
    pub fn set_focus_thickness(&mut self, thickness: f32) {
        self.focus_thickness = thickness.clamp(1.0, 8.0);
    }
    
    /// Get focus thickness
    pub fn focus_thickness(&self) -> f32 {
        self.focus_thickness
    }
    
    /// Enable/disable transparency reduction
    pub fn set_reduce_transparency(&mut self, enabled: bool) {
        self.reduce_transparency = enabled;
    }
    
    /// Check if transparency is reduced
    pub fn is_transparency_reduced(&self) -> bool {
        self.reduce_transparency
    }
    
    /// Enable edge enhancement
    pub fn enable_edge_enhancement(&mut self, strength: f32) {
        self.edge_enhancement = true;
        self.edge_strength = strength.clamp(0.0, 1.0);
    }
    
    /// Disable edge enhancement
    pub fn disable_edge_enhancement(&mut self) {
        self.edge_enhancement = false;
    }
    
    /// Check if edge enhancement is enabled
    pub fn is_edge_enhancement(&self) -> bool {
        self.edge_enhancement
    }
    
    /// Toggle color inversion
    pub fn toggle_inversion(&mut self) {
        self.inverted = !self.inverted;
        if self.inverted {
            self.color_mode = ColorMode::Inverted;
        } else {
            self.color_mode = ColorMode::Normal;
        }
        self.last_change = Instant::now();
    }
    
    /// Check if colors are inverted
    pub fn is_inverted(&self) -> bool {
        self.inverted
    }
    
    /// Toggle high contrast
    pub fn toggle_high_contrast(&mut self) {
        self.high_contrast = !self.high_contrast;
        if self.high_contrast {
            self.contrast_mode = ContrastMode::High;
            self.color_mode = ColorMode::HighContrastDark;
        } else {
            self.contrast_mode = ContrastMode::Normal;
            self.color_mode = ColorMode::Normal;
        }
        self.last_change = Instant::now();
    }
    
    /// Check if high contrast is enabled
    pub fn is_high_contrast(&self) -> bool {
        self.high_contrast
    }
    
    /// Get effective opacity for overlays (respects reduce transparency)
    pub fn effective_opacity(&self, original: f32) -> f32 {
        if self.reduce_transparency {
            original.max(0.9)
        } else {
            original
        }
    }
    
    /// Apply color transform to RGBA color
    pub fn transform_color(&self, r: f32, g: f32, b: f32, a: f32) -> (f32, f32, f32, f32) {
        let matrix = self.color_mode.transform_matrix();
        
        let new_r = matrix[0][0] * r + matrix[0][1] * g + matrix[0][2] * b + matrix[0][3];
        let new_g = matrix[1][0] * r + matrix[1][1] * g + matrix[1][2] * b + matrix[1][3];
        let new_b = matrix[2][0] * r + matrix[2][1] * g + matrix[2][2] * b + matrix[2][3];
        
        (
            new_r.clamp(0.0, 1.0),
            new_g.clamp(0.0, 1.0),
            new_b.clamp(0.0, 1.0),
            a,
        )
    }
}

impl Default for VisionAssist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vision_assist_creation() {
        let va = VisionAssist::new();
        assert_eq!(va.text_scale(), 1.0);
        assert_eq!(va.color_mode(), ColorMode::Normal);
    }
    
    #[test]
    fn test_text_scale() {
        let mut va = VisionAssist::new();
        
        va.set_text_scale(2.0);
        assert_eq!(va.text_scale(), 2.0);
        
        // Test clamping
        va.set_text_scale(5.0);
        assert_eq!(va.text_scale(), 3.0);
        
        va.set_text_scale(0.1);
        assert_eq!(va.text_scale(), 0.5);
    }
    
    #[test]
    fn test_color_modes() {
        let mut va = VisionAssist::new();
        
        va.set_color_mode(ColorMode::Grayscale);
        assert_eq!(va.color_mode(), ColorMode::Grayscale);
    }
    
    #[test]
    fn test_color_transform() {
        let mut va = VisionAssist::new();
        
        // Normal should preserve colors
        let (r, g, b, a) = va.transform_color(1.0, 0.5, 0.0, 1.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 0.5).abs() < 0.01);
        
        // Grayscale should make them equal
        va.set_color_mode(ColorMode::Grayscale);
        let (r, g, b, _) = va.transform_color(1.0, 0.0, 0.0, 1.0);
        assert!((r - g).abs() < 0.01);
        assert!((g - b).abs() < 0.01);
    }
    
    #[test]
    fn test_high_contrast_toggle() {
        let mut va = VisionAssist::new();
        
        assert!(!va.is_high_contrast());
        va.toggle_high_contrast();
        assert!(va.is_high_contrast());
        assert_eq!(va.contrast_mode(), ContrastMode::High);
    }
    
    #[test]
    fn test_inversion_toggle() {
        let mut va = VisionAssist::new();
        
        assert!(!va.is_inverted());
        va.toggle_inversion();
        assert!(va.is_inverted());
        assert_eq!(va.color_mode(), ColorMode::Inverted);
    }
    
    #[test]
    fn test_reduce_transparency() {
        let mut va = VisionAssist::new();
        
        // Normal opacity
        assert_eq!(va.effective_opacity(0.5), 0.5);
        
        // Reduced transparency
        va.set_reduce_transparency(true);
        assert_eq!(va.effective_opacity(0.5), 0.9);
    }
}
