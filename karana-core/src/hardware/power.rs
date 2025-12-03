use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PowerProfile {
    Performance, // Full speed, high polling (60fps, all sensors)
    Balanced,    // Standard (30fps, selective sensors)
    LowPower,    // Reduced polling, dim screen (15fps, minimal)
    Critical,    // Save state, prepare for shutdown (5fps)
    Doze,        // Oracle sleep mode - wake on gaze/voice only
}

/// Power optimization stats for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerStats {
    /// Total mWh saved from optimizations
    pub energy_saved_mwh: f32,
    /// Number of ZK proofs batched (saves ~30% per batch)
    pub zk_proofs_batched: u64,
    /// Doze cycles entered
    pub doze_cycles: u64,
    /// Time spent in doze mode (seconds)
    pub doze_time_secs: u64,
    /// Gaze wake events (low-power attention detection)
    pub gaze_wakes: u64,
    /// Voice wake events
    pub voice_wakes: u64,
    /// Frames dropped to save power
    pub frames_dropped: u64,
}

/// eBPF-like gaze hook for low-power attention detection
/// Instead of full eye tracking, we use peripheral sensor polling
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GazeState {
    /// User actively looking at glasses display
    Active,
    /// User looking away but nearby
    Peripheral,
    /// No gaze detected - trigger doze
    Away,
    /// Eyes closed - deep doze
    Closed,
}

/// Configuration for power-aware sensor polling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorPollingConfig {
    /// Camera FPS based on power profile
    pub camera_fps: u8,
    /// IMU polling rate (Hz)
    pub imu_rate_hz: u16,
    /// Microphone sample rate (Hz)
    pub mic_sample_rate: u32,
    /// Eye tracking rate (Hz) - most power hungry
    pub eye_tracking_hz: u8,
    /// Enable always-on-display
    pub aod_enabled: bool,
}

impl SensorPollingConfig {
    /// Get optimal polling config for power profile
    pub fn for_profile(profile: PowerProfile) -> Self {
        match profile {
            PowerProfile::Performance => Self {
                camera_fps: 60,
                imu_rate_hz: 200,
                mic_sample_rate: 48000,
                eye_tracking_hz: 90,
                aod_enabled: true,
            },
            PowerProfile::Balanced => Self {
                camera_fps: 30,
                imu_rate_hz: 100,
                mic_sample_rate: 16000,
                eye_tracking_hz: 30,
                aod_enabled: true,
            },
            PowerProfile::LowPower => Self {
                camera_fps: 15,
                imu_rate_hz: 50,
                mic_sample_rate: 8000,
                eye_tracking_hz: 10,
                aod_enabled: false,
            },
            PowerProfile::Critical => Self {
                camera_fps: 5,
                imu_rate_hz: 20,
                mic_sample_rate: 8000,
                eye_tracking_hz: 0, // Disabled
                aod_enabled: false,
            },
            PowerProfile::Doze => Self {
                camera_fps: 0,       // Camera off
                imu_rate_hz: 10,     // Minimal for wake detection
                mic_sample_rate: 8000, // Low for wake word
                eye_tracking_hz: 1,  // Ultra-low for gaze wake
                aod_enabled: false,
            },
        }
    }
    
    /// Estimate power consumption in mW
    pub fn estimated_power_mw(&self) -> f32 {
        let camera = self.camera_fps as f32 * 3.0; // ~180mW at 60fps
        let imu = self.imu_rate_hz as f32 * 0.1;   // ~20mW at 200Hz
        let mic = (self.mic_sample_rate as f32 / 1000.0) * 0.5; // ~24mW at 48kHz
        let eye = self.eye_tracking_hz as f32 * 2.5; // ~225mW at 90Hz
        let aod = if self.aod_enabled { 50.0 } else { 0.0 };
        
        camera + imu + mic + eye + aod
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub percentage: f32, // 0.0 - 100.0
    pub is_charging: bool,
    pub time_remaining_secs: Option<u64>,
}

/// ZK Proof Batch Queue for power optimization
/// Batching proofs saves ~30% energy vs individual proving
pub struct ZkBatchQueue {
    /// Pending items waiting to be proven
    pending_count: AtomicU64,
    /// Max batch size before forced flush
    max_batch_size: u64,
    /// Time-based flush interval (seconds)
    flush_interval_secs: u64,
    /// Last flush timestamp
    last_flush: std::sync::Mutex<Instant>,
    /// Whether batch mode is enabled
    enabled: AtomicBool,
}

impl ZkBatchQueue {
    pub fn new() -> Self {
        Self {
            pending_count: AtomicU64::new(0),
            max_batch_size: 16,
            flush_interval_secs: 30,
            last_flush: std::sync::Mutex::new(Instant::now()),
            enabled: AtomicBool::new(true),
        }
    }
    
