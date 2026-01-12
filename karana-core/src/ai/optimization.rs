// Kāraṇa OS - Inference Optimization
// Phase 5: Performance - Batching, quantization, and acceleration

use anyhow::Result;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Request batching for efficient inference
pub struct InferenceBatcher {
    queue: VecDeque<InferenceRequest>,
    max_batch_size: usize,
    max_wait_time: Duration,
    last_batch: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: usize,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub responses: Vec<String>,
    pub batch_size: usize,
    pub total_time_ms: f32,
    pub avg_time_per_request_ms: f32,
}

impl InferenceBatcher {
    pub fn new(max_batch_size: usize, max_wait_ms: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            max_batch_size,
            max_wait_time: Duration::from_millis(max_wait_ms),
            last_batch: None,
        }
    }

    /// Add request to batch queue
    pub fn add_request(&mut self, prompt: String, max_tokens: usize) {
        self.queue.push_back(InferenceRequest {
            prompt,
            max_tokens,
            timestamp: Instant::now(),
        });
    }

    /// Check if batch should be processed
    pub fn should_process_batch(&self) -> bool {
        // Process if queue is full
        if self.queue.len() >= self.max_batch_size {
            return true;
        }

        // Process if max wait time exceeded
        if let Some(oldest) = self.queue.front() {
            if oldest.timestamp.elapsed() >= self.max_wait_time {
                return true;
            }
        }

        false
    }

    /// Get next batch of requests
    pub fn get_batch(&mut self) -> Vec<InferenceRequest> {
        let batch_size = self.max_batch_size.min(self.queue.len());
        let batch: Vec<_> = self.queue.drain(..batch_size).collect();
        self.last_batch = Some(Instant::now());
        batch
    }

    /// Get queue length
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }
}

/// Quantization settings for model optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationLevel {
    None,       // FP32 - full precision
    FP16,       // Half precision
    INT8,       // 8-bit quantization
    INT4,       // 4-bit quantization (GGUF Q4_0)
}

impl QuantizationLevel {
    /// Get memory savings ratio compared to FP32
    pub fn memory_savings_ratio(&self) -> f32 {
        match self {
            QuantizationLevel::None => 1.0,
            QuantizationLevel::FP16 => 0.5,
            QuantizationLevel::INT8 => 0.25,
            QuantizationLevel::INT4 => 0.125,
        }
    }

    /// Get expected speedup factor
    pub fn speedup_factor(&self) -> f32 {
        match self {
            QuantizationLevel::None => 1.0,
            QuantizationLevel::FP16 => 1.5,
            QuantizationLevel::INT8 => 2.0,
            QuantizationLevel::INT4 => 2.5,
        }
    }

    /// Get quality degradation (0.0 = no loss, 1.0 = severe loss)
    pub fn quality_loss(&self) -> f32 {
        match self {
            QuantizationLevel::None => 0.0,
            QuantizationLevel::FP16 => 0.01,
            QuantizationLevel::INT8 => 0.05,
            QuantizationLevel::INT4 => 0.10,
        }
    }
}

/// Performance profiler for inference operations
pub struct InferenceProfiler {
    operation_times: Vec<(String, Duration)>,
    start_time: Option<Instant>,
    current_operation: Option<String>,
}

impl InferenceProfiler {
    pub fn new() -> Self {
        Self {
            operation_times: Vec::new(),
            start_time: None,
            current_operation: None,
        }
    }

    /// Start profiling an operation
    pub fn start_operation(&mut self, name: impl Into<String>) {
        if let Some(current) = &self.current_operation {
            // End previous operation first
            self.end_operation();
        }
        
        self.current_operation = Some(name.into());
        self.start_time = Some(Instant::now());
    }

    /// End current operation
    pub fn end_operation(&mut self) {
        if let (Some(name), Some(start)) = (self.current_operation.take(), self.start_time.take()) {
            let duration = start.elapsed();
            self.operation_times.push((name, duration));
        }
    }

    /// Get total time spent
    pub fn total_time(&self) -> Duration {
        self.operation_times.iter().map(|(_, d)| *d).sum()
    }

    /// Get breakdown of time by operation
    pub fn breakdown(&self) -> Vec<(String, f32)> {
        self.operation_times
            .iter()
            .map(|(name, duration)| (name.clone(), duration.as_secs_f32() * 1000.0))
            .collect()
    }

