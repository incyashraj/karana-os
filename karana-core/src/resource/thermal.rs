// Kāraṇa OS - Phase 55: Thermal Governor
// Predictive thermal management and proactive throttling

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thermal state of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalState {
    /// Temperature normal, full performance
    Normal,
    
    /// Temperature elevated, start monitoring closely
    Elevated,
    
    /// Temperature high, begin throttling
    Hot,
    
    /// Temperature critical, aggressive throttling
    Critical,
    
    /// Emergency shutdown imminent
    Emergency,
}

impl ThermalState {
    /// Get state from temperature
    pub fn from_temperature(temp_c: f32) -> Self {
        match temp_c {
            t if t < 30.0 => Self::Normal,
            t if t < 35.0 => Self::Elevated,
            t if t < 40.0 => Self::Hot,
            t if t < 45.0 => Self::Critical,
            _ => Self::Emergency,
        }
    }
    
    /// Get maximum allowed compute intensity (0.0 to 1.0)
    pub fn max_compute_intensity(&self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Elevated => 0.9,
            Self::Hot => 0.7,
            Self::Critical => 0.4,
            Self::Emergency => 0.1,
        }
    }
    
    /// Get cooldown time required (seconds)
    pub fn cooldown_time_s(&self) -> u64 {
        match self {
            Self::Normal => 0,
            Self::Elevated => 10,
            Self::Hot => 30,
            Self::Critical => 60,
            Self::Emergency => 120,
        }
    }
}

/// Thermal prediction model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalPrediction {
    pub current_temp_c: f32,
    pub predicted_temp_c: f32,
    pub time_horizon_s: u64,
    pub confidence: f32,
    pub factors: Vec<String>,
}

/// Thermal history sample
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThermalSample {
    timestamp: u64,
    temperature_c: f32,
    compute_load: f32,
    ambient_temp_c: f32,
}

/// Thermal governor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalConfig {
    /// Temperature thresholds (°C)
    pub normal_temp: f32,
    pub elevated_temp: f32,
    pub hot_temp: f32,
    pub critical_temp: f32,
    pub emergency_temp: f32,
    
    /// Prediction window (seconds)
    pub prediction_window_s: u64,
    
    /// History window for modeling (samples)
    pub history_window: usize,
    
    /// Proactive throttling enabled
    pub proactive_throttling: bool,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            normal_temp: 30.0,
            elevated_temp: 35.0,
            hot_temp: 40.0,
            critical_temp: 45.0,
            emergency_temp: 50.0,
            prediction_window_s: 30,
            history_window: 100,
            proactive_throttling: true,
        }
    }
}

/// Thermal throttling action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThrottlingAction {
    /// No action needed
    None,
    
    /// Pause specified models
    PauseModels(Vec<String>),
    
    /// Reduce update frequency
    ReduceFrequency { from_hz: f32, to_hz: f32 },
    
    /// Offload to companion device
    OffloadToCompanion(Vec<String>),
    
    /// Shutdown non-critical services
    ShutdownServices(Vec<String>),
    
    /// Emergency system suspend
    EmergencyShutdown,
}

/// Thermal governor
pub struct ThermalGovernor {
    config: ThermalConfig,
    history: Arc<RwLock<VecDeque<ThermalSample>>>,
    current_state: Arc<RwLock<ThermalState>>,
    throttled_components: Arc<RwLock<Vec<String>>>,
    stats: Arc<RwLock<ThermalStats>>,
}

/// Thermal statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThermalStats {
    pub total_throttle_events: usize,
    pub time_in_normal_s: u64,
    pub time_in_elevated_s: u64,
    pub time_in_hot_s: u64,
    pub time_in_critical_s: u64,
    pub max_temp_reached_c: f32,
    pub avg_temp_c: f32,
    pub predictions_made: usize,
    pub predictions_accurate: usize,
}

