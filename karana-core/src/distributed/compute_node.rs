// Kāraṇa OS - Phase 52: Distributed Compute
// Compute node protocol for edge cloud integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Compute node in the distributed network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeNode {
    pub id: String,
    pub capabilities: NodeCapabilities,
    pub status: NodeStatus,
    pub resources: NodeResources,
    pub location: Option<NodeLocation>,
}

/// Node computational capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub cpu_cores: u32,
    pub gpu_available: bool,
    pub gpu_memory_gb: f32,
    pub ram_gb: f32,
    pub storage_gb: f32,
    pub network_bandwidth_mbps: u32,
    pub supported_models: Vec<String>,
    pub hardware_acceleration: Vec<HardwareAccel>,
}

/// Hardware acceleration types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HardwareAccel {
    CUDA,
    Metal,
    ROCm,
    OpenCL,
    Vulkan,
    NPU,
    TPU,
}

/// Node operational status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Available,
    Busy,
    Offline,
    Unreachable,
    Maintenance,
}

/// Current resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu_usage_percent: f32,
    pub gpu_usage_percent: f32,
    pub ram_usage_percent: f32,
    pub network_usage_mbps: f32,
    pub active_tasks: u32,
    pub queue_length: u32,
}

/// Physical/network location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLocation {
    pub latitude: f32,
    pub longitude: f32,
    pub region: String,
    pub network_latency_ms: f32,
}

/// Compute node protocol manager
pub struct ComputeNodeProtocol {
    nodes: Arc<RwLock<HashMap<String, ComputeNode>>>,
    local_node: Arc<RwLock<Option<ComputeNode>>>,
    discovery_enabled: Arc<RwLock<bool>>,
}

impl ComputeNodeProtocol {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            local_node: Arc::new(RwLock::new(None)),
            discovery_enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Register local node as compute provider
    pub async fn register_local_node(&self, capabilities: NodeCapabilities) -> String {
        let node_id = format!("node-{}", uuid::Uuid::new_v4());
        let node = ComputeNode {
            id: node_id.clone(),
            capabilities,
            status: NodeStatus::Available,
            resources: NodeResources {
                cpu_usage_percent: 0.0,
                gpu_usage_percent: 0.0,
                ram_usage_percent: 0.0,
                network_usage_mbps: 0.0,
                active_tasks: 0,
                queue_length: 0,
            },
            location: None,
        };

        *self.local_node.write().await = Some(node.clone());
        self.nodes.write().await.insert(node_id.clone(), node);
        node_id
    }

    /// Discover and register remote compute nodes
    pub async fn discover_nodes(&self) -> Vec<String> {
        *self.discovery_enabled.write().await = true;
        
        // Simulate discovering nodes on local network and edge cloud
        let discovered = vec![
            self.create_edge_node("edge-1", 8, true, 24.0, "us-west"),
            self.create_edge_node("edge-2", 16, true, 48.0, "us-east"),
            self.create_mobile_node("mobile-1", 4, false, 8.0),
        ];

        let mut nodes = self.nodes.write().await;
        let mut ids = Vec::new();
        
        for node in discovered {
            ids.push(node.id.clone());
            nodes.insert(node.id.clone(), node);
        }
        
        ids
    }

    /// Get specific node by ID
    pub async fn get_node(&self, node_id: &str) -> Option<ComputeNode> {
        self.nodes.read().await.get(node_id).cloned()
    }

    /// List all available nodes
    pub async fn list_available_nodes(&self) -> Vec<ComputeNode> {
        self.nodes
            .read()
            .await
            .values()
            .filter(|n| n.status == NodeStatus::Available)
            .cloned()
            .collect()
    }

    /// Update node status
    pub async fn update_node_status(&self, node_id: &str, status: NodeStatus) {
        if let Some(node) = self.nodes.write().await.get_mut(node_id) {
            node.status = status;
        }
    }

    /// Update node resource usage
    pub async fn update_node_resources(&self, node_id: &str, resources: NodeResources) {
        if let Some(node) = self.nodes.write().await.get_mut(node_id) {
            node.resources = resources;
        }
    }

