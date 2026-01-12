// Kāraṇa OS - Adaptive Model Loader
// Phase 5: Performance Optimization - Battery-aware model selection

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Model size variants available for different resource constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSize {
    Tiny,    // ~100MB, fastest inference, lowest quality
    Small,   // ~500MB, balanced performance
    Medium,  // ~2GB, best quality, slower
}

/// System resource status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    pub battery_percent: f32,
    pub is_charging: bool,
    pub available_memory_mb: u64,
    pub cpu_usage_percent: f32,
    pub temperature_celsius: f32,
}

impl ResourceStatus {
    /// Get current system resource status
    pub fn current() -> Result<Self> {
        // Use sysinfo crate for cross-platform monitoring
        use sysinfo::System;
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        // Battery status (mock on desktop, real on mobile)
        let battery_percent = Self::get_battery_level()?;
        let is_charging = Self::is_charging()?;
        
        // Memory
        let available_memory_mb = sys.available_memory() / 1024 / 1024;
        
        // CPU usage (use simple average - sysinfo 0.30 API)
        let cpu_usage_percent = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len().max(1) as f32;
        
        // Temperature (if available)
        let temperature_celsius = Self::get_cpu_temperature().unwrap_or(50.0);
        
        Ok(Self {
            battery_percent,
            is_charging,
            available_memory_mb,
            cpu_usage_percent,
            temperature_celsius,
        })
    }

    fn get_battery_level() -> Result<f32> {
        // Platform-specific battery reading
        #[cfg(target_os = "linux")]
        {
            // Read from /sys/class/power_supply/BAT0/capacity
            if let Ok(capacity) = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
                if let Ok(percent) = capacity.trim().parse::<f32>() {
                    return Ok(percent);
                }
            }
        }
        
        // Fallback: assume 100% (desktop or unsupported platform)
        Ok(100.0)
    }

    fn is_charging() -> Result<bool> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/sys/class/power_supply/BAT0/status") {
                return Ok(status.trim() == "Charging" || status.trim() == "Full");
            }
        }
        
        Ok(false)
    }

    fn get_cpu_temperature() -> Option<f32> {
        #[cfg(target_os = "linux")]
        {
            // Try common thermal zones
            for i in 0..10 {
                let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
                if let Ok(temp_str) = std::fs::read_to_string(&path) {
                    if let Ok(temp_millis) = temp_str.trim().parse::<f32>() {
                        return Some(temp_millis / 1000.0);
                    }
                }
            }
        }
        
        None
    }
}

/// Adaptive model selection policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptivePolicy {
    /// Battery threshold below which to use Tiny model (%)
    pub tiny_threshold: f32,
    
    /// Battery threshold below which to use Small model (%)
    pub small_threshold: f32,
    
    /// Memory threshold to avoid Medium model (MB)
    pub memory_threshold: u64,
    
    /// CPU usage threshold to avoid heavy models (%)
    pub cpu_threshold: f32,
    
    /// Temperature threshold to throttle (°C)
    pub temp_threshold: f32,
    
    /// Force specific model (overrides adaptive selection)
    pub force_model: Option<ModelSize>,
}

impl Default for AdaptivePolicy {
    fn default() -> Self {
        Self {
            tiny_threshold: 20.0,      // Use Tiny below 20% battery
            small_threshold: 50.0,     // Use Small below 50% battery
            memory_threshold: 1024,    // Avoid Medium if <1GB available
            cpu_threshold: 80.0,       // Throttle if CPU >80%
            temp_threshold: 70.0,      // Throttle if temp >70°C
            force_model: None,
        }
    }
}

/// Adaptive model loader with performance tracking
pub struct AdaptiveModelLoader {
    policy: AdaptivePolicy,
    current_model: Option<ModelSize>,
    inference_times: Vec<f32>,  // Rolling window of inference times
    last_resource_check: Option<Instant>,
    check_interval_secs: u64,
}

impl AdaptiveModelLoader {
    pub fn new(policy: AdaptivePolicy) -> Self {
        Self {
            policy,
            current_model: None,
            inference_times: Vec::with_capacity(100),
            last_resource_check: None,
            check_interval_secs: 30,  // Check resources every 30s
        }
    }

    /// Select optimal model size based on current resources
    pub fn select_model(&mut self) -> Result<ModelSize> {
        // Check if we need to re-evaluate (throttle checks)
        let should_check = match self.last_resource_check {
            None => true,
            Some(last) => last.elapsed().as_secs() >= self.check_interval_secs,
        };

        if !should_check && self.current_model.is_some() {
            return Ok(self.current_model.unwrap());
        }

        // Override if forced
        if let Some(forced) = self.policy.force_model {
            log::info!("[AdaptiveLoader] Using forced model: {:?}", forced);
            self.current_model = Some(forced);
            return Ok(forced);
        }

        // Get current resource status
        let resources = ResourceStatus::current()?;
        self.last_resource_check = Some(Instant::now());

        let selected = self.select_based_on_resources(&resources);
        
        if self.current_model != Some(selected) {
            log::info!(
                "[AdaptiveLoader] Model changed: {:?} → {:?} (battery: {:.0}%, mem: {}MB, cpu: {:.0}%)",
                self.current_model, selected,
                resources.battery_percent,
                resources.available_memory_mb,
                resources.cpu_usage_percent
            );
            self.current_model = Some(selected);
        }

        Ok(selected)
    }

