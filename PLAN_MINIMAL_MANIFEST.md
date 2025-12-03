# Plan: Minimal AR Manifest System

## Overview
The Oracle outputs responses through minimal AR overlays and haptic feedback - not through rich UI panels. This is the "output side" of the Oracle Veil.

---

## Target Architecture

```
Oracle Response → ManifestEngine → [AR Whisper + Haptic Pulse]
```

**Principles:**
1. **Minimal text** - Max 50 characters for AR whispers
2. **Haptic-first** - Primary feedback is tactile
3. **Contextual overlays** - Progress bars, checkmarks, not text walls
4. **Fading** - AR elements appear briefly then fade

---

## Implementation Plan

### File: `karana-core/src/oracle/manifest.rs`

### Step 1: Core Structures

```rust
use anyhow::Result;
use std::time::Duration;

/// What the Oracle manifests to the user
#[derive(Debug, Clone)]
pub struct Manifest {
    /// Short text whisper (max 50 chars)
    pub text: String,
    
    /// Haptic pattern to play
    pub haptic: Option<HapticPattern>,
    
    /// AR overlay element
    pub ar_overlay: Option<AROverlay>,
    
    /// Duration before auto-dismiss
    pub duration: Duration,
    
    /// Priority (affects haptic intensity)
    pub priority: ManifestPriority,
}

/// Priority levels for manifest
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ManifestPriority {
    Low,      // Quiet whisper, subtle haptic
    Normal,   // Standard presentation
    High,     // Prominent overlay, strong haptic
    Urgent,   // Persistent until acknowledged
}

/// AR overlay types
#[derive(Debug, Clone)]
pub enum AROverlay {
    /// Simple text whisper (faint, center-bottom)
    TextWhisper(String),
    
    /// Progress bar (e.g., battery, balance)
    Progress {
        label: String,
        value: f32,  // 0.0 to 1.0
        color: ARColor,
    },
    
    /// Confirmation checkmark (success)
    Confirmation {
        label: String,
    },
    
    /// Warning icon (error/alert)
    Warning {
        message: String,
    },
    
    /// Timer countdown
    Timer {
        remaining_secs: u32,
        label: String,
    },
    
    /// Navigation arrow
    Navigation {
        direction: [f32; 3],
        distance_m: f32,
    },
    
    /// Object highlight (for vision context)
    Highlight {
        bounds: [f32; 4],  // [x, y, width, height] normalized
        label: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ARColor {
    Amber,
    Green,
    Red,
    Blue,
    White,
}

/// Haptic feedback patterns
#[derive(Debug, Clone, Copy)]
pub enum HapticPattern {
    /// Short single pulse (acknowledgment)
    Tap,
    
    /// Double tap (success)
    DoubleTap,
    
    /// Long pulse (confirmation needed)
    LongPulse,
    
    /// Rapid pulses (error/warning)
    Buzz,
    
    /// Gentle wave (notification)
    Wave,
    
    /// Strong burst (urgent)
    Burst,
    
    /// Custom pattern: durations in ms, intensities 0-255
    Custom(Vec<(u32, u8)>),
}

impl HapticPattern {
    /// Convert to timing sequence (ms, intensity)
    pub fn to_sequence(&self) -> Vec<(u32, u8)> {
        match self {
            Self::Tap => vec![(50, 200)],
            Self::DoubleTap => vec![(50, 200), (50, 0), (50, 200)],
            Self::LongPulse => vec![(200, 150)],
            Self::Buzz => vec![(30, 255), (20, 0), (30, 255), (20, 0), (30, 255)],
            Self::Wave => vec![(100, 50), (100, 100), (100, 150), (100, 100), (100, 50)],
            Self::Burst => vec![(100, 255)],
            Self::Custom(seq) => seq.clone(),
        }
    }
    
    /// Get appropriate pattern for intent result
    pub fn for_result(success: bool, priority: ManifestPriority) -> Self {
        match (success, priority) {
            (true, ManifestPriority::Low) => Self::Tap,
            (true, ManifestPriority::Normal) => Self::DoubleTap,
            (true, ManifestPriority::High) => Self::Wave,
            (true, ManifestPriority::Urgent) => Self::Burst,
            (false, ManifestPriority::Low) => Self::Tap,
            (false, ManifestPriority::Normal) => Self::LongPulse,
            (false, ManifestPriority::High) => Self::Buzz,
            (false, ManifestPriority::Urgent) => Self::Buzz,
        }
    }
}
```

### Step 2: Manifest Engine

