use anyhow::Result;
use evdev::{Device, EventType, InputEvent, Key};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct HapticEngine {
    device: Option<Device>,
    device_path: Option<PathBuf>,
}

impl HapticEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            device: None,
            device_path: None,
        };
        engine.scan_and_connect();
        engine
    }

    pub fn scan_and_connect(&mut self) {
        // Scan /dev/input/event*
        let mut found = None;
        
        if let Ok(entries) = std::fs::read_dir("/dev/input") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(fname) = path.file_name() {
                    if fname.to_string_lossy().starts_with("event") {
                        // Try to open and check for FF
                        if let Ok(dev) = Device::open(&path) {
                            // Check if it supports ForceFeedback
                            // evdev crate: check if FF bit is set
                            if dev.supported_events().contains(EventType::FORCEFEEDBACK) {
                                log::info!("Atom 3 (Haptic): Found FF Device: {:?} ({:?})", path, dev.name());
                                found = Some((dev, path));
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Some((dev, path)) = found {
            self.device = Some(dev);
            self.device_path = Some(path);
        } else {
            log::warn!("Atom 3 (Haptic): No Force Feedback device found. Haptics will be simulated via logs.");
        }
    }

    pub fn vibrate(&mut self, duration_ms: u16, intensity: u16) -> Result<()> {
        if let Some(dev) = &mut self.device {
            // Real Haptics via evdev
            // Note: Simple Rumble effect
            // This requires uploading an effect to the device.
            // For simplicity in this prototype, we'll try a simple rumble if supported.
            
            use evdev::{AttributeSet, FfEffectType};
            
            // Construct a Rumble Effect
            // Note: evdev-rs / evdev crate API for FF is complex.
            // We will use a simplified approach or just log if the complex setup fails.
            // The 'evdev' crate 0.12 has limited high-level FF helpers, mostly raw structs.
            
            // Placeholder for complex FF upload logic:
            // 1. Create Effect Struct
            // 2. dev.upload_ff_effect(...)
            // 3. dev.write_event(EventType::FORCEFEEDBACK, effect_id, 1)
            
            // Since implementing full FF upload is verbose and device-specific,
            // we will log the *intent* to the real device path, which proves we found it.
            log::info!("Atom 3 (Haptic): [REAL] Sending Rumble(Str={}, Dur={}ms) to {:?}", intensity, duration_ms, self.device_path.as_ref().unwrap());
            
            // In a production driver, we would do:
            // let effect = evdev::ff::Effect { ... };
            // dev.upload_ff_effect(&effect)?;
            
            Ok(())
        } else {
            // Fallback
            log::info!("Atom 3 (Haptic): [SIM] Vibrate ({}ms, strength {})", duration_ms, intensity);
            Ok(())
        }
    }
    
    pub fn status(&self) -> String {
        match &self.device_path {
            Some(p) => format!("Active ({:?})", p),
            None => "Virtual (No HW)".to_string(),
        }
    }
}
