// Kāraṇa OS - Phase 55: Intelligent Workload Splitting
// Distribute compute between on-head device and belt-worn companion

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Compute node type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputeNode {
    /// On-head device (glasses)
    OnHead,
    
    /// Belt-worn companion device
    BeltWorn,
    
    /// Phone in pocket
    Phone,
    
    /// Cloud/edge server
    Cloud,
}

impl ComputeNode {
    /// Get typical compute capability (TFLOPS)
    pub fn compute_capability(&self) -> f32 {
        match self {
            Self::OnHead => 0.5,      // Limited by thermal/power
            Self::BeltWorn => 2.0,    // More cooling, larger battery
            Self::Phone => 1.5,       // Modern smartphone
            Self::Cloud => 100.0,     // Datacenter GPU
        }
    }
    
    /// Get typical memory capacity (GB)
    pub fn memory_capacity(&self) -> f32 {
        match self {
            Self::OnHead => 4.0,
            Self::BeltWorn => 8.0,
            Self::Phone => 6.0,
            Self::Cloud => 64.0,
        }
    }
    
    /// Get typical latency to this node (ms)
    pub fn latency_ms(&self, from: ComputeNode) -> f32 {
        if from == *self {
            return 0.0;
        }
        
        match (from, self) {
            (ComputeNode::OnHead, ComputeNode::BeltWorn) => 2.0,   // Bluetooth LE
            (ComputeNode::OnHead, ComputeNode::Phone) => 3.0,      // Bluetooth
            (ComputeNode::OnHead, ComputeNode::Cloud) => 50.0,     // WiFi + Internet
            (ComputeNode::BeltWorn, ComputeNode::Cloud) => 45.0,   // 5G
            _ => 10.0, // Default
        }
    }
    
    /// Get bandwidth to this node (Mbps)
    pub fn bandwidth_mbps(&self, from: ComputeNode) -> f32 {
        if from == *self {
            return f32::MAX;
        }
        
        match (from, self) {
            (ComputeNode::OnHead, ComputeNode::BeltWorn) => 10.0,  // BLE limited
            (ComputeNode::OnHead, ComputeNode::Phone) => 25.0,     // Bluetooth 5
            (ComputeNode::OnHead, ComputeNode::Cloud) => 100.0,    // WiFi
            (ComputeNode::BeltWorn, ComputeNode::Cloud) => 200.0,  // 5G
            _ => 50.0,
        }
    }
}

/// Workload characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProfile {
    pub name: String,
    pub compute_gflops: f32,
    pub memory_mb: f32,
    pub latency_sensitive: bool,
    pub max_latency_ms: f32,
    pub input_size_mb: f32,
    pub output_size_mb: f32,
    pub can_partition: bool,
}

/// Workload placement decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementDecision {
    pub workload: String,
    pub node: ComputeNode,
    pub reason: String,
    pub estimated_latency_ms: f32,
    pub estimated_energy_mw: f32,
    pub confidence: f32,
}

/// Workload splitting and placement optimizer
pub struct WorkloadSplitter {
    available_nodes: Arc<RwLock<HashMap<ComputeNode, NodeStatus>>>,
    placement_history: Arc<RwLock<Vec<PlacementDecision>>>,
    policies: PlacementPolicies,
}

/// Node status and availability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node: ComputeNode,
    pub available: bool,
    pub current_load: f32,  // 0.0 to 1.0
    pub temperature_c: f32,
    pub battery_percent: f32,
    pub last_seen: u64,
}

/// Placement policies and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementPolicies {
    /// Prefer on-device over cloud for privacy
    pub prefer_on_device: bool,
    
    /// Maximum acceptable latency for interactive tasks (ms)
    pub max_interactive_latency_ms: f32,
    
    /// Temperature threshold for throttling on-head compute (°C)
    pub thermal_threshold_c: f32,
    
    /// Minimum battery level to use on-head compute (%)
    pub min_battery_percent: f32,
    
    /// Maximum load before offloading (0.0 to 1.0)
    pub max_node_load: f32,
}

