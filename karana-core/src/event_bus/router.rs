// Event Router - Priority-based event routing and scheduling
// Phase 47: Intelligent event distribution

use super::core::{Event, EventCategory, EventPriority, EventBus};
use crate::capability::{LayerId, CapabilityRegistry};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, Semaphore};

/// Routing policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingPolicy {
    /// Broadcast to all subscribers
    Broadcast,
    
    /// Round-robin across providers
    RoundRobin,
    
    /// Route to least loaded provider
    LeastLoaded,
    
    /// Route to specific target
    Direct(LayerId),
    
    /// Route based on capability
    CapabilityBased,
}

/// Scheduled event with priority
#[derive(Clone)]
struct ScheduledEvent {
    event: Event,
    scheduled_at: SystemTime,
    deadline: Option<SystemTime>,
}

impl ScheduledEvent {
    fn priority_score(&self) -> (u8, SystemTime) {
        let priority_val = match self.event.metadata.priority {
            EventPriority::Critical => 3,
            EventPriority::High => 2,
            EventPriority::Normal => 1,
            EventPriority::Low => 0,
        };
        (priority_val, self.scheduled_at)
    }
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.metadata.id == other.event.metadata.id
    }
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first (for max-heap), then earlier timestamp
        let (self_pri, self_time) = self.priority_score();
        let (other_pri, other_time) = other.priority_score();
        
        match self_pri.cmp(&other_pri) {
            Ordering::Equal => other_time.cmp(&self_time), // Earlier times are greater
            other => other,
        }
    }
}

/// Routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule name
    pub name: String,
    
    /// Categories this rule applies to
    pub categories: Vec<EventCategory>,
    
    /// Routing policy
    pub policy: RoutingPolicy,
    
    /// Priority threshold
    pub min_priority: EventPriority,
    
    /// Whether rule is enabled
    pub enabled: bool,
}

/// Event router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Maximum queue size
    pub max_queue_size: usize,
    
    /// Maximum concurrent events
    pub max_concurrent: usize,
    
    /// Event timeout
    pub event_timeout: Duration,
    
    /// Enable priority scheduling
    pub priority_scheduling: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            max_concurrent: 100,
            event_timeout: Duration::from_secs(5),
            priority_scheduling: true,
        }
    }
}

/// Router statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouterStatistics {
    pub events_routed: u64,
    pub events_queued: u64,
    pub events_dropped: u64,
    pub events_timeout: u64,
    pub avg_queue_time_ms: f64,
    pub current_queue_size: usize,
}

/// Event router with priority scheduling
pub struct EventRouter {
    config: Arc<RwLock<RouterConfig>>,
    event_bus: Arc<EventBus>,
    capability_registry: Arc<RwLock<CapabilityRegistry>>,
    
    /// Priority queue for events
    event_queue: Arc<RwLock<BinaryHeap<ScheduledEvent>>>,
    
    /// Routing rules
    rules: Arc<RwLock<Vec<RoutingRule>>>,
    
    /// Round-robin counters
    rr_counters: Arc<RwLock<HashMap<EventCategory, usize>>>,
    
    /// Semaphore for concurrency control
    semaphore: Arc<Semaphore>,
    