    /// Queue a proof for batching, returns true if batch should be flushed
    pub fn queue(&self) -> bool {
        let count = self.pending_count.fetch_add(1, Ordering::SeqCst) + 1;
        
        // Check if batch is full
        if count >= self.max_batch_size {
            return true;
        }
        
        // Check time-based flush
        if let Ok(last) = self.last_flush.lock() {
            if last.elapsed() >= Duration::from_secs(self.flush_interval_secs) {
                return count > 0;
            }
        }
        
        false
    }
    
    /// Flush the queue and return count of items to prove
    pub fn flush(&self) -> u64 {
        let count = self.pending_count.swap(0, Ordering::SeqCst);
        if let Ok(mut last) = self.last_flush.lock() {
            *last = Instant::now();
        }
        count
    }
    
    /// Get pending count
    pub fn pending(&self) -> u64 {
        self.pending_count.load(Ordering::SeqCst)
    }
    
    /// Enable/disable batching (disable for time-critical proofs)
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

impl Default for ZkBatchQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PowerManager {
    sys: System,
    pub profile: PowerProfile,
    pub battery: BatteryState,
    /// Current gaze state for doze detection
    pub gaze_state: GazeState,
    /// Time since last user interaction
    idle_since: Instant,
    /// Power optimization statistics
    pub stats: PowerStats,
    /// Current sensor polling configuration
    pub sensor_config: SensorPollingConfig,
    /// ZK proof batch queue
    pub zk_queue: ZkBatchQueue,
    /// Doze entry threshold (seconds of no interaction)
    doze_threshold_secs: u64,
}

impl PowerManager {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );
        sys.refresh_all();
        
        let profile = PowerProfile::Balanced;

        Self {
            sys,
            profile,
            battery: BatteryState {
                percentage: 100.0, // Default assumption
                is_charging: true,
                time_remaining_secs: None,
            },
            gaze_state: GazeState::Active,
            idle_since: Instant::now(),
            stats: PowerStats::default(),
            sensor_config: SensorPollingConfig::for_profile(profile),
            zk_queue: ZkBatchQueue::new(),
            doze_threshold_secs: 30, // Enter doze after 30s idle
        }
    }

    pub fn update(&mut self) -> String {
        // Refresh system stats
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        
        // Check for doze transition
        self.check_doze_transition();
        
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
            // Power drain varies by profile
            if !self.battery.is_charging {
                let drain = match self.profile {
                    PowerProfile::Performance => 0.5,
                    PowerProfile::Balanced => 0.1,
                    PowerProfile::LowPower => 0.02,
                    PowerProfile::Critical => 0.01,
                    PowerProfile::Doze => 0.005, // Ultra low in doze
                };
                self.battery.percentage = (self.battery.percentage - drain).max(0.0);
                
                // Track energy saved vs Performance baseline
                let saved = 0.5 - drain;
                self.stats.energy_saved_mwh += saved * 0.1; // Approximate mWh
            } else {
                // Charge up
                self.battery.percentage = (self.battery.percentage + 1.0).min(100.0);
            }
        }

        // Auto-switch profiles based on battery
        if self.battery.percentage < 10.0 && self.profile != PowerProfile::Critical {
            self.set_profile(PowerProfile::Critical);
        } else if self.battery.percentage < 30.0 && self.profile == PowerProfile::Performance {
            self.set_profile(PowerProfile::LowPower);
        }

