// Kāraṇa OS - Audio Effects Module
// DSP effects for audio processing

use std::collections::VecDeque;
use std::f32::consts::PI;

use super::capture::AudioFrame;

/// Audio effect trait
pub trait AudioEffect: Send + Sync {
    /// Process audio frame
    fn process(&mut self, input: &AudioFrame) -> AudioFrame;
    
    /// Reset effect state
    fn reset(&mut self);
    
    /// Get effect name
    fn name(&self) -> &str;
    
    /// Set parameter
    fn set_parameter(&mut self, name: &str, value: f32) -> bool;
    
    /// Get parameter
    fn get_parameter(&self, name: &str) -> Option<f32>;
    
    /// Bypass state
    fn is_bypassed(&self) -> bool;
    
    /// Set bypass
    fn set_bypassed(&mut self, bypassed: bool);
}

/// Base effect state
#[derive(Debug)]
struct EffectBase {
    /// Effect name
    name: String,
    /// Sample rate
    sample_rate: u32,
    /// Bypass flag
    bypassed: bool,
}

/// Gain/Volume effect
#[derive(Debug)]
pub struct GainEffect {
    base: EffectBase,
    /// Gain amount (linear)
    gain: f32,
}

impl GainEffect {
    /// Create new gain effect
    pub fn new(sample_rate: u32) -> Self {
        Self {
            base: EffectBase {
                name: "Gain".to_string(),
                sample_rate,
                bypassed: false,
            },
            gain: 1.0,
        }
    }

    /// Set gain in dB
    pub fn set_gain_db(&mut self, db: f32) {
        self.gain = 10.0f32.powf(db / 20.0);
    }

    /// Get gain in dB
    pub fn gain_db(&self) -> f32 {
        20.0 * self.gain.log10()
    }
}

impl AudioEffect for GainEffect {
    fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        if self.base.bypassed {
            return input.clone();
        }

        let mut output = input.clone();
        for sample in &mut output.data {
            *sample *= self.gain;
        }
        output
    }

    fn reset(&mut self) {}

    fn name(&self) -> &str {
        &self.base.name
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "gain" => {
                self.gain = value;
                true
            }
            "gain_db" => {
                self.set_gain_db(value);
                true
            }
            _ => false,
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            "gain" => Some(self.gain),
            "gain_db" => Some(self.gain_db()),
            _ => None,
        }
    }

    fn is_bypassed(&self) -> bool {
        self.base.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.base.bypassed = bypassed;
    }
}

/// High-pass filter
#[derive(Debug)]
pub struct HighPassFilter {
    base: EffectBase,
    /// Cutoff frequency in Hz
    cutoff: f32,
    /// Filter coefficient
    alpha: f32,
    /// Previous samples per channel
    prev: Vec<f32>,
    /// Previous input per channel
    prev_in: Vec<f32>,
}

impl HighPassFilter {
    /// Create new high-pass filter
    pub fn new(sample_rate: u32, cutoff: f32) -> Self {
        let mut filter = Self {
            base: EffectBase {
                name: "High Pass".to_string(),
                sample_rate,
                bypassed: false,
            },
            cutoff,
            alpha: 0.0,
            prev: vec![0.0; 2],
            prev_in: vec![0.0; 2],
        };
        filter.update_coefficients();
        filter
    }

    /// Set cutoff frequency
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        let rc = 1.0 / (2.0 * PI * self.cutoff);
        let dt = 1.0 / self.base.sample_rate as f32;
        self.alpha = rc / (rc + dt);
    }
}

impl AudioEffect for HighPassFilter {
    fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        if self.base.bypassed {
            return input.clone();
        }

        // Ensure we have enough prev samples
        let channels = input.channels as usize;
        while self.prev.len() < channels {
            self.prev.push(0.0);
            self.prev_in.push(0.0);
        }

        let mut output = input.clone();
        let samples = input.samples_per_channel();

        for i in 0..samples {
            for ch in 0..channels {
                let idx = i * channels + ch;
                let x = input.data[idx];
                
                // High-pass filter: y[n] = alpha * (y[n-1] + x[n] - x[n-1])
                let y = self.alpha * (self.prev[ch] + x - self.prev_in[ch]);
                
                output.data[idx] = y;
                self.prev[ch] = y;
                self.prev_in[ch] = x;
            }
        }

