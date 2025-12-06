// Kāraṇa OS - Phase 55: Model Distillation & Optimization
// Knowledge distillation and quantization for efficient on-device inference

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Quantization precision levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuantizationLevel {
    /// Float32 - No quantization (baseline)
    FP32,
    
    /// Float16 - 50% size reduction, minimal accuracy loss
    FP16,
    
    /// INT8 - 75% size reduction, ~1-2% accuracy loss
    INT8,
    
    /// INT4 - 87.5% size reduction, ~3-5% accuracy loss
    INT4,
    
    /// Binary/Ternary - 97% size reduction, significant accuracy loss
    BINARY,
}

impl QuantizationLevel {
    /// Get memory reduction factor
    pub fn size_reduction(&self) -> f32 {
        match self {
            Self::FP32 => 1.0,
            Self::FP16 => 0.5,
            Self::INT8 => 0.25,
            Self::INT4 => 0.125,
            Self::BINARY => 0.03125,
        }
    }
    
    /// Get expected accuracy degradation percentage
    pub fn expected_accuracy_loss(&self) -> f32 {
        match self {
            Self::FP32 => 0.0,
            Self::FP16 => 0.1,
            Self::INT8 => 1.5,
            Self::INT4 => 4.0,
            Self::BINARY => 15.0,
        }
    }
    
    /// Get inference speedup factor
    pub fn speedup_factor(&self) -> f32 {
        match self {
            Self::FP32 => 1.0,
            Self::FP16 => 1.5,
            Self::INT8 => 2.5,
            Self::INT4 => 4.0,
            Self::BINARY => 8.0,
        }
    }
}

/// Distillation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationConfig {
    pub teacher_model: String,
    pub student_model: String,
    pub temperature: f32,
    pub alpha: f32,  // Balance between hard and soft targets
    pub epochs: usize,
    pub batch_size: usize,
    pub learning_rate: f32,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            teacher_model: "large_model".to_string(),
            student_model: "distilled_model".to_string(),
            temperature: 2.0,
            alpha: 0.7,
            epochs: 10,
            batch_size: 32,
            learning_rate: 1e-4,
        }
    }
}

/// Model optimization metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedModelInfo {
    pub original_model: String,
    pub optimized_name: String,
    pub quantization_level: QuantizationLevel,
    pub original_size_mb: f32,
    pub optimized_size_mb: f32,
    pub original_latency_ms: f32,
    pub optimized_latency_ms: f32,
    pub accuracy_before: f32,
    pub accuracy_after: f32,
    pub distilled: bool,
    pub calibration_dataset: Option<String>,
    pub created_at: u64,
}

impl OptimizedModelInfo {
    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f32 {
        self.original_size_mb / self.optimized_size_mb
    }
    
    /// Calculate speedup
    pub fn speedup(&self) -> f32 {
        self.original_latency_ms / self.optimized_latency_ms
    }
    
    /// Calculate accuracy retention
    pub fn accuracy_retention(&self) -> f32 {
        (self.accuracy_after / self.accuracy_before) * 100.0
    }
    
    /// Check if optimization meets quality thresholds
    pub fn meets_quality_threshold(&self, min_accuracy_retention: f32) -> bool {
        self.accuracy_retention() >= min_accuracy_retention
    }
}

/// Model distillation and optimization framework
pub struct ModelOptimizer {
    optimized_models: Arc<RwLock<HashMap<String, OptimizedModelInfo>>>,
    cache_dir: PathBuf,
    quality_thresholds: QualityThresholds,
}

/// Quality thresholds for model optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub min_accuracy_retention: f32,  // e.g., 95% = keep at least 95% accuracy
    pub max_latency_ms: f32,
    pub max_memory_mb: f32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_accuracy_retention: 95.0,
            max_latency_ms: 100.0,
            max_memory_mb: 500.0,
        }
    }
}

