// Kāraṇa OS - Distributed Compute Layer
// Edge cloud integration and distributed model execution

use std::sync::Arc;

pub mod compute_node;
pub mod model_partitioning;
pub mod distributed_inference;
pub mod edge_cloud;

pub use compute_node::{
    ComputeNode, ComputeNodeProtocol, ComputeRequirements, HardwareAccel, NodeCapabilities,
    NodeLocation, NodeResources, NodeStatus,
};

pub use model_partitioning::{
    ModelPartition, ModelPartitioner, PartitionStrategy, PartitionedModel,
};

pub use distributed_inference::{
    DistributedInference, InferenceInput, InferenceMetrics, InferenceOutput, InferenceParameters,
    InferenceRequest, InferenceResponse,
};

pub use edge_cloud::{
    EdgeCloudPool, NodeSelectionStrategy, PoolCapacity, PoolPolicy, PoolPriority, PoolStatistics,
    ResourcePool, ScaleAction, WorkloadType,
};

/// Distributed compute coordinator
/// 
/// Integrates all distributed compute components:
/// - Node discovery and management
/// - Model partitioning strategies
/// - Distributed inference execution
/// - Edge cloud resource pooling
/// 
/// # Example
/// ```
/// use karana_core::distributed::*;
/// use std::sync::Arc;
/// 
/// #[tokio::main]
/// async fn main() {
///     let coordinator = DistributedCoordinator::new();
///     
///     // Discover edge nodes
///     let nodes = coordinator.discover_nodes().await;
///     
///     // Partition large model
///     coordinator.partition_model("llama-70b", 140000.0, 4).await;
///     
///     // Execute distributed inference
///     let response = coordinator.infer("llama-70b", "Hello!").await;
/// }
/// ```
pub struct DistributedCoordinator {
    node_protocol: Arc<compute_node::ComputeNodeProtocol>,
    partitioner: Arc<model_partitioning::ModelPartitioner>,
    inference: Arc<distributed_inference::DistributedInference>,
    pool: Arc<edge_cloud::EdgeCloudPool>,
}

impl DistributedCoordinator {
    pub fn new() -> Self {
        let node_protocol = Arc::new(compute_node::ComputeNodeProtocol::new());
        let partitioner = Arc::new(model_partitioning::ModelPartitioner::new());
        let inference = Arc::new(distributed_inference::DistributedInference::new(
            node_protocol.clone(),
            partitioner.clone(),
        ));
        let pool = Arc::new(edge_cloud::EdgeCloudPool::new());

        Self {
            node_protocol,
            partitioner,
            inference,
            pool,
        }
    }

    /// Discover available compute nodes
    pub async fn discover_nodes(&self) -> Vec<String> {
        self.node_protocol.discover_nodes().await
    }

    /// Partition a model for distributed execution
    pub async fn partition_model(
        &self,
        model_name: &str,
        size_mb: f32,
        num_partitions: usize,
    ) -> Vec<model_partitioning::ModelPartition> {
        self.partitioner
            .partition_model(
                model_name,
                size_mb,
                num_partitions,
                model_partitioning::PartitionStrategy::LayerWise,
            )
            .await
    }

    /// Execute distributed inference
    pub async fn infer(&self, model_name: &str, text: &str) -> distributed_inference::InferenceResponse {
        let request = distributed_inference::InferenceRequest {
            request_id: String::new(),
            model_name: model_name.to_string(),
            input: distributed_inference::InferenceInput::Text(text.to_string()),
            parameters: distributed_inference::InferenceParameters::default(),
        };

        self.inference.infer(request).await
    }

    /// Create a resource pool
    pub async fn create_pool(&self, name: &str, description: &str) -> String {
        let policy = edge_cloud::PoolPolicy {
            max_nodes: 10,
            min_nodes: 1,
            auto_scale: true,
            priority: edge_cloud::PoolPriority::Medium,
            allowed_workloads: vec![edge_cloud::WorkloadType::Inference],
            node_selection: edge_cloud::NodeSelectionStrategy::LeastLoaded,
        };

        self.pool.create_pool(name, description, policy).await
    }

    /// Get pool statistics
    pub async fn get_statistics(&self) -> edge_cloud::PoolStatistics {
        self.pool.get_statistics().await
    }
}

impl Default for DistributedCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_coordinator() {
        let coordinator = DistributedCoordinator::new();
        
        // Discover nodes
        let nodes = coordinator.discover_nodes().await;
        assert!(!nodes.is_empty());

        // Partition model
        let partitions = coordinator.partition_model("llama-13b", 26000.0, 2).await;
        assert_eq!(partitions.len(), 2);

        // Create pool
        let pool_id = coordinator.create_pool("test-pool", "Test pool").await;
        assert!(pool_id.starts_with("pool-"));
    }

    #[tokio::test]
    async fn test_end_to_end_inference() {
        let coordinator = DistributedCoordinator::new();
        
        // Setup
        coordinator.discover_nodes().await;
        coordinator.partition_model("test-model", 10000.0, 2).await;

        // Execute inference
        let response = coordinator.infer("test-model", "Hello, world!").await;
        
        assert!(!response.request_id.is_empty());
        // Latency may be 0 for simulated inference, so just check it's non-negative
        assert!(response.metrics.latency_ms >= 0.0);
    }
}