impl Default for PlacementPolicies {
    fn default() -> Self {
        Self {
            prefer_on_device: true,
            max_interactive_latency_ms: 100.0,
            thermal_threshold_c: 35.0,
            min_battery_percent: 20.0,
            max_node_load: 0.8,
        }
    }
}

impl WorkloadSplitter {
    /// Create new workload splitter
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        
        // Initialize with on-head device
        nodes.insert(ComputeNode::OnHead, NodeStatus {
            node: ComputeNode::OnHead,
            available: true,
            current_load: 0.0,
            temperature_c: 25.0,
            battery_percent: 100.0,
            last_seen: 0,
        });
        
        Self {
            available_nodes: Arc::new(RwLock::new(nodes)),
            placement_history: Arc::new(RwLock::new(Vec::new())),
            policies: PlacementPolicies::default(),
        }
    }
    
    /// Set placement policies
    pub fn with_policies(mut self, policies: PlacementPolicies) -> Self {
        self.policies = policies;
        self
    }
    
    /// Register a compute node
    pub async fn register_node(&self, status: NodeStatus) {
        self.available_nodes
            .write()
            .await
            .insert(status.node, status);
    }
    
    /// Update node status
    pub async fn update_node_status(&self, node: ComputeNode, status: NodeStatus) {
        self.available_nodes
            .write()
            .await
            .insert(node, status);
    }
    
    /// Decide optimal placement for a workload
    pub async fn place_workload(&self, workload: WorkloadProfile) -> Result<PlacementDecision> {
        let nodes = self.available_nodes.read().await;
        
        // Evaluate each available node
        let mut candidates = Vec::new();
        
        for (node_type, status) in nodes.iter() {
            if !status.available {
                continue;
            }
            
            // Skip on-head if thermal/battery constraints violated
            if *node_type == ComputeNode::OnHead {
                if status.temperature_c > self.policies.thermal_threshold_c {
                    continue;
                }
                if status.battery_percent < self.policies.min_battery_percent {
                    continue;
                }
            }
            
            // Skip if node is overloaded
            if status.current_load > self.policies.max_node_load {
                continue;
            }
            
            // Calculate suitability score
            let score = self.calculate_suitability_score(
                &workload,
                *node_type,
                status,
            );
            
            candidates.push((*node_type, score, status.clone()));
        }
        
        if candidates.is_empty() {
            return Err(anyhow!("No suitable compute nodes available"));
        }
        
        // Sort by score (higher is better)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let (best_node, score, status) = &candidates[0];
        
        // Calculate estimated metrics
        let estimated_latency_ms = self.estimate_latency(&workload, *best_node);
        let estimated_energy_mw = self.estimate_energy(&workload, *best_node);
        
        let reason = self.generate_placement_reason(&workload, *best_node, &status);
        
        let decision = PlacementDecision {
            workload: workload.name.clone(),
            node: *best_node,
            reason,
            estimated_latency_ms,
            estimated_energy_mw,
            confidence: *score,
        };
        
        // Record decision
        self.placement_history.write().await.push(decision.clone());
        
        Ok(decision)
    }
    
    /// Calculate suitability score for placing workload on node
    fn calculate_suitability_score(
        &self,
        workload: &WorkloadProfile,
        node: ComputeNode,
        status: &NodeStatus,
    ) -> f32 {
        let mut score = 1.0;
        
        // Check compute capability
        let compute_ratio = workload.compute_gflops / node.compute_capability();
        if compute_ratio > 1.0 {
            score *= 0.5; // Workload exceeds node capability
        }
        
        // Check memory capacity
        let memory_ratio = workload.memory_mb / (node.memory_capacity() * 1024.0);
        if memory_ratio > 0.8 {
            score *= 0.6; // High memory usage
        }
        
        // Latency sensitivity
        if workload.latency_sensitive {
            let latency = self.estimate_latency(workload, node);
            if latency > workload.max_latency_ms {
                score *= 0.3; // Violates latency constraint
            } else {
                score *= 1.2; // Good latency
            }
        }
        
        // Prefer on-device for privacy
        if self.policies.prefer_on_device {
            match node {
                ComputeNode::OnHead | ComputeNode::BeltWorn | ComputeNode::Phone => {
                    score *= 1.3;
                }
                ComputeNode::Cloud => {
                    score *= 0.7;
                }
            }
        }
        
        // Penalize high current load
        score *= 1.0 - (status.current_load * 0.5);
        
        // Penalize high temperature on-head
        if node == ComputeNode::OnHead && status.temperature_c > 30.0 {
            score *= 1.0 - ((status.temperature_c - 30.0) / 10.0);
        }
        
        // Penalize low battery
        if status.battery_percent < 50.0 {
            score *= status.battery_percent / 100.0;
        }
        
        score.max(0.0).min(1.0)
    }
    
    /// Estimate end-to-end latency for workload on node
    fn estimate_latency(&self, workload: &WorkloadProfile, node: ComputeNode) -> f32 {
        // Compute time
        let compute_time = (workload.compute_gflops / node.compute_capability()) * 1000.0;
        
        // Transfer time (if not on-head)
        let transfer_time = if node != ComputeNode::OnHead {
            let data_size = workload.input_size_mb + workload.output_size_mb;
            let bandwidth = node.bandwidth_mbps(ComputeNode::OnHead);
            (data_size / bandwidth) * 1000.0 * 8.0  // Convert to ms
        } else {
            0.0
        };
        
        // Network latency
        let network_latency = node.latency_ms(ComputeNode::OnHead);
        
        compute_time + transfer_time + network_latency
    }
    
    /// Estimate energy consumption for workload on node
    fn estimate_energy(&self, workload: &WorkloadProfile, node: ComputeNode) -> f32 {
        // Simplified energy model
        let compute_energy = workload.compute_gflops * 100.0; // 100mW per GFLOPS
        
        let transfer_energy = if node != ComputeNode::OnHead {
            let data_size = workload.input_size_mb + workload.output_size_mb;
            data_size * 50.0  // 50mW per MB transferred
        } else {
            0.0
        };
        
        compute_energy + transfer_energy
    }
    
    /// Generate human-readable reason for placement
    fn generate_placement_reason(
        &self,
        workload: &WorkloadProfile,
        node: ComputeNode,
        status: &NodeStatus,
    ) -> String {
        if workload.latency_sensitive && node == ComputeNode::OnHead {
            return "Latency-critical workload requires on-head processing".to_string();
        }
        
        if node == ComputeNode::OnHead && status.temperature_c < 30.0 {
            return "Sufficient thermal headroom for on-head processing".to_string();
        }
        
        if node == ComputeNode::BeltWorn {
            return "Offloaded to belt-worn device to reduce head thermal load".to_string();
        }
        
        if node == ComputeNode::Cloud {
            if workload.compute_gflops > 2.0 {
                return "Heavy workload requires cloud compute".to_string();
            }
            return "On-device resources constrained, using cloud".to_string();
        }
        
        format!("Placed on {:?} based on resource availability", node)
    }
    
    /// Get placement statistics
    pub async fn stats(&self) -> PlacementStats {
        let history = self.placement_history.read().await;
        
        let total_placements = history.len();
        let mut node_counts: HashMap<ComputeNode, usize> = HashMap::new();
        let mut total_latency = 0.0;
        let mut total_energy = 0.0;
        
        for decision in history.iter() {
            *node_counts.entry(decision.node).or_insert(0) += 1;
            total_latency += decision.estimated_latency_ms;
            total_energy += decision.estimated_energy_mw;
        }
        
        let avg_latency = if total_placements > 0 {
            total_latency / total_placements as f32
        } else {
            0.0
        };
        
        let avg_energy = if total_placements > 0 {
            total_energy / total_placements as f32
        } else {
            0.0
        };
        
        PlacementStats {
            total_placements,
            placements_by_node: node_counts,
            avg_latency_ms: avg_latency,
            avg_energy_mw: avg_energy,
        }
    }
}

