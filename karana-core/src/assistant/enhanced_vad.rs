// Kāraṇa OS - Enhanced Voice Activity Detection
// ML-based VAD for better accuracy in noisy environments

use anyhow::{Result, anyhow};

/// Enhanced VAD configuration
#[derive(Debug, Clone)]
pub struct EnhancedVadConfig {
    /// Sample rate (must match audio input)
    pub sample_rate: u32,
    /// Minimum speech duration (ms)
    pub min_speech_duration_ms: u32,
    /// Maximum speech duration (ms)
    pub max_speech_duration_ms: u32,
    /// Silence duration before ending speech (ms)
    pub min_silence_duration_ms: u32,
    /// Speech probability threshold (0.0-1.0)
    pub speech_threshold: f32,
    /// Use energy-based fallback if ML VAD unavailable
    pub fallback_energy_threshold: f32,
}

impl Default for EnhancedVadConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            min_speech_duration_ms: 250,
            max_speech_duration_ms: 30000, // 30 seconds max
            min_silence_duration_ms: 500,
            speech_threshold: 0.5,
            fallback_energy_threshold: 0.01,
        }
    }
}

/// VAD detection result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VadDecision {
    /// Speech detected
    Speech,
    /// Silence detected
    Silence,
    /// Uncertain (borderline)
    Uncertain,
}

/// Enhanced Voice Activity Detector
pub struct EnhancedVad {
    config: EnhancedVadConfig,
    
    // State tracking
    is_speech_active: bool,
    speech_start_sample: Option<usize>,
    silence_start_sample: Option<usize>,
    total_samples_processed: usize,
    
    // Energy-based detection (fallback)
    smoothed_energy: f32,
    energy_alpha: f32, // Smoothing factor
    
    // ML-based detection (if available)
    #[cfg(feature = "webrtc-vad")]
    webrtc_vad: Option<webrtc_vad::Vad>,
}

impl EnhancedVad {
    /// Create new enhanced VAD
    pub fn new(config: EnhancedVadConfig) -> Result<Self> {
        #[cfg(feature = "webrtc-vad")]
        let webrtc_vad = match webrtc_vad::Vad::new_with_rate_and_mode(
            Self::get_vad_rate(config.sample_rate),
            webrtc_vad::VadMode::Quality,
        ) {
            Ok(vad) => Some(vad),
            Err(e) => {
                log::warn!("[VAD] WebRTC VAD init failed: {}, using fallback", e);
                None
            }
        };

        Ok(Self {
            config,
            is_speech_active: false,
            speech_start_sample: None,
            silence_start_sample: None,
            total_samples_processed: 0,
            smoothed_energy: 0.0,
            energy_alpha: 0.1,
            #[cfg(feature = "webrtc-vad")]
            webrtc_vad,
        })
    }

    /// Process audio samples and detect voice activity
    pub fn process_chunk(&mut self, samples: &[f32]) -> VadDecision {
        self.total_samples_processed += samples.len();

        // Try ML-based VAD first
        #[cfg(feature = "webrtc-vad")]
        if let Some(ref mut vad) = self.webrtc_vad {
            if let Ok(decision) = self.process_with_webrtc(vad, samples) {
                return decision;
            }
        }

        // Fallback to energy-based VAD
        self.process_with_energy(samples)
    }

    /// Process with WebRTC VAD (when available)
    #[cfg(feature = "webrtc-vad")]
    fn process_with_webrtc(&mut self, vad: &mut webrtc_vad::Vad, samples: &[f32]) -> Result<VadDecision> {
        // WebRTC VAD expects i16 samples
        let i16_samples: Vec<i16> = samples.iter()
            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();

        // WebRTC VAD requires specific frame sizes (10, 20, or 30ms)
        // For 16kHz: 160, 320, or 480 samples
        let frame_size = 320; // 20ms at 16kHz
        
        if i16_samples.len() < frame_size {
            return Ok(VadDecision::Uncertain);
        }

        // Process frame
        let frame = &i16_samples[0..frame_size];
        let is_speech = vad.is_voice_segment(frame)?;

        self.update_state(is_speech);

        Ok(if is_speech {
            VadDecision::Speech
        } else {
            VadDecision::Silence
        })
    }

    /// Process with energy-based VAD (fallback)
    fn process_with_energy(&mut self, samples: &[f32]) -> VadDecision {
        // Calculate RMS energy
        let energy = if samples.is_empty() {
            0.0
        } else {
            let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
            (sum_squares / samples.len() as f32).sqrt()
        };

        // Smooth energy with exponential moving average
        self.smoothed_energy = self.energy_alpha * energy + (1.0 - self.energy_alpha) * self.smoothed_energy;

        // Detect speech based on threshold
        let is_speech = self.smoothed_energy > self.config.fallback_energy_threshold;

        self.update_state(is_speech);

        if is_speech {
            VadDecision::Speech
        } else {
            VadDecision::Silence
        }
    }