    /// Statistics
    stats: Arc<RwLock<RouterStatistics>>,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl EventRouter {
    pub fn new(event_bus: Arc<EventBus>, capability_registry: Arc<RwLock<CapabilityRegistry>>) -> Self {
        let config = RouterConfig::default();
        let max_concurrent = config.max_concurrent;
        
        Self {
            config: Arc::new(RwLock::new(config)),
            event_bus,
            capability_registry,
            event_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            rr_counters: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            stats: Arc::new(RwLock::new(RouterStatistics::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Add routing rule
    pub async fn add_rule(&self, rule: RoutingRule) {
        self.rules.write().await.push(rule);
    }
    
    /// Remove routing rule
    pub async fn remove_rule(&self, name: &str) {
        self.rules.write().await.retain(|r| r.name != name);
    }
    
    /// Start the router
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);
        
        // Start processing loop
        let router = self.clone();
        tokio::spawn(async move {
            router.processing_loop().await;
        });
        
        Ok(())
    }
    
    /// Stop the router
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }
    
    /// Route an event
    pub async fn route(&self, mut event: Event) -> Result<()> {
        let queue_start = SystemTime::now();
        
        // Apply routing rules
        self.apply_routing_rules(&mut event).await;
        
        // Check queue size
        let config = self.config.read().await;
        let max_queue_size = config.max_queue_size;
        drop(config);
        
        let mut queue = self.event_queue.write().await;
        if queue.len() >= max_queue_size {
            self.stats.write().await.events_dropped += 1;
            return Ok(()); // Drop event
        }
        
        // Add to queue
        let scheduled = ScheduledEvent {
            event,
            scheduled_at: queue_start,
            deadline: None,
        };
        
        queue.push(scheduled);
        self.stats.write().await.events_queued += 1;
        self.stats.write().await.current_queue_size = queue.len();
        
        Ok(())
    }
    
    /// Processing loop
    async fn processing_loop(&self) {
        while *self.running.read().await {
            // Get next event from queue
            let event_opt = {
                let mut queue = self.event_queue.write().await;
                queue.pop()
            };
            
            if let Some(scheduled) = event_opt {
                // Update queue time statistics
                if let Ok(queue_time) = scheduled.scheduled_at.elapsed() {
                    let mut stats = self.stats.write().await;
                    let queue_ms = queue_time.as_millis() as f64;
                    stats.avg_queue_time_ms = 
                        (stats.avg_queue_time_ms * stats.events_routed as f64 + queue_ms)
                        / (stats.events_routed + 1) as f64;
                }
                
                // Process event
                let event_bus = Arc::clone(&self.event_bus);
                let stats = Arc::clone(&self.stats);
                let semaphore = Arc::clone(&self.semaphore);
                
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    if let Err(e) = event_bus.publish(scheduled.event).await {
                        eprintln!("Event routing error: {}", e);
                    }
                    stats.write().await.events_routed += 1;
                });
                
                // Update queue size
                let queue_size = self.event_queue.read().await.len();
                self.stats.write().await.current_queue_size = queue_size;
            } else {
                // Queue empty, sleep briefly
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
    
    /// Apply routing rules to event
    async fn apply_routing_rules(&self, event: &mut Event) {
        let rules = self.rules.read().await;
        
        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }
            
            // Check if rule applies
            if !rule.categories.is_empty() && !rule.categories.contains(&event.metadata.category) {
                continue;
            }
            
            if event.metadata.priority < rule.min_priority {
                continue;
            }
            
            // Apply policy
            match &rule.policy {
                RoutingPolicy::Broadcast => {
                    // Already broadcast by default
                }
                RoutingPolicy::RoundRobin => {
                    if let Some(target) = self.next_round_robin(&event.metadata.category).await {
                        event.metadata.target = Some(target);
                    }
                }
                RoutingPolicy::LeastLoaded => {
                    if let Some(cap) = &event.metadata.required_capability {
                        let registry = self.capability_registry.read().await;
                        if let Some(target) = registry.find_best_provider(cap) {
                            event.metadata.target = Some(target);
                        }
                    }
                }
                RoutingPolicy::Direct(target) => {
                    event.metadata.target = Some(*target);
                }
                RoutingPolicy::CapabilityBased => {
                    if let Some(cap) = &event.metadata.required_capability {
                        let registry = self.capability_registry.read().await;
                        if let Some(target) = registry.find_best_provider(cap) {
                            event.metadata.target = Some(target);
                        }
                    }
                }
            }
        }
    }
    
    /// Get next target for round-robin
    async fn next_round_robin(&self, category: &EventCategory) -> Option<LayerId> {
        let registry = self.capability_registry.read().await;
        let layers = registry.registered_layers();
        
        if layers.is_empty() {
            return None;
        }
        
        let mut counters = self.rr_counters.write().await;
        let counter = counters.entry(category.clone()).or_insert(0);
        let target = layers[*counter % layers.len()];
        *counter += 1;
        
        Some(target)
    }
    
    /// Get statistics
    pub async fn get_statistics(&self) -> RouterStatistics {
        self.stats.read().await.clone()
    }
    
    /// Clear queue
    pub async fn clear_queue(&self) {
        self.event_queue.write().await.clear();
        self.stats.write().await.current_queue_size = 0;
    }
}

impl Clone for EventRouter {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            event_bus: Arc::clone(&self.event_bus),
            capability_registry: Arc::clone(&self.capability_registry),
            event_queue: Arc::clone(&self.event_queue),
            rules: Arc::clone(&self.rules),
            rr_counters: Arc::clone(&self.rr_counters),
            semaphore: Arc::clone(&self.semaphore),
            stats: Arc::clone(&self.stats),
            running: Arc::clone(&self.running),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityAdvertisement;
    
