// Kāraṇa OS - Phase 52: Distributed Inference
// Coordinate inference across partitioned models on multiple nodes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

use super::compute_node::{ComputeNodeProtocol, ComputeRequirements};
use super::model_partitioning::{ModelPartitioner, PartitionStrategy};

/// Distributed inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub request_id: String,
    pub model_name: String,
    pub input: InferenceInput,
    pub parameters: InferenceParameters,
}

/// Input data for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceInput {
    Text(String),
    Tokens(Vec<u32>),
    Image(Vec<u8>),
    Audio(Vec<f32>),
    Multimodal {
        text: Option<String>,
        image: Option<Vec<u8>>,
        audio: Option<Vec<f32>>,
    },
}

/// Inference generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParameters {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    pub streaming: bool,
}

impl Default for InferenceParameters {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            streaming: false,
        }
    }
}

/// Inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: String,
    pub output: InferenceOutput,
    pub metrics: InferenceMetrics,
}

/// Output from inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceOutput {
    Text(String),
    Tokens(Vec<u32>),
    Image(Vec<u8>),
    Embedding(Vec<f32>),
    Error(String),
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetrics {
    pub latency_ms: f32,
    pub tokens_per_second: f32,
    pub nodes_used: usize,
    pub coordination_overhead_ms: f32,
    pub memory_used_mb: f32,
}

/// Intermediate partition result
#[derive(Debug, Clone)]
struct PartitionResult {
    partition_id: String,
    node_id: String,
    output: Vec<f32>, // Hidden states or activations
    latency_ms: f32,
}

/// Distributed inference coordinator
pub struct DistributedInference {
    node_protocol: Arc<ComputeNodeProtocol>,
    partitioner: Arc<ModelPartitioner>,
    active_requests: Arc<RwLock<HashMap<String, InferenceRequest>>>,
    partition_results: Arc<RwLock<HashMap<String, Vec<PartitionResult>>>>,
}

