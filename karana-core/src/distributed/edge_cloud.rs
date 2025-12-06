// Kāraṇa OS - Phase 52: Edge Cloud Resource Pooling
// Pool and manage distributed compute resources

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::compute_node::{ComputeNode, NodeCapabilities, NodeStatus};

/// Resource pool for edge cloud nodes
pub struct EdgeCloudPool {
    pools: Arc<RwLock<HashMap<String, ResourcePool>>>,
    node_allocations: Arc<RwLock<HashMap<String, Vec<String>>>>, // node_id -> task_ids
    global_stats: Arc<RwLock<PoolStatistics>>,
}

/// Named resource pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub pool_id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<String>, // node IDs
    pub policy: PoolPolicy,
    pub capacity: PoolCapacity,
    pub created_at: u64,
}

/// Pool management policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolPolicy {
    pub max_nodes: usize,
    pub min_nodes: usize,
    pub auto_scale: bool,
    pub priority: PoolPriority,
    pub allowed_workloads: Vec<WorkloadType>,
    pub node_selection: NodeSelectionStrategy,
}

/// Pool priority level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PoolPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Types of workloads
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WorkloadType {
    Inference,
    Training,
    DataProcessing,
    Rendering,
    Gaming,
    General,
}

/// Node selection strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum NodeSelectionStrategy {
    RoundRobin,
    LeastLoaded,
    LowestLatency,
    MostCapable,
    Random,
}

/// Pool capacity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolCapacity {
    pub total_cpu_cores: u32,
    pub total_gpu_count: u32,
    pub total_ram_gb: f32,
    pub total_storage_gb: f32,
    pub available_cpu_cores: u32,
    pub available_gpu_count: u32,
    pub available_ram_gb: f32,
    pub available_storage_gb: f32,
}

/// Pool-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub total_pools: usize,
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_tasks_completed: u64,
    pub total_compute_hours: f32,
    pub avg_utilization_percent: f32,
}