        output
    }

    fn reset(&mut self) {
        self.prev.fill(0.0);
        self.prev_in.fill(0.0);
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "cutoff" => {
                self.set_cutoff(value);
                true
            }
            _ => false,
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            "cutoff" => Some(self.cutoff),
            _ => None,
        }
    }

    fn is_bypassed(&self) -> bool {
        self.base.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.base.bypassed = bypassed;
    }
}

/// Low-pass filter
#[derive(Debug)]
pub struct LowPassFilter {
    base: EffectBase,
    /// Cutoff frequency in Hz
    cutoff: f32,
    /// Filter coefficient
    alpha: f32,
    /// Previous samples per channel
    prev: Vec<f32>,
}

impl LowPassFilter {
    /// Create new low-pass filter
    pub fn new(sample_rate: u32, cutoff: f32) -> Self {
        let mut filter = Self {
            base: EffectBase {
                name: "Low Pass".to_string(),
                sample_rate,
                bypassed: false,
            },
            cutoff,
            alpha: 0.0,
            prev: vec![0.0; 2],
        };
        filter.update_coefficients();
        filter
    }

    /// Set cutoff frequency
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff;
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        let rc = 1.0 / (2.0 * PI * self.cutoff);
        let dt = 1.0 / self.base.sample_rate as f32;
        self.alpha = dt / (rc + dt);
    }
}

impl AudioEffect for LowPassFilter {
    fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        if self.base.bypassed {
            return input.clone();
        }

        let channels = input.channels as usize;
        while self.prev.len() < channels {
            self.prev.push(0.0);
        }

        let mut output = input.clone();
        let samples = input.samples_per_channel();

        for i in 0..samples {
            for ch in 0..channels {
                let idx = i * channels + ch;
                let x = input.data[idx];
                
                // Low-pass filter: y[n] = alpha * x[n] + (1 - alpha) * y[n-1]
                let y = self.alpha * x + (1.0 - self.alpha) * self.prev[ch];
                
                output.data[idx] = y;
                self.prev[ch] = y;
            }
        }

        output
    }

    fn reset(&mut self) {
        self.prev.fill(0.0);
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "cutoff" => {
                self.set_cutoff(value);
                true
            }
            _ => false,
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            "cutoff" => Some(self.cutoff),
            _ => None,
        }
    }

    fn is_bypassed(&self) -> bool {
        self.base.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.base.bypassed = bypassed;
    }
}

/// Parametric EQ band
#[derive(Debug, Clone)]
pub struct EQBand {
    /// Center frequency
    pub frequency: f32,
    /// Gain in dB
    pub gain: f32,
    /// Q factor (bandwidth)
    pub q: f32,
    /// Band type
    pub band_type: EQBandType,
}

/// EQ band type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EQBandType {
    /// Peaking/Bell filter
    Peak,
    /// Low shelf
    LowShelf,
    /// High shelf
    HighShelf,
    /// Notch filter
    Notch,
}

/// Parametric equalizer
#[derive(Debug)]
pub struct ParametricEQ {
    base: EffectBase,
    /// EQ bands
    bands: Vec<EQBand>,
    /// Biquad filter states per band per channel
    states: Vec<Vec<BiquadState>>,
}

/// Biquad filter state
#[derive(Debug, Clone, Default)]
struct BiquadState {
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
}

impl ParametricEQ {
    /// Create new parametric EQ
    pub fn new(sample_rate: u32) -> Self {
        Self {
            base: EffectBase {
                name: "Parametric EQ".to_string(),
                sample_rate,
                bypassed: false,
            },
            bands: Vec::new(),
            states: Vec::new(),
        }
    }

    /// Add EQ band
    pub fn add_band(&mut self, band: EQBand, channels: u8) {
        self.bands.push(band);
        self.states.push(vec![BiquadState::default(); channels as usize]);
        self.update_band_coefficients(self.bands.len() - 1);
    }

    /// Update band parameters
    pub fn update_band(&mut self, index: usize, band: EQBand) {
        if index < self.bands.len() {
            self.bands[index] = band;
            self.update_band_coefficients(index);
        }
    }