    /// Update internal state based on detection
    fn update_state(&mut self, is_speech: bool) {
        if is_speech {
            // Speech detected
            if !self.is_speech_active {
                // Speech just started
                self.is_speech_active = true;
                self.speech_start_sample = Some(self.total_samples_processed);
                self.silence_start_sample = None;
                log::debug!("[VAD] Speech started");
            }
        } else {
            // Silence detected
            if self.is_speech_active {
                // Mark silence start
                if self.silence_start_sample.is_none() {
                    self.silence_start_sample = Some(self.total_samples_processed);
                }

                // Check if silence duration exceeded threshold
                if let Some(silence_start) = self.silence_start_sample {
                    let silence_samples = self.total_samples_processed - silence_start;
                    let silence_ms = (silence_samples as f32 / self.config.sample_rate as f32) * 1000.0;

                    if silence_ms >= self.config.min_silence_duration_ms as f32 {
                        // Speech ended
                        self.is_speech_active = false;
                        self.speech_start_sample = None;
                        self.silence_start_sample = None;
                        log::debug!("[VAD] Speech ended after {:.0}ms silence", silence_ms);
                    }
                }
            }
        }
    }

    /// Check if speech is currently active
    pub fn is_speech_active(&self) -> bool {
        self.is_speech_active
    }

    /// Get current energy level (0.0-1.0)
    pub fn get_energy_level(&self) -> f32 {
        self.smoothed_energy.min(1.0)
    }

    /// Check if current utterance is long enough
    pub fn is_utterance_valid(&self) -> bool {
        if let Some(start) = self.speech_start_sample {
            let duration_samples = self.total_samples_processed - start;
            let duration_ms = (duration_samples as f32 / self.config.sample_rate as f32) * 1000.0;
            duration_ms >= self.config.min_speech_duration_ms as f32
        } else {
            false
        }
    }

    /// Reset VAD state
    pub fn reset(&mut self) {
        self.is_speech_active = false;
        self.speech_start_sample = None;
        self.silence_start_sample = None;
        self.total_samples_processed = 0;
        self.smoothed_energy = 0.0;
    }

    /// Get WebRTC VAD sample rate enum
    #[cfg(feature = "webrtc-vad")]
    fn get_vad_rate(sample_rate: u32) -> webrtc_vad::SampleRate {
        match sample_rate {
            8000 => webrtc_vad::SampleRate::Rate8kHz,
            16000 => webrtc_vad::SampleRate::Rate16kHz,
            32000 => webrtc_vad::SampleRate::Rate32kHz,
            48000 => webrtc_vad::SampleRate::Rate48kHz,
            _ => webrtc_vad::SampleRate::Rate16kHz, // Default
        }
    }
}

impl Default for EnhancedVad {
    fn default() -> Self {
        Self::new(EnhancedVadConfig::default()).expect("Failed to create default VAD")
    }
}

/// Simple wrapper for backward compatibility
pub struct SimpleVadWrapper {
    vad: EnhancedVad,
}

impl SimpleVadWrapper {
    pub fn new(energy_threshold: f32, sample_rate: u32) -> Self {
        let config = EnhancedVadConfig {
            sample_rate,
            fallback_energy_threshold: energy_threshold,
            ..Default::default()
        };
        
        Self {
            vad: EnhancedVad::new(config).expect("Failed to create VAD"),
        }
    }

    pub fn process(&mut self, samples: &[f32]) -> bool {
        matches!(self.vad.process_chunk(samples), VadDecision::Speech)
    }

    pub fn is_active(&self) -> bool {
        self.vad.is_speech_active()
    }

    pub fn energy_level(&self) -> f32 {
        self.vad.get_energy_level()
    }

    pub fn reset(&mut self) {
        self.vad.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_vad_creation() {
        let config = EnhancedVadConfig::default();
        let vad = EnhancedVad::new(config);
        assert!(vad.is_ok());
    }

    #[test]
    fn test_energy_detection() {
        let mut vad = EnhancedVad::new(EnhancedVadConfig {
            fallback_energy_threshold: 0.1,
            ..Default::default()
        }).unwrap();

        // Silent samples
        let silence = vec![0.01; 320];
        let decision = vad.process_chunk(&silence);
        assert_eq!(decision, VadDecision::Silence);

        // Loud samples (speech)
        let speech = vec![0.5; 320];
        let decision = vad.process_chunk(&speech);
        assert_eq!(decision, VadDecision::Speech);
    }

    #[test]
    fn test_state_transitions() {
        let mut vad = EnhancedVad::new(EnhancedVadConfig::default()).unwrap();

        assert!(!vad.is_speech_active());

        // Simulate speech
        for _ in 0..10 {
            let samples = vec![0.5; 320];
            vad.process_chunk(&samples);
        }

        assert!(vad.is_speech_active());

        // Simulate silence
        for _ in 0..30 {
            let samples = vec![0.01; 320];
            vad.process_chunk(&samples);
        }

        // Speech should end after min_silence_duration_ms
        assert!(!vad.is_speech_active());
    }

    #[test]
    fn test_simple_wrapper() {
        let mut vad = SimpleVadWrapper::new(0.1, 16000);
        
        let silence = vec![0.01; 320];
        assert!(!vad.process(&silence));

        let speech = vec![0.5; 320];
        assert!(vad.process(&speech));
    }
}