```rust
/// Engine for generating and rendering manifests
pub struct ManifestEngine {
    /// Hardware haptic driver
    haptic_driver: Box<dyn HapticDriver>,
    
    /// AR renderer
    ar_renderer: Box<dyn ARRenderer>,
    
    /// Current active overlays
    active_overlays: Vec<ActiveOverlay>,
    
    /// Configuration
    config: ManifestConfig,
}

struct ActiveOverlay {
    overlay: AROverlay,
    start_time: std::time::Instant,
    duration: Duration,
}

pub struct ManifestConfig {
    /// Default overlay duration
    pub default_duration: Duration,
    
    /// Fade out duration
    pub fade_duration: Duration,
    
    /// Maximum concurrent overlays
    pub max_overlays: usize,
    
    /// Haptic intensity multiplier
    pub haptic_intensity: f32,
}

impl Default for ManifestConfig {
    fn default() -> Self {
        Self {
            default_duration: Duration::from_secs(3),
            fade_duration: Duration::from_millis(500),
            max_overlays: 3,
            haptic_intensity: 1.0,
        }
    }
}

impl ManifestEngine {
    pub fn new(
        haptic_driver: Box<dyn HapticDriver>,
        ar_renderer: Box<dyn ARRenderer>,
        config: ManifestConfig,
    ) -> Self {
        Self {
            haptic_driver,
            ar_renderer,
            active_overlays: Vec::new(),
            config,
        }
    }
    
    /// Output a manifest
    pub async fn output(&mut self, manifest: Manifest) -> Result<()> {
        log::info!("[MANIFEST] Output: {:?}", manifest.text);
        
        // Play haptic if present
        if let Some(pattern) = manifest.haptic {
            self.play_haptic(pattern).await?;
        }
        
        // Render AR overlay if present
        if let Some(overlay) = manifest.ar_overlay {
            self.render_overlay(overlay, manifest.duration).await?;
        }
        
        Ok(())
    }
    
    async fn play_haptic(&mut self, pattern: HapticPattern) -> Result<()> {
        let sequence = pattern.to_sequence();
        
        // Apply intensity multiplier
        let scaled: Vec<(u32, u8)> = sequence.into_iter()
            .map(|(dur, intensity)| {
                let scaled_intensity = ((intensity as f32) * self.config.haptic_intensity) as u8;
                (dur, scaled_intensity.min(255))
            })
            .collect();
        
        self.haptic_driver.play(&scaled).await
    }
    
    async fn render_overlay(&mut self, overlay: AROverlay, duration: Duration) -> Result<()> {
        // Remove expired overlays
        self.cleanup_overlays();
        
        // Limit concurrent overlays
        while self.active_overlays.len() >= self.config.max_overlays {
            self.active_overlays.remove(0);
        }
        
        // Add new overlay
        self.active_overlays.push(ActiveOverlay {
            overlay: overlay.clone(),
            start_time: std::time::Instant::now(),
            duration,
        });
        
        // Render
        self.ar_renderer.render(&overlay).await
    }
    
    fn cleanup_overlays(&mut self) {
        self.active_overlays.retain(|o| {
            o.start_time.elapsed() < o.duration + self.config.fade_duration
        });
    }
    
    /// Get current overlay opacity (for fading)
    pub fn get_overlay_opacity(&self, overlay_idx: usize) -> f32 {
        if let Some(overlay) = self.active_overlays.get(overlay_idx) {
            let elapsed = overlay.start_time.elapsed();
            
            if elapsed < overlay.duration {
                1.0  // Full opacity
            } else {
                // Fade out
                let fade_progress = (elapsed - overlay.duration).as_secs_f32() 
                    / self.config.fade_duration.as_secs_f32();
                (1.0 - fade_progress).max(0.0)
            }
        } else {
            0.0
        }
    }
}
```

### Step 3: Haptic Driver Trait

