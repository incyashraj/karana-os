// Event Bus Core - Message passing between layers
// Phase 47: Decouple layer communication

use crate::capability::{Capability, LayerId};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, RwLock};

/// Event identifier
pub type EventId = u64;

/// Event priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Event category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    // System events
    SystemStartup,
    SystemShutdown,
    LayerStateChange,
    ResourceChange,
    HealthChange,
    
    // Hardware events
    CameraFrame,
    AudioSample,
    SensorData,
    BatteryChange,
    ThermalChange,
    
    // Network events
    PeerConnected,
    PeerDisconnected,
    MessageReceived,
    NetworkError,
    
    // Blockchain events
    BlockMined,
    TransactionConfirmed,
    StateUpdated,
    
    // Oracle events
    IntentReceived,
    CommandExecuted,
    QueryResult,
    
    // AI events
    VisionDetection,
    SpeechTranscribed,
    IntentClassified,
    KnowledgeUpdated,
    
    // UI events
    UserInput,
    GestureDetected,
    GazeChanged,
    TabCreated,
    TabClosed,
    
    // App events
    AppLaunched,
    AppClosed,
    AppError,
    
    // Custom event
    Custom(String),
}

impl EventCategory {
    pub fn name(&self) -> String {
        match self {
            EventCategory::SystemStartup => "system_startup".to_string(),
            EventCategory::SystemShutdown => "system_shutdown".to_string(),
            EventCategory::LayerStateChange => "layer_state_change".to_string(),
            EventCategory::ResourceChange => "resource_change".to_string(),
            EventCategory::HealthChange => "health_change".to_string(),
            EventCategory::CameraFrame => "camera_frame".to_string(),
            EventCategory::AudioSample => "audio_sample".to_string(),
            EventCategory::SensorData => "sensor_data".to_string(),
            EventCategory::BatteryChange => "battery_change".to_string(),
            EventCategory::ThermalChange => "thermal_change".to_string(),
            EventCategory::PeerConnected => "peer_connected".to_string(),
            EventCategory::PeerDisconnected => "peer_disconnected".to_string(),
            EventCategory::MessageReceived => "message_received".to_string(),
            EventCategory::NetworkError => "network_error".to_string(),
            EventCategory::BlockMined => "block_mined".to_string(),
            EventCategory::TransactionConfirmed => "transaction_confirmed".to_string(),
            EventCategory::StateUpdated => "state_updated".to_string(),
            EventCategory::IntentReceived => "intent_received".to_string(),
            EventCategory::CommandExecuted => "command_executed".to_string(),
            EventCategory::QueryResult => "query_result".to_string(),
            EventCategory::VisionDetection => "vision_detection".to_string(),
            EventCategory::SpeechTranscribed => "speech_transcribed".to_string(),
            EventCategory::IntentClassified => "intent_classified".to_string(),
            EventCategory::KnowledgeUpdated => "knowledge_updated".to_string(),
            EventCategory::UserInput => "user_input".to_string(),
            EventCategory::GestureDetected => "gesture_detected".to_string(),
            EventCategory::GazeChanged => "gaze_changed".to_string(),
            EventCategory::TabCreated => "tab_created".to_string(),
            EventCategory::TabClosed => "tab_closed".to_string(),
            EventCategory::AppLaunched => "app_launched".to_string(),
            EventCategory::AppClosed => "app_closed".to_string(),
            EventCategory::AppError => "app_error".to_string(),
            EventCategory::Custom(name) => name.clone(),
        }
    }
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Event ID
    pub id: EventId,
    
    /// Source layer
    pub source: LayerId,
    
    /// Target layer (None for broadcast)
    pub target: Option<LayerId>,
    
    /// Event category
    pub category: EventCategory,
    
    /// Priority
    pub priority: EventPriority,
    
    /// Timestamp
    pub timestamp: SystemTime,
    
    /// Required capability to process
    pub required_capability: Option<Capability>,
    
    /// Trace ID for request tracking
    pub trace_id: Option<String>,
}

/// Event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    /// Empty event
    Empty,
    
    /// String data
    String(String),
    
    /// Binary data
    Binary(Vec<u8>),
    
    /// JSON data
    Json(serde_json::Value),
    
    /// Key-value pairs
    KeyValue(HashMap<String, String>),
}

/// Complete event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub metadata: EventMetadata,
    pub payload: EventPayload,
}