    /// Remove band
    pub fn remove_band(&mut self, index: usize) {
        if index < self.bands.len() {
            self.bands.remove(index);
            self.states.remove(index);
        }
    }

    fn update_band_coefficients(&mut self, index: usize) {
        let band = &self.bands[index];
        let fs = self.base.sample_rate as f32;
        
        let omega = 2.0 * PI * band.frequency / fs;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * band.q);
        let a = 10.0f32.powf(band.gain / 40.0);

        let (b0, b1, b2, a0, a1, a2) = match band.band_type {
            EQBandType::Peak => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha / a;
                (b0, b1, b2, a0, a1, a2)
            }
            EQBandType::LowShelf => {
                let sqrt_a = a.sqrt();
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha);
                let a0 = (a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
                let a2 = (a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            EQBandType::HighShelf => {
                let sqrt_a = a.sqrt();
                let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha);
                let a0 = (a + 1.0) - (a - 1.0) * cos_omega + 2.0 * sqrt_a * alpha;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
                let a2 = (a + 1.0) - (a - 1.0) * cos_omega - 2.0 * sqrt_a * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            EQBandType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        // Normalize coefficients
        for state in &mut self.states[index] {
            state.b0 = b0 / a0;
            state.b1 = b1 / a0;
            state.b2 = b2 / a0;
            state.a1 = a1 / a0;
            state.a2 = a2 / a0;
        }
    }
}

impl AudioEffect for ParametricEQ {
    fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        if self.base.bypassed || self.bands.is_empty() {
            return input.clone();
        }

        let mut output = input.clone();
        let channels = input.channels as usize;
        let samples = input.samples_per_channel();

        // Process each band
        for (band_idx, states) in self.states.iter_mut().enumerate() {
            // Ensure we have states for all channels
            while states.len() < channels {
                let template = states.first().cloned().unwrap_or_default();
                states.push(template);
            }

            for i in 0..samples {
                for ch in 0..channels {
                    let idx = i * channels + ch;
                    let state = &mut states[ch];
                    let x = output.data[idx];

                    // Direct Form I biquad
                    let y = state.b0 * x + state.b1 * state.x1 + state.b2 * state.x2
                        - state.a1 * state.y1 - state.a2 * state.y2;

                    state.x2 = state.x1;
                    state.x1 = x;
                    state.y2 = state.y1;
                    state.y1 = y;

                    output.data[idx] = y;
                }
            }
        }

        output
    }

