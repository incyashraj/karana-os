// Phase 54: Async Orchestrator
//
// Event-driven orchestrator replacing the synchronous tick loop:
// 1. Async message passing between layers
// 2. Priority-based scheduling
// 3. Deadline enforcement
// 4. Resource-aware execution

use crate::capabilities::{
    LayerCapability, LayerMessage, MessagePriority, CapabilityStatus, CapabilityState
};
use anyhow::{Result, Context};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::{Duration, Instant, timeout};

/// Maximum pending messages per layer
const MAX_PENDING_MESSAGES: usize = 1000;

/// Message with scheduling metadata
#[derive(Debug)]
struct ScheduledMessage {
    message: LayerMessage,
    deadline: Option<Instant>,
    retry_count: u8,
}

impl PartialEq for ScheduledMessage {
    fn eq(&self, other: &Self) -> bool {
        self.message.priority == other.message.priority
    }
}

impl Eq for ScheduledMessage {}

impl PartialOrd for ScheduledMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority comes first
        self.message.priority.cmp(&other.message.priority)
    }
}

/// Async orchestrator for layer coordination
pub struct AsyncOrchestrator {
    /// Registered layer capabilities (layer number -> capability)
    layers: Arc<RwLock<HashMap<u8, Box<dyn LayerCapability>>>>,
    
    /// Message queues per layer
    message_queues: Arc<RwLock<HashMap<u8, BinaryHeap<ScheduledMessage>>>>,
    
    /// Sender channels for each layer
    senders: HashMap<u8, mpsc::Sender<LayerMessage>>,
    
    /// Scheduling policies
    policies: SchedulingPolicies,
    
    /// Resource semaphores
    resource_limits: ResourceLimits,
    
    /// Statistics
    stats: Arc<RwLock<OrchestratorStats>>,
}

/// Scheduling policies
#[derive(Debug, Clone)]
pub struct SchedulingPolicies {
    /// Maximum message processing time before timeout
    pub default_timeout_ms: u64,
    
    /// Timeout for critical messages
    pub critical_timeout_ms: u64,
    
    /// Timeout for render operations (Layer 7)
    pub render_timeout_ms: u64,
    
    /// Maximum retry attempts for failed messages
    pub max_retries: u8,
    
    /// Enable deadline enforcement
    pub enforce_deadlines: bool,
}

impl Default for SchedulingPolicies {
    fn default() -> Self {
        Self {
            default_timeout_ms: 1000,
            critical_timeout_ms: 100,
            render_timeout_ms: 16, // ~60 FPS
            max_retries: 3,
            enforce_deadlines: true,
        }
    }
}

/// Resource limits for concurrent execution
#[derive(Clone)]
pub struct ResourceLimits {
    /// Max concurrent layer operations
    pub max_concurrent_layers: usize,
    
    /// Max concurrent messages per layer
    pub max_concurrent_messages: usize,
    
    /// Semaphores for enforcement
    layer_semaphore: Arc<Semaphore>,
    message_semaphore: Arc<Semaphore>,
}

impl ResourceLimits {
    pub fn new(max_concurrent_layers: usize, max_concurrent_messages: usize) -> Self {
        Self {
            max_concurrent_layers,
            max_concurrent_messages,
            layer_semaphore: Arc::new(Semaphore::new(max_concurrent_layers)),
            message_semaphore: Arc::new(Semaphore::new(max_concurrent_messages)),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::new(9, 100) // Allow all layers, 100 concurrent messages
    }
}

/// Orchestrator statistics
#[derive(Debug, Clone, Default)]
pub struct OrchestratorStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_failed: u64,
    pub messages_timeout: u64,
    pub messages_retried: u64,
    pub total_latency_ms: u64,
    pub layer_status: HashMap<u8, CapabilityStatus>,
}

impl AsyncOrchestrator {
    /// Create a new async orchestrator
    pub fn new(policies: SchedulingPolicies) -> Self {
        Self {
            layers: Arc::new(RwLock::new(HashMap::new())),
            message_queues: Arc::new(RwLock::new(HashMap::new())),
            senders: HashMap::new(),
            policies,
            resource_limits: ResourceLimits::default(),
            stats: Arc::new(RwLock::new(OrchestratorStats::default())),
        }
    }
    