```rust
/// Trait for haptic hardware drivers
#[async_trait::async_trait]
pub trait HapticDriver: Send + Sync {
    /// Play a haptic sequence
    /// Each tuple is (duration_ms, intensity_0_255)
    async fn play(&mut self, sequence: &[(u32, u8)]) -> Result<()>;
    
    /// Stop any ongoing haptic
    async fn stop(&mut self) -> Result<()>;
    
    /// Check if haptic hardware is available
    fn is_available(&self) -> bool;
}

/// Simulated haptic driver (for testing)
pub struct SimulatedHaptic {
    log_output: bool,
}

impl SimulatedHaptic {
    pub fn new(log_output: bool) -> Self {
        Self { log_output }
    }
}

#[async_trait::async_trait]
impl HapticDriver for SimulatedHaptic {
    async fn play(&mut self, sequence: &[(u32, u8)]) -> Result<()> {
        if self.log_output {
            let total_duration: u32 = sequence.iter().map(|(d, _)| d).sum();
            let max_intensity = sequence.iter().map(|(_, i)| *i).max().unwrap_or(0);
            log::info!("[HAPTIC] Playing: {}ms @ max intensity {}", total_duration, max_intensity);
        }
        
        // Simulate playback time
        for (dur_ms, _) in sequence {
            tokio::time::sleep(Duration::from_millis(*dur_ms as u64)).await;
        }
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        if self.log_output {
            log::info!("[HAPTIC] Stopped");
        }
        Ok(())
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

/// Real Linux haptic driver (vibration motor via GPIO)
#[cfg(target_os = "linux")]
pub struct LinuxHaptic {
    pwm_path: std::path::PathBuf,
}

#[cfg(target_os = "linux")]
impl LinuxHaptic {
    pub fn new(pwm_chip: u32, pwm_channel: u32) -> Result<Self> {
        let pwm_path = std::path::PathBuf::from(format!(
            "/sys/class/pwm/pwmchip{}/pwm{}",
            pwm_chip, pwm_channel
        ));
        
        // Export PWM channel if not already
        let export_path = pwm_path.parent().unwrap().join("export");
        if !pwm_path.exists() {
            std::fs::write(&export_path, format!("{}", pwm_channel))?;
        }
        
        Ok(Self { pwm_path })
    }
}

#[cfg(target_os = "linux")]
#[async_trait::async_trait]
impl HapticDriver for LinuxHaptic {
    async fn play(&mut self, sequence: &[(u32, u8)]) -> Result<()> {
        let period_path = self.pwm_path.join("period");
        let duty_path = self.pwm_path.join("duty_cycle");
        let enable_path = self.pwm_path.join("enable");
        
        // Set period (1MHz = 1000ns period)
        std::fs::write(&period_path, "1000")?;
        std::fs::write(&enable_path, "1")?;
        
        for (dur_ms, intensity) in sequence {
            // duty_cycle controls intensity (0-1000 for our period)
            let duty = (*intensity as u32) * 1000 / 255;
            std::fs::write(&duty_path, format!("{}", duty))?;
            
            tokio::time::sleep(Duration::from_millis(*dur_ms as u64)).await;
        }
        
        std::fs::write(&duty_path, "0")?;
        std::fs::write(&enable_path, "0")?;
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        let enable_path = self.pwm_path.join("enable");
        std::fs::write(&enable_path, "0")?;
        Ok(())
    }
    
    fn is_available(&self) -> bool {
        self.pwm_path.exists()
    }
}
```

### Step 4: AR Renderer Trait

```rust
/// Trait for AR overlay rendering
#[async_trait::async_trait]
pub trait ARRenderer: Send + Sync {
    /// Render an overlay
    async fn render(&mut self, overlay: &AROverlay) -> Result<()>;
    
    /// Clear all overlays
    async fn clear(&mut self) -> Result<()>;
    
    /// Check if AR rendering is available
    fn is_available(&self) -> bool;
}

/// Simulated AR renderer (logs to console)
pub struct SimulatedAR {
    log_output: bool,
}

impl SimulatedAR {
    pub fn new(log_output: bool) -> Self {
        Self { log_output }
    }
}

#[async_trait::async_trait]
impl ARRenderer for SimulatedAR {
    async fn render(&mut self, overlay: &AROverlay) -> Result<()> {
        if self.log_output {
            match overlay {
                AROverlay::TextWhisper(text) => {
                    log::info!("[AR] 💬 {}", text);
                }
                AROverlay::Progress { label, value, color } => {
                    let bar_len = 20;
                    let filled = (value * bar_len as f32) as usize;
                    let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
                    log::info!("[AR] 📊 {} [{}] {:.0}%", label, bar, value * 100.0);
                }
                AROverlay::Confirmation { label } => {
                    log::info!("[AR] ✓ {}", label);
                }
                AROverlay::Warning { message } => {
                    log::info!("[AR] ⚠️ {}", message);
                }
                AROverlay::Timer { remaining_secs, label } => {
                    let mins = remaining_secs / 60;
                    let secs = remaining_secs % 60;
                    log::info!("[AR] ⏱️ {} {:02}:{:02}", label, mins, secs);
                }
                AROverlay::Navigation { direction, distance_m } => {
                    let arrow = if direction[0] > 0.5 { "→" }
                        else if direction[0] < -0.5 { "←" }
                        else if direction[1] > 0.5 { "↑" }
                        else { "↓" };
                    log::info!("[AR] 🧭 {} {:.0}m", arrow, distance_m);
                }
                AROverlay::Highlight { bounds, label } => {
                    log::info!("[AR] 🔍 {} @ [{:.2}, {:.2}]", label, bounds[0], bounds[1]);
                }
            }
        }
        Ok(())
    }
    
    async fn clear(&mut self) -> Result<()> {
        if self.log_output {
            log::info!("[AR] Cleared");
        }
        Ok(())
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

/// Metal-based AR renderer for real glasses
#[cfg(target_os = "macos")]
pub struct MetalARRenderer {
    // Metal device, command queue, etc.
    // Would use metal-rs crate
}

// Would implement real Metal rendering here
```

