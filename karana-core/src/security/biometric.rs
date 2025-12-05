// Biometric authentication for Kāraṇa OS

use super::SecurityError;
use std::collections::HashMap;

/// Biometric authentication manager
pub struct BiometricAuth {
    enrolled_users: HashMap<String, UserBiometricProfile>,
    config: BiometricConfig,
    active_sensors: Vec<BiometricSensor>,
}

/// Configuration for biometric authentication
#[derive(Debug, Clone)]
pub struct BiometricConfig {
    /// Minimum confidence threshold (0.0 - 1.0)
    pub confidence_threshold: f32,
    /// Enable liveness detection
    pub liveness_detection: bool,
    /// Maximum enrollment templates per type
    pub max_templates_per_type: u32,
    /// Template refresh interval (days)
    pub template_refresh_days: u32,
    /// Enable anti-spoofing
    pub anti_spoofing: bool,
    /// Supported biometric types
    pub supported_types: Vec<BiometricType>,
}

impl Default for BiometricConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.9,
            liveness_detection: true,
            max_templates_per_type: 5,
            template_refresh_days: 90,
            anti_spoofing: true,
            supported_types: vec![
                BiometricType::IrisScan,
                BiometricType::FacialRecognition,
                BiometricType::VoicePrint,
            ],
        }
    }
}

/// Types of biometric authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiometricType {
    IrisScan,
    FacialRecognition,
    VoicePrint,
    Fingerprint,
    RetinaScan,
    EyeMovement,
    GaitAnalysis,
}

impl BiometricType {
    pub fn sensor_name(&self) -> &'static str {
        match self {
            Self::IrisScan => "iris_scanner",
            Self::FacialRecognition => "face_camera",
            Self::VoicePrint => "microphone",
            Self::Fingerprint => "fingerprint_sensor",
            Self::RetinaScan => "retina_scanner",
            Self::EyeMovement => "eye_tracker",
            Self::GaitAnalysis => "motion_sensor",
        }
    }
    
    pub fn requires_camera(&self) -> bool {
        matches!(self, 
            Self::IrisScan | 
            Self::FacialRecognition | 
            Self::RetinaScan |
            Self::EyeMovement
        )
    }
}

/// Biometric data for authentication
#[derive(Debug, Clone)]
pub struct BiometricData {
    pub biometric_type: BiometricType,
    pub template: Vec<u8>,
    pub quality_score: f32,
    pub captured_at: u64,
    pub sensor_id: String,
    pub metadata: BiometricMetadata,
}

/// Additional metadata for biometric samples
#[derive(Debug, Clone)]
pub struct BiometricMetadata {
    pub lighting_conditions: Option<LightingCondition>,
    pub distance_mm: Option<f32>,
    pub ambient_noise_db: Option<f32>,
    pub liveness_score: Option<f32>,
}

impl Default for BiometricMetadata {
    fn default() -> Self {
        Self {
            lighting_conditions: None,
            distance_mm: None,
            ambient_noise_db: None,
            liveness_score: None,
        }
    }
}

/// Lighting condition during capture
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightingCondition {
    Optimal,
    Bright,
    Dim,
    Mixed,
    Unknown,
}

/// User's enrolled biometric profile
#[derive(Debug, Clone)]
pub struct UserBiometricProfile {
    pub user_id: String,
    pub templates: HashMap<BiometricType, Vec<BiometricTemplate>>,
    pub created_at: u64,
    pub last_updated: u64,
    pub enrollment_complete: bool,
}

/// Stored biometric template
#[derive(Debug, Clone)]
pub struct BiometricTemplate {
    pub id: String,
    pub template_data: Vec<u8>,
    pub enrolled_at: u64,
    pub last_matched: Option<u64>,
    pub match_count: u32,
    pub quality_score: f32,
}

/// Biometric sensor information
#[derive(Debug, Clone)]
pub struct BiometricSensor {
    pub id: String,
    pub sensor_type: BiometricType,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub status: SensorStatus,
    pub capabilities: SensorCapabilities,
}

/// Sensor status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorStatus {
    Ready,
    Busy,
    Calibrating,
    Error,
    Disabled,
}

/// Sensor capabilities
#[derive(Debug, Clone)]
pub struct SensorCapabilities {
    pub max_resolution: (u32, u32),
    pub supports_liveness: bool,
    pub supports_anti_spoofing: bool,
    pub min_capture_time_ms: u32,
    pub max_capture_time_ms: u32,
}

