//! Reverb Processing
//!
//! Room acoustic simulation for realistic audio environments.

use std::f32::consts::PI;

/// Reverb processor for room simulation
#[derive(Debug)]
pub struct ReverbProcessor {
    sample_rate: u32,
    buffer_size: usize,
    /// Comb filter delay lines
    comb_filters: Vec<CombFilter>,
    /// Allpass filter delay lines
    allpass_filters: Vec<AllpassFilter>,
    /// Current preset
    preset: ReverbPreset,
    /// Wet/dry mix (0-1)
    wet_mix: f32,
    /// Input gain
    input_gain: f32,
    /// Early reflections buffer
    early_reflections: Vec<f32>,
    early_delay_samples: usize,
}

impl ReverbProcessor {
    pub fn new(sample_rate: u32, buffer_size: usize) -> Self {
        // Schroeder reverb: 4 parallel comb filters + 2 series allpass filters
        let comb_delays = [1557, 1617, 1491, 1422]; // samples at 44.1kHz
        let allpass_delays = [225, 556]; // samples at 44.1kHz
        
        let scale = sample_rate as f32 / 44100.0;
        
        let comb_filters: Vec<CombFilter> = comb_delays.iter()
            .map(|&delay| CombFilter::new((delay as f32 * scale) as usize, 0.84))
            .collect();
        
        let allpass_filters: Vec<AllpassFilter> = allpass_delays.iter()
            .map(|&delay| AllpassFilter::new((delay as f32 * scale) as usize, 0.5))
            .collect();
        
        Self {
            sample_rate,
            buffer_size,
            comb_filters,
            allpass_filters,
            preset: ReverbPreset::Room,
            wet_mix: 0.3,
            input_gain: 0.5,
            early_reflections: vec![0.0; (sample_rate as f32 * 0.1) as usize], // 100ms
            early_delay_samples: (sample_rate as f32 * 0.02) as usize, // 20ms
        }
    }
    
    /// Set reverb preset
    pub fn set_preset(&mut self, preset: ReverbPreset) {
        let (decay, wet) = match &preset {
            ReverbPreset::Small => (0.70, 0.2),
            ReverbPreset::Room => (0.84, 0.3),
            ReverbPreset::Hall => (0.90, 0.4),
            ReverbPreset::Cathedral => (0.95, 0.5),
            ReverbPreset::Plate => (0.88, 0.35),
            ReverbPreset::Spring => (0.82, 0.25),
            ReverbPreset::Custom(_) => (0.84, 0.3),
        };
        
        self.preset = preset;
        
        for comb in &mut self.comb_filters {
            comb.feedback = decay;
        }
        self.wet_mix = wet;
    }
    
    /// Set room acoustics
    pub fn set_room_acoustics(&mut self, acoustics: &RoomAcoustics) {
        // Convert RT60 to feedback coefficient
        // RT60 = -60dB decay time
        // feedback = 10^(-3 * delay / (RT60 * sample_rate))
        
        let base_feedback = if acoustics.rt60 > 0.0 {
            let target_decay: f32 = 0.001; // -60dB
            target_decay.powf(1.0 / (acoustics.rt60 * self.sample_rate as f32 / 1000.0))
        } else {
            0.84
        };
        
        for comb in &mut self.comb_filters {
            comb.feedback = (base_feedback * acoustics.absorption).clamp(0.0, 0.99);
        }
        
        self.wet_mix = (1.0 - acoustics.absorption) * 0.5;
        self.early_delay_samples = (acoustics.size * self.sample_rate as f32 / 343.0) as usize;
    }
    
    /// Process stereo buffer in-place
    pub fn process(&mut self, buffer: &mut [f32]) {
        if buffer.is_empty() {
            return;
        }
        
        // Process as stereo
        let samples = buffer.len() / 2;
        
        for i in 0..samples {
            let left_idx = i * 2;
            let right_idx = i * 2 + 1;
            
            if right_idx >= buffer.len() {
                break;
            }
            
            // Mix to mono for reverb processing
            let mono_in = (buffer[left_idx] + buffer[right_idx]) * 0.5 * self.input_gain;
            
            // Parallel comb filters
            let mut comb_out = 0.0;
            for comb in &mut self.comb_filters {
                comb_out += comb.process(mono_in);
            }
            comb_out /= self.comb_filters.len() as f32;
            
            // Series allpass filters
            let mut reverb_out = comb_out;
            for allpass in &mut self.allpass_filters {
                reverb_out = allpass.process(reverb_out);
            }
            
            // Mix wet/dry
            buffer[left_idx] = buffer[left_idx] * (1.0 - self.wet_mix) + reverb_out * self.wet_mix;
            buffer[right_idx] = buffer[right_idx] * (1.0 - self.wet_mix) + reverb_out * self.wet_mix;
        }
    }
    