impl ThermalGovernor {
    /// Create new thermal governor
    pub fn new(config: ThermalConfig) -> Self {
        Self {
            config,
            history: Arc::new(RwLock::new(VecDeque::new())),
            current_state: Arc::new(RwLock::new(ThermalState::Normal)),
            throttled_components: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(ThermalStats::default())),
        }
    }
    
    /// Update thermal reading
    pub async fn update(
        &self,
        temperature_c: f32,
        compute_load: f32,
        ambient_temp_c: f32,
    ) -> Result<ThrottlingAction> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Record sample
        let sample = ThermalSample {
            timestamp: now,
            temperature_c,
            compute_load,
            ambient_temp_c,
        };
        
        let mut history = self.history.write().await;
        history.push_back(sample);
        
        // Maintain history window
        while history.len() > self.config.history_window {
            history.pop_front();
        }
        drop(history);
        
        // Update state
        let new_state = ThermalState::from_temperature(temperature_c);
        let mut state = self.current_state.write().await;
        *state = new_state;
        drop(state);
        
        // Update statistics
        let mut stats = self.stats.write().await;
        if temperature_c > stats.max_temp_reached_c {
            stats.max_temp_reached_c = temperature_c;
        }
        stats.avg_temp_c = (stats.avg_temp_c * stats.total_throttle_events as f32 + temperature_c) 
            / (stats.total_throttle_events as f32 + 1.0);
        drop(stats);
        
        // Decide on throttling action
        self.decide_throttling_action(new_state, temperature_c, compute_load).await
    }
    
    /// Predict future temperature
    pub async fn predict_temperature(
        &self,
        time_horizon_s: u64,
        planned_load: f32,
    ) -> Result<ThermalPrediction> {
        let history = self.history.read().await;
        
        if history.len() < 2 {
            // Not enough data for prediction
            let current_temp = history.back().map(|s| s.temperature_c).unwrap_or(25.0);
            return Ok(ThermalPrediction {
                current_temp_c: current_temp,
                predicted_temp_c: current_temp,
                time_horizon_s,
                confidence: 0.5,
                factors: vec!["Insufficient history for prediction".to_string()],
            });
        }
        
        // Simple linear regression model
        let current_sample = history.back().unwrap();
        let current_temp = current_sample.temperature_c;
        
        // Calculate temperature rate of change
        let mut temp_rate = 0.0;
        if history.len() >= 5 {
            let recent: Vec<&ThermalSample> = history.iter().rev().take(5).collect();
            let time_diff = recent[0].timestamp - recent[4].timestamp;
            let temp_diff = recent[0].temperature_c - recent[4].temperature_c;
            temp_rate = temp_diff / time_diff as f32;
        }
        
        // Factor in planned load vs current load
        let load_factor = planned_load / current_sample.compute_load.max(0.1);
        
        // Predict temperature
        let base_prediction = current_temp + (temp_rate * time_horizon_s as f32);
        let load_adjusted = base_prediction * load_factor;
        
        // Account for ambient temperature and cooling
        let ambient = current_sample.ambient_temp_c;
        let cooling_factor = 0.7; // Simplified passive cooling model
        let predicted_temp = ambient + (load_adjusted - ambient) * cooling_factor;
        
        // Calculate confidence based on history consistency
        let confidence = (history.len() as f32 / self.config.history_window as f32)
            .min(1.0) * 0.8;
        
        let mut factors = Vec::new();
        if temp_rate > 0.5 {
            factors.push("Temperature rising rapidly".to_string());
        }
        if load_factor > 1.2 {
            factors.push("Planned load increase".to_string());
        }
        if ambient > 25.0 {
            factors.push("High ambient temperature".to_string());
        }
        
        let mut stats = self.stats.write().await;
        stats.predictions_made += 1;
        drop(stats);
        
        Ok(ThermalPrediction {
            current_temp_c: current_temp,
            predicted_temp_c: predicted_temp,
            time_horizon_s,
            confidence,
            factors,
        })
    }
    
    /// Decide on throttling action based on thermal state
    async fn decide_throttling_action(
        &self,
        state: ThermalState,
        temp_c: f32,
        load: f32,
    ) -> Result<ThrottlingAction> {
        // Check if we need proactive throttling
        if self.config.proactive_throttling {
            let prediction = self.predict_temperature(
                self.config.prediction_window_s,
                load,
            ).await?;
            
            if prediction.predicted_temp_c > self.config.critical_temp {
                // Proactively throttle to prevent critical state
                let mut stats = self.stats.write().await;
                stats.total_throttle_events += 1;
                drop(stats);
                
                return Ok(ThrottlingAction::PauseModels(vec![
                    "vision_model".to_string(),
                    "scene_understanding".to_string(),
                ]));
            }
        }
        
        match state {
            ThermalState::Normal => Ok(ThrottlingAction::None),
            
            ThermalState::Elevated => {
                // Start monitoring, reduce non-essential updates
                Ok(ThrottlingAction::ReduceFrequency {
                    from_hz: 10.0,
                    to_hz: 5.0,
                })
            }
            
            ThermalState::Hot => {
                // Pause heavy models
                let mut stats = self.stats.write().await;
                stats.total_throttle_events += 1;
                drop(stats);
                
                Ok(ThrottlingAction::PauseModels(vec![
                    "scene_understanding".to_string(),
                    "object_detection".to_string(),
                ]))
            }
            
            ThermalState::Critical => {
                // Offload to companion device
                let mut stats = self.stats.write().await;
                stats.total_throttle_events += 1;
                drop(stats);
                
                Ok(ThrottlingAction::OffloadToCompanion(vec![
                    "ai_inference".to_string(),
                    "vision_processing".to_string(),
                ]))
            }
            
            ThermalState::Emergency => {
                // Emergency shutdown
                let mut stats = self.stats.write().await;
                stats.total_throttle_events += 1;
                drop(stats);
                
                Ok(ThrottlingAction::EmergencyShutdown)
            }
        }
    }
    
    /// Get current thermal state
    pub async fn state(&self) -> ThermalState {
        *self.current_state.read().await
    }
    
    /// Get statistics
    pub async fn stats(&self) -> ThermalStats {
        self.stats.read().await.clone()
    }
    
    /// Check if component is currently throttled
    pub async fn is_throttled(&self, component: &str) -> bool {
        self.throttled_components
            .read()
            .await
            .contains(&component.to_string())
    }
    
    /// Manually throttle a component
    pub async fn throttle_component(&self, component: String) {
        self.throttled_components.write().await.push(component);
    }
    
    /// Unthrottle a component
    pub async fn unthrottle_component(&self, component: &str) {
        self.throttled_components
            .write()
            .await
            .retain(|c| c != component);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thermal_state_thresholds() {
        assert_eq!(ThermalState::from_temperature(25.0), ThermalState::Normal);
        assert_eq!(ThermalState::from_temperature(33.0), ThermalState::Elevated);
        assert_eq!(ThermalState::from_temperature(38.0), ThermalState::Hot);
        assert_eq!(ThermalState::from_temperature(43.0), ThermalState::Critical);
        assert_eq!(ThermalState::from_temperature(52.0), ThermalState::Emergency);
    }
    
    #[test]
    fn test_thermal_state_max_intensity() {
        assert_eq!(ThermalState::Normal.max_compute_intensity(), 1.0);
        assert!(ThermalState::Hot.max_compute_intensity() < 1.0);
        assert!(ThermalState::Critical.max_compute_intensity() < 0.5);
    }
    
    #[tokio::test]
    async fn test_thermal_governor_normal() {
        let governor = ThermalGovernor::new(ThermalConfig::default());
        
        let action = governor.update(28.0, 0.5, 22.0).await.unwrap();
        
        assert!(matches!(action, ThrottlingAction::None));
        assert_eq!(governor.state().await, ThermalState::Normal);
    }
    
    #[tokio::test]
    async fn test_thermal_governor_hot() {
        let governor = ThermalGovernor::new(ThermalConfig::default());
        
        let action = governor.update(38.0, 0.8, 24.0).await.unwrap();
        
        assert!(matches!(action, ThrottlingAction::PauseModels(_)));
        assert_eq!(governor.state().await, ThermalState::Hot);
    }
    
    #[tokio::test]
    async fn test_temperature_prediction() {
        let governor = ThermalGovernor::new(ThermalConfig::default());
        
        // Build history
        for i in 0..10 {
            let temp = 25.0 + i as f32 * 0.5;
            let _ = governor.update(temp, 0.6, 22.0).await;
        }
        
        let prediction = governor.predict_temperature(30, 0.8).await.unwrap();
        
        assert!(prediction.predicted_temp_c > prediction.current_temp_c);
        assert!(prediction.confidence > 0.0);
    }
    
    #[tokio::test]
    async fn test_proactive_throttling() {
        let config = ThermalConfig {
            proactive_throttling: true,
            prediction_window_s: 30,
            critical_temp: 40.0,
            ..Default::default()
        };
        
        let governor = ThermalGovernor::new(config);
        
        // Simulate rapid heating
        for i in 0..5 {
            let temp = 35.0 + i as f32 * 1.0;
            let _ = governor.update(temp, 0.9, 25.0).await;
        }
        
        let stats = governor.stats().await;
        assert!(stats.total_throttle_events > 0);
    }
    
    #[tokio::test]
    async fn test_component_throttling() {
        let governor = ThermalGovernor::new(ThermalConfig::default());
        
        governor.throttle_component("test_component".to_string()).await;
        assert!(governor.is_throttled("test_component").await);
        
        governor.unthrottle_component("test_component").await;
        assert!(!governor.is_throttled("test_component").await);
    }
}
