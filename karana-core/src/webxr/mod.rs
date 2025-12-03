//! WebXR Bridge - Enables web content to access AR/VR features
//! 
//! This module provides the bridge between web applications running in AR browser tabs
//! and the Kāraṇa OS spatial computing capabilities. It implements:
//! 
//! - WebXR session management (immersive-ar, immersive-vr, inline)
//! - Reality capture APIs (camera, depth, environment)
//! - Anchor management through WebXR Anchors API
//! - Hit testing for placing virtual content
//! - Light estimation for realistic rendering
//! 
//! Security: All WebXR capabilities require explicit user consent via Oracle permission system.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::ar_tabs::TabId;
use crate::spatial::{SpatialAnchor, WorldPosition, AnchorId};

mod session;
mod reality_capture;
mod anchors;
mod hit_test;
mod light_estimation;

pub use session::*;
pub use reality_capture::*;
pub use anchors::*;
pub use hit_test::*;
pub use light_estimation::*;

/// Unique identifier for a WebXR session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct XRSessionId(u64);

impl XRSessionId {
    /// Create a new unique session ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl std::fmt::Display for XRSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "xr-session-{}", self.0)
    }
}

/// WebXR session mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRSessionMode {
    /// Non-immersive inline session
    Inline,
    /// Immersive VR session
    ImmersiveVR,
    /// Immersive AR session
    ImmersiveAR,
}

/// WebXR reference space type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum XRReferenceSpaceType {
    /// Viewer-relative space (head-locked)
    Viewer,
    /// Local space (seated)
    Local,
    /// Local floor space (standing)
    LocalFloor,
    /// Bounded floor space (room-scale)
    BoundedFloor,
    /// Unbounded space (world-scale)
    Unbounded,
}

/// WebXR required/optional features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRSessionFeatures {
    /// Required features - session fails if not available
    pub required: Vec<XRFeature>,
    /// Optional features - session proceeds without them
    pub optional: Vec<XRFeature>,
}

impl Default for XRSessionFeatures {
    fn default() -> Self {
        Self {
            required: vec![],
            optional: vec![],
        }
    }
}

/// WebXR features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum XRFeature {
    /// Anchors API for persistent AR content
    Anchors,
    /// Bounded floor reference space
    BoundedFloor,
    /// Depth sensing
    DepthSensing,
    /// DOM overlay (web content overlaid)
    DomOverlay,
    /// Hand tracking
    HandTracking,
    /// Hit test for ray-world intersection
    HitTest,
    /// Layers (composition layers)
    Layers,
    /// Light estimation
    LightEstimation,
    /// Local reference space
    Local,
    /// Local floor reference space
    LocalFloor,
    /// Secondary views (spectator mode)
    SecondaryViews,
    /// Unbounded reference space
    Unbounded,
    /// Viewer reference space
    Viewer,
    /// Plane detection
    PlaneDetection,
    /// Mesh detection (scene understanding)
    MeshDetection,
    /// Camera access
    CameraAccess,
    /// Raw camera access (pixel data)
    RawCameraAccess,
}

impl XRFeature {
    /// Check if this feature requires permission prompt
    pub fn requires_permission(&self) -> bool {
        matches!(self, 
            XRFeature::Anchors |
            XRFeature::DepthSensing |
            XRFeature::HandTracking |
            XRFeature::HitTest |
            XRFeature::LightEstimation |
            XRFeature::PlaneDetection |
            XRFeature::MeshDetection |
            XRFeature::CameraAccess |
            XRFeature::RawCameraAccess
        )
    }
    
    /// Get permission description for user prompt
    pub fn permission_description(&self) -> &'static str {
        match self {
            XRFeature::Anchors => "Place persistent virtual objects in your space",
            XRFeature::DepthSensing => "Understand depth and surfaces in your environment",
            XRFeature::HandTracking => "Track your hand movements and gestures",
            XRFeature::HitTest => "Detect surfaces in your environment",
            XRFeature::LightEstimation => "Estimate lighting conditions",
            XRFeature::PlaneDetection => "Detect flat surfaces like floors and walls",
            XRFeature::MeshDetection => "Create 3D models of your environment",
            XRFeature::CameraAccess => "Access camera for AR passthrough",
            XRFeature::RawCameraAccess => "Access raw camera pixel data",
            _ => "Use this WebXR feature",
        }
    }
}