### Step 5: Manifest Builder

```rust
/// Builder for creating manifests from intent results
pub struct ManifestBuilder {
    config: ManifestConfig,
}

impl ManifestBuilder {
    pub fn new() -> Self {
        Self {
            config: ManifestConfig::default(),
        }
    }
    
    /// Build manifest for a balance check result
    pub fn balance(&self, balance: u64, max: u64) -> Manifest {
        let ratio = balance as f32 / max as f32;
        
        Manifest {
            text: format!("{} KARA", balance),
            haptic: Some(HapticPattern::Tap),
            ar_overlay: Some(AROverlay::Progress {
                label: "Balance".into(),
                value: ratio.min(1.0),
                color: if ratio > 0.5 { ARColor::Green } else { ARColor::Amber },
            }),
            duration: Duration::from_secs(3),
            priority: ManifestPriority::Normal,
        }
    }
    
    /// Build manifest for a transfer result
    pub fn transfer(&self, amount: u64, recipient: &str, success: bool) -> Manifest {
        if success {
            Manifest {
                text: format!("✓ {} to {}", amount, &recipient[..8.min(recipient.len())]),
                haptic: Some(HapticPattern::DoubleTap),
                ar_overlay: Some(AROverlay::Confirmation {
                    label: format!("Sent {} KARA", amount),
                }),
                duration: Duration::from_secs(2),
                priority: ManifestPriority::High,
            }
        } else {
            Manifest {
                text: "✗ Transfer failed".into(),
                haptic: Some(HapticPattern::Buzz),
                ar_overlay: Some(AROverlay::Warning {
                    message: "Transfer failed".into(),
                }),
                duration: Duration::from_secs(4),
                priority: ManifestPriority::High,
            }
        }
    }
    
    /// Build manifest for a timer
    pub fn timer(&self, remaining_secs: u32, label: &str) -> Manifest {
        let urgency = if remaining_secs <= 10 {
            ManifestPriority::High
        } else {
            ManifestPriority::Normal
        };
        
        Manifest {
            text: format!("⏱️ {} {}", label, format_time(remaining_secs)),
            haptic: if remaining_secs == 0 {
                Some(HapticPattern::Burst)
            } else if remaining_secs <= 10 {
                Some(HapticPattern::Tap)
            } else {
                None
            },
            ar_overlay: Some(AROverlay::Timer {
                remaining_secs,
                label: label.into(),
            }),
            duration: Duration::from_secs(2),
            priority: urgency,
        }
    }
    
    /// Build manifest for general success
    pub fn success(&self, message: &str) -> Manifest {
        Manifest {
            text: format!("✓ {}", truncate(message, 47)),
            haptic: Some(HapticPattern::DoubleTap),
            ar_overlay: Some(AROverlay::Confirmation {
                label: message.to_string(),
            }),
            duration: Duration::from_secs(2),
            priority: ManifestPriority::Normal,
        }
    }
    
    /// Build manifest for general error
    pub fn error(&self, message: &str) -> Manifest {
        Manifest {
            text: format!("✗ {}", truncate(message, 47)),
            haptic: Some(HapticPattern::Buzz),
            ar_overlay: Some(AROverlay::Warning {
                message: message.to_string(),
            }),
            duration: Duration::from_secs(4),
            priority: ManifestPriority::High,
        }
    }
    
    /// Build manifest for navigation
    pub fn navigation(&self, direction: [f32; 3], distance: f32, instruction: &str) -> Manifest {
        Manifest {
            text: truncate(instruction, 50).to_string(),
            haptic: Some(HapticPattern::Tap),
            ar_overlay: Some(AROverlay::Navigation {
                direction,
                distance_m: distance,
            }),
            duration: Duration::from_secs(5),
            priority: ManifestPriority::Normal,
        }
    }
    
    /// Build manifest for object identification
    pub fn identify(&self, object: &str, bounds: [f32; 4]) -> Manifest {
        Manifest {
            text: object.to_string(),
            haptic: Some(HapticPattern::Tap),
            ar_overlay: Some(AROverlay::Highlight {
                bounds,
                label: object.into(),
            }),
            duration: Duration::from_secs(3),
            priority: ManifestPriority::Normal,
        }
    }
    
    /// Build manifest for conversation response
    pub fn conversation(&self, response: &str) -> Manifest {
        Manifest {
            text: truncate(response, 50).to_string(),
            haptic: None,  // No haptic for conversation
            ar_overlay: Some(AROverlay::TextWhisper(truncate(response, 100).into())),
            duration: Duration::from_secs(5),
            priority: ManifestPriority::Low,
        }
    }
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len-3]  // Leave room for "..."
    }
}

fn format_time(secs: u32) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{}:{:02}", mins, secs)
}
```