impl EdgeCloudPool {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            node_allocations: Arc::new(RwLock::new(HashMap::new())),
            global_stats: Arc::new(RwLock::new(PoolStatistics {
                total_pools: 0,
                total_nodes: 0,
                active_nodes: 0,
                total_tasks_completed: 0,
                total_compute_hours: 0.0,
                avg_utilization_percent: 0.0,
            })),
        }
    }

    /// Create a new resource pool
    pub async fn create_pool(
        &self,
        name: &str,
        description: &str,
        policy: PoolPolicy,
    ) -> String {
        let pool_id = format!("pool-{}", uuid::Uuid::new_v4());
        
        let pool = ResourcePool {
            pool_id: pool_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            nodes: Vec::new(),
            policy,
            capacity: PoolCapacity {
                total_cpu_cores: 0,
                total_gpu_count: 0,
                total_ram_gb: 0.0,
                total_storage_gb: 0.0,
                available_cpu_cores: 0,
                available_gpu_count: 0,
                available_ram_gb: 0.0,
                available_storage_gb: 0.0,
            },
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.pools.write().await.insert(pool_id.clone(), pool);
        self.global_stats.write().await.total_pools += 1;
        
        pool_id
    }

    /// Add node to a pool
    pub async fn add_node_to_pool(&self, pool_id: &str, node_id: &str, node: &ComputeNode) -> bool {
        let mut pools = self.pools.write().await;
        
        if let Some(pool) = pools.get_mut(pool_id) {
            // Check pool capacity limits
            if pool.nodes.len() >= pool.policy.max_nodes {
                return false;
            }

            // Check if node already in pool
            if pool.nodes.contains(&node_id.to_string()) {
                return false;
            }

            // Add node
            pool.nodes.push(node_id.to_string());
            
            // Update capacity
            pool.capacity.total_cpu_cores += node.capabilities.cpu_cores;
            pool.capacity.available_cpu_cores += node.capabilities.cpu_cores;
            
            if node.capabilities.gpu_available {
                pool.capacity.total_gpu_count += 1;
                pool.capacity.available_gpu_count += 1;
            }
            
            pool.capacity.total_ram_gb += node.capabilities.ram_gb;
            pool.capacity.available_ram_gb += node.capabilities.ram_gb;
            pool.capacity.total_storage_gb += node.capabilities.storage_gb;
            pool.capacity.available_storage_gb += node.capabilities.storage_gb;

            self.global_stats.write().await.total_nodes += 1;
            if node.status == NodeStatus::Available {
                self.global_stats.write().await.active_nodes += 1;
            }

            true
        } else {
            false
        }
    }

    /// Remove node from pool
    pub async fn remove_node_from_pool(&self, pool_id: &str, node_id: &str) -> bool {
        let mut pools = self.pools.write().await;
        
        if let Some(pool) = pools.get_mut(pool_id) {
            if let Some(pos) = pool.nodes.iter().position(|id| id == node_id) {
                pool.nodes.remove(pos);
                self.global_stats.write().await.total_nodes -= 1;
                return true;
            }
        }
        
        false
    }

    /// Select a node from pool based on strategy
    pub async fn select_node_from_pool(
        &self,
        pool_id: &str,
        nodes: &HashMap<String, ComputeNode>,
    ) -> Option<String> {
        let pools = self.pools.read().await;
        
        let pool = pools.get(pool_id)?;
        let available_nodes: Vec<_> = pool
            .nodes
            .iter()
            .filter_map(|node_id| {
                nodes.get(node_id).and_then(|node| {
                    if node.status == NodeStatus::Available {
                        Some((node_id.clone(), node.clone()))
                    } else {
                        None
                    }
                })
            })
            .collect();

        if available_nodes.is_empty() {
            return None;
        }

        match pool.policy.node_selection {
            NodeSelectionStrategy::RoundRobin => {
                // Simple: return first available
                Some(available_nodes[0].0.clone())
            }
            NodeSelectionStrategy::LeastLoaded => {
                // Find node with lowest CPU usage
                available_nodes
                    .iter()
                    .min_by(|a, b| {
                        a.1.resources
                            .cpu_usage_percent
                            .partial_cmp(&b.1.resources.cpu_usage_percent)
                            .unwrap()
                    })
                    .map(|(id, _)| id.clone())
            }
            NodeSelectionStrategy::LowestLatency => {
                // Find node with lowest latency
                available_nodes
                    .iter()
                    .min_by(|a, b| {
                        let a_latency = a.1.location.as_ref().map(|l| l.network_latency_ms).unwrap_or(999.9);
                        let b_latency = b.1.location.as_ref().map(|l| l.network_latency_ms).unwrap_or(999.9);
                        a_latency.partial_cmp(&b_latency).unwrap()
                    })
                    .map(|(id, _)| id.clone())
            }
            NodeSelectionStrategy::MostCapable => {
                // Find node with most resources
                available_nodes
                    .iter()
                    .max_by(|a, b| {
                        let a_score = a.1.capabilities.cpu_cores as f32 
                            + if a.1.capabilities.gpu_available { 100.0 } else { 0.0 };
                        let b_score = b.1.capabilities.cpu_cores as f32 
                            + if b.1.capabilities.gpu_available { 100.0 } else { 0.0 };
                        a_score.partial_cmp(&b_score).unwrap()
                    })
                    .map(|(id, _)| id.clone())
            }
            NodeSelectionStrategy::Random => {
                // Random selection
                let idx = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as usize
                    % available_nodes.len();
                Some(available_nodes[idx].0.clone())
            }
        }
    }

    /// Get pool information
    pub async fn get_pool(&self, pool_id: &str) -> Option<ResourcePool> {
        self.pools.read().await.get(pool_id).cloned()
    }

    /// List all pools
    pub async fn list_pools(&self) -> Vec<ResourcePool> {
        self.pools.read().await.values().cloned().collect()
    }

    /// Get pools by priority
    pub async fn get_pools_by_priority(&self, priority: PoolPriority) -> Vec<ResourcePool> {
        self.pools
            .read()
            .await
            .values()
            .filter(|p| p.policy.priority == priority)
            .cloned()
            .collect()
    }

    /// Allocate task to node in pool
    pub async fn allocate_task(&self, node_id: &str, task_id: &str) {
        let mut allocations = self.node_allocations.write().await;
        allocations
            .entry(node_id.to_string())
            .or_insert_with(Vec::new)
            .push(task_id.to_string());
    }

    /// Release task from node
    pub async fn release_task(&self, node_id: &str, task_id: &str) -> bool {
        let mut allocations = self.node_allocations.write().await;
        
        if let Some(tasks) = allocations.get_mut(node_id) {
            if let Some(pos) = tasks.iter().position(|id| id == task_id) {
                tasks.remove(pos);
                self.global_stats.write().await.total_tasks_completed += 1;
                return true;
            }
        }
        
        false
    }

    /// Get node allocation count
    pub async fn get_node_task_count(&self, node_id: &str) -> usize {
        self.node_allocations
            .read()
            .await
            .get(node_id)
            .map(|tasks| tasks.len())
            .unwrap_or(0)
    }

    /// Update pool capacity after resource usage
    pub async fn update_pool_capacity(
        &self,
        pool_id: &str,
        cpu_used: u32,
        gpu_used: u32,
        ram_used_gb: f32,
    ) {
        let mut pools = self.pools.write().await;
        
        if let Some(pool) = pools.get_mut(pool_id) {
            pool.capacity.available_cpu_cores = pool.capacity.total_cpu_cores.saturating_sub(cpu_used);
            pool.capacity.available_gpu_count = pool.capacity.total_gpu_count.saturating_sub(gpu_used);
            pool.capacity.available_ram_gb = (pool.capacity.total_ram_gb - ram_used_gb).max(0.0);
        }
    }

    /// Get global statistics
    pub async fn get_statistics(&self) -> PoolStatistics {
        self.global_stats.read().await.clone()
    }

    /// Calculate pool utilization
    pub async fn calculate_pool_utilization(&self, pool_id: &str) -> f32 {
        let pools = self.pools.read().await;
        
        if let Some(pool) = pools.get(pool_id) {
            if pool.capacity.total_cpu_cores == 0 {
                return 0.0;
            }

            let cpu_util = 1.0 - (pool.capacity.available_cpu_cores as f32 / pool.capacity.total_cpu_cores as f32);
            let ram_util = if pool.capacity.total_ram_gb > 0.0 {
                1.0 - (pool.capacity.available_ram_gb / pool.capacity.total_ram_gb)
            } else {
                0.0
            };

            (cpu_util + ram_util) / 2.0
        } else {
            0.0
        }
    }

    /// Auto-scale pool based on utilization
    pub async fn auto_scale_pool(&self, pool_id: &str, target_utilization: f32) -> Option<ScaleAction> {
        let current_util = self.calculate_pool_utilization(pool_id).await;
        let pool = self.get_pool(pool_id).await?;

        if !pool.policy.auto_scale {
            return None;
        }

        if current_util > target_utilization + 0.2 && pool.nodes.len() < pool.policy.max_nodes {
            // Scale up
            Some(ScaleAction::ScaleUp {
                current_nodes: pool.nodes.len(),
                target_nodes: (pool.nodes.len() + 1).min(pool.policy.max_nodes),
            })
        } else if current_util < target_utilization - 0.2 && pool.nodes.len() > pool.policy.min_nodes {
            // Scale down
            Some(ScaleAction::ScaleDown {
                current_nodes: pool.nodes.len(),
                target_nodes: (pool.nodes.len() - 1).max(pool.policy.min_nodes),
            })
        } else {
            None
        }
    }
}