impl DistributedInference {
    pub fn new(
        node_protocol: Arc<ComputeNodeProtocol>,
        partitioner: Arc<ModelPartitioner>,
    ) -> Self {
        Self {
            node_protocol,
            partitioner,
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            partition_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute distributed inference request
    pub async fn infer(&self, mut request: InferenceRequest) -> InferenceResponse {
        let start = Instant::now();
        
        // Generate request ID if not provided
        if request.request_id.is_empty() {
            request.request_id = format!("req-{}", uuid::Uuid::new_v4());
        }

        // Register active request
        self.active_requests
            .write()
            .await
            .insert(request.request_id.clone(), request.clone());

        // Check if model is partitioned
        let partitioned = self
            .partitioner
            .get_partitioned_model(&request.model_name)
            .await;

        let (output, metrics) = if let Some(model) = partitioned {
            // Distributed inference across partitions
            self.execute_distributed_inference(&request, &model).await
        } else {
            // Single-node inference
            self.execute_single_node_inference(&request).await
        };

        // Cleanup
        self.active_requests.write().await.remove(&request.request_id);
        self.partition_results.write().await.remove(&request.request_id);

        InferenceResponse {
            request_id: request.request_id,
            output,
            metrics,
        }
    }

    /// Execute inference on a partitioned model
    async fn execute_distributed_inference(
        &self,
        request: &InferenceRequest,
        model: &super::model_partitioning::PartitionedModel,
    ) -> (InferenceOutput, InferenceMetrics) {
        let start = Instant::now();
        let mut total_latency = 0.0;
        let mut memory_used = 0.0;

        match model.strategy {
            PartitionStrategy::LayerWise | PartitionStrategy::Pipeline => {
                // Sequential execution through layers
                let mut hidden_states = self.initial_embedding(&request.input);

                for partition in &model.partitions {
                    let partition_start = Instant::now();
                    
                    // Execute on assigned node
                    if let Some(node_id) = &partition.node_id {
                        hidden_states = self
                            .execute_partition(node_id, partition, &hidden_states)
                            .await;
                        
                        let partition_latency = partition_start.elapsed().as_millis() as f32;
                        total_latency += partition_latency;
                        
                        memory_used += self.partitioner.estimate_partition_memory(partition);
                    }
                }

                let output_text = self.decode_output(hidden_states, &request.parameters);
                let total_ms = start.elapsed().as_millis() as f32;

                (
                    InferenceOutput::Text(output_text.clone()),
                    InferenceMetrics {
                        latency_ms: total_ms,
                        tokens_per_second: self.calculate_tokens_per_second(&output_text, total_ms),
                        nodes_used: model.partitions.len(),
                        coordination_overhead_ms: model.coordination_overhead_ms,
                        memory_used_mb: memory_used,
                    },
                )
            }
            PartitionStrategy::TensorParallel => {
                // Parallel execution with all-reduce
                let hidden_states = self.initial_embedding(&request.input);
                let mut partial_results = Vec::new();

                for partition in &model.partitions {
                    if let Some(node_id) = &partition.node_id {
                        let result = self
                            .execute_partition(node_id, partition, &hidden_states)
                            .await;
                        partial_results.push(result);
                        memory_used += self.partitioner.estimate_partition_memory(partition);
                    }
                }

                // All-reduce to combine results
                let combined = self.all_reduce(partial_results);
                let output_text = self.decode_output(combined, &request.parameters);
                let total_ms = start.elapsed().as_millis() as f32;

                (
                    InferenceOutput::Text(output_text.clone()),
                    InferenceMetrics {
                        latency_ms: total_ms,
                        tokens_per_second: self.calculate_tokens_per_second(&output_text, total_ms),
                        nodes_used: model.partitions.len(),
                        coordination_overhead_ms: model.coordination_overhead_ms,
                        memory_used_mb: memory_used,
                    },
                )
            }
            PartitionStrategy::Hybrid => {
                // Combination of pipeline and tensor parallelism
                let output_text = self.execute_hybrid_inference(request, model).await;
                let total_ms = start.elapsed().as_millis() as f32;

                for partition in &model.partitions {
                    memory_used += self.partitioner.estimate_partition_memory(partition);
                }

                (
                    InferenceOutput::Text(output_text.clone()),
                    InferenceMetrics {
                        latency_ms: total_ms,
                        tokens_per_second: self.calculate_tokens_per_second(&output_text, total_ms),
                        nodes_used: model.partitions.len(),
                        coordination_overhead_ms: model.coordination_overhead_ms,
                        memory_used_mb: memory_used,
                    },
                )
            }
        }
    }

    /// Execute inference on a single node
    async fn execute_single_node_inference(
        &self,
        request: &InferenceRequest,
    ) -> (InferenceOutput, InferenceMetrics) {
        let start = Instant::now();

        // Find best node for this model
        let requirements = ComputeRequirements {
            requires_gpu: true,
            min_ram_gb: 8.0,
            min_bandwidth_mbps: 100,
            model_name: Some(request.model_name.clone()),
            max_latency_ms: None,
        };

        let node_id = self.node_protocol.find_best_node(&requirements).await;

        let output_text = if node_id.is_some() {
            // Execute on selected node
            self.execute_single_inference(&request.input, &request.parameters)
                .await
        } else {
            // No suitable node found
            "Error: No available compute nodes".to_string()
        };

        let total_ms = start.elapsed().as_millis() as f32;

        (
            InferenceOutput::Text(output_text.clone()),
            InferenceMetrics {
                latency_ms: total_ms,
                tokens_per_second: self.calculate_tokens_per_second(&output_text, total_ms),
                nodes_used: 1,
                coordination_overhead_ms: 0.0,
                memory_used_mb: 2048.0, // Estimate
            },
        )
    }

    /// Get metrics for an active request
    pub async fn get_request_metrics(&self, request_id: &str) -> Option<HashMap<String, f32>> {
        let results = self.partition_results.read().await;
        
        if let Some(partition_results) = results.get(request_id) {
            let mut metrics = HashMap::new();
            
            let total_latency: f32 = partition_results.iter().map(|r| r.latency_ms).sum();
            let avg_latency = total_latency / partition_results.len() as f32;
            
            metrics.insert("total_latency_ms".to_string(), total_latency);
            metrics.insert("avg_partition_latency_ms".to_string(), avg_latency);
            metrics.insert("num_partitions".to_string(), partition_results.len() as f32);
            
            Some(metrics)
        } else {
            None
        }
    }

    /// Cancel an active inference request
    pub async fn cancel_request(&self, request_id: &str) -> bool {
        self.active_requests.write().await.remove(request_id).is_some()
    }

    // Private helper methods
    fn initial_embedding(&self, input: &InferenceInput) -> Vec<f32> {
        // Simulate embedding layer
        match input {
            InferenceInput::Text(text) => {
                vec![0.5; text.len() * 768] // 768-dim embeddings
            }
            InferenceInput::Tokens(tokens) => {
                vec![0.5; tokens.len() * 768]
            }
            _ => vec![0.5; 768],
        }
    }

    async fn execute_partition(
        &self,
        node_id: &str,
        partition: &super::model_partitioning::ModelPartition,
        hidden_states: &[f32],
    ) -> Vec<f32> {
        // Simulate network transfer and computation
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        // Transform hidden states (simplified)
        hidden_states.to_vec()
    }

    fn all_reduce(&self, partial_results: Vec<Vec<f32>>) -> Vec<f32> {
        // Combine partial results from tensor parallel execution
        if partial_results.is_empty() {
            return vec![];
        }

        let len = partial_results[0].len();
        let num_results = partial_results.len() as f32;
        let mut combined = vec![0.0; len];

        for result in &partial_results {
            for (i, val) in result.iter().enumerate() {
                combined[i] += val;
            }
        }

        // Average
        combined.iter().map(|v| v / num_results).collect()
    }

    fn decode_output(&self, _hidden_states: Vec<f32>, params: &InferenceParameters) -> String {
        // Simulate decoding hidden states to text
        let num_tokens = (params.max_tokens.min(50)) as usize;
        format!("Generated response with {} tokens (simulated)", num_tokens)
    }

    async fn execute_hybrid_inference(
        &self,
        request: &InferenceRequest,
        model: &super::model_partitioning::PartitionedModel,
    ) -> String {
        // Simplified hybrid execution
        format!(
            "Hybrid inference on {} using {} partitions",
            request.model_name,
            model.partitions.len()
        )
    }

    async fn execute_single_inference(
        &self,
        input: &InferenceInput,
        params: &InferenceParameters,
    ) -> String {
        // Simulate single-node inference
        tokio::time::sleep(Duration::from_millis(100)).await;
        format!("Single-node inference result ({} max tokens)", params.max_tokens)
    }

    fn calculate_tokens_per_second(&self, output: &str, latency_ms: f32) -> f32 {
        if latency_ms == 0.0 {
            return 0.0;
        }
        
        // Rough estimate based on output length
        let estimated_tokens = output.split_whitespace().count() as f32;
        (estimated_tokens / latency_ms) * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_node_inference() {
        let protocol = Arc::new(ComputeNodeProtocol::new());
        let partitioner = Arc::new(ModelPartitioner::new());
        let inference = DistributedInference::new(protocol.clone(), partitioner);

        // Register a compute node
        let caps = super::super::compute_node::NodeCapabilities {
            cpu_cores: 8,
            gpu_available: true,
            gpu_memory_gb: 16.0,
            ram_gb: 32.0,
            storage_gb: 512.0,
            network_bandwidth_mbps: 1000,
            supported_models: vec!["llama".to_string()],
            hardware_acceleration: vec![super::super::compute_node::HardwareAccel::CUDA],
        };
        protocol.register_local_node(caps).await;

        let request = InferenceRequest {
            request_id: "test-1".to_string(),
            model_name: "llama-7b".to_string(),
            input: InferenceInput::Text("Hello, world!".to_string()),
            parameters: InferenceParameters::default(),
        };

        let response = inference.infer(request).await;
        
        match response.output {
            InferenceOutput::Text(text) => {
                assert!(!text.is_empty());
            }
            _ => panic!("Expected text output"),
        }
        
        assert!(response.metrics.latency_ms > 0.0);
        assert_eq!(response.metrics.nodes_used, 1);
    }

    #[tokio::test]
    async fn test_distributed_inference() {
        let protocol = Arc::new(ComputeNodeProtocol::new());
        let partitioner = Arc::new(ModelPartitioner::new());
        let inference = DistributedInference::new(protocol.clone(), partitioner.clone());

        // Create partitioned model
        protocol.discover_nodes().await;
        let partitions = partitioner
            .partition_model("llama-70b", 140000.0, 4, PartitionStrategy::LayerWise)
            .await;

        // Assign to nodes
        let mut assignments = std::collections::HashMap::new();
        for (i, partition) in partitions.iter().enumerate() {
            let node_id = format!("edge-{}", (i % 2) + 1);
            assignments.insert(partition.partition_id.clone(), node_id);
        }
        partitioner
            .assign_partitions_to_nodes("llama-70b", assignments)
            .await;

        let request = InferenceRequest {
            request_id: "test-2".to_string(),
            model_name: "llama-70b".to_string(),
            input: InferenceInput::Text("Distributed inference test".to_string()),
            parameters: InferenceParameters::default(),
        };

        let response = inference.infer(request).await;
        
        assert!(response.metrics.nodes_used > 1);
        assert!(response.metrics.coordination_overhead_ms > 0.0);
    }

    #[tokio::test]
    async fn test_inference_parameters() {
        let protocol = Arc::new(ComputeNodeProtocol::new());
        let partitioner = Arc::new(ModelPartitioner::new());
        let inference = DistributedInference::new(protocol, partitioner);

        let params = InferenceParameters {
            max_tokens: 100,
            temperature: 0.8,
            top_p: 0.95,
            top_k: 50,
            streaming: true,
        };

        let request = InferenceRequest {
            request_id: "test-3".to_string(),
            model_name: "phi-2".to_string(),
            input: InferenceInput::Text("Test".to_string()),
            parameters: params,
        };

        let response = inference.infer(request).await;
        assert!(!response.request_id.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_request() {
        let protocol = Arc::new(ComputeNodeProtocol::new());
        let partitioner = Arc::new(ModelPartitioner::new());
        let inference = DistributedInference::new(protocol, partitioner);

        let request = InferenceRequest {
            request_id: "cancel-test".to_string(),
            model_name: "test-model".to_string(),
            input: InferenceInput::Text("Test".to_string()),
            parameters: InferenceParameters::default(),
        };

        // Register request
        inference
            .active_requests
            .write()
            .await
            .insert(request.request_id.clone(), request);

        // Cancel it
        let cancelled = inference.cancel_request("cancel-test").await;
        assert!(cancelled);

        // Verify removed
        let exists = inference
            .active_requests
            .read()
            .await
            .contains_key("cancel-test");
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_multimodal_input() {
        let protocol = Arc::new(ComputeNodeProtocol::new());
        let partitioner = Arc::new(ModelPartitioner::new());
        let inference = DistributedInference::new(protocol, partitioner);

        let request = InferenceRequest {
            request_id: "multimodal-test".to_string(),
            model_name: "llava".to_string(),
            input: InferenceInput::Multimodal {
                text: Some("Describe this image".to_string()),
                image: Some(vec![0u8; 1024]),
                audio: None,
            },
            parameters: InferenceParameters::default(),
        };

        let response = inference.infer(request).await;
        assert!(!response.request_id.is_empty());
    }
}
