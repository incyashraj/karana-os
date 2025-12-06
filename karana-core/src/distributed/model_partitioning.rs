// Kāraṇa OS - Phase 52: Model Partitioning
// Split large models across multiple compute nodes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Model partition strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PartitionStrategy {
    LayerWise,      // Split by transformer layers
    TensorParallel, // Split tensors horizontally
    Pipeline,       // Pipeline parallel with stages
    Hybrid,         // Combination of strategies
}

/// Model partition metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPartition {
    pub partition_id: String,
    pub model_name: String,
    pub strategy: PartitionStrategy,
    pub layer_range: Option<(usize, usize)>,
    pub total_layers: usize,
    pub size_mb: f32,
    pub node_id: Option<String>,
    pub dependencies: Vec<String>, // Other partition IDs this depends on
}

/// Partitioned model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionedModel {
    pub model_name: String,
    pub original_size_mb: f32,
    pub strategy: PartitionStrategy,
    pub partitions: Vec<ModelPartition>,
    pub coordination_overhead_ms: f32,
}

/// Model partitioning manager
pub struct ModelPartitioner {
    partitioned_models: Arc<RwLock<HashMap<String, PartitionedModel>>>,
    partition_cache: Arc<RwLock<HashMap<String, ModelPartition>>>,
}