    /// Find best node for a task based on requirements
    pub async fn find_best_node(&self, requirements: &ComputeRequirements) -> Option<String> {
        let nodes = self.nodes.read().await;
        
        // Score nodes based on requirements
        let mut best_node: Option<(&String, f32)> = None;
        
        for (id, node) in nodes.iter() {
            if node.status != NodeStatus::Available {
                continue;
            }

            let score = self.score_node(node, requirements);
            if score > 0.0 {
                if let Some((_, best_score)) = best_node {
                    if score > best_score {
                        best_node = Some((id, score));
                    }
                } else {
                    best_node = Some((id, score));
                }
            }
        }
        
        best_node.map(|(id, _)| id.clone())
    }

    /// Remove offline/unreachable nodes
    pub async fn cleanup_offline_nodes(&self) -> usize {
        let mut nodes = self.nodes.write().await;
        let initial_count = nodes.len();
        
        nodes.retain(|_, node| {
            node.status != NodeStatus::Offline && node.status != NodeStatus::Unreachable
        });
        
        initial_count - nodes.len()
    }

    // Helper methods
    fn create_edge_node(&self, id: &str, cores: u32, gpu: bool, gpu_mem: f32, region: &str) -> ComputeNode {
        ComputeNode {
            id: id.to_string(),
            capabilities: NodeCapabilities {
                cpu_cores: cores,
                gpu_available: gpu,
                gpu_memory_gb: gpu_mem,
                ram_gb: cores as f32 * 4.0,
                storage_gb: 500.0,
                network_bandwidth_mbps: 1000,
                supported_models: vec!["llama".to_string(), "stable-diffusion".to_string()],
                hardware_acceleration: if gpu { vec![HardwareAccel::CUDA, HardwareAccel::Vulkan] } else { vec![] },
            },
            status: NodeStatus::Available,
            resources: NodeResources {
                cpu_usage_percent: 20.0,
                gpu_usage_percent: if gpu { 15.0 } else { 0.0 },
                ram_usage_percent: 30.0,
                network_usage_mbps: 50.0,
                active_tasks: 1,
                queue_length: 0,
            },
            location: Some(NodeLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                region: region.to_string(),
                network_latency_ms: 25.0,
            }),
        }
    }

    fn create_mobile_node(&self, id: &str, cores: u32, gpu: bool, ram: f32) -> ComputeNode {
        ComputeNode {
            id: id.to_string(),
            capabilities: NodeCapabilities {
                cpu_cores: cores,
                gpu_available: gpu,
                gpu_memory_gb: if gpu { 4.0 } else { 0.0 },
                ram_gb: ram,
                storage_gb: 128.0,
                network_bandwidth_mbps: 100,
                supported_models: vec!["phi-2".to_string(), "tinyllama".to_string()],
                hardware_acceleration: vec![HardwareAccel::Metal, HardwareAccel::Vulkan],
            },
            status: NodeStatus::Available,
            resources: NodeResources {
                cpu_usage_percent: 40.0,
                gpu_usage_percent: 0.0,
                ram_usage_percent: 60.0,
                network_usage_mbps: 20.0,
                active_tasks: 0,
                queue_length: 0,
            },
            location: None,
        }
    }

    fn score_node(&self, node: &ComputeNode, req: &ComputeRequirements) -> f32 {
        let mut score = 100.0;

        // Check GPU requirement
        if req.requires_gpu && !node.capabilities.gpu_available {
            return 0.0;
        }

        // Check minimum resources
        if node.capabilities.ram_gb < req.min_ram_gb {
            return 0.0;
        }

        // Penalize for current usage
        score -= node.resources.cpu_usage_percent * 0.3;
        score -= node.resources.gpu_usage_percent * 0.5;
        score -= node.resources.ram_usage_percent * 0.2;

        // Prefer nodes with lower latency
        if let Some(loc) = &node.location {
            score -= loc.network_latency_ms * 0.1;
        }

        // Bonus for model support
        if req.model_name.is_some() {
            let model = req.model_name.as_ref().unwrap();
            if node.capabilities.supported_models.contains(model) {
                score += 20.0;
            }
        }

        score.max(0.0)
    }
}