    /// Register a layer capability
    pub async fn register_layer(&mut self, layer_num: u8, capability: Box<dyn LayerCapability>) -> Result<()> {
        // Create message channel for this layer
        let (tx, rx) = mpsc::channel(MAX_PENDING_MESSAGES);
        self.senders.insert(layer_num, tx);
        
        // Initialize message queue
        {
            let mut queues = self.message_queues.write().await;
            queues.insert(layer_num, BinaryHeap::new());
        }
        
        // Register capability
        {
            let mut layers = self.layers.write().await;
            layers.insert(layer_num, capability);
        }
        
        // Start message processing loop for this layer
        self.spawn_layer_handler(layer_num, rx).await;
        
        log::info!("[ORCHESTRATOR] Registered layer {}", layer_num);
        Ok(())
    }
    
    /// Spawn message handler for a layer
    async fn spawn_layer_handler(&self, layer_num: u8, mut rx: mpsc::Receiver<LayerMessage>) {
        let layers = self.layers.clone();
        let stats = self.stats.clone();
        let policies = self.policies.clone();
        let resource_limits = self.resource_limits.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                // Acquire message processing permit
                let _permit = resource_limits.message_semaphore.acquire().await.unwrap();
                
                let start = Instant::now();
                
                // Determine timeout based on priority
                let timeout_ms = match msg.priority {
                    MessagePriority::Critical => policies.critical_timeout_ms,
                    _ => policies.default_timeout_ms,
                };
                
                // Process message with timeout
                let result = {
                    let mut layers_write = layers.write().await;
                    if let Some(capability) = layers_write.get_mut(&layer_num) {
                        timeout(
                            Duration::from_millis(timeout_ms),
                            capability.handle_message(msg.clone())
                        ).await
                    } else {
                        log::error!("[ORCHESTRATOR] Layer {} not found", layer_num);
                        continue;
                    }
                };
                
                // Update statistics
                let elapsed = start.elapsed().as_millis() as u64;
                let mut stats_write = stats.write().await;
                stats_write.messages_received += 1;
                stats_write.total_latency_ms += elapsed;
                
                match result {
                    Ok(Ok(_response)) => {
                        // Success - could route response if needed
                    }
                    Ok(Err(e)) => {
                        log::error!("[ORCHESTRATOR] Layer {} message failed: {}", layer_num, e);
                        stats_write.messages_failed += 1;
                    }
                    Err(_) => {
                        log::warn!("[ORCHESTRATOR] Layer {} message timeout after {}ms", layer_num, timeout_ms);
                        stats_write.messages_timeout += 1;
                    }
                }
            }
        });
    }
    
    /// Send a message to a layer
    pub async fn send_message(&self, msg: LayerMessage) -> Result<()> {
        let target_layer = msg.to;
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.messages_sent += 1;
        }
        
        // Get sender for target layer
        if let Some(sender) = self.senders.get(&target_layer) {
            sender.send(msg).await
                .context("Failed to send message to layer")?;
        } else if target_layer == 0 {
            // Broadcast to all layers
            for sender in self.senders.values() {
                let _ = sender.send(msg.clone()).await;
            }
        } else {
            return Err(anyhow::anyhow!("Layer {} not registered", target_layer));
        }
        
        Ok(())
    }
    
    /// Send a message and wait for response
    pub async fn send_and_wait(&self, msg: LayerMessage, timeout_ms: u64) -> Result<LayerMessage> {
        // This would require a more sophisticated request-response mechanism
        // For now, just send and return an error
        self.send_message(msg).await?;
        Err(anyhow::anyhow!("Request-response not yet implemented"))
    }
    
    /// Broadcast a message to all layers
    pub async fn broadcast(&self, msg: LayerMessage) -> Result<()> {
        let mut broadcast_msg = msg;
        broadcast_msg.to = 0; // 0 = broadcast
        self.send_message(broadcast_msg).await
    }
    
    /// Get status of all layers
    pub async fn get_all_status(&self) -> Result<HashMap<u8, CapabilityStatus>> {
        let mut statuses = HashMap::new();
        let layers = self.layers.read().await;
        
        for (layer_num, capability) in layers.iter() {
            let status = capability.status().await;
            statuses.insert(*layer_num, status);
        }
        
        Ok(statuses)
    }
    
    /// Get orchestrator statistics
    pub async fn get_stats(&self) -> OrchestratorStats {
        self.stats.read().await.clone()
    }
    
    /// Initialize all registered layers
    pub async fn init_all(&self) -> Result<()> {
        let mut layers = self.layers.write().await;
        
        for (layer_num, capability) in layers.iter_mut() {
            log::info!("[ORCHESTRATOR] Initializing layer {}...", layer_num);
            capability.init().await
                .context(format!("Failed to initialize layer {}", layer_num))?;
        }
        
        Ok(())
    }
    
    /// Shutdown all registered layers
    pub async fn shutdown_all(&self) -> Result<()> {
        let mut layers = self.layers.write().await;
        
        for (layer_num, capability) in layers.iter_mut() {
            log::info!("[ORCHESTRATOR] Shutting down layer {}...", layer_num);
            capability.shutdown().await
                .context(format!("Failed to shutdown layer {}", layer_num))?;
        }
        
        Ok(())
    }
    
    /// Health check - verify all layers are responsive
    pub async fn health_check(&self) -> Result<HealthCheckReport> {
        let statuses = self.get_all_status().await?;
        let stats = self.get_stats().await;
        
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut error_count = 0;
        
        for status in statuses.values() {
            match status.state {
                CapabilityState::Ready => healthy_count += 1,
                CapabilityState::Degraded => degraded_count += 1,
                CapabilityState::Error => error_count += 1,
                _ => {}
            }
        }
        
        let avg_latency_ms = if stats.messages_received > 0 {
            stats.total_latency_ms / stats.messages_received
        } else {
            0
        };
        
        Ok(HealthCheckReport {
            healthy_layers: healthy_count,
            degraded_layers: degraded_count,
            error_layers: error_count,
            total_messages: stats.messages_sent,
            failed_messages: stats.messages_failed,
            timeout_messages: stats.messages_timeout,
            avg_latency_ms,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HealthCheckReport {
    pub healthy_layers: usize,
    pub degraded_layers: usize,
    pub error_layers: usize,
    pub total_messages: u64,
    pub failed_messages: u64,
    pub timeout_messages: u64,
    pub avg_latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{MessageType, CapabilityVersion, CapabilityMetrics, HealthStatus};
    use std::any::Any;
    
    // Mock capability for testing
    struct MockCapability {
        layer: u8,
        message_count: Arc<RwLock<u64>>,
    }
    
    #[async_trait::async_trait]
    impl LayerCapability for MockCapability {
        fn name(&self) -> &str {
            "mock"
        }
        
        fn version(&self) -> CapabilityVersion {
            CapabilityVersion::new(1, 0, 0)
        }
        
        fn layer(&self) -> u8 {
            self.layer
        }
        
        async fn init(&mut self) -> Result<()> {
            Ok(())
        }
        
        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }
        
        async fn status(&self) -> CapabilityStatus {
            CapabilityStatus {
                state: CapabilityState::Ready,
                health: HealthStatus::Healthy,
                message: "OK".to_string(),
                metrics: CapabilityMetrics {
                    cpu_percent: 0.0,
                    memory_mb: 0.0,
                    message_count: *self.message_count.read().await,
                    error_count: 0,
                    last_activity: 0,
                },
            }
        }
        
        async fn handle_message(&mut self, _msg: LayerMessage) -> Result<LayerMessage> {
            let mut count = self.message_count.write().await;
            *count += 1;
            Ok(LayerMessage::new(self.layer, 0, MessageType::Response, vec![]))
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
    
    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = AsyncOrchestrator::new(SchedulingPolicies::default());
        let stats = orchestrator.get_stats().await;
        assert_eq!(stats.messages_sent, 0);
    }
    
    #[tokio::test]
    async fn test_register_layer() {
        let mut orchestrator = AsyncOrchestrator::new(SchedulingPolicies::default());
        
        let mock = MockCapability {
            layer: 1,
            message_count: Arc::new(RwLock::new(0)),
        };
        
        orchestrator.register_layer(1, Box::new(mock)).await.unwrap();
        
        let statuses = orchestrator.get_all_status().await.unwrap();
        assert!(statuses.contains_key(&1));
    }
    
    #[tokio::test]
    async fn test_send_message() {
        let mut orchestrator = AsyncOrchestrator::new(SchedulingPolicies::default());
        
        let mock = MockCapability {
            layer: 1,
            message_count: Arc::new(RwLock::new(0)),
        };
        
        orchestrator.register_layer(1, Box::new(mock)).await.unwrap();
        
        let msg = LayerMessage::new(0, 1, MessageType::Request, vec![1, 2, 3]);
        orchestrator.send_message(msg).await.unwrap();
        
        // Give time for message to process
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let stats = orchestrator.get_stats().await;
        assert_eq!(stats.messages_sent, 1);
    }
}