    /// Set wet/dry mix
    pub fn set_wet_mix(&mut self, mix: f32) {
        self.wet_mix = mix.clamp(0.0, 1.0);
    }
    
    /// Reset all delay lines
    pub fn reset(&mut self) {
        for comb in &mut self.comb_filters {
            comb.reset();
        }
        for allpass in &mut self.allpass_filters {
            allpass.reset();
        }
        self.early_reflections.fill(0.0);
    }
    
    /// Get current preset
    pub fn preset(&self) -> &ReverbPreset {
        &self.preset
    }
}

/// Comb filter for reverb
#[derive(Debug)]
struct CombFilter {
    buffer: Vec<f32>,
    pos: usize,
    feedback: f32,
    lowpass_state: f32,
    damping: f32,
}

impl CombFilter {
    fn new(delay: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; delay.max(1)],
            pos: 0,
            feedback,
            lowpass_state: 0.0,
            damping: 0.2,
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.pos];
        
        // One-pole lowpass filter for damping high frequencies
        self.lowpass_state = delayed * (1.0 - self.damping) + self.lowpass_state * self.damping;
        
        let output = self.lowpass_state;
        self.buffer[self.pos] = input + output * self.feedback;
        
        self.pos = (self.pos + 1) % self.buffer.len();
        
        output
    }
    
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.lowpass_state = 0.0;
        self.pos = 0;
    }
}

/// Allpass filter for diffusion
#[derive(Debug)]
struct AllpassFilter {
    buffer: Vec<f32>,
    pos: usize,
    feedback: f32,
}

impl AllpassFilter {
    fn new(delay: usize, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; delay.max(1)],
            pos: 0,
            feedback,
        }
    }
    
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.pos];
        let output = -input + delayed;
        
        self.buffer[self.pos] = input + delayed * self.feedback;
        self.pos = (self.pos + 1) % self.buffer.len();
        
        output
    }
    
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.pos = 0;
    }
}

/// Reverb presets
#[derive(Debug, Clone, PartialEq)]
pub enum ReverbPreset {
    /// Small room
    Small,
    /// Medium room
    Room,
    /// Large hall
    Hall,
    /// Cathedral/church
    Cathedral,
    /// Plate reverb
    Plate,
    /// Spring reverb
    Spring,
    /// Custom settings
    Custom(ReverbSettings),
}

/// Custom reverb settings
#[derive(Debug, Clone, PartialEq)]
pub struct ReverbSettings {
    /// Decay time (seconds)
    pub decay: f32,
    /// Pre-delay (ms)
    pub pre_delay: f32,
    /// High frequency damping
    pub damping: f32,
    /// Wet/dry mix
    pub mix: f32,
    /// Room size factor
    pub size: f32,
}

impl Default for ReverbSettings {
    fn default() -> Self {
        Self {
            decay: 1.5,
            pre_delay: 20.0,
            damping: 0.5,
            mix: 0.3,
            size: 1.0,
        }
    }
}

/// Room acoustic properties
#[derive(Debug, Clone)]
pub struct RoomAcoustics {
    /// Room dimensions (meters): width, height, depth
    pub dimensions: [f32; 3],
    /// Room size (characteristic dimension)
    pub size: f32,
    /// RT60 reverberation time (seconds)
    pub rt60: f32,
    /// Surface absorption coefficient (0-1)
    pub absorption: f32,
    /// Wall materials
    pub materials: RoomMaterials,
}

impl RoomAcoustics {
    pub fn new(width: f32, height: f32, depth: f32) -> Self {
        let volume = width * height * depth;
        let surface_area = 2.0 * (width * height + height * depth + depth * width);
        
        // Sabine formula: RT60 = 0.161 * V / (A * alpha)
        // Assuming average absorption of 0.2
        let rt60 = 0.161 * volume / (surface_area * 0.2);
        
        Self {
            dimensions: [width, height, depth],
            size: (volume).powf(1.0 / 3.0),
            rt60,
            absorption: 0.2,
            materials: RoomMaterials::default(),
        }
    }
    
    pub fn from_scene_bounds(min: [f32; 3], max: [f32; 3]) -> Self {
        let width = max[0] - min[0];
        let height = max[1] - min[1];
        let depth = max[2] - min[2];
        Self::new(width, height, depth)
    }
    