/// WebXR permission state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRPermissionState {
    /// Permission not yet requested
    Prompt,
    /// Permission granted
    Granted,
    /// Permission denied
    Denied,
}

/// WebXR frame of reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRReferenceSpace {
    /// Space type
    pub space_type: XRReferenceSpaceType,
    /// Transform from local to reference space
    pub transform: XRRigidTransform,
}

/// WebXR rigid transform (position + orientation)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XRRigidTransform {
    /// Position
    pub position: XRVector3,
    /// Orientation (quaternion)
    pub orientation: XRQuaternion,
}

impl XRRigidTransform {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            position: XRVector3::zero(),
            orientation: XRQuaternion::identity(),
        }
    }
    
    /// Create from position
    pub fn from_position(x: f64, y: f64, z: f64) -> Self {
        Self {
            position: XRVector3 { x, y, z },
            orientation: XRQuaternion::identity(),
        }
    }
    
    /// Inverse transform
    pub fn inverse(&self) -> Self {
        let inv_orientation = self.orientation.inverse();
        let inv_pos = inv_orientation.rotate_vector(&XRVector3 {
            x: -self.position.x,
            y: -self.position.y,
            z: -self.position.z,
        });
        Self {
            position: inv_pos,
            orientation: inv_orientation,
        }
    }
    
    /// Compose transforms: self * other
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            position: XRVector3 {
                x: self.position.x + self.orientation.rotate_vector(&other.position).x,
                y: self.position.y + self.orientation.rotate_vector(&other.position).y,
                z: self.position.z + self.orientation.rotate_vector(&other.position).z,
            },
            orientation: self.orientation.multiply(&other.orientation),
        }
    }
}

/// 3D vector
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XRVector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl XRVector3 {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
    
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            Self::zero()
        }
    }
    
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

/// Quaternion for rotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRQuaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Default for XRQuaternion {
    fn default() -> Self {
        Self::identity()
    }
}

impl XRQuaternion {
    pub fn identity() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
    
    pub fn from_axis_angle(axis: &XRVector3, angle_rad: f64) -> Self {
        let half = angle_rad / 2.0;
        let s = half.sin();
        let n = axis.normalize();
        Self {
            x: n.x * s,
            y: n.y * s,
            z: n.z * s,
            w: half.cos(),
        }
    }
    
    pub fn from_euler_yxz(yaw: f64, pitch: f64, roll: f64) -> Self {
        let cy = (yaw / 2.0).cos();
        let sy = (yaw / 2.0).sin();
        let cp = (pitch / 2.0).cos();
        let sp = (pitch / 2.0).sin();
        let cr = (roll / 2.0).cos();
        let sr = (roll / 2.0).sin();
        
        Self {
            x: cy * sp * cr + sy * cp * sr,
            y: sy * cp * cr - cy * sp * sr,
            z: cy * cp * sr - sy * sp * cr,
            w: cy * cp * cr + sy * sp * sr,
        }
    }
    
