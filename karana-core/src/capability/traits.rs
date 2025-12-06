// Capability-Based Layer Interface
// Phase 47: Decouple layers with capability negotiation

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Layer identifier in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerId {
    Hardware,
    P2P,
    Ledger,
    Oracle,
    Intelligence,
    AI,
    Interface,
    Apps,
    System,
}

impl LayerId {
    pub fn name(&self) -> &'static str {
        match self {
            LayerId::Hardware => "hardware",
            LayerId::P2P => "p2p",
            LayerId::Ledger => "ledger",
            LayerId::Oracle => "oracle",
            LayerId::Intelligence => "intelligence",
            LayerId::AI => "ai",
            LayerId::Interface => "interface",
            LayerId::Apps => "apps",
            LayerId::System => "system",
        }
    }
    
    /// Get all layers in dependency order
    pub fn all() -> Vec<LayerId> {
        vec![
            LayerId::Hardware,
            LayerId::P2P,
            LayerId::Ledger,
            LayerId::Oracle,
            LayerId::Intelligence,
            LayerId::AI,
            LayerId::Interface,
            LayerId::Apps,
            LayerId::System,
        ]
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Capability that a layer can provide
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // Hardware capabilities
    CameraCapture,
    AudioCapture,
    Display,
    Sensors,
    
    // Network capabilities
    P2PMessaging,
    InternetAccess,
    CloudStorage,
    
    // Blockchain capabilities
    TransactionProcessing,
    SmartContracts,
    StateManagement,
    
    // Oracle capabilities
    DataQuery,
    IntentProcessing,
    CommandExecution,
    
    // Intelligence capabilities
    ContextTracking,
    Learning,
    Prediction,
    
    // AI capabilities
    VisionProcessing,
    SpeechRecognition,
    NaturalLanguageUnderstanding,
    KnowledgeRetrieval,
    
    // Interface capabilities
    ARRendering,
    HUDDisplay,
    VoiceInterface,
    GestureTracking,
    
    // App capabilities
    AppExecution,
    AppStore,
    
    // System capabilities
    ResourceManagement,
    HealthMonitoring,
    FaultRecovery,
    
    // Custom capability
    Custom(String),
}

impl Capability {
    pub fn name(&self) -> String {
        match self {
            Capability::CameraCapture => "camera_capture".to_string(),
            Capability::AudioCapture => "audio_capture".to_string(),
            Capability::Display => "display".to_string(),
            Capability::Sensors => "sensors".to_string(),
            Capability::P2PMessaging => "p2p_messaging".to_string(),
            Capability::InternetAccess => "internet_access".to_string(),
            Capability::CloudStorage => "cloud_storage".to_string(),
            Capability::TransactionProcessing => "transaction_processing".to_string(),
            Capability::SmartContracts => "smart_contracts".to_string(),
            Capability::StateManagement => "state_management".to_string(),
            Capability::DataQuery => "data_query".to_string(),
            Capability::IntentProcessing => "intent_processing".to_string(),
            Capability::CommandExecution => "command_execution".to_string(),
            Capability::ContextTracking => "context_tracking".to_string(),
            Capability::Learning => "learning".to_string(),
            Capability::Prediction => "prediction".to_string(),
            Capability::VisionProcessing => "vision_processing".to_string(),
            Capability::SpeechRecognition => "speech_recognition".to_string(),
            Capability::NaturalLanguageUnderstanding => "natural_language_understanding".to_string(),
            Capability::KnowledgeRetrieval => "knowledge_retrieval".to_string(),
            Capability::ARRendering => "ar_rendering".to_string(),
            Capability::HUDDisplay => "hud_display".to_string(),
            Capability::VoiceInterface => "voice_interface".to_string(),
            Capability::GestureTracking => "gesture_tracking".to_string(),
            Capability::AppExecution => "app_execution".to_string(),
            Capability::AppStore => "app_store".to_string(),
            Capability::ResourceManagement => "resource_management".to_string(),
            Capability::HealthMonitoring => "health_monitoring".to_string(),
            Capability::FaultRecovery => "fault_recovery".to_string(),
            Capability::Custom(name) => name.clone(),
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Capability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirements {
    /// Required capabilities
    pub required: Vec<Capability>,
    
    /// Optional capabilities
    pub optional: Vec<Capability>,
    
    /// Minimum version required
    pub min_version: Option<String>,
}

impl CapabilityRequirements {
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            optional: Vec::new(),
            min_version: None,
        }
    }
    
    pub fn with_required(mut self, cap: Capability) -> Self {
        self.required.push(cap);
        self
    }
    
    pub fn with_optional(mut self, cap: Capability) -> Self {
        self.optional.push(cap);
        self
    }
}

impl Default for CapabilityRequirements {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability advertisement from a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAdvertisement {
    /// Layer providing the capabilities
    pub layer: LayerId,
    