---

## Integration with Oracle

```rust
// In oracle/veil.rs

impl OracleVeil {
    async fn create_manifest(
        &self,
        intent: &ParsedIntent,
        response: &MonadResponse,
    ) -> Result<Manifest> {
        let builder = ManifestBuilder::new();
        
        match intent.action.as_str() {
            "balance" | "get_balance" => {
                let balance: u64 = response.data.parse().unwrap_or(0);
                Ok(builder.balance(balance, 10000))
            },
            "transfer" => {
                let amount = intent.params.get("amount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let recipient = intent.params.get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                Ok(builder.transfer(amount, recipient, response.success))
            },
            "timer" | "set_timer" => {
                let secs = intent.params.get("seconds")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(300) as u32;
                let label = intent.params.get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Timer");
                Ok(builder.timer(secs, label))
            },
            "navigate" => {
                let direction = [0.0, 0.0, 1.0];  // Would come from nav engine
                let distance = 100.0;
                Ok(builder.navigation(direction, distance, &response.data))
            },
            "identify" | "analyze" => {
                Ok(builder.identify(&response.data, [0.4, 0.4, 0.2, 0.2]))
            },
            _ => {
                if response.success {
                    Ok(builder.success(&response.data))
                } else {
                    Ok(builder.error(&response.data))
                }
            }
        }
    }
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_haptic_sequence() {
        let mut haptic = SimulatedHaptic::new(false);
        let pattern = HapticPattern::DoubleTap;
        haptic.play(&pattern.to_sequence()).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_ar_render() {
        let mut ar = SimulatedAR::new(false);
        let overlay = AROverlay::Progress {
            label: "Test".into(),
            value: 0.75,
            color: ARColor::Green,
        };
        ar.render(&overlay).await.unwrap();
    }
    
    #[test]
    fn test_manifest_builder_balance() {
        let builder = ManifestBuilder::new();
        let manifest = builder.balance(500, 1000);
        
        assert!(manifest.text.contains("500"));
        assert!(manifest.haptic.is_some());
        assert!(matches!(manifest.ar_overlay, Some(AROverlay::Progress { .. })));
    }
    
    #[test]
    fn test_manifest_builder_transfer() {
        let builder = ManifestBuilder::new();
        let manifest = builder.transfer(100, "alice", true);
        
        assert!(manifest.text.contains("✓"));
        assert!(manifest.text.contains("100"));
        assert!(matches!(manifest.haptic, Some(HapticPattern::DoubleTap)));
    }
}
```

---

## Timeline

| Task | Duration |
|------|----------|
| Core structures | 2 hours |
| Manifest engine | 3 hours |
| Haptic driver | 2 hours |
| AR renderer | 3 hours |
| Manifest builder | 2 hours |
| Integration | 2 hours |
| Testing | 2 hours |
| **Total** | **16 hours** |

---

## Success Criteria

- [ ] Haptic patterns play correctly (simulated/real)
- [ ] AR overlays render (at least simulated)
- [ ] Manifests created for all intent types
- [ ] Overlays auto-fade after duration
- [ ] Maximum 50 chars for text whispers
- [ ] < 50ms haptic response time

---

*Minimal Manifest Plan v1.0 - December 3, 2025*
