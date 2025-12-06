// Phase 54: Capability Trait System
//
// Defines capability-based interfaces for each layer to:
// 1. Eliminate direct struct field access between layers
// 2. Enable parallel development of layers
// 3. Support layer swapping without orchestrator changes
// 4. Provide versioned API contracts

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::any::Any;

/// Capability version for API compatibility checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl CapabilityVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
    
    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

/// Base trait that all layer capabilities must implement
#[async_trait]
pub trait LayerCapability: Send + Sync {
    /// Name of this capability
    fn name(&self) -> &str;
    
    /// Version of this capability's API
    fn version(&self) -> CapabilityVersion;
    
    /// Layer number (1-9)
    fn layer(&self) -> u8;
    
    /// Initialize this capability
    async fn init(&mut self) -> Result<()>;
    
    /// Shutdown this capability
    async fn shutdown(&mut self) -> Result<()>;
    
    /// Get current status
    async fn status(&self) -> CapabilityStatus;
    
    /// Handle a message from another layer
    async fn handle_message(&mut self, msg: LayerMessage) -> Result<LayerMessage>;
    
    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Cast to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Status of a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStatus {
    pub state: CapabilityState,
    pub health: HealthStatus,
    pub message: String,
    pub metrics: CapabilityMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState {
    Uninitialized,
    Initializing,
    Ready,
    Degraded,
    Error,
    ShuttingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetrics {
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub message_count: u64,
    pub error_count: u64,
    pub last_activity: u64,
}

/// Message passed between layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMessage {
    /// Unique message ID
    pub id: String,
    
    /// Source layer (1-9)
    pub from: u8,
    
    /// Destination layer (1-9, or 0 for broadcast)
    pub to: u8,
    
    /// Message type
    pub msg_type: MessageType,
    
    /// Message payload
    pub payload: Vec<u8>,
    
    /// Priority
    pub priority: MessagePriority,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Correlation ID (for request-response)
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Command,
    Query,
    Notification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Critical = 4,
    High = 3,
    Normal = 2,
    Low = 1,
    Background = 0,
}

impl LayerMessage {
    pub fn new(from: u8, to: u8, msg_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            msg_type,
            payload,
            priority: MessagePriority::Normal,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            correlation_id: None,
        }
    }
    
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_correlation(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn as_response(&self, payload: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from: self.to,
            to: self.from,
            msg_type: MessageType::Response,
            payload,
            priority: self.priority,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            correlation_id: Some(self.id.clone()),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Specific Layer Capabilities
// ═══════════════════════════════════════════════════════════════════════

/// Layer 1: Hardware Capability
#[async_trait]
pub trait HardwareCapability: LayerCapability {
    async fn capture_camera_frame(&self) -> Result<Vec<u8>>;
    async fn get_sensor_data(&self) -> Result<SensorData>;
    async fn render_display(&mut self, frame: &[u8]) -> Result<()>;
    async fn play_haptic(&mut self, pattern: HapticPattern) -> Result<()>;
    async fn get_power_status(&self) -> Result<PowerStatus>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub imu: ImuData,
    pub gps: Option<GpsData>,
    pub ambient_light: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImuData {
    pub accel: [f32; 3],
    pub gyro: [f32; 3],
    pub mag: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsData {
    pub lat: f64,
    pub lon: f64,
    pub alt: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticPattern {
    pub name: String,
    pub duration_ms: u64,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStatus {
    pub battery_percent: f32,
    pub is_charging: bool,
    pub temperature_c: f32,
}

/// Layer 2: Network Capability
#[async_trait]
pub trait NetworkCapability: LayerCapability {
    async fn discover_peers(&mut self) -> Result<Vec<PeerInfo>>;
    async fn connect_peer(&mut self, peer_id: &str) -> Result<()>;
    async fn broadcast_message(&mut self, topic: &str, data: &[u8]) -> Result<()>;
    async fn get_network_stats(&self) -> Result<NetworkStats>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub peer_count: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Layer 3: Blockchain Capability
#[async_trait]
pub trait BlockchainCapability: LayerCapability {
    async fn submit_transaction(&mut self, tx: TransactionData) -> Result<String>;
    async fn get_balance(&self, address: &str) -> Result<u128>;
    async fn get_block(&self, height: u64) -> Result<Option<BlockData>>;
    async fn get_chain_height(&self) -> Result<u64>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub from: String,
    pub to: Option<String>,
    pub amount: u128,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub tx_count: usize,
}

/// Layer 4: Oracle Capability
#[async_trait]
pub trait OracleCapability: LayerCapability {
    async fn classify_intent(&self, text: &str) -> Result<IntentClassification>;
    async fn generate_proof(&mut self, intent: &IntentClassification) -> Result<Vec<u8>>;
    async fn query_external(&self, query: &str) -> Result<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub intent_type: String,
    pub confidence: f32,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: String,
    pub value: String,
    pub confidence: f32,
}

/// Layer 5: Intelligence Capability
#[async_trait]
pub trait IntelligenceCapability: LayerCapability {
    async fn fuse_inputs(&mut self, inputs: MultimodalInputs) -> Result<FusedContext>;
    async fn analyze_scene(&self, image: &[u8]) -> Result<SceneAnalysis>;
    async fn retrieve_context(&self, query: &str) -> Result<Vec<ContextItem>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalInputs {
    pub audio: Option<Vec<u8>>,
    pub image: Option<Vec<u8>>,
    pub gaze: Option<[f32; 2]>,
    pub gesture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedContext {
    pub summary: String,
    pub confidence: f32,
    pub modalities_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAnalysis {
    pub description: String,
    pub objects: Vec<DetectedObject>,
    pub text: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedObject {
    pub label: String,
    pub confidence: f32,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub text: String,
    pub relevance: f32,
    pub source: String,
}

/// Layer 6: AI Engine Capability
#[async_trait]
pub trait AIEngineCapability: LayerCapability {
    async fn process_language(&self, text: &str) -> Result<LanguageUnderstanding>;
    async fn generate_response(&self, context: &str) -> Result<String>;
    async fn execute_action(&mut self, action: &ActionPlan) -> Result<ActionResult>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageUnderstanding {
    pub intent: String,
    pub entities: Vec<Entity>,
    pub sentiment: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub steps: Vec<ActionStep>,
    pub estimated_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    pub action: String,
    pub parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub data: Vec<u8>,
}

/// Layer 7: Interface Capability
#[async_trait]
pub trait InterfaceCapability: LayerCapability {
    async fn detect_voice(&self, audio: &[u8]) -> Result<bool>;
    async fn transcribe(&self, audio: &[u8]) -> Result<String>;
    async fn track_gaze(&self) -> Result<[f32; 2]>;
    async fn recognize_gesture(&self, image: &[u8]) -> Result<Option<String>>;
    async fn render_hud(&mut self, elements: &[HudElement]) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudElement {
    pub element_type: String,
    pub content: String,
    pub position: [f32; 2],
}

/// Layer 8: Applications Capability
#[async_trait]
pub trait ApplicationsCapability: LayerCapability {
    async fn list_apps(&self) -> Result<Vec<AppInfo>>;
    async fn launch_app(&mut self, app_id: &str) -> Result<()>;
    async fn send_to_app(&mut self, app_id: &str, data: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub is_running: bool,
}

/// Layer 9: System Services Capability
#[async_trait]
pub trait SystemServicesCapability: LayerCapability {
    async fn run_diagnostics(&self) -> Result<DiagnosticsReport>;
    async fn initiate_recovery(&mut self, strategy: RecoveryStrategy) -> Result<()>;
    async fn check_for_updates(&self) -> Result<Option<UpdateInfo>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub overall_health: HealthStatus,
    pub issues: Vec<DiagnosticIssue>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    pub severity: HealthStatus,
    pub component: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    Restart,
    Minimal,
    SafeMode,
    FactoryReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub size_mb: f64,
    pub changelog: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_compatibility() {
        let v1 = CapabilityVersion::new(1, 0, 0);
        let v2 = CapabilityVersion::new(1, 1, 0);
        let v3 = CapabilityVersion::new(2, 0, 0);
        
        assert!(v2.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v2));
        assert!(!v3.is_compatible_with(&v1));
    }
    
    #[test]
    fn test_message_creation() {
        let msg = LayerMessage::new(1, 2, MessageType::Request, vec![1, 2, 3]);
        
        assert_eq!(msg.from, 1);
        assert_eq!(msg.to, 2);
        assert_eq!(msg.msg_type, MessageType::Request);
        assert_eq!(msg.priority, MessagePriority::Normal);
    }
    
    #[test]
    fn test_message_response() {
        let request = LayerMessage::new(1, 2, MessageType::Request, vec![1, 2, 3]);
        let response = request.as_response(vec![4, 5, 6]);
        
        assert_eq!(response.from, 2);
        assert_eq!(response.to, 1);
        assert_eq!(response.msg_type, MessageType::Response);
        assert_eq!(response.correlation_id, Some(request.id));
    }
    
    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }
}