impl ModelOptimizer {
    /// Create a new model optimizer
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            optimized_models: Arc::new(RwLock::new(HashMap::new())),
            cache_dir,
            quality_thresholds: QualityThresholds::default(),
        }
    }
    
    /// Set quality thresholds
    pub fn with_thresholds(mut self, thresholds: QualityThresholds) -> Self {
        self.quality_thresholds = thresholds;
        self
    }
    
    /// Optimize a model with knowledge distillation
    pub async fn distill_model(
        &self,
        config: DistillationConfig,
    ) -> Result<OptimizedModelInfo> {
        // In real implementation, this would:
        // 1. Load teacher model
        // 2. Initialize student model (smaller architecture)
        // 3. Run knowledge distillation training
        // 4. Evaluate on validation set
        // 5. Save optimized model
        
        // Simulated distillation results
        let original_size_mb = 500.0;
        let optimized_size_mb = 150.0;
        let original_latency_ms = 250.0;
        let optimized_latency_ms = 80.0;
        
        let info = OptimizedModelInfo {
            original_model: config.teacher_model.clone(),
            optimized_name: config.student_model.clone(),
            quantization_level: QuantizationLevel::FP32,
            original_size_mb,
            optimized_size_mb,
            original_latency_ms,
            optimized_latency_ms,
            accuracy_before: 92.5,
            accuracy_after: 90.2,
            distilled: true,
            calibration_dataset: Some("validation_set".to_string()),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Check quality thresholds
        if !info.meets_quality_threshold(self.quality_thresholds.min_accuracy_retention) {
            return Err(anyhow!(
                "Distilled model accuracy ({:.1}%) below threshold ({:.1}%)",
                info.accuracy_retention(),
                self.quality_thresholds.min_accuracy_retention
            ));
        }
        
        self.optimized_models
            .write()
            .await
            .insert(config.student_model.clone(), info.clone());
        
        Ok(info)
    }
    
    /// Quantize a model to lower precision
    pub async fn quantize_model(
        &self,
        model_name: &str,
        level: QuantizationLevel,
        calibration_data: Option<Vec<Vec<f32>>>,
    ) -> Result<OptimizedModelInfo> {
        // In real implementation, this would:
        // 1. Load model weights
        // 2. Calibrate quantization ranges (if INT8/INT4)
        // 3. Convert weights to lower precision
        // 4. Evaluate accuracy on validation set
        // 5. Save quantized model
        
        // Simulated quantization results
        let original_size_mb = 500.0;
        let optimized_size_mb = original_size_mb * level.size_reduction();
        let original_latency_ms = 250.0;
        let optimized_latency_ms = original_latency_ms / level.speedup_factor();
        
        let info = OptimizedModelInfo {
            original_model: model_name.to_string(),
            optimized_name: format!("{}_q{:?}", model_name, level),
            quantization_level: level,
            original_size_mb,
            optimized_size_mb,
            original_latency_ms,
            optimized_latency_ms,
            accuracy_before: 92.5,
            accuracy_after: 92.5 - level.expected_accuracy_loss(),
            distilled: false,
            calibration_dataset: calibration_data.map(|_| "calibration_set".to_string()),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Check quality thresholds
        if !info.meets_quality_threshold(self.quality_thresholds.min_accuracy_retention) {
            return Err(anyhow!(
                "Quantized model accuracy ({:.1}%) below threshold ({:.1}%)",
                info.accuracy_retention(),
                self.quality_thresholds.min_accuracy_retention
            ));
        }
        
        if info.optimized_size_mb > self.quality_thresholds.max_memory_mb {
            return Err(anyhow!(
                "Quantized model size ({:.1}MB) exceeds threshold ({:.1}MB)",
                info.optimized_size_mb,
                self.quality_thresholds.max_memory_mb
            ));
        }
        
        self.optimized_models
            .write()
            .await
            .insert(info.optimized_name.clone(), info.clone());
        
        Ok(info)
    }
    
    /// Combine distillation and quantization for maximum optimization
    pub async fn optimize_model(
        &self,
        model_name: &str,
        target_size_mb: f32,
        min_accuracy: f32,
    ) -> Result<OptimizedModelInfo> {
        // Strategy: Try quantization levels from least to most aggressive
        // until we meet size target while maintaining accuracy
        
        let quantization_levels = [
            QuantizationLevel::FP16,
            QuantizationLevel::INT8,
            QuantizationLevel::INT4,
        ];
        
        for level in quantization_levels {
            let result = self.quantize_model(model_name, level, None).await?;
            
            if result.optimized_size_mb <= target_size_mb 
                && result.accuracy_retention() >= min_accuracy {
                return Ok(result);
            }
        }
        
        // If quantization alone isn't enough, try distillation first
        let distill_config = DistillationConfig {
            teacher_model: model_name.to_string(),
            student_model: format!("{}_distilled", model_name),
            ..Default::default()
        };
        
        let distilled = self.distill_model(distill_config).await?;
        
        // Then quantize the distilled model
        for level in quantization_levels {
            let result = self.quantize_model(&distilled.optimized_name, level, None).await?;
            
            if result.optimized_size_mb <= target_size_mb 
                && result.accuracy_retention() >= min_accuracy {
                return Ok(result);
            }
        }
        
        Err(anyhow!(
            "Could not optimize model to {:.1}MB while maintaining {:.1}% accuracy",
            target_size_mb,
            min_accuracy
        ))
    }
    
    /// Get optimized model info
    pub async fn get_model_info(&self, model_name: &str) -> Option<OptimizedModelInfo> {
        self.optimized_models.read().await.get(model_name).cloned()
    }
    
    /// List all optimized models
    pub async fn list_models(&self) -> Vec<OptimizedModelInfo> {
        self.optimized_models.read().await.values().cloned().collect()
    }
    
    /// Get optimization statistics
    pub async fn stats(&self) -> OptimizationStats {
        let models = self.optimized_models.read().await;
        
        let total_models = models.len();
        let total_original_size: f32 = models.values().map(|m| m.original_size_mb).sum();
        let total_optimized_size: f32 = models.values().map(|m| m.optimized_size_mb).sum();
        let avg_compression = if total_original_size > 0.0 {
            total_original_size / total_optimized_size
        } else {
            1.0
        };
        let avg_speedup: f32 = models.values()
            .map(|m| m.speedup())
            .sum::<f32>() / total_models as f32;
        let avg_accuracy_retention: f32 = models.values()
            .map(|m| m.accuracy_retention())
            .sum::<f32>() / total_models as f32;
        
        OptimizationStats {
            total_models,
            total_original_size_mb: total_original_size,
            total_optimized_size_mb: total_optimized_size,
            avg_compression_ratio: avg_compression,
            avg_speedup,
            avg_accuracy_retention,
        }
    }
}

/// Optimization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub total_models: usize,
    pub total_original_size_mb: f32,
    pub total_optimized_size_mb: f32,
    pub avg_compression_ratio: f32,
    pub avg_speedup: f32,
    pub avg_accuracy_retention: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_quantization_levels() {
        assert_eq!(QuantizationLevel::INT8.size_reduction(), 0.25);
        assert_eq!(QuantizationLevel::INT4.size_reduction(), 0.125);
        assert!(QuantizationLevel::INT8.speedup_factor() > 1.0);
    }
    
    #[tokio::test]
    async fn test_model_quantization() {
        let temp_dir = tempdir().unwrap();
        let optimizer = ModelOptimizer::new(temp_dir.path().to_path_buf());
        
        let result = optimizer.quantize_model(
            "test_model",
            QuantizationLevel::INT8,
            None,
        ).await;
        
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.quantization_level, QuantizationLevel::INT8);
        assert!(info.optimized_size_mb < info.original_size_mb);
        assert!(info.optimized_latency_ms < info.original_latency_ms);
    }
    
    #[tokio::test]
    async fn test_model_distillation() {
        let temp_dir = tempdir().unwrap();
        let optimizer = ModelOptimizer::new(temp_dir.path().to_path_buf());
        
        let config = DistillationConfig::default();
        let result = optimizer.distill_model(config).await;
        
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.distilled);
        assert!(info.compression_ratio() > 1.0);
        assert!(info.speedup() > 1.0);
    }
    
    #[tokio::test]
    async fn test_quality_thresholds() {
        let temp_dir = tempdir().unwrap();
        let optimizer = ModelOptimizer::new(temp_dir.path().to_path_buf())
            .with_thresholds(QualityThresholds {
                min_accuracy_retention: 99.0,  // Very strict
                max_latency_ms: 100.0,
                max_memory_mb: 500.0,
            });
        
        // INT4 quantization should fail quality threshold
        let result = optimizer.quantize_model(
            "test_model",
            QuantizationLevel::INT4,
            None,
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_optimization_stats() {
        let temp_dir = tempdir().unwrap();
        let optimizer = ModelOptimizer::new(temp_dir.path().to_path_buf());
        
        // Optimize a few models
        let _ = optimizer.quantize_model("model1", QuantizationLevel::INT8, None).await;
        let _ = optimizer.quantize_model("model2", QuantizationLevel::FP16, None).await;
        
        let stats = optimizer.stats().await;
        assert_eq!(stats.total_models, 2);
        assert!(stats.avg_compression_ratio > 1.0);
        assert!(stats.avg_speedup > 1.0);
    }
    
    #[tokio::test]
    async fn test_combined_optimization() {
        let temp_dir = tempdir().unwrap();
        let optimizer = ModelOptimizer::new(temp_dir.path().to_path_buf());
        
        let result = optimizer.optimize_model(
            "large_model",
            100.0,  // Target: 100MB
            90.0,   // Min accuracy: 90%
        ).await;
        
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.optimized_size_mb <= 100.0);
        assert!(info.accuracy_retention() >= 90.0);
    }
}