        format!("Power: {:?} | Bat: {:.1}% ({}) | Gaze: {:?}", 
            self.profile, 
            self.battery.percentage, 
            if self.battery.is_charging { "⚡" } else { "🔋" },
            self.gaze_state
        )
    }
    
    /// Check and handle doze mode transitions
    fn check_doze_transition(&mut self) {
        let idle_secs = self.idle_since.elapsed().as_secs();
        
        match self.gaze_state {
            GazeState::Active => {
                // User active, reset idle timer handled by on_user_interaction
            }
            GazeState::Peripheral => {
                // User nearby but not focused
                if idle_secs > self.doze_threshold_secs * 2 {
                    self.enter_doze();
                }
            }
            GazeState::Away | GazeState::Closed => {
                // User not looking, enter doze quickly
                if idle_secs > self.doze_threshold_secs / 2 && self.profile != PowerProfile::Doze {
                    self.enter_doze();
                }
            }
        }
    }
    
    /// Enter Oracle doze mode
    pub fn enter_doze(&mut self) {
        if self.profile == PowerProfile::Doze {
            return; // Already in doze
        }
        
        log::info!("Atom 3 (Power): Entering Doze mode");
        self.profile = PowerProfile::Doze;
        self.sensor_config = SensorPollingConfig::for_profile(PowerProfile::Doze);
        self.stats.doze_cycles += 1;
    }
    
    /// Exit doze mode, called on gaze/voice wake
    pub fn exit_doze(&mut self, wake_reason: WakeReason) {
        if self.profile != PowerProfile::Doze {
            return; // Not in doze
        }
        
        // Track doze time
        let doze_duration = self.idle_since.elapsed().as_secs();
        self.stats.doze_time_secs += doze_duration;
        
        // Track wake reason
        match wake_reason {
            WakeReason::Gaze => self.stats.gaze_wakes += 1,
            WakeReason::Voice => self.stats.voice_wakes += 1,
            WakeReason::Touch => {}
            WakeReason::Timer => {}
        }
        
        log::info!("Atom 3 (Power): Exiting Doze ({:?} wake, {}s doze)", wake_reason, doze_duration);
        
        // Return to balanced profile
        self.set_profile(PowerProfile::Balanced);
        self.idle_since = Instant::now();
    }
    
    /// Called when user interaction detected
    pub fn on_user_interaction(&mut self) {
        self.idle_since = Instant::now();
        
        // Exit doze if active
        if self.profile == PowerProfile::Doze {
            self.exit_doze(WakeReason::Touch);
        }
    }
    
    /// Update gaze state from eye tracking (eBPF-like hook)
    pub fn update_gaze(&mut self, gaze: GazeState) {
        let prev_gaze = self.gaze_state;
        self.gaze_state = gaze;
        
        // Wake from doze on gaze activation
        if prev_gaze != GazeState::Active && gaze == GazeState::Active {
            if self.profile == PowerProfile::Doze {
                self.exit_doze(WakeReason::Gaze);
            }
            self.idle_since = Instant::now();
        }
    }
    
    /// Queue a ZK proof for batch processing
    pub fn queue_zk_proof(&mut self) -> bool {
        if !self.zk_queue.is_enabled() {
            return true; // Immediate proof required
        }
        
        let should_flush = self.zk_queue.queue();
        if should_flush {
            self.stats.zk_proofs_batched += self.zk_queue.pending();
        }
        should_flush
    }
    
    /// Get power-aware frame drop decision
    /// Returns true if frame should be dropped to save power
    pub fn should_drop_frame(&mut self, frame_idx: u64) -> bool {
        let drop = match self.profile {
            PowerProfile::Performance => false,
            PowerProfile::Balanced => frame_idx % 2 == 1, // Drop every other
            PowerProfile::LowPower => frame_idx % 4 != 0, // Keep every 4th
            PowerProfile::Critical => frame_idx % 12 != 0, // Keep every 12th
            PowerProfile::Doze => true, // Drop all
        };
        
        if drop {
            self.stats.frames_dropped += 1;
        }
        drop
    }

    pub fn set_profile(&mut self, profile: PowerProfile) {
        self.profile = profile;
        self.sensor_config = SensorPollingConfig::for_profile(profile);
        log::info!("Atom 3 (Power): Switched to {:?} Profile ({}mW est.)", 
            profile, self.sensor_config.estimated_power_mw());
    }

    pub fn toggle_charging_sim(&mut self) {
        self.battery.is_charging = !self.battery.is_charging;
        log::info!("Atom 3 (Power): Charging Simulation Toggled: {}", self.battery.is_charging);
    }
    
    pub fn get_status_string(&self) -> String {
         format!("{:?} {:.0}%", self.profile, self.battery.percentage)
    }
    
    /// Get comprehensive power stats
    pub fn get_stats(&self) -> &PowerStats {
        &self.stats
    }
    
    /// Get current sensor configuration
    pub fn get_sensor_config(&self) -> &SensorPollingConfig {
        &self.sensor_config
    }
    
    /// Check if currently in doze mode
    pub fn is_dozing(&self) -> bool {
        self.profile == PowerProfile::Doze
    }
    
    /// Get estimated remaining runtime in minutes
    pub fn estimated_runtime_mins(&self) -> Option<u32> {
        if self.battery.is_charging {
            return None;
        }
        
        let power_mw = self.sensor_config.estimated_power_mw();
        // Assume 2000mWh battery capacity
        let battery_mwh = 2000.0 * (self.battery.percentage / 100.0);
        
        Some((battery_mwh / power_mw * 60.0) as u32)
    }
}