    /// Get summary report
    pub fn report(&self) -> String {
        let mut report = String::from("Inference Profile:\n");
        let total_ms = self.total_time().as_secs_f32() * 1000.0;
        
        report.push_str(&format!("  Total: {:.2}ms\n", total_ms));
        report.push_str("  Breakdown:\n");
        
        for (name, ms) in self.breakdown() {
            let percent = (ms / total_ms) * 100.0;
            report.push_str(&format!("    {}: {:.2}ms ({:.1}%)\n", name, ms, percent));
        }
        
        report
    }

    /// Clear all recorded times
    pub fn clear(&mut self) {
        self.operation_times.clear();
        self.start_time = None;
        self.current_operation = None;
    }
}

impl Default for InferenceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// KV cache manager for transformer models
pub struct KVCacheManager {
    cache_size_mb: usize,
    max_context_length: usize,
    cache_hits: usize,
    cache_misses: usize,
}

impl KVCacheManager {
    pub fn new(cache_size_mb: usize, max_context_length: usize) -> Self {
        Self {
            cache_size_mb,
            max_context_length,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Record cache hit
    pub fn record_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record cache miss
    pub fn record_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        (self.cache_hits as f32 / total as f32) * 100.0
    }

    /// Get stats
    pub fn stats(&self) -> String {
        format!(
            "KV Cache: {}MB, max_ctx={}, hits={}, misses={}, hit_rate={:.1}%",
            self.cache_size_mb,
            self.max_context_length,
            self.cache_hits,
            self.cache_misses,
            self.hit_rate()
        )
    }
}

/// Token generation optimization settings
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: usize,
    pub repetition_penalty: f32,
    pub stop_sequences: Vec<String>,
    pub early_stopping: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.8,
            top_p: 0.95,
            top_k: 50,
            repetition_penalty: 1.1,
            stop_sequences: vec!["</s>".to_string(), "<|endoftext|>".to_string()],
            early_stopping: true,
        }
    }
}

impl GenerationConfig {
    /// Create config for fast, low-quality generation
    pub fn fast() -> Self {
        Self {
            max_tokens: 128,
            temperature: 1.0,
            top_p: 0.9,
            top_k: 20,
            early_stopping: true,
            ..Default::default()
        }
    }

    /// Create config for high-quality generation
    pub fn quality() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.95,
            top_k: 50,
            repetition_penalty: 1.2,
            early_stopping: false,
            ..Default::default()
        }
    }

    /// Create config for creative generation
    pub fn creative() -> Self {
        Self {
            max_tokens: 384,
            temperature: 1.2,
            top_p: 0.98,
            top_k: 100,
            repetition_penalty: 1.05,
            early_stopping: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_batcher() {
        let mut batcher = InferenceBatcher::new(3, 100);
        
        batcher.add_request("test1".to_string(), 50);
        batcher.add_request("test2".to_string(), 50);
        assert_eq!(batcher.queue_len(), 2);
        
        batcher.add_request("test3".to_string(), 50);
        assert!(batcher.should_process_batch());
        
        let batch = batcher.get_batch();
        assert_eq!(batch.len(), 3);
        assert_eq!(batcher.queue_len(), 0);
    }

    #[test]
    fn test_quantization_metrics() {
        let int4 = QuantizationLevel::INT4;
        assert_eq!(int4.memory_savings_ratio(), 0.125);
        assert!(int4.speedup_factor() > 1.0);
        assert!(int4.quality_loss() > 0.0);
    }

    #[test]
    fn test_profiler() {
        let mut profiler = InferenceProfiler::new();
        
        profiler.start_operation("tokenization");
        std::thread::sleep(Duration::from_millis(10));
        profiler.end_operation();
        
        profiler.start_operation("inference");
        std::thread::sleep(Duration::from_millis(20));
        profiler.end_operation();
        
        let total = profiler.total_time();
        assert!(total.as_millis() >= 30);
        
        let breakdown = profiler.breakdown();
        assert_eq!(breakdown.len(), 2);
    }

    #[test]
    fn test_kv_cache_stats() {
        let mut cache = KVCacheManager::new(128, 2048);
        
        cache.record_hit();
        cache.record_hit();
        cache.record_miss();
        
        assert_eq!(cache.hit_rate(), 66.66667);
    }

    #[test]
    fn test_generation_configs() {
        let fast = GenerationConfig::fast();
        let quality = GenerationConfig::quality();
        
        assert!(fast.max_tokens < quality.max_tokens);
        assert!(fast.temperature > quality.temperature);
    }
}