    #[tokio::test]
    async fn test_router_creation() {
        let bus = Arc::new(EventBus::new());
        let registry = Arc::new(RwLock::new(CapabilityRegistry::new()));
        
        let router = EventRouter::new(bus, registry);
        let stats = router.get_statistics().await;
        
        assert_eq!(stats.events_routed, 0);
    }
    
    #[tokio::test]
    async fn test_add_routing_rule() {
        let bus = Arc::new(EventBus::new());
        let registry = Arc::new(RwLock::new(CapabilityRegistry::new()));
        let router = EventRouter::new(bus, registry);
        
        let rule = RoutingRule {
            name: "test_rule".to_string(),
            categories: vec![EventCategory::CameraFrame],
            policy: RoutingPolicy::Broadcast,
            min_priority: EventPriority::Normal,
            enabled: true,
        };
        
        router.add_rule(rule).await;
        assert_eq!(router.rules.read().await.len(), 1);
    }
    
    #[tokio::test]
    async fn test_event_routing() {
        let bus = Arc::new(EventBus::new());
        let registry = Arc::new(RwLock::new(CapabilityRegistry::new()));
        let router = EventRouter::new(bus, registry);
        
        router.start().await.unwrap();
        
        let event = Event::new(
            LayerId::Hardware,
            EventCategory::CameraFrame,
            EventPriority::Normal,
        );
        
        router.route(event).await.unwrap();
        
        // Give time to process
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let stats = router.get_statistics().await;
        assert!(stats.events_queued > 0);
        
        router.stop().await;
    }
    
    #[tokio::test]
    async fn test_priority_ordering() {
        let low = ScheduledEvent {
            event: Event::new(LayerId::Hardware, EventCategory::SensorData, EventPriority::Low),
            scheduled_at: SystemTime::now(),
            deadline: None,
        };
        
        let high = ScheduledEvent {
            event: Event::new(LayerId::Hardware, EventCategory::SystemShutdown, EventPriority::Critical),
            scheduled_at: SystemTime::now(),
            deadline: None,
        };
        
        assert!(high > low);
    }
    
    #[tokio::test]
    async fn test_capability_based_routing() {
        use crate::capability::{Capability, CapabilityAdvertisement};
        
        let bus = Arc::new(EventBus::new());
        let registry = Arc::new(RwLock::new(CapabilityRegistry::new()));
        
        // Register a provider
        {
            let mut reg = registry.write().await;
            reg.register(CapabilityAdvertisement {
                layer: LayerId::AI,
                capabilities: vec![Capability::VisionProcessing],
                version: "0.1.0".to_string(),
                load: 0.3,
                healthy: true,
            });
        }
        
        let router = EventRouter::new(bus, registry);
        
        let rule = RoutingRule {
            name: "vision_route".to_string(),
            categories: vec![EventCategory::CameraFrame],
            policy: RoutingPolicy::CapabilityBased,
            min_priority: EventPriority::Normal,
            enabled: true,
        };
        
        router.add_rule(rule).await;
        
        let mut event = Event::new(
            LayerId::Hardware,
            EventCategory::CameraFrame,
            EventPriority::Normal,
        ).with_capability(Capability::VisionProcessing);
        
        router.apply_routing_rules(&mut event).await;
        
        assert_eq!(event.metadata.target, Some(LayerId::AI));
    }
}