impl Event {
    pub fn new(
        source: LayerId,
        category: EventCategory,
        priority: EventPriority,
    ) -> Self {
        Self {
            metadata: EventMetadata {
                id: Self::generate_id(),
                source,
                target: None,
                category,
                priority,
                timestamp: SystemTime::now(),
                required_capability: None,
                trace_id: None,
            },
            payload: EventPayload::Empty,
        }
    }
    
    pub fn with_target(mut self, target: LayerId) -> Self {
        self.metadata.target = Some(target);
        self
    }
    
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.metadata.required_capability = Some(capability);
        self
    }
    
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.metadata.trace_id = Some(trace_id);
        self
    }
    
    pub fn with_payload(mut self, payload: EventPayload) -> Self {
        self.payload = payload;
        self
    }
    
    fn generate_id() -> EventId {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

/// Event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle an event (returns a boxed future for dyn compatibility)
    fn handle<'a>(&'a self, event: &'a Event) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Check if this handler is interested in the event
    fn interested_in(&self, event: &Event) -> bool;
}

/// Event subscription
pub struct EventSubscription {
    /// Subscriber layer
    pub layer: LayerId,
    
    /// Categories to subscribe to
    pub categories: Vec<EventCategory>,
    
    /// Minimum priority
    pub min_priority: EventPriority,
    
    /// Channel sender
    sender: broadcast::Sender<Event>,
}

/// Event bus statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventBusStatistics {
    pub total_events: u64,
    pub events_by_priority: HashMap<String, u64>,
    pub events_by_category: HashMap<String, u64>,
    pub dropped_events: u64,
    pub avg_processing_time_ms: f64,
}

/// Event bus for layer communication
pub struct EventBus {
    /// Event subscriptions
    subscriptions: Arc<RwLock<HashMap<LayerId, EventSubscription>>>,
    
    /// Event handlers
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
    
    /// Event history (for debugging)
    history: Arc<RwLock<Vec<Event>>>,
    
    /// Statistics
    stats: Arc<RwLock<EventBusStatistics>>,
    