impl BiometricAuth {
    pub fn new() -> Self {
        Self {
            enrolled_users: HashMap::new(),
            config: BiometricConfig::default(),
            active_sensors: Vec::new(),
        }
    }
    
    pub fn with_config(mut self, config: BiometricConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Enroll biometric data for a user
    pub fn enroll(&mut self, user_id: &str, data: &BiometricData) -> Result<(), SecurityError> {
        // Validate biometric type is supported
        if !self.config.supported_types.contains(&data.biometric_type) {
            return Err(SecurityError::BiometricEnrollmentFailed);
        }
        
        // Check quality
        if data.quality_score < 0.7 {
            return Err(SecurityError::BiometricEnrollmentFailed);
        }
        
        // Check liveness if enabled
        if self.config.liveness_detection {
            if let Some(liveness) = data.metadata.liveness_score {
                if liveness < 0.8 {
                    return Err(SecurityError::BiometricEnrollmentFailed);
                }
            }
        }
        
        // Get or create user profile
        let profile = self.enrolled_users
            .entry(user_id.to_string())
            .or_insert_with(|| UserBiometricProfile {
                user_id: user_id.to_string(),
                templates: HashMap::new(),
                created_at: self.current_timestamp(),
                last_updated: self.current_timestamp(),
                enrollment_complete: false,
            });
        
        // Get or create template list for this type
        let templates = profile.templates
            .entry(data.biometric_type)
            .or_insert_with(Vec::new);
        
        // Check max templates
        if templates.len() >= self.config.max_templates_per_type as usize {
            // Remove oldest template
            templates.sort_by_key(|t| t.enrolled_at);
            templates.remove(0);
        }
        
        // Add new template
        templates.push(BiometricTemplate {
            id: format!("{}_{}", data.biometric_type.sensor_name(), self.current_timestamp()),
            template_data: data.template.clone(),
            enrolled_at: self.current_timestamp(),
            last_matched: None,
            match_count: 0,
            quality_score: data.quality_score,
        });
        
        profile.last_updated = self.current_timestamp();
        
        Ok(())
    }
    
    /// Verify biometric data against enrolled templates
    pub fn verify(&self, user_id: &str, data: &BiometricData) -> bool {
        let profile = match self.enrolled_users.get(user_id) {
            Some(p) => p,
            None => return false,
        };
        
        let templates = match profile.templates.get(&data.biometric_type) {
            Some(t) => t,
            None => return false,
        };
        
        // Check quality threshold
        if data.quality_score < 0.5 {
            return false;
        }
        
        // Check liveness if enabled
        if self.config.liveness_detection {
            if let Some(liveness) = data.metadata.liveness_score {
                if liveness < 0.8 {
                    return false;
                }
            }
        }
        
        // Match against stored templates
        for template in templates {
            let score = self.compare_templates(&template.template_data, &data.template);
            if score >= self.config.confidence_threshold {
                return true;
            }
        }
        
        false
    }
    
    /// Compare two biometric templates
    fn compare_templates(&self, template1: &[u8], template2: &[u8]) -> f32 {
        // Simulated comparison - would use actual biometric matching algorithm
        if template1 == template2 {
            1.0
        } else if template1.len() == template2.len() {
            // Simple similarity for simulation
            let matches: usize = template1.iter()
                .zip(template2.iter())
                .filter(|(a, b)| a == b)
                .count();
            matches as f32 / template1.len() as f32
        } else {
            0.0
        }
    }
    
    /// Check if user has any biometric enrolled
    pub fn is_enrolled(&self, user_id: &str) -> bool {
        self.enrolled_users.get(user_id)
            .map(|p| !p.templates.is_empty())
            .unwrap_or(false)
    }
    
    /// Check if specific biometric type is enrolled
    pub fn is_type_enrolled(&self, user_id: &str, biometric_type: BiometricType) -> bool {
        self.enrolled_users.get(user_id)
            .and_then(|p| p.templates.get(&biometric_type))
            .map(|t| !t.is_empty())
            .unwrap_or(false)
    }
    
    /// Get enrolled biometric types for user
    pub fn enrolled_types(&self, user_id: &str) -> Vec<BiometricType> {
        self.enrolled_users.get(user_id)
            .map(|p| p.templates.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Remove all biometric data for user
    pub fn unenroll(&mut self, user_id: &str) -> bool {
        self.enrolled_users.remove(user_id).is_some()
    }
    
    /// Remove specific biometric type for user
    pub fn unenroll_type(&mut self, user_id: &str, biometric_type: BiometricType) -> bool {
        if let Some(profile) = self.enrolled_users.get_mut(user_id) {
            profile.templates.remove(&biometric_type).is_some()
        } else {
            false
        }
    }
    
    /// Register a biometric sensor
    pub fn register_sensor(&mut self, sensor: BiometricSensor) {
        self.active_sensors.push(sensor);
    }
    
    /// Get available sensors
    pub fn available_sensors(&self) -> Vec<&BiometricSensor> {
        self.active_sensors
            .iter()
            .filter(|s| s.status == SensorStatus::Ready)
            .collect()
    }
    
    /// Get sensor for specific biometric type
    pub fn sensor_for_type(&self, biometric_type: BiometricType) -> Option<&BiometricSensor> {
        self.active_sensors
            .iter()
            .find(|s| s.sensor_type == biometric_type && s.status == SensorStatus::Ready)
    }
    
    /// Get enrollment template count for user
    pub fn template_count(&self, user_id: &str) -> usize {
        self.enrolled_users.get(user_id)
            .map(|p| p.templates.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }
    
    fn current_timestamp(&self) -> u64 {
        0 // Simulated
    }
}

impl Default for BiometricAuth {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of biometric verification
#[derive(Debug, Clone)]
pub struct BiometricVerificationResult {
    pub success: bool,
    pub biometric_type: BiometricType,
    pub confidence: f32,
    pub liveness_passed: bool,
    pub anti_spoofing_passed: bool,
    pub matched_template_id: Option<String>,
    pub capture_quality: f32,
}

/// Biometric capture session
pub struct BiometricCaptureSession {
    pub session_id: String,
    pub user_id: String,
    pub biometric_type: BiometricType,
    pub started_at: u64,
    pub captures: Vec<BiometricCapture>,
    pub status: CaptureSessionStatus,
}

/// Individual capture in session
#[derive(Debug, Clone)]
pub struct BiometricCapture {
    pub capture_id: String,
    pub data: Vec<u8>,
    pub quality_score: f32,
    pub captured_at: u64,
    pub feedback: Option<CaptureFeedback>,
}

/// Status of capture session
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureSessionStatus {
    InProgress,
    Complete,
    Failed,
    Cancelled,
}

/// Feedback for capture quality
#[derive(Debug, Clone)]
pub struct CaptureFeedback {
    pub overall_quality: CaptureQuality,
    pub issues: Vec<CaptureIssue>,
    pub suggestions: Vec<String>,
}

/// Overall capture quality assessment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaptureQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Unusable,
}

/// Issues detected during capture
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureIssue {
    TooFar,
    TooClose,
    OffCenter,
    MotionBlur,
    PoorLighting,
    Obstruction,
    LowContrast,
    NoiseDetected,
}

impl BiometricCaptureSession {
    pub fn new(user_id: &str, biometric_type: BiometricType) -> Self {
        Self {
            session_id: format!("capture_{}", 0), // Would use real timestamp
            user_id: user_id.to_string(),
            biometric_type,
            started_at: 0,
            captures: Vec::new(),
            status: CaptureSessionStatus::InProgress,
        }
    }
    
    pub fn add_capture(&mut self, capture: BiometricCapture) {
        self.captures.push(capture);
    }
    
    pub fn best_capture(&self) -> Option<&BiometricCapture> {
        self.captures.iter().max_by(|a, b| {
            a.quality_score.partial_cmp(&b.quality_score).unwrap()
        })
    }
    
    pub fn complete(&mut self) {
        self.status = CaptureSessionStatus::Complete;
    }
    
    pub fn fail(&mut self) {
        self.status = CaptureSessionStatus::Failed;
    }
    
    pub fn cancel(&mut self) {
        self.status = CaptureSessionStatus::Cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_biometric_data(biometric_type: BiometricType) -> BiometricData {
        BiometricData {
            biometric_type,
            template: vec![1, 2, 3, 4, 5],
            quality_score: 0.9,
            captured_at: 0,
            sensor_id: "test_sensor".to_string(),
            metadata: BiometricMetadata {
                liveness_score: Some(0.95),
                ..Default::default()
            },
        }
    }
    
    #[test]
    fn test_biometric_auth_creation() {
        let auth = BiometricAuth::new();
        assert!(!auth.is_enrolled("user1"));
    }
    
    #[test]
    fn test_biometric_type_properties() {
        assert!(BiometricType::IrisScan.requires_camera());
        assert!(BiometricType::FacialRecognition.requires_camera());
        assert!(!BiometricType::VoicePrint.requires_camera());
        
        assert_eq!(BiometricType::IrisScan.sensor_name(), "iris_scanner");
    }
    
    #[test]
    fn test_enroll_biometric() {
        let mut auth = BiometricAuth::new();
        
        let data = create_test_biometric_data(BiometricType::IrisScan);
        let result = auth.enroll("user1", &data);
        
        assert!(result.is_ok());
        assert!(auth.is_enrolled("user1"));
        assert!(auth.is_type_enrolled("user1", BiometricType::IrisScan));
    }
    
    #[test]
    fn test_verify_biometric() {
        let mut auth = BiometricAuth::new();
        
        let data = create_test_biometric_data(BiometricType::IrisScan);
        auth.enroll("user1", &data).unwrap();
        
        // Verify with same data
        assert!(auth.verify("user1", &data));
        
        // Verify with different data
        let mut different_data = data.clone();
        different_data.template = vec![9, 8, 7, 6, 5];
        assert!(!auth.verify("user1", &different_data));
    }
    
    #[test]
    fn test_enroll_low_quality() {
        let mut auth = BiometricAuth::new();
        
        let mut data = create_test_biometric_data(BiometricType::IrisScan);
        data.quality_score = 0.3; // Below threshold
        
        let result = auth.enroll("user1", &data);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_enrolled_types() {
        let mut auth = BiometricAuth::new();
        
        auth.enroll("user1", &create_test_biometric_data(BiometricType::IrisScan)).unwrap();
        auth.enroll("user1", &create_test_biometric_data(BiometricType::VoicePrint)).unwrap();
        
        let types = auth.enrolled_types("user1");
        assert_eq!(types.len(), 2);
    }
    
    #[test]
    fn test_unenroll() {
        let mut auth = BiometricAuth::new();
        
        let data = create_test_biometric_data(BiometricType::IrisScan);
        auth.enroll("user1", &data).unwrap();
        
        assert!(auth.is_enrolled("user1"));
        assert!(auth.unenroll("user1"));
        assert!(!auth.is_enrolled("user1"));
    }
    
    #[test]
    fn test_unenroll_type() {
        let mut auth = BiometricAuth::new();
        
        auth.enroll("user1", &create_test_biometric_data(BiometricType::IrisScan)).unwrap();
        auth.enroll("user1", &create_test_biometric_data(BiometricType::VoicePrint)).unwrap();
        
        assert!(auth.unenroll_type("user1", BiometricType::IrisScan));
        assert!(!auth.is_type_enrolled("user1", BiometricType::IrisScan));
        assert!(auth.is_type_enrolled("user1", BiometricType::VoicePrint));
    }
    
    #[test]
    fn test_register_sensor() {
        let mut auth = BiometricAuth::new();
        
        auth.register_sensor(BiometricSensor {
            id: "sensor1".to_string(),
            sensor_type: BiometricType::IrisScan,
            manufacturer: "Test".to_string(),
            model: "Model1".to_string(),
            firmware_version: "1.0".to_string(),
            status: SensorStatus::Ready,
            capabilities: SensorCapabilities {
                max_resolution: (1920, 1080),
                supports_liveness: true,
                supports_anti_spoofing: true,
                min_capture_time_ms: 100,
                max_capture_time_ms: 500,
            },
        });
        
        assert_eq!(auth.available_sensors().len(), 1);
        assert!(auth.sensor_for_type(BiometricType::IrisScan).is_some());
    }
    
    #[test]
    fn test_capture_session() {
        let mut session = BiometricCaptureSession::new("user1", BiometricType::IrisScan);
        
        session.add_capture(BiometricCapture {
            capture_id: "cap1".to_string(),
            data: vec![1, 2, 3],
            quality_score: 0.7,
            captured_at: 0,
            feedback: None,
        });
        
        session.add_capture(BiometricCapture {
            capture_id: "cap2".to_string(),
            data: vec![4, 5, 6],
            quality_score: 0.9,
            captured_at: 1,
            feedback: None,
        });
        
        let best = session.best_capture().unwrap();
        assert_eq!(best.quality_score, 0.9);
    }
    
    #[test]
    fn test_capture_quality() {
        assert_ne!(CaptureQuality::Excellent, CaptureQuality::Poor);
    }
    
    #[test]
    fn test_biometric_config_default() {
        let config = BiometricConfig::default();
        assert!(config.liveness_detection);
        assert_eq!(config.confidence_threshold, 0.9);
    }
}