    fn reset(&mut self) {
        for states in &mut self.states {
            for state in states {
                state.x1 = 0.0;
                state.x2 = 0.0;
                state.y1 = 0.0;
                state.y2 = 0.0;
            }
        }
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        // Parse band parameters like "band0_frequency"
        if let Some(rest) = name.strip_prefix("band") {
            if let Some((idx_str, param)) = rest.split_once('_') {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx < self.bands.len() {
                        match param {
                            "frequency" => {
                                self.bands[idx].frequency = value;
                                self.update_band_coefficients(idx);
                                return true;
                            }
                            "gain" => {
                                self.bands[idx].gain = value;
                                self.update_band_coefficients(idx);
                                return true;
                            }
                            "q" => {
                                self.bands[idx].q = value;
                                self.update_band_coefficients(idx);
                                return true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        false
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        if let Some(rest) = name.strip_prefix("band") {
            if let Some((idx_str, param)) = rest.split_once('_') {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    if idx < self.bands.len() {
                        return match param {
                            "frequency" => Some(self.bands[idx].frequency),
                            "gain" => Some(self.bands[idx].gain),
                            "q" => Some(self.bands[idx].q),
                            _ => None,
                        };
                    }
                }
            }
        }
        None
    }

    fn is_bypassed(&self) -> bool {
        self.base.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.base.bypassed = bypassed;
    }
}

/// Compressor/Limiter
#[derive(Debug)]
pub struct Compressor {
    base: EffectBase,
    /// Threshold in dB
    threshold: f32,
    /// Ratio (e.g., 4:1 = 4.0)
    ratio: f32,
    /// Attack time in ms
    attack: f32,
    /// Release time in ms
    release: f32,
    /// Knee width in dB
    knee: f32,
    /// Makeup gain in dB
    makeup: f32,
    /// Current envelope per channel
    envelope: Vec<f32>,
    /// Attack coefficient
    attack_coef: f32,
    /// Release coefficient
    release_coef: f32,
}

impl Compressor {
    /// Create new compressor
    pub fn new(sample_rate: u32) -> Self {
        let mut comp = Self {
            base: EffectBase {
                name: "Compressor".to_string(),
                sample_rate,
                bypassed: false,
            },
            threshold: -20.0,
            ratio: 4.0,
            attack: 10.0,
            release: 100.0,
            knee: 6.0,
            makeup: 0.0,
            envelope: vec![0.0; 2],
            attack_coef: 0.0,
            release_coef: 0.0,
        };
        comp.update_coefficients();
        comp
    }

    /// Set threshold
    pub fn set_threshold(&mut self, db: f32) {
        self.threshold = db;
    }

    /// Set ratio
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.max(1.0);
    }

    /// Set attack time
    pub fn set_attack(&mut self, ms: f32) {
        self.attack = ms;
        self.update_coefficients();
    }

    /// Set release time
    pub fn set_release(&mut self, ms: f32) {
        self.release = ms;
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        let fs = self.base.sample_rate as f32;
        self.attack_coef = (-1.0 / (fs * self.attack / 1000.0)).exp();
        self.release_coef = (-1.0 / (fs * self.release / 1000.0)).exp();
    }

    /// Calculate gain reduction
    fn compute_gain(&self, level_db: f32) -> f32 {
        let knee_start = self.threshold - self.knee / 2.0;
        let knee_end = self.threshold + self.knee / 2.0;

        if level_db <= knee_start {
            0.0 // No compression
        } else if level_db >= knee_end {
            // Full compression
            (level_db - self.threshold) * (1.0 - 1.0 / self.ratio)
        } else {
            // Soft knee region
            let knee_factor = (level_db - knee_start) / self.knee;
            let gain_reduction = (level_db - self.threshold) * (1.0 - 1.0 / self.ratio);
            gain_reduction * knee_factor * knee_factor
        }
    }
}

impl AudioEffect for Compressor {
    fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        if self.base.bypassed {
            return input.clone();
        }

        let channels = input.channels as usize;
        while self.envelope.len() < channels {
            self.envelope.push(0.0);
        }

        let mut output = input.clone();
        let samples = input.samples_per_channel();
        let makeup_linear = 10.0f32.powf(self.makeup / 20.0);

        for i in 0..samples {
            // Find peak across channels for this sample
            let mut peak = 0.0f32;
            for ch in 0..channels {
                let idx = i * channels + ch;
                peak = peak.max(input.data[idx].abs());
            }

            // Convert to dB
            let level_db = if peak > 0.0 { 20.0 * peak.log10() } else { -100.0 };

            // Envelope follower (use channel 0 for simplicity)
            let coef = if level_db > self.envelope[0] {
                self.attack_coef
            } else {
                self.release_coef
            };
            self.envelope[0] = level_db + coef * (self.envelope[0] - level_db);

            // Calculate gain reduction
            let gain_reduction = self.compute_gain(self.envelope[0]);
            let gain_linear = 10.0f32.powf(-gain_reduction / 20.0) * makeup_linear;

            // Apply gain to all channels
            for ch in 0..channels {
                let idx = i * channels + ch;
                output.data[idx] *= gain_linear;
            }
        }

        output
    }

    fn reset(&mut self) {
        self.envelope.fill(0.0);
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "threshold" => { self.threshold = value; true }
            "ratio" => { self.set_ratio(value); true }
            "attack" => { self.set_attack(value); true }
            "release" => { self.set_release(value); true }
            "knee" => { self.knee = value; true }
            "makeup" => { self.makeup = value; true }
            _ => false,
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            "threshold" => Some(self.threshold),
            "ratio" => Some(self.ratio),
            "attack" => Some(self.attack),
            "release" => Some(self.release),
            "knee" => Some(self.knee),
            "makeup" => Some(self.makeup),
            _ => None,
        }
    }

    fn is_bypassed(&self) -> bool {
        self.base.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.base.bypassed = bypassed;
    }
}

/// Delay effect
#[derive(Debug)]
pub struct DelayEffect {
    base: EffectBase,
    /// Delay time in ms
    delay_time: f32,
    /// Feedback amount (0.0 to 1.0)
    feedback: f32,
    /// Wet/dry mix (0.0 to 1.0)
    mix: f32,
    /// Delay buffers per channel
    buffers: Vec<VecDeque<f32>>,
    /// Delay in samples
    delay_samples: usize,
}

impl DelayEffect {
    /// Create new delay effect
    pub fn new(sample_rate: u32, max_delay_ms: f32) -> Self {
        let max_samples = (sample_rate as f32 * max_delay_ms / 1000.0) as usize;
        Self {
            base: EffectBase {
                name: "Delay".to_string(),
                sample_rate,
                bypassed: false,
            },
            delay_time: 250.0,
            feedback: 0.5,
            mix: 0.5,
            buffers: vec![VecDeque::from(vec![0.0; max_samples]); 2],
            delay_samples: (sample_rate as f32 * 0.25) as usize,
        }
    }

