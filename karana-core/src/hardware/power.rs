use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PowerProfile {
    Performance, // Full speed, high polling
    Balanced,    // Standard
    LowPower,    // Reduced polling, dim screen
    Critical,    // Save state, prepare for shutdown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub percentage: f32, // 0.0 - 100.0
    pub is_charging: bool,
    pub time_remaining_secs: Option<u64>,
}

pub struct PowerManager {
    sys: System,
    pub profile: PowerProfile,
    pub battery: BatteryState,
}

impl PowerManager {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );
        sys.refresh_all();

        Self {
            sys,
            profile: PowerProfile::Balanced,
            battery: BatteryState {
                percentage: 100.0, // Default assumption
                is_charging: true,
                time_remaining_secs: None,
            },
        }
    }

    pub fn update(&mut self) -> String {
        // Refresh system stats
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        
        // Real Battery Logic
        // We attempt to find a battery component. If found, we use its values.
        // If not (e.g. Desktop/Container), we fall back to simulation.
        let found_real_battery = false;
        
        // Note: sysinfo components might need specific permissions or hardware support.
        // In a container, this list is often empty.
        // Removed for sysinfo 0.37 compatibility in prototype
        /*
        if let Some(batteries) = self.sys.components().iter().find(|c| c.label().to_lowercase().contains("battery")) {
             // Found a battery!
             // ...
        }
        */

        // Simulation Logic for Prototype (Fallback)
        if !found_real_battery {
            if !self.battery.is_charging {
                let drain = match self.profile {
                    PowerProfile::Performance => 0.5,
                    PowerProfile::Balanced => 0.1,
                    PowerProfile::LowPower => 0.02,
                    PowerProfile::Critical => 0.0,
                };
                self.battery.percentage = (self.battery.percentage - drain).max(0.0);
            }
        }

        // Auto-switch profiles based on battery
        if self.battery.percentage < 10.0 {
            self.profile = PowerProfile::Critical;
        } else if self.battery.percentage < 30.0 {
            self.profile = PowerProfile::LowPower;
        }

        format!("Power: {:?} | Bat: {:.1}% ({})", 
            self.profile, 
            self.battery.percentage, 
            if self.battery.is_charging { "⚡" } else { "🔋" }
        )
    }

    pub fn set_profile(&mut self, profile: PowerProfile) {
        self.profile = profile;
        log::info!("Atom 3 (Power): Switched to {:?} Profile", profile);
    }

    pub fn toggle_charging_sim(&mut self) {
        self.battery.is_charging = !self.battery.is_charging;
    }
    
    pub fn get_status_string(&self) -> String {
         format!("{:?} {:.0}%", self.profile, self.battery.percentage)
    }
}