    /// Capabilities provided
    pub capabilities: Vec<Capability>,
    
    /// Version of the layer
    pub version: String,
    
    /// Current load (0.0 to 1.0)
    pub load: f32,
    
    /// Whether layer is healthy
    pub healthy: bool,
}

/// Layer lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerState {
    Uninitialized,
    Initializing,
    Ready,
    Degraded,
    Failed,
    Shutdown,
}

/// Base trait for all layers
#[async_trait]
pub trait Layer: Send + Sync {
    /// Get layer identifier
    fn id(&self) -> LayerId;
    
    /// Get layer name
    fn name(&self) -> &str {
        self.id().name()
    }
    
    /// Get layer version
    fn version(&self) -> String {
        "0.1.0".to_string()
    }
    
    /// Get current state
    async fn state(&self) -> LayerState;
    
    /// Initialize the layer
    async fn initialize(&mut self) -> Result<()>;
    
    /// Shutdown the layer
    async fn shutdown(&mut self) -> Result<()>;
    
    /// Get capabilities provided by this layer
    fn capabilities(&self) -> Vec<Capability>;
    
    /// Check if layer can provide a capability
    fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities().contains(capability)
    }
    
    /// Get capability requirements
    fn requirements(&self) -> CapabilityRequirements {
        CapabilityRequirements::new()
    }
    
    /// Advertise capabilities
    async fn advertise(&self) -> CapabilityAdvertisement {
        CapabilityAdvertisement {
            layer: self.id(),
            capabilities: self.capabilities(),
            version: self.version(),
            load: self.current_load().await,
            healthy: matches!(self.state().await, LayerState::Ready),
        }
    }
    
    /// Get current load (0.0 to 1.0)
    async fn current_load(&self) -> f32 {
        0.0
    }
    