impl ModelPartitioner {
    pub fn new() -> Self {
        Self {
            partitioned_models: Arc::new(RwLock::new(HashMap::new())),
            partition_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Partition a large model across multiple nodes
    pub async fn partition_model(
        &self,
        model_name: &str,
        model_size_mb: f32,
        num_partitions: usize,
        strategy: PartitionStrategy,
    ) -> Vec<ModelPartition> {
        let partitions = match strategy {
            PartitionStrategy::LayerWise => {
                self.create_layer_wise_partitions(model_name, model_size_mb, num_partitions)
            }
            PartitionStrategy::TensorParallel => {
                self.create_tensor_parallel_partitions(model_name, model_size_mb, num_partitions)
            }
            PartitionStrategy::Pipeline => {
                self.create_pipeline_partitions(model_name, model_size_mb, num_partitions)
            }
            PartitionStrategy::Hybrid => {
                self.create_hybrid_partitions(model_name, model_size_mb, num_partitions)
            }
        };

        let partitioned = PartitionedModel {
            model_name: model_name.to_string(),
            original_size_mb: model_size_mb,
            strategy,
            partitions: partitions.clone(),
            coordination_overhead_ms: self.estimate_coordination_overhead(strategy, num_partitions),
        };

        self.partitioned_models
            .write()
            .await
            .insert(model_name.to_string(), partitioned);

        // Cache individual partitions
        let mut cache = self.partition_cache.write().await;
        for partition in &partitions {
            cache.insert(partition.partition_id.clone(), partition.clone());
        }

        partitions
    }

    /// Assign partitions to compute nodes
    pub async fn assign_partitions_to_nodes(
        &self,
        model_name: &str,
        node_assignments: HashMap<String, String>, // partition_id -> node_id
    ) -> bool {
        let mut models = self.partitioned_models.write().await;
        
        if let Some(model) = models.get_mut(model_name) {
            for partition in &mut model.partitions {
                if let Some(node_id) = node_assignments.get(&partition.partition_id) {
                    partition.node_id = Some(node_id.clone());
                }
            }

            // Update cache
            let mut cache = self.partition_cache.write().await;
            for partition in &model.partitions {
                cache.insert(partition.partition_id.clone(), partition.clone());
            }

            true
        } else {
            false
        }
    }

    /// Get partitioned model configuration
    pub async fn get_partitioned_model(&self, model_name: &str) -> Option<PartitionedModel> {
        self.partitioned_models.read().await.get(model_name).cloned()
    }

    /// Get specific partition by ID
    pub async fn get_partition(&self, partition_id: &str) -> Option<ModelPartition> {
        self.partition_cache.read().await.get(partition_id).cloned()
    }

    /// Get all partitions assigned to a specific node
    pub async fn get_node_partitions(&self, node_id: &str) -> Vec<ModelPartition> {
        self.partition_cache
            .read()
            .await
            .values()
            .filter(|p| p.node_id.as_ref() == Some(&node_id.to_string()))
            .cloned()
            .collect()
    }

    /// Calculate optimal partition count based on model size and available nodes
    pub fn calculate_optimal_partitions(&self, model_size_mb: f32, num_nodes: usize) -> usize {
        // Heuristic: partition if model > 2GB and we have multiple nodes
        if model_size_mb < 2000.0 || num_nodes == 1 {
            return 1;
        }

        // Aim for ~1-2GB per partition
        let ideal_size_per_partition = 1500.0;
        let calculated = (model_size_mb / ideal_size_per_partition).ceil() as usize;
        
        // Don't exceed number of available nodes
        calculated.min(num_nodes).max(1)
    }

    /// Estimate memory requirement for a partition
    pub fn estimate_partition_memory(&self, partition: &ModelPartition) -> f32 {
        // Model weights + activations + KV cache
        partition.size_mb * 1.5 // 50% overhead for inference
    }

    // Private helper methods
    fn create_layer_wise_partitions(
        &self,
        model_name: &str,
        total_size_mb: f32,
        num_partitions: usize,
    ) -> Vec<ModelPartition> {
        let total_layers = self.estimate_layer_count(total_size_mb);
        let layers_per_partition = (total_layers as f32 / num_partitions as f32).ceil() as usize;
        let size_per_partition = total_size_mb / num_partitions as f32;

        (0..num_partitions)
            .map(|i| {
                let start_layer = i * layers_per_partition;
                let end_layer = ((i + 1) * layers_per_partition).min(total_layers);
                
                ModelPartition {
                    partition_id: format!("{}-layer-{}", model_name, i),
                    model_name: model_name.to_string(),
                    strategy: PartitionStrategy::LayerWise,
                    layer_range: Some((start_layer, end_layer)),
                    total_layers,
                    size_mb: size_per_partition,
                    node_id: None,
                    dependencies: if i > 0 {
                        vec![format!("{}-layer-{}", model_name, i - 1)]
                    } else {
                        vec![]
                    },
                }
            })
            .collect()
    }

    fn create_tensor_parallel_partitions(
        &self,
        model_name: &str,
        total_size_mb: f32,
        num_partitions: usize,
    ) -> Vec<ModelPartition> {
        let total_layers = self.estimate_layer_count(total_size_mb);
        let size_per_partition = total_size_mb / num_partitions as f32;

        (0..num_partitions)
            .map(|i| ModelPartition {
                partition_id: format!("{}-tensor-{}", model_name, i),
                model_name: model_name.to_string(),
                strategy: PartitionStrategy::TensorParallel,
                layer_range: Some((0, total_layers)), // All layers, split tensors
                total_layers,
                size_mb: size_per_partition,
                node_id: None,
                dependencies: vec![], // All partitions work in parallel
            })
            .collect()
    }

    fn create_pipeline_partitions(
        &self,
        model_name: &str,
        total_size_mb: f32,
        num_partitions: usize,
    ) -> Vec<ModelPartition> {
        let total_layers = self.estimate_layer_count(total_size_mb);
        let layers_per_stage = (total_layers as f32 / num_partitions as f32).ceil() as usize;
        let size_per_partition = total_size_mb / num_partitions as f32;

        (0..num_partitions)
            .map(|i| {
                let start_layer = i * layers_per_stage;
                let end_layer = ((i + 1) * layers_per_stage).min(total_layers);
                
                ModelPartition {
                    partition_id: format!("{}-stage-{}", model_name, i),
                    model_name: model_name.to_string(),
                    strategy: PartitionStrategy::Pipeline,
                    layer_range: Some((start_layer, end_layer)),
                    total_layers,
                    size_mb: size_per_partition,
                    node_id: None,
                    dependencies: if i > 0 {
                        vec![format!("{}-stage-{}", model_name, i - 1)]
                    } else {
                        vec![]
                    },
                }
            })
            .collect()
    }

    fn create_hybrid_partitions(
        &self,
        model_name: &str,
        total_size_mb: f32,
        num_partitions: usize,
    ) -> Vec<ModelPartition> {
        // Combine pipeline and tensor parallelism
        let num_stages = (num_partitions as f32).sqrt().ceil() as usize;
        let tensor_splits_per_stage = (num_partitions + num_stages - 1) / num_stages; // Round up
        
        let total_layers = self.estimate_layer_count(total_size_mb);
        let layers_per_stage = (total_layers as f32 / num_stages as f32).ceil() as usize;
        let size_per_partition = total_size_mb / num_partitions as f32;

        let mut partitions = Vec::new();
        let mut partition_idx = 0;

        for stage in 0..num_stages {
            let start_layer = stage * layers_per_stage;
            let end_layer = ((stage + 1) * layers_per_stage).min(total_layers);

            // Don't exceed total partitions
            let partitions_this_stage = if partition_idx + tensor_splits_per_stage > num_partitions {
                num_partitions - partition_idx
            } else {
                tensor_splits_per_stage
            };

            for _tensor_idx in 0..partitions_this_stage {
                let prev_stage_deps: Vec<String> = if stage > 0 {
                    // Depend on all partitions from previous stage
                    let prev_stage_start = partitions.iter()
                        .filter(|p: &&ModelPartition| p.layer_range.as_ref().map(|(s, _)| *s < start_layer).unwrap_or(false))
                        .count();
                    let prev_partitions_this_stage = if prev_stage_start == 0 {
                        0
                    } else {
                        partitions.len() - prev_stage_start
                    };
                    (partition_idx.saturating_sub(prev_partitions_this_stage)..partition_idx)
                        .map(|i| format!("{}-hybrid-{}", model_name, i))
                        .collect()
                } else {
                    vec![]
                };

                partitions.push(ModelPartition {
                    partition_id: format!("{}-hybrid-{}", model_name, partition_idx),
                    model_name: model_name.to_string(),
                    strategy: PartitionStrategy::Hybrid,
                    layer_range: Some((start_layer, end_layer)),
                    total_layers,
                    size_mb: size_per_partition,
                    node_id: None,
                    dependencies: prev_stage_deps,
                });

                partition_idx += 1;
                
                // Break early if we've created all partitions
                if partition_idx >= num_partitions {
                    break;
                }
            }
            
            if partition_idx >= num_partitions {
                break;
            }
        }

        partitions
    }

    fn estimate_layer_count(&self, model_size_mb: f32) -> usize {
        // Rough heuristic: ~50MB per transformer layer for large models
        ((model_size_mb / 50.0).round() as usize).max(12)
    }

    fn estimate_coordination_overhead(&self, strategy: PartitionStrategy, num_partitions: usize) -> f32 {
        match strategy {
            PartitionStrategy::LayerWise => {
                // Sequential, high communication overhead
                num_partitions as f32 * 2.0 // 2ms per layer transfer
            }
            PartitionStrategy::TensorParallel => {
                // Parallel, but needs all-reduce
                5.0 + (num_partitions as f32 * 0.5) // Base + per-partition sync
            }
            PartitionStrategy::Pipeline => {
                // Pipeline parallel, medium overhead
                num_partitions as f32 * 1.5
            }
            PartitionStrategy::Hybrid => {
                // Combination of both
                num_partitions as f32 * 1.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_layer_wise_partitioning() {
        let partitioner = ModelPartitioner::new();
        
        let partitions = partitioner
            .partition_model("llama-70b", 140000.0, 4, PartitionStrategy::LayerWise)
            .await;

        assert_eq!(partitions.len(), 4);
        assert_eq!(partitions[0].strategy, PartitionStrategy::LayerWise);
        
        // Check dependencies (sequential)
        assert_eq!(partitions[0].dependencies.len(), 0);
        assert_eq!(partitions[1].dependencies.len(), 1);
        assert!(partitions[1].dependencies[0].contains("layer-0"));
    }

    #[tokio::test]
    async fn test_tensor_parallel_partitioning() {
        let partitioner = ModelPartitioner::new();
        
        let partitions = partitioner
            .partition_model("llama-70b", 140000.0, 4, PartitionStrategy::TensorParallel)
            .await;

        assert_eq!(partitions.len(), 4);
        
        // All partitions are independent
        for partition in partitions {
            assert_eq!(partition.dependencies.len(), 0);
            assert_eq!(partition.strategy, PartitionStrategy::TensorParallel);
        }
    }

    #[tokio::test]
    async fn test_assign_partitions_to_nodes() {
        let partitioner = ModelPartitioner::new();
        
        partitioner
            .partition_model("llama-13b", 26000.0, 2, PartitionStrategy::LayerWise)
            .await;

        let mut assignments = HashMap::new();
        assignments.insert("llama-13b-layer-0".to_string(), "node-1".to_string());
        assignments.insert("llama-13b-layer-1".to_string(), "node-2".to_string());

        let success = partitioner
            .assign_partitions_to_nodes("llama-13b", assignments)
            .await;

        assert!(success);

        let model = partitioner.get_partitioned_model("llama-13b").await.unwrap();
        assert_eq!(model.partitions[0].node_id, Some("node-1".to_string()));
        assert_eq!(model.partitions[1].node_id, Some("node-2".to_string()));
    }

    #[tokio::test]
    async fn test_get_node_partitions() {
        let partitioner = ModelPartitioner::new();
        
        partitioner
            .partition_model("model-a", 10000.0, 2, PartitionStrategy::LayerWise)
            .await;

        let mut assignments = HashMap::new();
        assignments.insert("model-a-layer-0".to_string(), "node-1".to_string());
        assignments.insert("model-a-layer-1".to_string(), "node-2".to_string());

        partitioner
            .assign_partitions_to_nodes("model-a", assignments)
            .await;

        let node1_partitions = partitioner.get_node_partitions("node-1").await;
        assert_eq!(node1_partitions.len(), 1);
        assert_eq!(node1_partitions[0].partition_id, "model-a-layer-0");
    }

    #[tokio::test]
    async fn test_calculate_optimal_partitions() {
        let partitioner = ModelPartitioner::new();
        
        // Small model, don't partition
        assert_eq!(partitioner.calculate_optimal_partitions(1000.0, 4), 1);
        
        // Large model with multiple nodes
        let optimal = partitioner.calculate_optimal_partitions(12000.0, 8);
        assert!(optimal >= 4 && optimal <= 8);
    }

    #[tokio::test]
    async fn test_estimate_partition_memory() {
        let partitioner = ModelPartitioner::new();
        
        let partition = ModelPartition {
            partition_id: "test".to_string(),
            model_name: "test-model".to_string(),
            strategy: PartitionStrategy::LayerWise,
            layer_range: Some((0, 10)),
            total_layers: 40,
            size_mb: 1000.0,
            node_id: None,
            dependencies: vec![],
        };

        let memory = partitioner.estimate_partition_memory(&partition);
        assert_eq!(memory, 1500.0); // 1.5x overhead
    }

    #[tokio::test]
    async fn test_hybrid_partitioning() {
        let partitioner = ModelPartitioner::new();
        
        let partitions = partitioner
            .partition_model("llama-70b", 140000.0, 8, PartitionStrategy::Hybrid)
            .await;

        assert_eq!(partitions.len(), 8);
        assert_eq!(partitions[0].strategy, PartitionStrategy::Hybrid);
        
        // First stage partitions have no dependencies
        assert_eq!(partitions[0].dependencies.len(), 0);
        assert_eq!(partitions[1].dependencies.len(), 0);
        
        // Later stage partitions depend on previous stage
        assert!(partitions[4].dependencies.len() > 0);
    }
}