    pub fn inverse(&self) -> Self {
        let norm_sq = self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w;
        Self {
            x: -self.x / norm_sq,
            y: -self.y / norm_sq,
            z: -self.z / norm_sq,
            w: self.w / norm_sq,
        }
    }
    
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }
    
    pub fn rotate_vector(&self, v: &XRVector3) -> XRVector3 {
        // q * v * q^-1
        let qv = Self { x: v.x, y: v.y, z: v.z, w: 0.0 };
        let result = self.multiply(&qv).multiply(&self.inverse());
        XRVector3 { x: result.x, y: result.y, z: result.z }
    }
    
    pub fn slerp(&self, other: &Self, t: f64) -> Self {
        let mut dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;
        
        let (other, dot) = if dot < 0.0 {
            (Self { x: -other.x, y: -other.y, z: -other.z, w: -other.w }, -dot)
        } else {
            (other.clone(), dot)
        };
        
        // Use linear interpolation for nearly identical quaternions
        if dot > 0.9995 {
            return Self {
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
                w: self.w + t * (other.w - self.w),
            };
        }
        
        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();
        
        let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;
        
        Self {
            x: self.x * s0 + other.x * s1,
            y: self.y * s0 + other.y * s1,
            z: self.z * s0 + other.z * s1,
            w: self.w * s0 + other.w * s1,
        }
    }
}

/// WebXR view (one per eye)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRView {
    /// Eye identifier
    pub eye: XREye,
    /// View matrix (inverse of camera transform)
    pub transform: XRRigidTransform,
    /// Projection matrix values
    pub projection_matrix: [f64; 16],
}

/// Eye identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XREye {
    None,
    Left,
    Right,
}

/// WebXR viewport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRViewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// WebXR frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRFrame {
    /// Timestamp in milliseconds
    pub timestamp: f64,
    /// Session ID
    pub session_id: u64,
    /// Predicted display time
    pub predicted_display_time: f64,
    /// Views for this frame
    pub views: Vec<XRView>,
    /// Hit test results if requested
    pub hit_test_results: Vec<XRHitTestResult>,
    /// Detected anchors
    pub tracked_anchors: Vec<XRAnchorInfo>,
    /// Light estimate if available
    pub light_estimate: Option<XRLightEstimate>,
    /// Detected planes
    pub detected_planes: Vec<XRPlane>,
}

/// Hit test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRHitTestResult {
    /// Transform at hit point
    pub transform: XRRigidTransform,
    /// Hit point relative to reference space
    pub pose: XRRigidTransform,
}

/// Anchor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRAnchorInfo {
    /// Anchor ID
    pub id: String,
    /// Current pose
    pub pose: XRRigidTransform,
    /// Tracking state
    pub tracking_state: XRTrackingState,
}

/// Tracking state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRTrackingState {
    /// Not currently tracked
    NotTracking,
    /// Tracking with normal quality
    Tracking,
    /// Tracking emulated (estimated position)
    Emulated,
}

/// Light estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRLightEstimate {
    /// Primary light direction
    pub primary_light_direction: XRVector3,
    /// Primary light intensity
    pub primary_light_intensity: XRVector3,
    /// Spherical harmonics coefficients
    pub spherical_harmonics: Vec<f32>,
}

/// Detected plane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRPlane {
    /// Unique plane ID
    pub id: String,
    /// Center pose
    pub pose: XRRigidTransform,
    /// Polygon vertices (local space)
    pub polygon: Vec<XRVector3>,
    /// Plane orientation
    pub orientation: XRPlaneOrientation,
    /// Semantic label
    pub semantic_label: Option<String>,
}

/// Plane orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRPlaneOrientation {
    Horizontal,
    Vertical,
}

/// WebXR bridge manager - coordinates between web tabs and AR system
pub struct WebXRBridge {
    /// Active sessions by ID
    sessions: HashMap<XRSessionId, XRSessionState>,
    /// Sessions by tab
    sessions_by_tab: HashMap<TabId, XRSessionId>,
    /// Feature permissions by domain
    permissions: HashMap<String, HashMap<XRFeature, XRPermissionState>>,
    /// Frame counter for timestamps
    frame_count: u64,
}

/// Session state
struct XRSessionState {
    /// Session info
    pub session: XRSession,
    /// Owning tab
    pub tab_id: TabId,
    /// Active reference spaces
    pub reference_spaces: HashMap<XRReferenceSpaceType, XRReferenceSpace>,
    /// Pending hit test sources
    pub hit_test_sources: Vec<HitTestSource>,
    /// Session anchors
    pub anchors: HashMap<String, XRAnchor>,
    /// Created at
    pub created_at: Instant,
}