/// Requirements for compute task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequirements {
    pub requires_gpu: bool,
    pub min_ram_gb: f32,
    pub min_bandwidth_mbps: u32,
    pub model_name: Option<String>,
    pub max_latency_ms: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_local_node() {
        let protocol = ComputeNodeProtocol::new();
        
        let caps = NodeCapabilities {
            cpu_cores: 8,
            gpu_available: true,
            gpu_memory_gb: 16.0,
            ram_gb: 32.0,
            storage_gb: 512.0,
            network_bandwidth_mbps: 1000,
            supported_models: vec!["llama".to_string()],
            hardware_acceleration: vec![HardwareAccel::CUDA],
        };

        let node_id = protocol.register_local_node(caps).await;
        assert!(node_id.starts_with("node-"));

        let node = protocol.get_node(&node_id).await.unwrap();
        assert_eq!(node.capabilities.cpu_cores, 8);
        assert_eq!(node.status, NodeStatus::Available);
    }

    #[tokio::test]
    async fn test_discover_nodes() {
        let protocol = ComputeNodeProtocol::new();
        let discovered = protocol.discover_nodes().await;
        
        assert_eq!(discovered.len(), 3);
        assert!(discovered.contains(&"edge-1".to_string()));
        assert!(discovered.contains(&"mobile-1".to_string()));
    }

    #[tokio::test]
    async fn test_list_available_nodes() {
        let protocol = ComputeNodeProtocol::new();
        protocol.discover_nodes().await;
        
        let available = protocol.list_available_nodes().await;
        assert_eq!(available.len(), 3);
        
        // Mark one as busy
        protocol.update_node_status("edge-1", NodeStatus::Busy).await;
        
        let available = protocol.list_available_nodes().await;
        assert_eq!(available.len(), 2);
    }

    #[tokio::test]
    async fn test_find_best_node() {
        let protocol = ComputeNodeProtocol::new();
        protocol.discover_nodes().await;

        // Require GPU
        let requirements = ComputeRequirements {
            requires_gpu: true,
            min_ram_gb: 16.0,
            min_bandwidth_mbps: 100,
            model_name: Some("llama".to_string()),
            max_latency_ms: Some(50.0),
        };

        let best = protocol.find_best_node(&requirements).await;
        assert!(best.is_some());
        
        let node_id = best.unwrap();
        assert!(node_id.starts_with("edge-")); // Edge nodes have GPUs
    }

    #[tokio::test]
    async fn test_update_node_resources() {
        let protocol = ComputeNodeProtocol::new();
        protocol.discover_nodes().await;

        let new_resources = NodeResources {
            cpu_usage_percent: 80.0,
            gpu_usage_percent: 90.0,
            ram_usage_percent: 75.0,
            network_usage_mbps: 500.0,
            active_tasks: 5,
            queue_length: 2,
        };

        protocol.update_node_resources("edge-1", new_resources).await;

        let node = protocol.get_node("edge-1").await.unwrap();
        assert_eq!(node.resources.cpu_usage_percent, 80.0);
        assert_eq!(node.resources.active_tasks, 5);
    }

    #[tokio::test]
    async fn test_cleanup_offline_nodes() {
        let protocol = ComputeNodeProtocol::new();
        protocol.discover_nodes().await;
        
        assert_eq!(protocol.nodes.read().await.len(), 3);
        
        // Mark nodes as offline
        protocol.update_node_status("edge-1", NodeStatus::Offline).await;
        protocol.update_node_status("mobile-1", NodeStatus::Unreachable).await;
        
        let removed = protocol.cleanup_offline_nodes().await;
        assert_eq!(removed, 2);
        assert_eq!(protocol.nodes.read().await.len(), 1);
    }

    #[tokio::test]
    async fn test_node_scoring() {
        let protocol = ComputeNodeProtocol::new();
        protocol.discover_nodes().await;

        // High requirements should exclude mobile nodes
        let requirements = ComputeRequirements {
            requires_gpu: true,
            min_ram_gb: 24.0,
            min_bandwidth_mbps: 500,
            model_name: None,
            max_latency_ms: None,
        };

        let best = protocol.find_best_node(&requirements).await;
        assert!(best.is_some());
        
        let node = protocol.get_node(&best.unwrap()).await.unwrap();
        assert!(node.capabilities.gpu_available);
        assert!(node.capabilities.ram_gb >= 24.0);
    }
}