    /// Set delay time
    pub fn set_delay_time(&mut self, ms: f32) {
        self.delay_time = ms;
        self.delay_samples = (self.base.sample_rate as f32 * ms / 1000.0) as usize;
        self.delay_samples = self.delay_samples.min(self.buffers[0].len());
    }
}

impl AudioEffect for DelayEffect {
    fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        if self.base.bypassed {
            return input.clone();
        }

        let channels = input.channels as usize;
        while self.buffers.len() < channels {
            self.buffers.push(VecDeque::from(vec![0.0; self.buffers[0].len()]));
        }

        let mut output = input.clone();
        let samples = input.samples_per_channel();

        for i in 0..samples {
            for ch in 0..channels {
                let idx = i * channels + ch;
                let dry = input.data[idx];
                
                // Read from delay line
                let buf_len = self.buffers[ch].len();
                let read_pos = if buf_len > self.delay_samples {
                    buf_len - self.delay_samples
                } else {
                    0
                };
                let delayed = self.buffers[ch].get(read_pos).copied().unwrap_or(0.0);

                // Write to delay line with feedback
                self.buffers[ch].pop_front();
                self.buffers[ch].push_back(dry + delayed * self.feedback);

                // Mix
                output.data[idx] = dry * (1.0 - self.mix) + delayed * self.mix;
            }
        }

        output
    }

    fn reset(&mut self) {
        for buffer in &mut self.buffers {
            buffer.iter_mut().for_each(|s| *s = 0.0);
        }
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "delay_time" => { self.set_delay_time(value); true }
            "feedback" => { self.feedback = value.clamp(0.0, 0.99); true }
            "mix" => { self.mix = value.clamp(0.0, 1.0); true }
            _ => false,
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            "delay_time" => Some(self.delay_time),
            "feedback" => Some(self.feedback),
            "mix" => Some(self.mix),
            _ => None,
        }
    }

    fn is_bypassed(&self) -> bool {
        self.base.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.base.bypassed = bypassed;
    }
}

/// Noise gate
#[derive(Debug)]
pub struct NoiseGate {
    base: EffectBase,
    /// Threshold in dB
    threshold: f32,
    /// Attack time in ms
    attack: f32,
    /// Hold time in ms
    hold: f32,
    /// Release time in ms
    release: f32,
    /// Range (max attenuation in dB)
    range: f32,
    /// Current gate state (0.0 = closed, 1.0 = open)
    gate: f32,
    /// Hold counter
    hold_counter: usize,
    /// Attack/release coefficients
    attack_coef: f32,
    release_coef: f32,
    hold_samples: usize,
}

impl NoiseGate {
    /// Create new noise gate
    pub fn new(sample_rate: u32) -> Self {
        let mut gate = Self {
            base: EffectBase {
                name: "Noise Gate".to_string(),
                sample_rate,
                bypassed: false,
            },
            threshold: -40.0,
            attack: 1.0,
            hold: 50.0,
            release: 100.0,
            range: -80.0,
            gate: 0.0,
            hold_counter: 0,
            attack_coef: 0.0,
            release_coef: 0.0,
            hold_samples: 0,
        };
        gate.update_coefficients();
        gate
    }