    /// Calculate RT60 from materials
    pub fn calculate_rt60(&mut self) {
        let [w, h, d] = self.dimensions;
        let volume = w * h * d;
        let surface_area = 2.0 * (w * h + h * d + d * w);
        
        // Weighted average absorption
        let avg_absorption = (
            self.materials.floor * w * d +
            self.materials.ceiling * w * d +
            self.materials.walls * 2.0 * (w * h + h * d)
        ) / surface_area;
        
        self.absorption = avg_absorption;
        self.rt60 = 0.161 * volume / (surface_area * avg_absorption.max(0.01));
    }
}

impl Default for RoomAcoustics {
    fn default() -> Self {
        Self::new(5.0, 2.5, 4.0) // Typical room
    }
}

/// Material absorption coefficients
#[derive(Debug, Clone)]
pub struct RoomMaterials {
    /// Floor absorption
    pub floor: f32,
    /// Ceiling absorption
    pub ceiling: f32,
    /// Wall absorption (average)
    pub walls: f32,
}

impl Default for RoomMaterials {
    fn default() -> Self {
        Self {
            floor: 0.1,   // Hard floor
            ceiling: 0.3, // Acoustic tiles
            walls: 0.2,   // Painted drywall
        }
    }
}

impl RoomMaterials {
    /// Carpeted room
    pub fn carpeted() -> Self {
        Self {
            floor: 0.6,
            ceiling: 0.3,
            walls: 0.2,
        }
    }
    
    /// Acoustic treatment
    pub fn treated() -> Self {
        Self {
            floor: 0.3,
            ceiling: 0.8,
            walls: 0.6,
        }
    }
    
    /// Reflective (hard surfaces)
    pub fn reflective() -> Self {
        Self {
            floor: 0.05,
            ceiling: 0.05,
            walls: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reverb_creation() {
        let reverb = ReverbProcessor::new(48000, 512);
        assert_eq!(*reverb.preset(), ReverbPreset::Room);
    }
    
    #[test]
    fn test_reverb_process() {
        let mut reverb = ReverbProcessor::new(48000, 512);
        reverb.set_wet_mix(0.5);
        
        // Buffer needs to be large enough for delays (~1700 samples at 48kHz)
        let mut buffer = vec![0.0f32; 8192]; // 4096 stereo frames
        
        // Add impulse at the start
        for i in 0..10 {
            buffer[i * 2] = 0.8;
            buffer[i * 2 + 1] = 0.8;
        }
        
        reverb.process(&mut buffer);
        
        // Check for energy after the initial input (well past delay lines)
        let tail_energy: f32 = buffer[4000..].iter().map(|s| s.powi(2)).sum();
        assert!(tail_energy > 0.0, "Reverb should produce some tail energy, got {}", tail_energy);
    }
    
    #[test]
    fn test_reverb_preset() {
        let mut reverb = ReverbProcessor::new(48000, 512);
        
        reverb.set_preset(ReverbPreset::Cathedral);
        assert_eq!(*reverb.preset(), ReverbPreset::Cathedral);
        
        // Cathedral should have high wet mix
        assert!(reverb.wet_mix > 0.4);
    }
    
    #[test]
    fn test_room_acoustics() {
        let room = RoomAcoustics::new(5.0, 2.5, 4.0);
        
        assert!(room.rt60 > 0.0);
        assert!(room.size > 0.0);
    }
    
    #[test]
    fn test_room_acoustics_rt60() {
        let mut room = RoomAcoustics::new(10.0, 5.0, 8.0);
        room.materials = RoomMaterials::treated();
        room.calculate_rt60();
        
        // Treated room should have shorter RT60
        let untreated = RoomAcoustics::new(10.0, 5.0, 8.0);
        assert!(room.rt60 < untreated.rt60);
    }
    
    #[test]
    fn test_reverb_reset() {
        let mut reverb = ReverbProcessor::new(48000, 512);
        
        // Process some audio
        let mut buffer = vec![0.5f32; 1024];
        reverb.process(&mut buffer);
        
        // Reset
        reverb.reset();
        
        // Process silence
        let mut silent = vec![0.0f32; 1024];
        reverb.process(&mut silent);
        
        // Should be mostly silent after reset
        let energy: f32 = silent.iter().map(|s| s.powi(2)).sum();
        assert!(energy < 0.01);
    }
    
    #[test]
    fn test_wet_mix() {
        let mut reverb = ReverbProcessor::new(48000, 512);
        
        reverb.set_wet_mix(0.7);
        assert!((reverb.wet_mix - 0.7).abs() < 0.001);
        
        reverb.set_wet_mix(2.0); // Should clamp
        assert!((reverb.wet_mix - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_reverb_settings() {
        let settings = ReverbSettings::default();
        assert!(settings.decay > 0.0);
        assert!(settings.mix > 0.0);
    }
}
