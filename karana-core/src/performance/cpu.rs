// CPU Management for Kāraṇa OS
// Controls CPU frequency, cores, and processing allocation

use std::collections::HashMap;

/// CPU governor types for frequency scaling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GovernorType {
    /// Maximum frequency always
    Performance,
    /// Minimum frequency always
    PowerSave,
    /// Dynamic scaling based on load
    OnDemand,
    /// Conservative scaling, slower ramp up
    Conservative,
    /// User-defined frequency
    UserSpace,
    /// Schedutil - scheduler-driven
    SchedUtil,
}

/// CPU core state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoreState {
    Active,
    Idle,
    DeepSleep,
    Offline,
}

/// Individual CPU core information
#[derive(Debug, Clone)]
pub struct CpuCore {
    pub core_id: u32,
    pub state: CoreState,
    pub frequency_mhz: u32,
    pub min_frequency_mhz: u32,
    pub max_frequency_mhz: u32,
    pub usage_percent: f32,
    pub temperature_celsius: f32,
}

impl CpuCore {
    pub fn new(core_id: u32) -> Self {
        Self {
            core_id,
            state: CoreState::Active,
            frequency_mhz: 1800,
            min_frequency_mhz: 600,
            max_frequency_mhz: 2400,
            usage_percent: 0.0,
            temperature_celsius: 45.0,
        }
    }

    pub fn set_frequency(&mut self, freq_mhz: u32) {
        self.frequency_mhz = freq_mhz.clamp(self.min_frequency_mhz, self.max_frequency_mhz);
    }

    pub fn set_state(&mut self, state: CoreState) {
        self.state = state;
        if state == CoreState::DeepSleep || state == CoreState::Offline {
            self.frequency_mhz = 0;
        }
    }
}

/// CPU cluster for big.LITTLE architectures
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpuCluster {
    /// High performance cores
    Big,
    /// Efficiency cores
    Little,
    /// Medium cores (some architectures)
    Medium,
}

/// Task affinity for CPU scheduling
#[derive(Debug, Clone)]
pub struct TaskAffinity {
    pub task_id: String,
    pub preferred_cluster: CpuCluster,
    pub allowed_cores: Vec<u32>,
    pub priority: i32,
}

/// CPU governor managing frequency and core states
pub struct CpuGovernor {
    governor: GovernorType,
    cores: Vec<CpuCore>,
    affinities: HashMap<String, TaskAffinity>,
    total_usage: f32,
    boost_enabled: bool,
    boost_timeout_ms: u64,
}

impl CpuGovernor {
    pub fn new() -> Self {
        // Assume 4-core CPU typical for smart glasses
        let cores = (0..4).map(CpuCore::new).collect();
        Self {
            governor: GovernorType::OnDemand,
            cores,
            affinities: HashMap::new(),
            total_usage: 0.0,
            boost_enabled: true,
            boost_timeout_ms: 500,
        }
    }

    pub fn with_cores(core_count: u32) -> Self {
        let cores = (0..core_count).map(CpuCore::new).collect();
        Self {
            governor: GovernorType::OnDemand,
            cores,
            affinities: HashMap::new(),
            total_usage: 0.0,
            boost_enabled: true,
            boost_timeout_ms: 500,
        }
    }

    pub fn set_governor(&mut self, governor: GovernorType) {
        self.governor = governor;
        self.apply_governor_policy();
    }

    pub fn get_governor(&self) -> GovernorType {
        self.governor
    }

    fn apply_governor_policy(&mut self) {
        for core in &mut self.cores {
            if core.state != CoreState::Offline {
                match self.governor {
                    GovernorType::Performance => {
                        core.set_frequency(core.max_frequency_mhz);
                    }
                    GovernorType::PowerSave => {
                        core.set_frequency(core.min_frequency_mhz);
                    }
                    GovernorType::OnDemand | GovernorType::SchedUtil => {
                        // Scale based on usage
                        let target = core.min_frequency_mhz + 
                            ((core.max_frequency_mhz - core.min_frequency_mhz) as f32 
                            * (core.usage_percent / 100.0)) as u32;
                        core.set_frequency(target);
                    }
                    GovernorType::Conservative => {
                        // Slower ramp up
                        let current = core.frequency_mhz;
                        let target = if core.usage_percent > 80.0 {
                            current + 100
                        } else if core.usage_percent < 20.0 {
                            current.saturating_sub(50)
                        } else {
                            current
                        };
                        core.set_frequency(target);
                    }
                    GovernorType::UserSpace => {
                        // Don't auto-adjust
                    }
                }
            }
        }
    }

    pub fn update_usage(&mut self, core_id: u32, usage_percent: f32) {
        if let Some(core) = self.cores.iter_mut().find(|c| c.core_id == core_id) {
            core.usage_percent = usage_percent.clamp(0.0, 100.0);
        }
        self.calculate_total_usage();
        self.apply_governor_policy();
    }

    fn calculate_total_usage(&mut self) {
        let active_cores: Vec<_> = self.cores.iter()
            .filter(|c| c.state == CoreState::Active)
            .collect();
        if active_cores.is_empty() {
            self.total_usage = 0.0;
        } else {
            let sum: f32 = active_cores.iter().map(|c| c.usage_percent).sum();
            self.total_usage = sum / active_cores.len() as f32;
        }
    }

    pub fn get_total_usage(&self) -> f32 {
        self.total_usage
    }