    /// Handle layer degradation
    async fn degrade(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Recover from degradation
    async fn recover(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Health check
    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

/// Capability registry for layer discovery
pub struct CapabilityRegistry {
    /// Map of capabilities to layers that provide them
    providers: HashMap<Capability, Vec<LayerId>>,
    
    /// Map of layers to their advertisements
    advertisements: HashMap<LayerId, CapabilityAdvertisement>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            advertisements: HashMap::new(),
        }
    }
    
    /// Register a layer's capabilities
    pub fn register(&mut self, advertisement: CapabilityAdvertisement) {
        let layer_id = advertisement.layer;
        
        // Update providers map
        for capability in &advertisement.capabilities {
            self.providers
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(layer_id);
        }
        
        // Store advertisement
        self.advertisements.insert(layer_id, advertisement);
    }
    
    /// Unregister a layer
    pub fn unregister(&mut self, layer_id: LayerId) {
        // Remove from providers
        for providers in self.providers.values_mut() {
            providers.retain(|&id| id != layer_id);
        }
        
        // Remove advertisement
        self.advertisements.remove(&layer_id);
    }
    
    /// Find layers that provide a capability
    pub fn find_providers(&self, capability: &Capability) -> Vec<LayerId> {
        self.providers
            .get(capability)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Find best provider for a capability
    pub fn find_best_provider(&self, capability: &Capability) -> Option<LayerId> {
        let providers = self.find_providers(capability);
        
        // Select provider with lowest load and healthy status
        providers
            .into_iter()
            .filter_map(|layer_id| {
                self.advertisements.get(&layer_id).map(|ad| (layer_id, ad))
            })
            .filter(|(_, ad)| ad.healthy)
            .min_by(|(_, a), (_, b)| {
                a.load.partial_cmp(&b.load).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(layer_id, _)| layer_id)
    }
    
    /// Check if requirements can be satisfied
    pub fn can_satisfy(&self, requirements: &CapabilityRequirements) -> bool {
        requirements
            .required
            .iter()
            .all(|cap| !self.find_providers(cap).is_empty())
    }
    
    /// Get advertisement for a layer
    pub fn get_advertisement(&self, layer_id: LayerId) -> Option<&CapabilityAdvertisement> {
        self.advertisements.get(&layer_id)
    }
    
    /// Get all registered layers
    pub fn registered_layers(&self) -> Vec<LayerId> {
        self.advertisements.keys().copied().collect()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layer_id_display() {
        assert_eq!(LayerId::Hardware.to_string(), "hardware");
        assert_eq!(LayerId::Oracle.to_string(), "oracle");
    }
    
    #[test]
    fn test_capability_name() {
        assert_eq!(Capability::CameraCapture.name(), "camera_capture");
        assert_eq!(Capability::Custom("test".to_string()).name(), "test");
    }
    
    #[test]
    fn test_capability_requirements() {
        let reqs = CapabilityRequirements::new()
            .with_required(Capability::CameraCapture)
            .with_optional(Capability::Display);
        
        assert_eq!(reqs.required.len(), 1);
        assert_eq!(reqs.optional.len(), 1);
    }
    
    #[test]
    fn test_capability_registry() {
        let mut registry = CapabilityRegistry::new();
        
        let ad = CapabilityAdvertisement {
            layer: LayerId::Hardware,
            capabilities: vec![Capability::CameraCapture, Capability::Sensors],
            version: "0.1.0".to_string(),
            load: 0.3,
            healthy: true,
        };
        
        registry.register(ad);
        
        let providers = registry.find_providers(&Capability::CameraCapture);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0], LayerId::Hardware);
    }
    
    #[test]
    fn test_find_best_provider() {
        let mut registry = CapabilityRegistry::new();
        
        // Register two providers with different loads
        registry.register(CapabilityAdvertisement {
            layer: LayerId::Hardware,
            capabilities: vec![Capability::CameraCapture],
            version: "0.1.0".to_string(),
            load: 0.8,
            healthy: true,
        });
        
        registry.register(CapabilityAdvertisement {
            layer: LayerId::System,
            capabilities: vec![Capability::CameraCapture],
            version: "0.1.0".to_string(),
            load: 0.2,
            healthy: true,
        });
        
        let best = registry.find_best_provider(&Capability::CameraCapture);
        assert_eq!(best, Some(LayerId::System)); // Lower load
    }
    
    #[test]
    fn test_can_satisfy_requirements() {
        let mut registry = CapabilityRegistry::new();
        
        registry.register(CapabilityAdvertisement {
            layer: LayerId::Hardware,
            capabilities: vec![Capability::CameraCapture, Capability::Display],
            version: "0.1.0".to_string(),
            load: 0.3,
            healthy: true,
        });
        
        let reqs = CapabilityRequirements::new()
            .with_required(Capability::CameraCapture)
            .with_optional(Capability::Display);
        
        assert!(registry.can_satisfy(&reqs));
        
        let reqs2 = CapabilityRequirements::new()
            .with_required(Capability::CloudStorage);
        
        assert!(!registry.can_satisfy(&reqs2));
    }
    
    #[test]
    fn test_unregister_layer() {
        let mut registry = CapabilityRegistry::new();
        
        registry.register(CapabilityAdvertisement {
            layer: LayerId::Hardware,
            capabilities: vec![Capability::CameraCapture],
            version: "0.1.0".to_string(),
            load: 0.3,
            healthy: true,
        });
        
        assert_eq!(registry.registered_layers().len(), 1);
        
        registry.unregister(LayerId::Hardware);
        assert_eq!(registry.registered_layers().len(), 0);
        assert!(registry.find_providers(&Capability::CameraCapture).is_empty());
    }
}