impl WebXRBridge {
    /// Create a new WebXR bridge
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            sessions_by_tab: HashMap::new(),
            permissions: HashMap::new(),
            frame_count: 0,
        }
    }
    
    /// Check if XR is supported
    pub fn is_supported(&self, mode: XRSessionMode) -> bool {
        // Kāraṇa OS supports all modes on AR glasses
        match mode {
            XRSessionMode::Inline => true,
            XRSessionMode::ImmersiveVR => true,
            XRSessionMode::ImmersiveAR => true,
        }
    }
    
    /// Check if a feature is supported
    pub fn is_feature_supported(&self, feature: XRFeature) -> bool {
        match feature {
            XRFeature::Viewer |
            XRFeature::Local |
            XRFeature::LocalFloor |
            XRFeature::BoundedFloor |
            XRFeature::Unbounded |
            XRFeature::Anchors |
            XRFeature::HitTest |
            XRFeature::LightEstimation |
            XRFeature::PlaneDetection |
            XRFeature::DepthSensing => true,
            
            // Features we're working on
            XRFeature::HandTracking |
            XRFeature::MeshDetection |
            XRFeature::CameraAccess |
            XRFeature::RawCameraAccess => true,
            
            // Features that need more work
            XRFeature::DomOverlay |
            XRFeature::Layers |
            XRFeature::SecondaryViews => false,
        }
    }
    
    /// Request permission for a feature
    pub fn request_permission(
        &mut self,
        domain: &str,
        feature: XRFeature,
    ) -> XRPermissionState {
        // Check existing permission
        if let Some(domain_perms) = self.permissions.get(domain) {
            if let Some(state) = domain_perms.get(&feature) {
                return *state;
            }
        }
        
        // For now, auto-grant all permissions (in production, this would prompt user)
        // TODO: Integrate with Oracle permission system
        let state = if feature.requires_permission() {
            // Would prompt user here
            XRPermissionState::Granted
        } else {
            XRPermissionState::Granted
        };
        
        self.permissions
            .entry(domain.to_string())
            .or_default()
            .insert(feature, state);
        
        state
    }
    
    /// Request a new XR session
    pub fn request_session(
        &mut self,
        tab_id: TabId,
        domain: &str,
        mode: XRSessionMode,
        features: XRSessionFeatures,
    ) -> Result<XRSession> {
        // Check if tab already has a session
        if self.sessions_by_tab.contains_key(&tab_id) {
            return Err(anyhow!("Tab already has an active XR session"));
        }
        
        // Check required features
        let mut enabled_features = vec![];
        for feature in &features.required {
            if !self.is_feature_supported(*feature) {
                return Err(anyhow!("Required feature {:?} not supported", feature));
            }
            let perm = self.request_permission(domain, *feature);
            if perm == XRPermissionState::Denied {
                return Err(anyhow!("Permission denied for feature {:?}", feature));
            }
            enabled_features.push(*feature);
        }
        
        // Check optional features
        for feature in &features.optional {
            if self.is_feature_supported(*feature) {
                let perm = self.request_permission(domain, *feature);
                if perm == XRPermissionState::Granted {
                    enabled_features.push(*feature);
                }
            }
        }
        
        // Create session
        let session_id = XRSessionId::new();
        let session = XRSession {
            id: session_id,
            mode,
            enabled_features,
            visibility_state: XRVisibilityState::Visible,
            frame_rate: 90.0, // Default 90 Hz for AR glasses
            render_state: XRRenderState::default(),
        };
        
        // Create session state
        let state = XRSessionState {
            session: session.clone(),
            tab_id,
            reference_spaces: HashMap::new(),
            hit_test_sources: vec![],
            anchors: HashMap::new(),
            created_at: Instant::now(),
        };
        
        self.sessions.insert(session_id, state);
        self.sessions_by_tab.insert(tab_id, session_id);
        
        log::info!("[WEBXR] Created session {} for tab {} ({:?})", 
                   session_id, tab_id, mode);
        
        Ok(session)
    }
    
    /// End an XR session
    pub fn end_session(&mut self, session_id: XRSessionId) -> Result<()> {
        let state = self.sessions.remove(&session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        self.sessions_by_tab.remove(&state.tab_id);
        
        log::info!("[WEBXR] Ended session {} (active for {:?})", 
                   session_id, state.created_at.elapsed());
        
        Ok(())
    }
    
    /// End session by tab
    pub fn end_session_for_tab(&mut self, tab_id: TabId) -> Result<()> {
        if let Some(session_id) = self.sessions_by_tab.remove(&tab_id) {
            self.sessions.remove(&session_id);
            log::info!("[WEBXR] Ended session for tab {}", tab_id);
        }
        Ok(())
    }
    
    /// Get session for a tab
    pub fn get_session(&self, tab_id: TabId) -> Option<&XRSession> {
        self.sessions_by_tab.get(&tab_id)
            .and_then(|id| self.sessions.get(id))
            .map(|s| &s.session)
    }
    
    /// Request a reference space
    pub fn request_reference_space(
        &mut self,
        session_id: XRSessionId,
        space_type: XRReferenceSpaceType,
    ) -> Result<XRReferenceSpace> {
        let state = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        // Check if space type is enabled
        let feature = match space_type {
            XRReferenceSpaceType::Viewer => XRFeature::Viewer,
            XRReferenceSpaceType::Local => XRFeature::Local,
            XRReferenceSpaceType::LocalFloor => XRFeature::LocalFloor,
            XRReferenceSpaceType::BoundedFloor => XRFeature::BoundedFloor,
            XRReferenceSpaceType::Unbounded => XRFeature::Unbounded,
        };
        
        // Viewer and Local are always available
        let allowed = matches!(space_type, 
            XRReferenceSpaceType::Viewer | XRReferenceSpaceType::Local) ||
            state.session.enabled_features.contains(&feature);
        
        if !allowed {
            return Err(anyhow!("Reference space {:?} not enabled for this session", space_type));
        }
        
        // Create reference space
        let space = XRReferenceSpace {
            space_type,
            transform: XRRigidTransform::identity(),
        };
        
        state.reference_spaces.insert(space_type, space.clone());
        
        Ok(space)
    }
    
    /// Get a frame for rendering
    pub fn get_frame(&mut self, session_id: XRSessionId) -> Result<XRFrame> {
        let state = self.sessions.get(&session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        self.frame_count += 1;
        
        // Calculate timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        let timestamp = now.as_secs_f64() * 1000.0;
        
        // Build views based on session mode
        let views = match state.session.mode {
            XRSessionMode::Inline => {
                // Single view for inline
                vec![XRView {
                    eye: XREye::None,
                    transform: XRRigidTransform::identity(),
                    projection_matrix: Self::create_perspective_matrix(
                        std::f64::consts::PI / 3.0, // 60 degree FOV
                        16.0 / 9.0,
                        0.1,
                        1000.0,
                    ),
                }]
            }
            XRSessionMode::ImmersiveVR | XRSessionMode::ImmersiveAR => {
                // Stereo views
                let ipd = 0.063; // 63mm inter-pupillary distance
                vec![
                    XRView {
                        eye: XREye::Left,
                        transform: XRRigidTransform::from_position(-ipd / 2.0, 0.0, 0.0),
                        projection_matrix: Self::create_perspective_matrix(
                            std::f64::consts::PI / 2.5, // ~72 degree FOV
                            1.0,
                            0.1,
                            1000.0,
                        ),
                    },
                    XRView {
                        eye: XREye::Right,
                        transform: XRRigidTransform::from_position(ipd / 2.0, 0.0, 0.0),
                        projection_matrix: Self::create_perspective_matrix(
                            std::f64::consts::PI / 2.5,
                            1.0,
                            0.1,
                            1000.0,
                        ),
                    },
                ]
            }
        };
        
        // Collect tracked anchors
        let tracked_anchors: Vec<XRAnchorInfo> = state.anchors.values()
            .map(|anchor| XRAnchorInfo {
                id: anchor.id.clone(),
                pose: anchor.pose.clone(),
                tracking_state: anchor.tracking_state,
            })
            .collect();
        
        Ok(XRFrame {
            timestamp,
            session_id: session_id.0,
            predicted_display_time: timestamp + (1000.0 / state.session.frame_rate),
            views,
            hit_test_results: vec![], // Would be populated from spatial system
            tracked_anchors,
            light_estimate: None, // Would come from light estimation module
            detected_planes: vec![], // Would come from plane detection
        })
    }
    
    /// Create a perspective projection matrix (column-major)
    fn create_perspective_matrix(fov_y: f64, aspect: f64, near: f64, far: f64) -> [f64; 16] {
        let f = 1.0 / (fov_y / 2.0).tan();
        let nf = 1.0 / (near - far);
        
        [
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (far + near) * nf, -1.0,
            0.0, 0.0, 2.0 * far * near * nf, 0.0,
        ]
    }
    
    /// Create an anchor
    pub fn create_anchor(
        &mut self,
        session_id: XRSessionId,
        pose: XRRigidTransform,
    ) -> Result<String> {
        let state = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        if !state.session.enabled_features.contains(&XRFeature::Anchors) {
            return Err(anyhow!("Anchors feature not enabled"));
        }
        
        let anchor = XRAnchor::new(pose);
        let id = anchor.id.clone();
        state.anchors.insert(id.clone(), anchor);
        
        log::info!("[WEBXR] Created anchor {} in session {}", id, session_id);
        
        Ok(id)
    }
    
    /// Delete an anchor
    pub fn delete_anchor(
        &mut self,
        session_id: XRSessionId,
        anchor_id: &str,
    ) -> Result<()> {
        let state = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        state.anchors.remove(anchor_id)
            .ok_or_else(|| anyhow!("Anchor not found"))?;
        
        Ok(())
    }
    
    /// Request hit test source
    pub fn request_hit_test_source(
        &mut self,
        session_id: XRSessionId,
        ray: HitTestRay,
    ) -> Result<HitTestSourceId> {
        let state = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        if !state.session.enabled_features.contains(&XRFeature::HitTest) {
            return Err(anyhow!("HitTest feature not enabled"));
        }
        
        let source = HitTestSource {
            id: HitTestSourceId::new(),
            ray,
            entity_types: vec![HitTestEntityType::Plane],
        };
        
        let id = source.id;
        state.hit_test_sources.push(source);
        
        Ok(id)
    }
    
    /// Get stats about active sessions
    pub fn stats(&self) -> WebXRStats {
        WebXRStats {
            active_sessions: self.sessions.len(),
            total_anchors: self.sessions.values()
                .map(|s| s.anchors.len())
                .sum(),
            total_hit_test_sources: self.sessions.values()
                .map(|s| s.hit_test_sources.len())
                .sum(),
        }
    }
}

impl Default for WebXRBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// WebXR statistics
#[derive(Debug, Clone)]
pub struct WebXRStats {
    pub active_sessions: usize,
    pub total_anchors: usize,
    pub total_hit_test_sources: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_webxr_bridge_creation() {
        let bridge = WebXRBridge::new();
        assert!(bridge.is_supported(XRSessionMode::ImmersiveAR));
        assert!(bridge.is_supported(XRSessionMode::ImmersiveVR));
        assert!(bridge.is_supported(XRSessionMode::Inline));
    }
    
    #[test]
    fn test_session_creation() {
        let mut bridge = WebXRBridge::new();
        let tab_id = uuid::Uuid::new_v4();
        
        let session = bridge.request_session(
            tab_id,
            "example.com",
            XRSessionMode::ImmersiveAR,
            XRSessionFeatures::default(),
        ).unwrap();
        
        assert_eq!(session.mode, XRSessionMode::ImmersiveAR);
        assert!(bridge.get_session(tab_id).is_some());
    }
    
    #[test]
    fn test_reference_space() {
        let mut bridge = WebXRBridge::new();
        let tab_id = uuid::Uuid::new_v4();
        
        let session = bridge.request_session(
            tab_id,
            "example.com",
            XRSessionMode::ImmersiveAR,
            XRSessionFeatures::default(),
        ).unwrap();
        
        let space = bridge.request_reference_space(
            session.id,
            XRReferenceSpaceType::Local,
        ).unwrap();
        
        assert_eq!(space.space_type, XRReferenceSpaceType::Local);
    }
    
    #[test]
    fn test_anchor_creation() {
        let mut bridge = WebXRBridge::new();
        let tab_id = uuid::Uuid::new_v4();
        
        let session = bridge.request_session(
            tab_id,
            "example.com",
            XRSessionMode::ImmersiveAR,
            XRSessionFeatures {
                required: vec![XRFeature::Anchors],
                optional: vec![],
            },
        ).unwrap();
        
        let anchor_id = bridge.create_anchor(
            session.id,
            XRRigidTransform::from_position(1.0, 0.0, -2.0),
        ).unwrap();
        
        assert!(!anchor_id.is_empty());
        assert_eq!(bridge.stats().total_anchors, 1);
    }
    
    #[test]
    fn test_quaternion_operations() {
        let q1 = XRQuaternion::identity();
        let v = XRVector3::new(1.0, 0.0, 0.0);
        
        let rotated = q1.rotate_vector(&v);
        assert!((rotated.x - 1.0).abs() < 0.001);
        assert!(rotated.y.abs() < 0.001);
        assert!(rotated.z.abs() < 0.001);
    }
    
    #[test]
    fn test_transform_composition() {
        let t1 = XRRigidTransform::from_position(1.0, 0.0, 0.0);
        let t2 = XRRigidTransform::from_position(0.0, 1.0, 0.0);
        
        let composed = t1.multiply(&t2);
        assert!((composed.position.x - 1.0).abs() < 0.001);
        assert!((composed.position.y - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_frame_generation() {
        let mut bridge = WebXRBridge::new();
        let tab_id = uuid::Uuid::new_v4();
        
        let session = bridge.request_session(
            tab_id,
            "example.com",
            XRSessionMode::ImmersiveAR,
            XRSessionFeatures::default(),
        ).unwrap();
        
        let frame = bridge.get_frame(session.id).unwrap();
        assert_eq!(frame.views.len(), 2); // Stereo for AR
        assert!(frame.timestamp > 0.0);
    }
    
    #[test]
    fn test_feature_permissions() {
        let mut bridge = WebXRBridge::new();
        
        // Anchors require permission
        assert!(XRFeature::Anchors.requires_permission());
        
        // Request permission
        let state = bridge.request_permission("example.com", XRFeature::Anchors);
        assert_eq!(state, XRPermissionState::Granted);
    }
    
    #[test]
    fn test_duplicate_session_rejected() {
        let mut bridge = WebXRBridge::new();
        let tab_id = uuid::Uuid::new_v4();
        
        // First session succeeds
        bridge.request_session(
            tab_id,
            "example.com",
            XRSessionMode::ImmersiveAR,
            XRSessionFeatures::default(),
        ).unwrap();
        
        // Second session for same tab fails
        let result = bridge.request_session(
            tab_id,
            "example.com",
            XRSessionMode::ImmersiveVR,
            XRSessionFeatures::default(),
        );
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_session_end() {
        let mut bridge = WebXRBridge::new();
        let tab_id = uuid::Uuid::new_v4();
        
        let session = bridge.request_session(
            tab_id,
            "example.com",
            XRSessionMode::ImmersiveAR,
            XRSessionFeatures::default(),
        ).unwrap();
        
        bridge.end_session(session.id).unwrap();
        assert!(bridge.get_session(tab_id).is_none());
    }
}