    pub fn set_core_state(&mut self, core_id: u32, state: CoreState) {
        if let Some(core) = self.cores.iter_mut().find(|c| c.core_id == core_id) {
            core.set_state(state);
        }
    }

    pub fn get_core(&self, core_id: u32) -> Option<&CpuCore> {
        self.cores.iter().find(|c| c.core_id == core_id)
    }

    pub fn get_cores(&self) -> &[CpuCore] {
        &self.cores
    }

    pub fn offline_cores_for_power_save(&mut self, keep_active: u32) {
        for (i, core) in self.cores.iter_mut().enumerate() {
            if i as u32 >= keep_active {
                core.set_state(CoreState::Offline);
            }
        }
    }

    pub fn bring_all_cores_online(&mut self) {
        for core in &mut self.cores {
            if core.state == CoreState::Offline {
                core.set_state(CoreState::Active);
            }
        }
    }

    pub fn set_task_affinity(&mut self, affinity: TaskAffinity) {
        self.affinities.insert(affinity.task_id.clone(), affinity);
    }

    pub fn get_task_affinity(&self, task_id: &str) -> Option<&TaskAffinity> {
        self.affinities.get(task_id)
    }

    pub fn enable_boost(&mut self, enabled: bool) {
        self.boost_enabled = enabled;
    }

    pub fn is_boost_enabled(&self) -> bool {
        self.boost_enabled
    }

    pub fn trigger_boost(&mut self) {
        if self.boost_enabled {
            for core in &mut self.cores {
                if core.state == CoreState::Active {
                    core.set_frequency(core.max_frequency_mhz);
                }
            }
        }
    }

    pub fn get_power_estimate_mw(&self) -> f32 {
        self.cores.iter().map(|core| {
            if core.state != CoreState::Active {
                return 10.0; // Minimal power in sleep
            }
            // Rough power model: P = k * f^2 * V^2 (simplified)
            let freq_ratio = core.frequency_mhz as f32 / core.max_frequency_mhz as f32;
            50.0 + (200.0 * freq_ratio * freq_ratio * (core.usage_percent / 100.0))
        }).sum()
    }
}

impl Default for CpuGovernor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_governor_creation() {
        let governor = CpuGovernor::new();
        assert_eq!(governor.get_governor(), GovernorType::OnDemand);
        assert_eq!(governor.get_cores().len(), 4);
    }

    #[test]
    fn test_set_governor() {
        let mut governor = CpuGovernor::new();
        governor.set_governor(GovernorType::Performance);
        assert_eq!(governor.get_governor(), GovernorType::Performance);
    }

    #[test]
    fn test_core_frequency_clamping() {
        let mut core = CpuCore::new(0);
        core.set_frequency(3000); // Above max
        assert_eq!(core.frequency_mhz, core.max_frequency_mhz);
        
        core.set_frequency(100); // Below min
        assert_eq!(core.frequency_mhz, core.min_frequency_mhz);
    }

    #[test]
    fn test_update_usage() {
        let mut governor = CpuGovernor::new();
        governor.update_usage(0, 50.0);
        assert_eq!(governor.get_core(0).unwrap().usage_percent, 50.0);
    }

    #[test]
    fn test_offline_cores() {
        let mut governor = CpuGovernor::new();
        governor.offline_cores_for_power_save(2);
        
        assert_eq!(governor.get_core(0).unwrap().state, CoreState::Active);
        assert_eq!(governor.get_core(1).unwrap().state, CoreState::Active);
        assert_eq!(governor.get_core(2).unwrap().state, CoreState::Offline);
        assert_eq!(governor.get_core(3).unwrap().state, CoreState::Offline);
    }

    #[test]
    fn test_bring_cores_online() {
        let mut governor = CpuGovernor::new();
        governor.offline_cores_for_power_save(1);
        governor.bring_all_cores_online();
        
        for core in governor.get_cores() {
            assert_ne!(core.state, CoreState::Offline);
        }
    }

    #[test]
    fn test_task_affinity() {
        let mut governor = CpuGovernor::new();
        let affinity = TaskAffinity {
            task_id: "ar_render".to_string(),
            preferred_cluster: CpuCluster::Big,
            allowed_cores: vec![0, 1],
            priority: 10,
        };
        governor.set_task_affinity(affinity);
        
        let retrieved = governor.get_task_affinity("ar_render");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().priority, 10);
    }

    #[test]
    fn test_boost() {
        let mut governor = CpuGovernor::new();
        governor.set_governor(GovernorType::PowerSave);
        assert!(governor.is_boost_enabled());
        
        governor.trigger_boost();
        // All active cores should be at max frequency
        for core in governor.get_cores() {
            if core.state == CoreState::Active {
                assert_eq!(core.frequency_mhz, core.max_frequency_mhz);
            }
        }
    }

    #[test]
    fn test_power_estimate() {
        let governor = CpuGovernor::new();
        let power = governor.get_power_estimate_mw();
        assert!(power > 0.0);
    }

    #[test]
    fn test_total_usage_calculation() {
        let mut governor = CpuGovernor::new();
        governor.update_usage(0, 100.0);
        governor.update_usage(1, 100.0);
        governor.update_usage(2, 0.0);
        governor.update_usage(3, 0.0);
        
        assert_eq!(governor.get_total_usage(), 50.0);
    }
}