    fn update_coefficients(&mut self) {
        let fs = self.base.sample_rate as f32;
        self.attack_coef = (-1.0 / (fs * self.attack / 1000.0)).exp();
        self.release_coef = (-1.0 / (fs * self.release / 1000.0)).exp();
        self.hold_samples = (fs * self.hold / 1000.0) as usize;
    }
}

impl AudioEffect for NoiseGate {
    fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        if self.base.bypassed {
            return input.clone();
        }

        let mut output = input.clone();
        let channels = input.channels as usize;
        let samples = input.samples_per_channel();
        let range_linear = 10.0f32.powf(self.range / 20.0);

        for i in 0..samples {
            // Find peak across channels
            let mut peak = 0.0f32;
            for ch in 0..channels {
                let idx = i * channels + ch;
                peak = peak.max(input.data[idx].abs());
            }

            let level_db = if peak > 0.0 { 20.0 * peak.log10() } else { -100.0 };

            // Gate logic
            let target = if level_db > self.threshold {
                self.hold_counter = self.hold_samples;
                1.0
            } else if self.hold_counter > 0 {
                self.hold_counter -= 1;
                1.0
            } else {
                0.0
            };

            // Smooth gate transitions
            let coef = if target > self.gate { self.attack_coef } else { self.release_coef };
            self.gate = target + coef * (self.gate - target);

            // Calculate gain
            let gain = range_linear + (1.0 - range_linear) * self.gate;

            // Apply to all channels
            for ch in 0..channels {
                let idx = i * channels + ch;
                output.data[idx] *= gain;
            }
        }

        output
    }

    fn reset(&mut self) {
        self.gate = 0.0;
        self.hold_counter = 0;
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn set_parameter(&mut self, name: &str, value: f32) -> bool {
        match name {
            "threshold" => { self.threshold = value; true }
            "attack" => { self.attack = value; self.update_coefficients(); true }
            "hold" => { self.hold = value; self.update_coefficients(); true }
            "release" => { self.release = value; self.update_coefficients(); true }
            "range" => { self.range = value; true }
            _ => false,
        }
    }

    fn get_parameter(&self, name: &str) -> Option<f32> {
        match name {
            "threshold" => Some(self.threshold),
            "attack" => Some(self.attack),
            "hold" => Some(self.hold),
            "release" => Some(self.release),
            "range" => Some(self.range),
            _ => None,
        }
    }

    fn is_bypassed(&self) -> bool {
        self.base.bypassed
    }

    fn set_bypassed(&mut self, bypassed: bool) {
        self.base.bypassed = bypassed;
    }
}

/// Effects chain
#[derive(Debug)]
pub struct EffectsChain {
    /// Effects in order
    effects: Vec<Box<dyn AudioEffect>>,
}

impl EffectsChain {
    /// Create new effects chain
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add effect to chain
    pub fn add(&mut self, effect: Box<dyn AudioEffect>) {
        self.effects.push(effect);
    }

    /// Insert effect at position
    pub fn insert(&mut self, index: usize, effect: Box<dyn AudioEffect>) {
        if index <= self.effects.len() {
            self.effects.insert(index, effect);
        }
    }

    /// Remove effect by index
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn AudioEffect>> {
        if index < self.effects.len() {
            Some(self.effects.remove(index))
        } else {
            None
        }
    }

    /// Process through entire chain
    pub fn process(&mut self, input: &AudioFrame) -> AudioFrame {
        let mut output = input.clone();
        for effect in &mut self.effects {
            output = effect.process(&output);
        }
        output
    }

    /// Reset all effects
    pub fn reset(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
    }