/// Reason for waking from doze mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WakeReason {
    /// User looked at display
    Gaze,
    /// Wake word detected
    Voice,
    /// Physical touch/gesture
    Touch,
    /// Scheduled timer/notification
    Timer,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_power_profiles() {
        let mut pm = PowerManager::new();
        assert_eq!(pm.profile, PowerProfile::Balanced);
        
        pm.set_profile(PowerProfile::Performance);
        assert_eq!(pm.profile, PowerProfile::Performance);
        assert_eq!(pm.sensor_config.camera_fps, 60);
        
        pm.set_profile(PowerProfile::LowPower);
        assert_eq!(pm.profile, PowerProfile::LowPower);
        assert_eq!(pm.sensor_config.camera_fps, 15);
    }
    
    #[test]
    fn test_doze_mode() {
        let mut pm = PowerManager::new();
        
        // Enter doze
        pm.enter_doze();
        assert!(pm.is_dozing());
        assert_eq!(pm.stats.doze_cycles, 1);
        assert_eq!(pm.sensor_config.camera_fps, 0); // Camera off in doze
        
        // Exit doze via gaze
        pm.exit_doze(WakeReason::Gaze);
        assert!(!pm.is_dozing());
        assert_eq!(pm.stats.gaze_wakes, 1);
        assert_eq!(pm.profile, PowerProfile::Balanced);
    }
    
    #[test]
    fn test_gaze_state_tracking() {
        let mut pm = PowerManager::new();
        assert_eq!(pm.gaze_state, GazeState::Active);
        
        pm.update_gaze(GazeState::Away);
        assert_eq!(pm.gaze_state, GazeState::Away);
        
        // Gaze returning to Active should reset idle
        pm.enter_doze();
        pm.update_gaze(GazeState::Active);
        assert!(!pm.is_dozing()); // Should have exited doze
        assert_eq!(pm.stats.gaze_wakes, 1);
    }
    
    #[test]
    fn test_zk_batch_queue() {
        let queue = ZkBatchQueue::new();
        
        // Queue items
        for _ in 0..15 {
            let should_flush = queue.queue();
            assert!(!should_flush);
        }
        
        // 16th item should trigger flush
        let should_flush = queue.queue();
        assert!(should_flush);
        
        // Flush and verify
        let count = queue.flush();
        assert_eq!(count, 16);
        assert_eq!(queue.pending(), 0);
    }
    
    #[test]
    fn test_frame_dropping() {
        let mut pm = PowerManager::new();
        
        // Performance mode: no drops
        pm.set_profile(PowerProfile::Performance);
        assert!(!pm.should_drop_frame(0));
        assert!(!pm.should_drop_frame(1));
        
        // Low power: keep every 4th frame
        pm.set_profile(PowerProfile::LowPower);
        assert!(!pm.should_drop_frame(0));
        assert!(pm.should_drop_frame(1));
        assert!(pm.should_drop_frame(2));
        assert!(pm.should_drop_frame(3));
        assert!(!pm.should_drop_frame(4));
    }
    
    #[test]
    fn test_sensor_power_estimation() {
        let perf = SensorPollingConfig::for_profile(PowerProfile::Performance);
        let doze = SensorPollingConfig::for_profile(PowerProfile::Doze);
        
        // Performance should use significantly more power
        assert!(perf.estimated_power_mw() > doze.estimated_power_mw() * 10.0);
        
        // Doze should be very low power
        assert!(doze.estimated_power_mw() < 50.0);
    }
    
    #[test]
    fn test_estimated_runtime() {
        let mut pm = PowerManager::new();
        pm.battery.is_charging = false;
        pm.battery.percentage = 100.0;
        
        pm.set_profile(PowerProfile::Doze);
        let doze_runtime = pm.estimated_runtime_mins();
        
        pm.set_profile(PowerProfile::Performance);
        let perf_runtime = pm.estimated_runtime_mins();
        
        // Doze should have much longer runtime
        assert!(doze_runtime.unwrap() > perf_runtime.unwrap() * 5);
    }
    
    #[test]
    fn test_user_interaction() {
        let mut pm = PowerManager::new();
        pm.enter_doze();
        assert!(pm.is_dozing());
        
        pm.on_user_interaction();
        assert!(!pm.is_dozing()); // Should exit doze
    }
    
    #[test]
    fn test_battery_auto_profile() {
        let mut pm = PowerManager::new();
        pm.battery.is_charging = false;
        pm.set_profile(PowerProfile::Performance);
        
        // Simulate low battery
        pm.battery.percentage = 8.0;
        pm.update();
        
        // Should auto-switch to Critical
        assert_eq!(pm.profile, PowerProfile::Critical);
    }
}