/// Scaling action recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleAction {
    ScaleUp { current_nodes: usize, target_nodes: usize },
    ScaleDown { current_nodes: usize, target_nodes: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::compute_node::*;

    fn create_test_node(id: &str, cores: u32, has_gpu: bool) -> ComputeNode {
        ComputeNode {
            id: id.to_string(),
            capabilities: NodeCapabilities {
                cpu_cores: cores,
                gpu_available: has_gpu,
                gpu_memory_gb: if has_gpu { 16.0 } else { 0.0 },
                ram_gb: cores as f32 * 4.0,
                storage_gb: 500.0,
                network_bandwidth_mbps: 1000,
                supported_models: vec![],
                hardware_acceleration: vec![],
            },
            status: NodeStatus::Available,
            resources: NodeResources {
                cpu_usage_percent: 20.0,
                gpu_usage_percent: 0.0,
                ram_usage_percent: 30.0,
                network_usage_mbps: 50.0,
                active_tasks: 0,
                queue_length: 0,
            },
            location: Some(NodeLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                region: "us-west".to_string(),
                network_latency_ms: 25.0,
            }),
        }
    }

    #[tokio::test]
    async fn test_create_pool() {
        let pool_manager = EdgeCloudPool::new();
        
        let policy = PoolPolicy {
            max_nodes: 10,
            min_nodes: 1,
            auto_scale: true,
            priority: PoolPriority::High,
            allowed_workloads: vec![WorkloadType::Inference],
            node_selection: NodeSelectionStrategy::LeastLoaded,
        };

        let pool_id = pool_manager.create_pool("inference-pool", "For AI inference", policy).await;
        
        assert!(pool_id.starts_with("pool-"));
        
        let pool = pool_manager.get_pool(&pool_id).await.unwrap();
        assert_eq!(pool.name, "inference-pool");
        assert_eq!(pool.policy.priority, PoolPriority::High);
    }

    #[tokio::test]
    async fn test_add_nodes_to_pool() {
        let pool_manager = EdgeCloudPool::new();
        
        let policy = PoolPolicy {
            max_nodes: 5,
            min_nodes: 1,
            auto_scale: false,
            priority: PoolPriority::Medium,
            allowed_workloads: vec![WorkloadType::General],
            node_selection: NodeSelectionStrategy::RoundRobin,
        };

        let pool_id = pool_manager.create_pool("test-pool", "Test", policy).await;
        
        let node1 = create_test_node("node-1", 8, true);
        let node2 = create_test_node("node-2", 16, false);

        assert!(pool_manager.add_node_to_pool(&pool_id, "node-1", &node1).await);
        assert!(pool_manager.add_node_to_pool(&pool_id, "node-2", &node2).await);

        let pool = pool_manager.get_pool(&pool_id).await.unwrap();
        assert_eq!(pool.nodes.len(), 2);
        assert_eq!(pool.capacity.total_cpu_cores, 24);
        assert_eq!(pool.capacity.total_gpu_count, 1);
    }

    #[tokio::test]
    async fn test_node_selection_least_loaded() {
        let pool_manager = EdgeCloudPool::new();
        
        let policy = PoolPolicy {
            max_nodes: 5,
            min_nodes: 1,
            auto_scale: false,
            priority: PoolPriority::Medium,
            allowed_workloads: vec![WorkloadType::General],
            node_selection: NodeSelectionStrategy::LeastLoaded,
        };

        let pool_id = pool_manager.create_pool("select-pool", "Test", policy).await;
        
        let mut node1 = create_test_node("node-1", 8, false);
        node1.resources.cpu_usage_percent = 80.0;
        
        let mut node2 = create_test_node("node-2", 8, false);
        node2.resources.cpu_usage_percent = 20.0;

        pool_manager.add_node_to_pool(&pool_id, "node-1", &node1).await;
        pool_manager.add_node_to_pool(&pool_id, "node-2", &node2).await;

        let mut nodes = HashMap::new();
        nodes.insert("node-1".to_string(), node1);
        nodes.insert("node-2".to_string(), node2);

        let selected = pool_manager.select_node_from_pool(&pool_id, &nodes).await;
        assert_eq!(selected, Some("node-2".to_string())); // Less loaded
    }

    #[tokio::test]
    async fn test_task_allocation() {
        let pool_manager = EdgeCloudPool::new();
        
        pool_manager.allocate_task("node-1", "task-1").await;
        pool_manager.allocate_task("node-1", "task-2").await;
        pool_manager.allocate_task("node-2", "task-3").await;

        assert_eq!(pool_manager.get_node_task_count("node-1").await, 2);
        assert_eq!(pool_manager.get_node_task_count("node-2").await, 1);

        pool_manager.release_task("node-1", "task-1").await;
        assert_eq!(pool_manager.get_node_task_count("node-1").await, 1);
    }

    #[tokio::test]
    async fn test_pool_utilization() {
        let pool_manager = EdgeCloudPool::new();
        
        let policy = PoolPolicy {
            max_nodes: 5,
            min_nodes: 1,
            auto_scale: false,
            priority: PoolPriority::Low,
            allowed_workloads: vec![WorkloadType::General],
            node_selection: NodeSelectionStrategy::RoundRobin,
        };

        let pool_id = pool_manager.create_pool("util-pool", "Test", policy).await;
        
        let node = create_test_node("node-1", 8, false);
        pool_manager.add_node_to_pool(&pool_id, "node-1", &node).await;

        // Initially low utilization
        let util = pool_manager.calculate_pool_utilization(&pool_id).await;
        assert!(util < 0.5);

        // Use some resources
        pool_manager.update_pool_capacity(&pool_id, 6, 0, 20.0).await;
        
        let util = pool_manager.calculate_pool_utilization(&pool_id).await;
        assert!(util > 0.5);
    }

    #[tokio::test]
    async fn test_auto_scaling() {
        let pool_manager = EdgeCloudPool::new();
        
        let policy = PoolPolicy {
            max_nodes: 10,
            min_nodes: 2,
            auto_scale: true,
            priority: PoolPriority::High,
            allowed_workloads: vec![WorkloadType::Inference],
            node_selection: NodeSelectionStrategy::LeastLoaded,
        };

        let pool_id = pool_manager.create_pool("scale-pool", "Test", policy).await;
        
        let node = create_test_node("node-1", 8, false);
        pool_manager.add_node_to_pool(&pool_id, "node-1", &node).await;

        // Simulate high utilization
        pool_manager.update_pool_capacity(&pool_id, 7, 0, 28.0).await;
        
        let action = pool_manager.auto_scale_pool(&pool_id, 0.6).await;
        assert!(matches!(action, Some(ScaleAction::ScaleUp { .. })));
    }

    #[tokio::test]
    async fn test_pool_priority_filtering() {
        let pool_manager = EdgeCloudPool::new();
        
        let high_policy = PoolPolicy {
            max_nodes: 5,
            min_nodes: 1,
            auto_scale: false,
            priority: PoolPriority::High,
            allowed_workloads: vec![WorkloadType::Inference],
            node_selection: NodeSelectionStrategy::RoundRobin,
        };

        let low_policy = PoolPolicy {
            max_nodes: 5,
            min_nodes: 1,
            auto_scale: false,
            priority: PoolPriority::Low,
            allowed_workloads: vec![WorkloadType::General],
            node_selection: NodeSelectionStrategy::RoundRobin,
        };

        pool_manager.create_pool("high-pool", "High priority", high_policy).await;
        pool_manager.create_pool("low-pool", "Low priority", low_policy).await;

        let high_pools = pool_manager.get_pools_by_priority(PoolPriority::High).await;
        assert_eq!(high_pools.len(), 1);
        assert_eq!(high_pools[0].name, "high-pool");
    }
}
