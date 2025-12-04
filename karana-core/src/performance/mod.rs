// Performance & Resource Optimization Module for Kāraṇa OS
// Comprehensive resource management for AR smart glasses

pub mod cpu;
pub mod memory;
pub mod framerate;
pub mod scheduler;
pub mod quality;

pub use cpu::*;
pub use memory::*;
pub use framerate::*;
pub use scheduler::*;
pub use quality::*;

use std::collections::HashMap;

/// Performance profile for overall system optimization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceProfile {
    /// Maximum performance, power consumption not limited
    Performance,
    /// Balanced performance and efficiency
    Balanced,
    /// Power saving mode
    PowerSaver,
    /// Thermal throttling active
    ThermalThrottle,
    /// Custom profile
    Custom,
}

/// System performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub cpu_usage_percent: f32,
    pub gpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub memory_available_mb: f32,
    pub frame_time_ms: f32,
    pub target_frame_time_ms: f32,
    pub frame_drops_last_second: u32,
    pub thermal_state: ThermalState,
    pub battery_drain_mw: f32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            gpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            memory_available_mb: 512.0,
            frame_time_ms: 16.67,
            target_frame_time_ms: 16.67,
            frame_drops_last_second: 0,
            thermal_state: ThermalState::Normal,
            battery_drain_mw: 500.0,
        }
    }
}

/// Thermal state affecting performance
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalState {
    Cool,
    Normal,
    Warm,
    Hot,
    Critical,
}

/// Performance budget for a component
#[derive(Debug, Clone)]
pub struct PerformanceBudget {
    pub component_id: String,
    pub cpu_budget_percent: f32,
    pub memory_budget_mb: f32,
    pub gpu_budget_percent: f32,
    pub priority: u8,
}

/// Performance manager coordinating all optimization
pub struct PerformanceManager {
    profile: PerformanceProfile,
    metrics: PerformanceMetrics,
    budgets: HashMap<String, PerformanceBudget>,
    cpu_governor: CpuGovernor,
    memory_manager: MemoryManager,
    frame_controller: FrameRateController,
    task_scheduler: TaskScheduler,
    quality_manager: QualityManager,
}

impl PerformanceManager {
    pub fn new() -> Self {
        Self {
            profile: PerformanceProfile::Balanced,
            metrics: PerformanceMetrics::default(),
            budgets: HashMap::new(),
            cpu_governor: CpuGovernor::new(),
            memory_manager: MemoryManager::new(),
            frame_controller: FrameRateController::new(),
            task_scheduler: TaskScheduler::new(),
            quality_manager: QualityManager::new(),
        }
    }

    pub fn set_profile(&mut self, profile: PerformanceProfile) {
        self.profile = profile;
        self.apply_profile_settings();
    }

    pub fn get_profile(&self) -> PerformanceProfile {
        self.profile
    }

    fn apply_profile_settings(&mut self) {
        match self.profile {
            PerformanceProfile::Performance => {
                self.cpu_governor.set_governor(GovernorType::Performance);
                self.frame_controller.set_target_fps(90);
                self.quality_manager.set_level(QualityLevel::Ultra);
            }
            PerformanceProfile::Balanced => {
                self.cpu_governor.set_governor(GovernorType::OnDemand);
                self.frame_controller.set_target_fps(60);
                self.quality_manager.set_level(QualityLevel::High);
            }
            PerformanceProfile::PowerSaver => {
                self.cpu_governor.set_governor(GovernorType::PowerSave);
                self.frame_controller.set_target_fps(30);
                self.quality_manager.set_level(QualityLevel::Medium);
            }
            PerformanceProfile::ThermalThrottle => {
                self.cpu_governor.set_governor(GovernorType::PowerSave);
                self.frame_controller.set_target_fps(30);
                self.quality_manager.set_level(QualityLevel::Low);
            }
            PerformanceProfile::Custom => {}
        }
    }

    pub fn update_metrics(&mut self, metrics: PerformanceMetrics) {
        self.metrics = metrics;
        self.check_thermal_throttle();
        self.optimize_based_on_metrics();
    }

    fn check_thermal_throttle(&mut self) {
        if self.metrics.thermal_state == ThermalState::Hot 
            || self.metrics.thermal_state == ThermalState::Critical {
            if self.profile != PerformanceProfile::ThermalThrottle {
                self.set_profile(PerformanceProfile::ThermalThrottle);
            }
        }
    }

    fn optimize_based_on_metrics(&mut self) {
        // Frame rate adaptation
        if self.metrics.frame_drops_last_second > 5 {
            self.frame_controller.decrease_target_fps();
        }

        // Memory pressure handling
        let memory_usage_ratio = self.metrics.memory_usage_mb / 
            (self.metrics.memory_usage_mb + self.metrics.memory_available_mb);
        if memory_usage_ratio > 0.85 {
            self.memory_manager.trigger_gc();
        }
    }

    pub fn allocate_budget(&mut self, budget: PerformanceBudget) {
        self.budgets.insert(budget.component_id.clone(), budget);
    }

    pub fn get_budget(&self, component_id: &str) -> Option<&PerformanceBudget> {
        self.budgets.get(component_id)
    }

    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    pub fn cpu_governor(&mut self) -> &mut CpuGovernor {
        &mut self.cpu_governor
    }

    pub fn memory_manager(&mut self) -> &mut MemoryManager {
        &mut self.memory_manager
    }

    pub fn frame_controller(&mut self) -> &mut FrameRateController {
        &mut self.frame_controller
    }

    pub fn task_scheduler(&mut self) -> &mut TaskScheduler {
        &mut self.task_scheduler
    }

    pub fn quality_manager(&mut self) -> &mut QualityManager {
        &mut self.quality_manager
    }
}

impl Default for PerformanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_manager_creation() {
        let manager = PerformanceManager::new();
        assert_eq!(manager.get_profile(), PerformanceProfile::Balanced);
    }

    #[test]
    fn test_profile_change() {
        let mut manager = PerformanceManager::new();
        manager.set_profile(PerformanceProfile::Performance);
        assert_eq!(manager.get_profile(), PerformanceProfile::Performance);
    }

    #[test]
    fn test_thermal_throttle() {
        let mut manager = PerformanceManager::new();
        let mut metrics = PerformanceMetrics::default();
        metrics.thermal_state = ThermalState::Hot;
        manager.update_metrics(metrics);
        assert_eq!(manager.get_profile(), PerformanceProfile::ThermalThrottle);
    }

    #[test]
    fn test_budget_allocation() {
        let mut manager = PerformanceManager::new();
        let budget = PerformanceBudget {
            component_id: "ar_renderer".to_string(),
            cpu_budget_percent: 30.0,
            memory_budget_mb: 128.0,
            gpu_budget_percent: 50.0,
            priority: 1,
        };
        manager.allocate_budget(budget);
        let retrieved = manager.get_budget("ar_renderer");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().cpu_budget_percent, 30.0);
    }

    #[test]
    fn test_default_metrics() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.cpu_usage_percent, 0.0);
        assert_eq!(metrics.frame_time_ms, 16.67);
    }
}
