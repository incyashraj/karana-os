//! Virtual Glasses Device
//!
//! The main virtual device that combines all simulated components.

use super::{SimulatorConfig, SimulatorStats};
use super::display::{VirtualDisplay, DisplayLayer};
use super::sensors::VirtualSensors;
use super::input::{VirtualCamera, VirtualMicrophone};
use std::time::{Duration, Instant};

/// Power state of the virtual device
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerState {
    Off,
    Booting,
    Running,
    Sleeping,
    LowPower,
    Charging,
}

/// Thermal state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalState {
    Cool,      // < 35°C
    Warm,      // 35-45°C
    Hot,       // 45-55°C
    Throttling, // > 55°C
    Critical,  // > 70°C - shutdown imminent
}

/// Battery state
#[derive(Debug, Clone)]
pub struct BatteryState {
    pub capacity_mah: u32,
    pub current_mah: f32,
    pub voltage: f32,
    pub is_charging: bool,
    pub temperature_c: f32,
}

impl BatteryState {
    pub fn new(capacity: u32) -> Self {
        Self {
            capacity_mah: capacity,
            current_mah: capacity as f32,
            voltage: 4.2,
            is_charging: false,
            temperature_c: 25.0,
        }
    }

    pub fn percentage(&self) -> f32 {
        if self.capacity_mah == 0 {
            return 100.0; // Tethered device
        }
        (self.current_mah / self.capacity_mah as f32 * 100.0).clamp(0.0, 100.0)
    }

    pub fn drain(&mut self, mah: f32) {
        self.current_mah = (self.current_mah - mah).max(0.0);
        // Voltage drops as battery drains
        self.voltage = 3.3 + (self.percentage() / 100.0) * 0.9;
    }

    pub fn charge(&mut self, mah: f32) {
        self.current_mah = (self.current_mah + mah).min(self.capacity_mah as f32);
    }
}

/// The complete virtual glasses device
pub struct VirtualGlasses {
    pub config: SimulatorConfig,
    pub display: VirtualDisplay,
    pub sensors: VirtualSensors,
    pub camera: VirtualCamera,
    pub microphone: VirtualMicrophone,
    pub power_state: PowerState,
    pub thermal_state: ThermalState,
    pub battery: BatteryState,
    pub temperature_c: f32,
    boot_time: Option<Instant>,
    last_update: Instant,
}

impl VirtualGlasses {
    pub fn new(config: SimulatorConfig) -> Self {
        Self {
            display: VirtualDisplay::new(config.display_width, config.display_height),
            sensors: VirtualSensors::new(),
            camera: VirtualCamera::new(1280, 720),
            microphone: VirtualMicrophone::new(),
            power_state: PowerState::Off,
            thermal_state: ThermalState::Cool,
            battery: BatteryState::new(config.battery_capacity),
            temperature_c: 25.0,
            boot_time: None,
            last_update: Instant::now(),
            config,
        }
    }

    /// Boot the virtual device
    pub fn boot(&mut self) -> Result<(), String> {
        match self.power_state {
            PowerState::Off | PowerState::Sleeping => {
                self.power_state = PowerState::Booting;
                self.boot_time = Some(Instant::now());
                
                // Simulate boot sequence
                self.display.show_boot_screen();
                
                // After "boot", go to running
                self.power_state = PowerState::Running;
                Ok(())
            }
            PowerState::Running => Err("Device already running".to_string()),
            PowerState::Booting => Err("Device is booting".to_string()),
            _ => {
                self.power_state = PowerState::Running;
                Ok(())
            }
        }
    }

    /// Shutdown the device
    pub fn shutdown(&mut self) {
        self.power_state = PowerState::Off;
        self.display.clear();
        self.boot_time = None;
    }

    /// Put device to sleep
    pub fn sleep(&mut self) {
        if self.power_state == PowerState::Running {
            self.power_state = PowerState::Sleeping;
            self.display.dim(0.0);
        }
    }

    /// Wake from sleep
    pub fn wake(&mut self) {
        if self.power_state == PowerState::Sleeping {
            self.power_state = PowerState::Running;
            self.display.dim(1.0);
        }
    }