    fn select_based_on_resources(&self, resources: &ResourceStatus) -> ModelSize {
        // Priority 1: Critical battery
        if !resources.is_charging && resources.battery_percent < self.policy.tiny_threshold {
            log::debug!("[AdaptiveLoader] Low battery → Tiny model");
            return ModelSize::Tiny;
        }

        // Priority 2: Temperature throttling
        if resources.temperature_celsius > self.policy.temp_threshold {
            log::debug!("[AdaptiveLoader] High temperature → Small model");
            return ModelSize::Small;
        }

        // Priority 3: CPU overload
        if resources.cpu_usage_percent > self.policy.cpu_threshold {
            log::debug!("[AdaptiveLoader] High CPU → Small model");
            return ModelSize::Small;
        }

        // Priority 4: Memory constraints
        if resources.available_memory_mb < self.policy.memory_threshold {
            log::debug!("[AdaptiveLoader] Low memory → Small model");
            return ModelSize::Small;
        }

        // Priority 5: Battery level
        if !resources.is_charging && resources.battery_percent < self.policy.small_threshold {
            log::debug!("[AdaptiveLoader] Medium battery → Small model");
            return ModelSize::Small;
        }

        // Default: Use best quality if resources available
        if resources.is_charging || resources.battery_percent > 70.0 {
            log::debug!("[AdaptiveLoader] Good resources → Medium model");
            ModelSize::Medium
        } else {
            log::debug!("[AdaptiveLoader] Moderate resources → Small model");
            ModelSize::Small
        }
    }

    /// Record inference time for performance tracking
    pub fn record_inference_time(&mut self, duration_ms: f32) {
        self.inference_times.push(duration_ms);
        
        // Keep rolling window of last 100 inferences
        if self.inference_times.len() > 100 {
            self.inference_times.remove(0);
        }
    }

    /// Get average inference time
    pub fn avg_inference_time_ms(&self) -> f32 {
        if self.inference_times.is_empty() {
            return 0.0;
        }
        
        self.inference_times.iter().sum::<f32>() / self.inference_times.len() as f32
    }

    /// Get p95 inference time
    pub fn p95_inference_time_ms(&self) -> f32 {
        if self.inference_times.is_empty() {
            return 0.0;
        }
        
        let mut sorted = self.inference_times.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let idx = (sorted.len() as f32 * 0.95) as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    /// Get performance statistics
    pub fn stats(&self) -> PerformanceStats {
        PerformanceStats {
            current_model: self.current_model,
            avg_inference_ms: self.avg_inference_time_ms(),
            p95_inference_ms: self.p95_inference_time_ms(),
            total_inferences: self.inference_times.len(),
        }
    }

    /// Force a specific model size
    pub fn force_model(&mut self, size: Option<ModelSize>) {
        self.policy.force_model = size;
        self.current_model = None;  // Trigger re-evaluation
        log::info!("[AdaptiveLoader] Forced model set to: {:?}", size);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub current_model: Option<ModelSize>,
    pub avg_inference_ms: f32,
    pub p95_inference_ms: f32,
    pub total_inferences: usize,
}

/// Model info with paths and metadata
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub size: ModelSize,
    pub model_path: String,
    pub tokenizer_path: String,
    pub estimated_memory_mb: u64,
    pub avg_inference_ms: f32,
}

impl ModelInfo {
    pub fn for_size(size: ModelSize) -> Self {
        match size {
            ModelSize::Tiny => Self {
                size,
                model_path: "models/tinyllama-1.1b-q4_0.gguf".to_string(),
                tokenizer_path: "models/tokenizer.json".to_string(),
                estimated_memory_mb: 100,
                avg_inference_ms: 50.0,
            },
            ModelSize::Small => Self {
                size,
                model_path: "models/phi-3-mini-4k-q4_0.gguf".to_string(),
                tokenizer_path: "models/tokenizer.json".to_string(),
                estimated_memory_mb: 500,
                avg_inference_ms: 200.0,
            },
            ModelSize::Medium => Self {
                size,
                model_path: "models/llama-3-8b-q4_0.gguf".to_string(),
                tokenizer_path: "models/tokenizer.json".to_string(),
                estimated_memory_mb: 2048,
                avg_inference_ms: 800.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_status() {
        let status = ResourceStatus::current();
        assert!(status.is_ok());
        
        let status = status.unwrap();
        assert!(status.battery_percent >= 0.0 && status.battery_percent <= 100.0);
    }

    #[test]
    fn test_model_selection() {
        let mut loader = AdaptiveModelLoader::new(AdaptivePolicy::default());
        
        // Should select a model
        let model = loader.select_model();
        assert!(model.is_ok());
    }

    #[test]
    fn test_forced_model() {
        let mut loader = AdaptiveModelLoader::new(AdaptivePolicy::default());
        
        loader.force_model(Some(ModelSize::Tiny));
        let model = loader.select_model().unwrap();
        assert_eq!(model, ModelSize::Tiny);
    }

    #[test]
    fn test_inference_tracking() {
        let mut loader = AdaptiveModelLoader::new(AdaptivePolicy::default());
        
        loader.record_inference_time(100.0);
        loader.record_inference_time(150.0);
        loader.record_inference_time(120.0);
        
        let avg = loader.avg_inference_time_ms();
        assert!((avg - 123.33).abs() < 1.0);
        
        let stats = loader.stats();
        assert_eq!(stats.total_inferences, 3);
    }
}