    /// Maximum history size
    max_history: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(EventBusStatistics::default())),
            max_history: 1000,
        }
    }
    
    /// Subscribe a layer to events
    pub async fn subscribe(
        &self,
        layer: LayerId,
        categories: Vec<EventCategory>,
        min_priority: EventPriority,
    ) -> broadcast::Receiver<Event> {
        let (tx, rx) = broadcast::channel(100);
        
        let subscription = EventSubscription {
            layer,
            categories,
            min_priority,
            sender: tx,
        };
        
        self.subscriptions.write().await.insert(layer, subscription);
        rx
    }
    
    /// Unsubscribe a layer
    pub async fn unsubscribe(&self, layer: LayerId) {
        self.subscriptions.write().await.remove(&layer);
    }
    
    /// Register an event handler
    pub async fn register_handler(&self, handler: Box<dyn EventHandler>) {
        self.handlers.write().await.push(handler);
    }
    
    /// Publish an event
    pub async fn publish(&self, event: Event) -> Result<()> {
        let start = SystemTime::now();
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
            
            let priority_key = format!("{:?}", event.metadata.priority);
            *stats.events_by_priority.entry(priority_key).or_insert(0) += 1;
            
            let category_key = event.metadata.category.name();
            *stats.events_by_category.entry(category_key).or_insert(0) += 1;
        }
        
        // Add to history
        {
            let mut history = self.history.write().await;
            history.push(event.clone());
            let history_len = history.len();
            if history_len > self.max_history {
                history.drain(0..history_len - self.max_history);
            }
        }
        
        // Send to subscribers
        let subscriptions = self.subscriptions.read().await;
        for subscription in subscriptions.values() {
            // Check if subscriber is interested
            if !subscription.categories.is_empty()
                && !subscription.categories.contains(&event.metadata.category)
            {
                continue;
            }
            
            // Check priority
            if event.metadata.priority < subscription.min_priority {
                continue;
            }
            
            // Check if targeted
            if let Some(target) = event.metadata.target {
                if target != subscription.layer {
                    continue;
                }
            }
            
            // Send event
            if subscription.sender.send(event.clone()).is_err() {
                // Subscriber dropped, will be cleaned up later
            }
        }
        drop(subscriptions);
        
        // Invoke handlers
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            if handler.interested_in(&event) {
                if let Err(e) = handler.handle(&event).await {
                    eprintln!("Event handler error: {}", e);
                }
            }
        }
        drop(handlers);
        
        // Update processing time
        if let Ok(elapsed) = start.elapsed() {
            let mut stats = self.stats.write().await;
            let elapsed_ms = elapsed.as_millis() as f64;
            stats.avg_processing_time_ms = 
                (stats.avg_processing_time_ms * (stats.total_events - 1) as f64 + elapsed_ms)
                / stats.total_events as f64;
        }
        
        Ok(())
    }
    
    /// Get event history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<Event> {
        let history = self.history.read().await;
        if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            history.clone()
        }
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> EventBusStatistics {
        self.stats.read().await.clone()
    }
    
    /// Clear history
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_creation() {
        let event = Event::new(
            LayerId::Hardware,
            EventCategory::CameraFrame,
            EventPriority::Normal,
        );
        
        assert_eq!(event.metadata.source, LayerId::Hardware);
        assert_eq!(event.metadata.priority, EventPriority::Normal);
    }
    
    #[tokio::test]
    async fn test_event_bus_publish() {
        let bus = EventBus::new();
        
        let event = Event::new(
            LayerId::Hardware,
            EventCategory::CameraFrame,
            EventPriority::Normal,
        );
        
        bus.publish(event).await.unwrap();
        
        let stats = bus.get_statistics().await;
        assert_eq!(stats.total_events, 1);
    }
    
    #[tokio::test]
    async fn test_event_subscription() {
        let bus = EventBus::new();
        
        let mut rx = bus.subscribe(
            LayerId::AI,
            vec![EventCategory::CameraFrame],
            EventPriority::Normal,
        ).await;
        
        let event = Event::new(
            LayerId::Hardware,
            EventCategory::CameraFrame,
            EventPriority::Normal,
        );
        
        bus.publish(event.clone()).await.unwrap();
        
        let received = tokio::time::timeout(
            Duration::from_millis(100),
            rx.recv(),
        ).await.unwrap().unwrap();
        
        assert_eq!(received.metadata.category, event.metadata.category);
    }
    
    #[tokio::test]
    async fn test_priority_filtering() {
        let bus = EventBus::new();
        
        let mut rx = bus.subscribe(
            LayerId::AI,
            vec![],
            EventPriority::High,
        ).await;
        
        // Send low priority event
        let low_event = Event::new(
            LayerId::Hardware,
            EventCategory::CameraFrame,
            EventPriority::Normal,
        );
        bus.publish(low_event).await.unwrap();
        
        // Should not receive
        let result = tokio::time::timeout(
            Duration::from_millis(50),
            rx.recv(),
        ).await;
        assert!(result.is_err());
        
        // Send high priority event
        let high_event = Event::new(
            LayerId::Hardware,
            EventCategory::SystemShutdown,
            EventPriority::Critical,
        );
        bus.publish(high_event.clone()).await.unwrap();
        
        // Should receive
        let received = tokio::time::timeout(
            Duration::from_millis(100),
            rx.recv(),
        ).await.unwrap().unwrap();
        
        assert_eq!(received.metadata.priority, EventPriority::Critical);
    }
    
    #[tokio::test]
    async fn test_targeted_event() {
        let bus = EventBus::new();
        
        let mut rx1 = bus.subscribe(
            LayerId::AI,
            vec![EventCategory::CommandExecuted],
            EventPriority::Normal,
        ).await;
        
        let mut rx2 = bus.subscribe(
            LayerId::Interface,
            vec![EventCategory::CommandExecuted],
            EventPriority::Normal,
        ).await;
        
        // Send targeted event
        let event = Event::new(
            LayerId::Oracle,
            EventCategory::CommandExecuted,
            EventPriority::Normal,
        ).with_target(LayerId::AI);
        
        bus.publish(event).await.unwrap();
        
        // Only AI should receive
        let result1 = tokio::time::timeout(
            Duration::from_millis(100),
            rx1.recv(),
        ).await;
        assert!(result1.is_ok());
        
        let result2 = tokio::time::timeout(
            Duration::from_millis(50),
            rx2.recv(),
        ).await;
        assert!(result2.is_err());
    }
    
    #[tokio::test]
    async fn test_event_history() {
        let bus = EventBus::new();
        
        for i in 0..5 {
            let event = Event::new(
                LayerId::Hardware,
                EventCategory::SensorData,
                EventPriority::Normal,
            ).with_payload(EventPayload::String(format!("sensor_{}", i)));
            
            bus.publish(event).await.unwrap();
        }
        
        let history = bus.get_history(Some(3)).await;
        assert_eq!(history.len(), 3);
    }
}
