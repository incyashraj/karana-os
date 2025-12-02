//! # Kāraṇa Smart Glasses Simulator
//! 
//! A comprehensive virtual device simulator for testing and developing
//! Kāraṇa OS without physical hardware. This enables rapid iteration
//! on design, UX, and functionality.
//!
//! ## Architecture
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    VIRTUAL GLASSES DEVICE                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
//! │  │   Display   │  │   Camera    │  │        Sensors          │  │
//! │  │  1920x1080  │  │  720p feed  │  │ IMU, GPS, Light, Touch  │  │
//! │  │  AR Layers  │  │  or images  │  │    Gesture Recognition  │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
//! │  │ Microphone  │  │   Speaker   │  │     Battery/Thermal     │  │
//! │  │ Text/Audio  │  │    TTS      │  │     Power Simulation    │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                     SCENARIO ENGINE                             │
//! │  [Walking] [Meeting] [Cooking] [Navigation] [Custom]            │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

pub mod device;
pub mod display;
pub mod sensors;
pub mod input;
pub mod scenarios;
pub mod tui;

pub use device::VirtualGlasses;
pub use display::{VirtualDisplay, DisplayLayer, ARElement};
pub use sensors::{VirtualSensors, SensorReading, GestureType};
pub use input::{VirtualCamera, VirtualMicrophone, InputEvent};
pub use scenarios::{Scenario, ScenarioEngine, ScenarioEvent};
pub use tui::SimulatorTUI;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Main simulator configuration
#[derive(Debug, Clone)]
pub struct SimulatorConfig {
    /// Display resolution
    pub display_width: u32,
    pub display_height: u32,
    /// Field of view in degrees
    pub fov_horizontal: f32,
    pub fov_vertical: f32,
    /// Simulated battery capacity (mAh)
    pub battery_capacity: u32,
    /// Enable realistic latency simulation
    pub simulate_latency: bool,
    /// Frame rate target
    pub target_fps: u32,
    /// Enable thermal throttling simulation
    pub simulate_thermal: bool,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            display_width: 1920,
            display_height: 1080,
            fov_horizontal: 52.0,  // Typical AR glasses FOV
            fov_vertical: 30.0,
            battery_capacity: 1500,
            simulate_latency: true,
            target_fps: 60,
            simulate_thermal: true,
        }
    }
}

/// Preset device profiles for different hardware targets
#[derive(Debug, Clone, Copy)]
pub enum DeviceProfile {
    /// XREAL Air style glasses
    XrealAir,
    /// Rokid Max style glasses
    RokidMax,
    /// Meta Ray-Ban style (camera-focused)
    MetaRayBan,
    /// Enterprise AR (HoloLens-like)
    EnterpriseAR,
    /// Custom configuration
    Custom,
}

impl DeviceProfile {
    pub fn config(&self) -> SimulatorConfig {
        match self {
            DeviceProfile::XrealAir => SimulatorConfig {
                display_width: 1920,
                display_height: 1080,
                fov_horizontal: 46.0,
                fov_vertical: 25.0,
                battery_capacity: 0, // Display only, powered by cable
                simulate_latency: true,
                target_fps: 120,
                simulate_thermal: false,
            },
            DeviceProfile::RokidMax => SimulatorConfig {
                display_width: 1920,
                display_height: 1080,
                fov_horizontal: 50.0,
                fov_vertical: 28.0,
                battery_capacity: 0,
                simulate_latency: true,
                target_fps: 120,
                simulate_thermal: false,
            },
            DeviceProfile::MetaRayBan => SimulatorConfig {
                display_width: 640,  // Smaller display
                display_height: 480,
                fov_horizontal: 30.0,
                fov_vertical: 20.0,
                battery_capacity: 500,
                simulate_latency: true,
                target_fps: 30,
                simulate_thermal: true,
            },
            DeviceProfile::EnterpriseAR => SimulatorConfig {
                display_width: 2048,
                display_height: 2048,
                fov_horizontal: 70.0,
                fov_vertical: 40.0,
                battery_capacity: 3000,
                simulate_latency: true,
                target_fps: 60,
                simulate_thermal: true,
            },
            DeviceProfile::Custom => SimulatorConfig::default(),
        }
    }
}

/// Statistics from the simulator
#[derive(Debug, Clone, Default)]
pub struct SimulatorStats {
    pub frames_rendered: u64,
    pub average_fps: f32,
    pub ai_inferences: u64,
    pub voice_commands: u64,
    pub gestures_detected: u64,
    pub battery_used_mah: f32,
    pub peak_temperature_c: f32,
    pub uptime_seconds: f64,
}

/// The main simulator runner
pub struct Simulator {
    pub config: SimulatorConfig,
    pub device: Arc<Mutex<VirtualGlasses>>,
    pub scenario_engine: Arc<Mutex<ScenarioEngine>>,
    pub stats: Arc<Mutex<SimulatorStats>>,
    start_time: Instant,
}

impl Simulator {
    pub fn new(profile: DeviceProfile) -> Self {
        let config = profile.config();
        Self {
            device: Arc::new(Mutex::new(VirtualGlasses::new(config.clone()))),
            scenario_engine: Arc::new(Mutex::new(ScenarioEngine::new())),
            stats: Arc::new(Mutex::new(SimulatorStats::default())),
            config,
            start_time: Instant::now(),
        }
    }

    pub fn with_config(config: SimulatorConfig) -> Self {
        Self {
            device: Arc::new(Mutex::new(VirtualGlasses::new(config.clone()))),
            scenario_engine: Arc::new(Mutex::new(ScenarioEngine::new())),
            stats: Arc::new(Mutex::new(SimulatorStats::default())),
            config,
            start_time: Instant::now(),
        }
    }

    /// Load a scenario for testing
    pub fn load_scenario(&self, scenario: Scenario) {
        let mut engine = self.scenario_engine.lock().unwrap();
        engine.load(scenario);
    }

    /// Get current uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Update stats
    pub fn update_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.uptime_seconds = self.start_time.elapsed().as_secs_f64();
        if stats.uptime_seconds > 0.0 {
            stats.average_fps = stats.frames_rendered as f32 / stats.uptime_seconds as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_profiles() {
        let xreal = DeviceProfile::XrealAir.config();
        assert_eq!(xreal.display_width, 1920);
        assert_eq!(xreal.target_fps, 120);

        let meta = DeviceProfile::MetaRayBan.config();
        assert_eq!(meta.battery_capacity, 500);
    }

    #[test]
    fn test_simulator_creation() {
        let sim = Simulator::new(DeviceProfile::XrealAir);
        assert_eq!(sim.config.display_width, 1920);
    }
}