    /// Simulate a frame/tick of the device
    pub fn tick(&mut self, delta: Duration) {
        if self.power_state != PowerState::Running {
            return;
        }

        let delta_secs = delta.as_secs_f32();

        // Simulate battery drain
        if self.config.battery_capacity > 0 {
            // Base drain + display + sensors + AI
            let base_drain = 0.5 * delta_secs;  // ~0.5 mAh/s base
            let display_drain = if self.display.brightness > 0.5 { 0.3 } else { 0.1 } * delta_secs;
            let sensor_drain = if self.sensors.is_active() { 0.2 } else { 0.0 } * delta_secs;
            let camera_drain = if self.camera.is_capturing { 0.5 } else { 0.0 } * delta_secs;
            
            self.battery.drain(base_drain + display_drain + sensor_drain + camera_drain);

            // Check low power
            if self.battery.percentage() < 10.0 {
                self.power_state = PowerState::LowPower;
            }
        }

        // Simulate thermal behavior
        if self.config.simulate_thermal {
            // Heat up from usage
            let heat_generation = 0.1 * delta_secs;
            let camera_heat = if self.camera.is_capturing { 0.2 } else { 0.0 } * delta_secs;
            let ai_heat = 0.15 * delta_secs; // AI inference generates heat
            
            // Passive cooling
            let cooling = (self.temperature_c - 25.0) * 0.05 * delta_secs;
            
            self.temperature_c += heat_generation + camera_heat + ai_heat - cooling;
            self.temperature_c = self.temperature_c.clamp(20.0, 85.0);

            // Update thermal state
            self.thermal_state = match self.temperature_c {
                t if t < 35.0 => ThermalState::Cool,
                t if t < 45.0 => ThermalState::Warm,
                t if t < 55.0 => ThermalState::Hot,
                t if t < 70.0 => ThermalState::Throttling,
                _ => ThermalState::Critical,
            };
        }

        // Update sensors
        self.sensors.tick(delta);

        self.last_update = Instant::now();
    }

    /// Get device uptime
    pub fn uptime(&self) -> Duration {
        self.boot_time.map(|t| t.elapsed()).unwrap_or(Duration::ZERO)
    }

    /// Check if device is usable
    pub fn is_operational(&self) -> bool {
        matches!(self.power_state, PowerState::Running | PowerState::LowPower)
            && self.thermal_state != ThermalState::Critical
    }

    /// Get a status summary
    pub fn status_summary(&self) -> DeviceStatus {
        DeviceStatus {
            power_state: self.power_state,
            thermal_state: self.thermal_state,
            battery_percent: self.battery.percentage(),
            temperature_c: self.temperature_c,
            uptime: self.uptime(),
            display_on: self.display.is_on,
            camera_active: self.camera.is_capturing,
            sensors_active: self.sensors.is_active(),
        }
    }
}

/// Summary of device status
#[derive(Debug, Clone)]
pub struct DeviceStatus {
    pub power_state: PowerState,
    pub thermal_state: ThermalState,
    pub battery_percent: f32,
    pub temperature_c: f32,
    pub uptime: Duration,
    pub display_on: bool,
    pub camera_active: bool,
    pub sensors_active: bool,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Power: {:?} | Battery: {:.0}% | Temp: {:.1}°C ({:?}) | Uptime: {:.0}s",
            self.power_state,
            self.battery_percent,
            self.temperature_c,
            self.thermal_state,
            self.uptime.as_secs_f32()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_boot() {
        let config = SimulatorConfig::default();
        let mut device = VirtualGlasses::new(config);
        
        assert_eq!(device.power_state, PowerState::Off);
        device.boot().unwrap();
        assert_eq!(device.power_state, PowerState::Running);
    }

    #[test]
    fn test_battery_drain() {
        let mut battery = BatteryState::new(1000);
        assert_eq!(battery.percentage(), 100.0);
        
        battery.drain(100.0);
        assert_eq!(battery.percentage(), 90.0);
    }

    #[test]
    fn test_thermal_states() {
        let config = SimulatorConfig {
            simulate_thermal: true,
            ..Default::default()
        };
        let mut device = VirtualGlasses::new(config);
        device.boot().unwrap();
        
        // Force temperature
        device.temperature_c = 50.0;
        device.tick(Duration::from_millis(16));
        assert_eq!(device.thermal_state, ThermalState::Hot);
    }
}