    /// Get effect count
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

impl Default for EffectsChain {
    fn default() -> Self {
        Self::new()
    }
}

// Can't implement Debug for trait objects with fmt::Debug
impl std::fmt::Debug for Box<dyn AudioEffect> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioEffect({})", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frame() -> AudioFrame {
        AudioFrame::new(vec![0.5f32; 960 * 2], 2, 48000)
    }

    #[test]
    fn test_gain_effect() {
        let mut gain = GainEffect::new(48000);
        gain.gain = 2.0;

        let input = test_frame();
        let output = gain.process(&input);

        assert!((output.data[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_gain_db() {
        let mut gain = GainEffect::new(48000);
        gain.set_gain_db(-6.0);

        assert!((gain.gain - 0.5).abs() < 0.1);
        assert!((gain.gain_db() - (-6.0)).abs() < 0.1);
    }

    #[test]
    fn test_highpass_filter() {
        let mut hpf = HighPassFilter::new(48000, 100.0);
        
        // Low frequency should be attenuated
        let input = test_frame();
        let output = hpf.process(&input);

        // Just verify it runs without crashing
        assert_eq!(output.data.len(), input.data.len());
    }

    #[test]
    fn test_lowpass_filter() {
        let mut lpf = LowPassFilter::new(48000, 1000.0);
        
        let input = test_frame();
        let output = lpf.process(&input);

        assert_eq!(output.data.len(), input.data.len());
    }

    #[test]
    fn test_parametric_eq() {
        let mut eq = ParametricEQ::new(48000);
        eq.add_band(EQBand {
            frequency: 1000.0,
            gain: 6.0,
            q: 1.0,
            band_type: EQBandType::Peak,
        }, 2);

        let input = test_frame();
        let output = eq.process(&input);

        assert_eq!(output.data.len(), input.data.len());
    }

    #[test]
    fn test_compressor() {
        let mut comp = Compressor::new(48000);
        comp.set_threshold(-20.0);
        comp.set_ratio(4.0);

        let input = test_frame();
        let output = comp.process(&input);

        // Output should be compressed (lower than input for loud signals)
        assert_eq!(output.data.len(), input.data.len());
    }

    #[test]
    fn test_delay_effect() {
        let mut delay = DelayEffect::new(48000, 1000.0);
        delay.set_delay_time(100.0);
        delay.mix = 0.5;
        delay.feedback = 0.3;

        let input = test_frame();
        let output = delay.process(&input);

        assert_eq!(output.data.len(), input.data.len());
    }

    #[test]
    fn test_noise_gate() {
        let mut gate = NoiseGate::new(48000);
        gate.threshold = -30.0;

        // Quiet input should be gated
        let quiet = AudioFrame::new(vec![0.001f32; 960 * 2], 2, 48000);
        let output = gate.process(&quiet);

        // Should be significantly attenuated
        assert!(output.rms() < quiet.rms());
    }

    #[test]
    fn test_effects_chain() {
        let mut chain = EffectsChain::new();
        chain.add(Box::new(GainEffect::new(48000)));
        chain.add(Box::new(LowPassFilter::new(48000, 5000.0)));

        assert_eq!(chain.len(), 2);

        let input = test_frame();
        let output = chain.process(&input);

        assert_eq!(output.data.len(), input.data.len());
    }

    #[test]
    fn test_effect_bypass() {
        let mut gain = GainEffect::new(48000);
        gain.gain = 2.0;
        gain.set_bypassed(true);

        let input = test_frame();
        let output = gain.process(&input);

        // Should be unchanged when bypassed
        assert_eq!(output.data[0], input.data[0]);
    }

    #[test]
    fn test_effect_parameters() {
        let mut comp = Compressor::new(48000);
        
        assert!(comp.set_parameter("threshold", -30.0));
        assert_eq!(comp.get_parameter("threshold"), Some(-30.0));

        assert!(comp.set_parameter("ratio", 8.0));
        assert_eq!(comp.get_parameter("ratio"), Some(8.0));

        assert!(!comp.set_parameter("invalid", 0.0));
        assert_eq!(comp.get_parameter("invalid"), None);
    }

    #[test]
    fn test_eq_band_types() {
        let mut eq = ParametricEQ::new(48000);
        
        eq.add_band(EQBand {
            frequency: 100.0,
            gain: 3.0,
            q: 0.7,
            band_type: EQBandType::LowShelf,
        }, 2);

        eq.add_band(EQBand {
            frequency: 10000.0,
            gain: -3.0,
            q: 0.7,
            band_type: EQBandType::HighShelf,
        }, 2);

        let input = test_frame();
        let output = eq.process(&input);
        assert_eq!(output.data.len(), input.data.len());
    }
}