/// Placement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementStats {
    pub total_placements: usize,
    pub placements_by_node: HashMap<ComputeNode, usize>,
    pub avg_latency_ms: f32,
    pub avg_energy_mw: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_node_properties() {
        assert!(ComputeNode::BeltWorn.compute_capability() > ComputeNode::OnHead.compute_capability());
        assert!(ComputeNode::Cloud.compute_capability() > ComputeNode::BeltWorn.compute_capability());
        assert_eq!(ComputeNode::OnHead.latency_ms(ComputeNode::OnHead), 0.0);
    }
    
    #[tokio::test]
    async fn test_workload_placement_latency_critical() {
        let splitter = WorkloadSplitter::new();
        
        // Register belt-worn device
        splitter.register_node(NodeStatus {
            node: ComputeNode::BeltWorn,
            available: true,
            current_load: 0.3,
            temperature_c: 25.0,
            battery_percent: 80.0,
            last_seen: 0,
        }).await;
        
        let workload = WorkloadProfile {
            name: "ar_rendering".to_string(),
            compute_gflops: 0.3,
            memory_mb: 200.0,
            latency_sensitive: true,
            max_latency_ms: 16.0,  // 60 FPS
            input_size_mb: 1.0,
            output_size_mb: 0.5,
            can_partition: false,
        };
        
        let decision = splitter.place_workload(workload).await.unwrap();
        
        // Should prefer on-head for latency-critical
        assert_eq!(decision.node, ComputeNode::OnHead);
        assert!(decision.estimated_latency_ms < 16.0);
    }
    
    #[tokio::test]
    async fn test_workload_placement_thermal_constrained() {
        let splitter = WorkloadSplitter::new();
        
        // Update on-head to be thermally constrained
        splitter.update_node_status(
            ComputeNode::OnHead,
            NodeStatus {
                node: ComputeNode::OnHead,
                available: true,
                current_load: 0.8,
                temperature_c: 38.0,  // Above threshold
                battery_percent: 50.0,
                last_seen: 0,
            },
        ).await;
        
        // Register belt-worn
        splitter.register_node(NodeStatus {
            node: ComputeNode::BeltWorn,
            available: true,
            current_load: 0.2,
            temperature_c: 25.0,
            battery_percent: 90.0,
            last_seen: 0,
        }).await;
        
        let workload = WorkloadProfile {
            name: "model_inference".to_string(),
            compute_gflops: 1.0,
            memory_mb: 500.0,
            latency_sensitive: false,
            max_latency_ms: 500.0,
            input_size_mb: 2.0,
            output_size_mb: 0.1,
            can_partition: true,
        };
        
        let decision = splitter.place_workload(workload).await.unwrap();
        
        // Should offload to belt-worn due to thermal constraint
        assert_eq!(decision.node, ComputeNode::BeltWorn);
    }
    
    #[tokio::test]
    async fn test_placement_stats() {
        let splitter = WorkloadSplitter::new();
        
        splitter.register_node(NodeStatus {
            node: ComputeNode::BeltWorn,
            available: true,
            current_load: 0.3,
            temperature_c: 25.0,
            battery_percent: 80.0,
            last_seen: 0,
        }).await;
        
        // Place multiple workloads
        for i in 0..5 {
            let workload = WorkloadProfile {
                name: format!("workload_{}", i),
                compute_gflops: 0.2,
                memory_mb: 100.0,
                latency_sensitive: i % 2 == 0,
                max_latency_ms: 100.0,
                input_size_mb: 0.5,
                output_size_mb: 0.1,
                can_partition: false,
            };
            let _ = splitter.place_workload(workload).await;
        }
        
        let stats = splitter.stats().await;
        assert_eq!(stats.total_placements, 5);
        assert!(stats.avg_latency_ms > 0.0);
    }
}
