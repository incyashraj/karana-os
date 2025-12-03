//! WebXR Session Management
//!
//! Manages XR session lifecycle, state transitions, and render configuration.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{XRSessionId, XRSessionMode, XRFeature, XRViewport};

/// WebXR session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRSession {
    /// Session ID
    pub id: XRSessionId,
    /// Session mode
    pub mode: XRSessionMode,
    /// Enabled features
    pub enabled_features: Vec<XRFeature>,
    /// Visibility state
    pub visibility_state: XRVisibilityState,
    /// Target frame rate
    pub frame_rate: f64,
    /// Render state
    pub render_state: XRRenderState,
}

impl XRSession {
    /// Check if a feature is enabled
    pub fn has_feature(&self, feature: XRFeature) -> bool {
        self.enabled_features.contains(&feature)
    }
    
    /// Get supported frame rates
    pub fn supported_frame_rates(&self) -> Vec<f64> {
        // Typical AR glasses frame rates
        vec![60.0, 72.0, 90.0, 120.0]
    }
    
    /// Update target frame rate
    pub fn update_frame_rate(&mut self, rate: f64) -> bool {
        if self.supported_frame_rates().contains(&rate) {
            self.frame_rate = rate;
            true
        } else {
            false
        }
    }
}

/// Session visibility state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRVisibilityState {
    /// Session is visible and has focus
    Visible,
    /// Session visible but doesn't have focus
    VisibleBlurred,
    /// Session not visible
    Hidden,
}

/// Render state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRRenderState {
    /// Base layer viewport
    pub base_layer: Option<XRWebGLLayer>,
    /// Depth near plane
    pub depth_near: f64,
    /// Depth far plane
    pub depth_far: f64,
    /// Inline vertical FOV (radians)
    pub inline_vertical_fov: Option<f64>,
    /// Additional composition layers
    pub layers: Vec<XRCompositionLayer>,
}

impl Default for XRRenderState {
    fn default() -> Self {
        Self {
            base_layer: None,
            depth_near: 0.1,
            depth_far: 1000.0,
            inline_vertical_fov: Some(std::f64::consts::PI / 2.0), // 90 degrees
            layers: vec![],
        }
    }
}

/// WebGL layer for XR rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRWebGLLayer {
    /// Antialias enabled
    pub antialias: bool,
    /// Depth buffer
    pub depth: bool,
    /// Stencil buffer
    pub stencil: bool,
    /// Alpha channel
    pub alpha: bool,
    /// Ignore depth values
    pub ignore_depth_values: bool,
    /// Fixed foveation level (0.0 - 1.0)
    pub fixed_foveation: Option<f64>,
    /// Framebuffer width
    pub framebuffer_width: u32,
    /// Framebuffer height
    pub framebuffer_height: u32,
}

impl Default for XRWebGLLayer {
    fn default() -> Self {
        Self {
            antialias: true,
            depth: true,
            stencil: false,
            alpha: true,
            ignore_depth_values: false,
            fixed_foveation: None,
            framebuffer_width: 2560,
            framebuffer_height: 2560,
        }
    }
}

/// Composition layer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XRCompositionLayer {
    /// Projection layer (main content)
    Projection(XRProjectionLayer),
    /// Quad layer (flat panel in space)
    Quad(XRQuadLayer),
    /// Cylinder layer (curved panel)
    Cylinder(XRCylinderLayer),
    /// Equirect layer (360 content)
    Equirect(XREquirectLayer),
    /// Cube layer (skybox)
    Cube(XRCubeLayer),
}

/// Projection layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRProjectionLayer {
    /// Fixed foveation
    pub fixed_foveation: Option<f64>,
    /// Texture type
    pub texture_type: XRTextureType,
}

/// Quad layer - flat panel in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRQuadLayer {
    /// Width in meters
    pub width: f32,
    /// Height in meters
    pub height: f32,
}

/// Cylinder layer - curved panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRCylinderLayer {
    /// Radius in meters
    pub radius: f32,
    /// Central angle in radians
    pub central_angle: f32,
    /// Aspect ratio
    pub aspect_ratio: f32,
}

/// Equirectangular layer for 360 content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XREquirectLayer {
    /// Radius in meters
    pub radius: f32,
    /// Central horizontal angle
    pub central_horizontal_angle: f32,
    /// Upper vertical angle
    pub upper_vertical_angle: f32,
    /// Lower vertical angle
    pub lower_vertical_angle: f32,
}

/// Cube layer for skybox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRCubeLayer {
    /// Cube orientation
    pub orientation: [f64; 4], // quaternion
}

/// Texture type for layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRTextureType {
    /// 2D texture
    Texture,
    /// 2D array texture
    TextureArray,
}

/// Session event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XRSessionEvent {
    /// Session ended
    End,
    /// Visibility changed
    VisibilityChange(XRVisibilityState),
    /// Input sources changed
    InputSourcesChange,
    /// Select event (primary action)
    Select { input_source: usize },
    /// Select start
    SelectStart { input_source: usize },
    /// Select end
    SelectEnd { input_source: usize },
    /// Squeeze event (grip action)
    Squeeze { input_source: usize },
    /// Frame rate change
    FrameRateChange(f64),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_features() {
        let session = XRSession {
            id: XRSessionId::new(),
            mode: XRSessionMode::ImmersiveAR,
            enabled_features: vec![XRFeature::Anchors, XRFeature::HitTest],
            visibility_state: XRVisibilityState::Visible,
            frame_rate: 90.0,
            render_state: XRRenderState::default(),
        };
        
        assert!(session.has_feature(XRFeature::Anchors));
        assert!(session.has_feature(XRFeature::HitTest));
        assert!(!session.has_feature(XRFeature::HandTracking));
    }
    
    #[test]
    fn test_frame_rate_update() {
        let mut session = XRSession {
            id: XRSessionId::new(),
            mode: XRSessionMode::ImmersiveVR,
            enabled_features: vec![],
            visibility_state: XRVisibilityState::Visible,
            frame_rate: 90.0,
            render_state: XRRenderState::default(),
        };
        
        assert!(session.update_frame_rate(72.0));
        assert_eq!(session.frame_rate, 72.0);
        
        // Invalid rate
        assert!(!session.update_frame_rate(144.0));
        assert_eq!(session.frame_rate, 72.0);
    }
    
    #[test]
    fn test_render_state_default() {
        let state = XRRenderState::default();
        assert!((state.depth_near - 0.1).abs() < 0.001);
        assert!((state.depth_far - 1000.0).abs() < 0.001);
    }
}
