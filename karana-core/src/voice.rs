//! # Kāraṇa Voice Processing Pipeline
//!
//! Real voice-to-intent processing using Whisper STT.
//!
//! ## Architecture
//! ```
//! Microphone → Resample → VAD → Whisper → Intent → Oracle
//!      ↓                    ↓
//!   16kHz PCM          Voice Activity Detection
//! ```
//!
//! ## Features
//! - Real-time microphone capture via CPAL
//! - WAV file input for testing
//! - Voice activity detection (VAD) for automatic segmentation
//! - Streaming transcription support
//! - Wake word detection ("karana" or custom)

use anyhow::{Result, anyhow};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::collections::VecDeque;

/// Voice activity detection state
#[derive(Debug, Clone)]
pub struct VadState {
    /// Energy threshold for voice detection
    pub energy_threshold: f32,
    /// Minimum frames of silence to end utterance
    pub silence_frames: u32,
    /// Current silence frame count
    pub current_silence: u32,
    /// Is voice currently detected
    pub voice_active: bool,
    /// Smoothed energy level
    pub energy_level: f32,
}

impl Default for VadState {
    fn default() -> Self {
        Self {
            energy_threshold: 0.01, // Adjust based on environment
            silence_frames: 30,      // ~1 second at 30fps
            current_silence: 0,
            voice_active: false,
            energy_level: 0.0,
        }
    }
}

/// Voice pipeline configuration
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Sample rate (Whisper requires 16kHz)
    pub sample_rate: u32,
    /// Wake word to activate listening
    pub wake_word: String,
    /// Enable continuous listening (no wake word needed)
    pub continuous_mode: bool,
    /// Maximum recording duration (seconds)
    pub max_duration_secs: u32,
    /// Energy threshold for VAD
    pub vad_threshold: f32,
    /// Enable noise reduction
    pub noise_reduction: bool,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            wake_word: "karana".to_string(),
            continuous_mode: true,
            max_duration_secs: 30,
            vad_threshold: 0.01,
            noise_reduction: true,
        }
    }
}

/// Voice recording result
#[derive(Debug, Clone)]
pub struct VoiceRecording {
    /// Audio samples (16kHz mono f32)
    pub samples: Vec<f32>,
    /// Duration in seconds
    pub duration_secs: f32,
    /// Peak energy level
    pub peak_energy: f32,
    /// Was wake word detected
    pub wake_word_detected: bool,
}

/// Real-time voice capture and processing
pub struct VoicePipeline {
    config: VoiceConfig,
    vad: VadState,
    /// Audio buffer (ring buffer for streaming)
    buffer: Arc<Mutex<VecDeque<f32>>>,
    /// Is recording active
    is_recording: Arc<AtomicBool>,
    /// Accumulated samples for current utterance
    utterance_buffer: Vec<f32>,
    /// Microphone stream handle (kept alive while recording)
    #[cfg(feature = "audio")]
    _stream: Option<cpal::Stream>,
}

impl VoicePipeline {
    pub fn new(config: VoiceConfig) -> Self {
        Self {
            vad: VadState {
                energy_threshold: config.vad_threshold,
                ..Default::default()
            },
            config,
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(16000 * 30))), // 30 sec buffer
            is_recording: Arc::new(AtomicBool::new(false)),
            utterance_buffer: Vec::new(),
            #[cfg(feature = "audio")]
            _stream: None,
        }
    }

    /// Start real-time microphone capture
    #[cfg(feature = "audio")]
    pub fn start_microphone(&mut self) -> Result<()> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| anyhow!("No input device available"))?;
        
        log::info!("[VOICE] Using input device: {}", device.name()?);
        
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        
        let buffer = self.buffer.clone();
        let is_recording = self.is_recording.clone();
        
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if is_recording.load(Ordering::Relaxed) {
                    let mut buf = buffer.lock().unwrap();
                    for sample in data {
                        buf.push_back(*sample);
                        // Keep buffer size reasonable
                        if buf.len() > 16000 * 60 {
                            buf.pop_front();
                        }
                    }
                }
            },
            |err| {
                log::error!("[VOICE] Audio stream error: {}", err);
            },
            None,
        )?;
        
        stream.play()?;
        self._stream = Some(stream);
        
        log::info!("[VOICE] 🎤 Microphone started @ {}Hz", self.config.sample_rate);
        Ok(())
    }

    /// Fallback when audio feature is not enabled
    #[cfg(not(feature = "audio"))]
    pub fn start_microphone(&mut self) -> Result<()> {
        log::warn!("[VOICE] Audio capture disabled, use load_wav() instead");
        Ok(())
    }

    /// Start recording (call after microphone is started)
    pub fn start_recording(&mut self) {
        self.is_recording.store(true, Ordering::Relaxed);
        self.utterance_buffer.clear();
        self.vad.voice_active = false;
        self.vad.current_silence = 0;
        log::info!("[VOICE] 🔴 Recording started");
    }

    /// Stop recording and return audio
    pub fn stop_recording(&mut self) -> VoiceRecording {
        self.is_recording.store(false, Ordering::Relaxed);
        
        // Drain buffer into utterance
        let mut buf = self.buffer.lock().unwrap();
        while let Some(sample) = buf.pop_front() {
            self.utterance_buffer.push(sample);
        }
        
        let samples = std::mem::take(&mut self.utterance_buffer);
        let duration_secs = samples.len() as f32 / self.config.sample_rate as f32;
        let peak_energy = samples.iter()
            .map(|s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        log::info!("[VOICE] ⬛ Recording stopped: {:.2}s, peak={:.3}", duration_secs, peak_energy);
        
        VoiceRecording {
            samples,
            duration_secs,
            peak_energy,
            wake_word_detected: true, // TODO: Implement wake word detection
        }
    }

    /// Load audio from WAV file (for testing)
    pub fn load_wav(&self, path: &str) -> Result<VoiceRecording> {
        use hound::WavReader;
        
        let reader = WavReader::open(path)?;
        let spec = reader.spec();
        
        log::info!("[VOICE] Loading WAV: {} ({}Hz, {} channels, {} bits)", 
            path, spec.sample_rate, spec.channels, spec.bits_per_sample);
        
        // Convert to f32 mono 16kHz
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => {
                let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
                reader.into_samples::<i32>()
                    .filter_map(|s| s.ok())
                    .map(|s| s as f32 / max_val)
                    .collect()
            },
            hound::SampleFormat::Float => {
                reader.into_samples::<f32>()
                    .filter_map(|s| s.ok())
                    .collect()
            },
        };
        
        // Convert to mono if stereo
        let mono_samples: Vec<f32> = if spec.channels == 2 {
            samples.chunks(2)
                .map(|chunk| (chunk[0] + chunk.get(1).unwrap_or(&0.0)) / 2.0)
                .collect()
        } else {
            samples
        };
        
        // Resample to 16kHz if needed
        let resampled = if spec.sample_rate != self.config.sample_rate {
            self.resample(&mono_samples, spec.sample_rate, self.config.sample_rate)
        } else {
            mono_samples
        };
        
        let duration_secs = resampled.len() as f32 / self.config.sample_rate as f32;
        let peak_energy = resampled.iter()
            .map(|s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        log::info!("[VOICE] Loaded: {:.2}s, {} samples, peak={:.3}", 
            duration_secs, resampled.len(), peak_energy);
        
        Ok(VoiceRecording {
            samples: resampled,
            duration_secs,
            peak_energy,
            wake_word_detected: true,
        })
    }

    /// Simple linear interpolation resampling
    fn resample(&self, samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        let ratio = from_rate as f64 / to_rate as f64;
        let new_len = (samples.len() as f64 / ratio) as usize;
        
        (0..new_len)
            .map(|i| {
                let src_idx = i as f64 * ratio;
                let idx0 = src_idx.floor() as usize;
                let idx1 = (idx0 + 1).min(samples.len() - 1);
                let frac = (src_idx - idx0 as f64) as f32;
                samples[idx0] * (1.0 - frac) + samples[idx1] * frac
            })
            .collect()
    }

    /// Process audio buffer with VAD (Voice Activity Detection)
    pub fn process_with_vad(&mut self) -> Option<VoiceRecording> {
        let mut buf = self.buffer.lock().unwrap();
        
        if buf.is_empty() {
            return None;
        }
        
        // Process in chunks of 512 samples (~32ms at 16kHz)
        const CHUNK_SIZE: usize = 512;
        
        while buf.len() >= CHUNK_SIZE {
            let chunk: Vec<f32> = buf.drain(..CHUNK_SIZE).collect();
            
            // Calculate energy
            let energy: f32 = chunk.iter().map(|s| s * s).sum::<f32>() / CHUNK_SIZE as f32;
            self.vad.energy_level = self.vad.energy_level * 0.9 + energy * 0.1; // Smooth
            
            let is_voice = self.vad.energy_level > self.vad.energy_threshold;
            
            if is_voice {
                self.vad.current_silence = 0;
                if !self.vad.voice_active {
                    self.vad.voice_active = true;
                    log::debug!("[VAD] Voice started, energy={:.4}", self.vad.energy_level);
                }
                self.utterance_buffer.extend_from_slice(&chunk);
            } else if self.vad.voice_active {
                self.vad.current_silence += 1;
                self.utterance_buffer.extend_from_slice(&chunk); // Include trailing silence
                
                if self.vad.current_silence >= self.vad.silence_frames {
                    // End of utterance
                    self.vad.voice_active = false;
                    log::debug!("[VAD] Voice ended, {} samples", self.utterance_buffer.len());
                    
                    let samples = std::mem::take(&mut self.utterance_buffer);
                    let duration_secs = samples.len() as f32 / self.config.sample_rate as f32;
                    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                    
                    return Some(VoiceRecording {
                        samples,
                        duration_secs,
                        peak_energy: peak,
                        wake_word_detected: true,
                    });
                }
            }
        }
        
        None
    }

    /// Apply simple noise reduction
    pub fn reduce_noise(samples: &mut [f32]) {
        // Simple spectral gating: reduce samples below threshold
        let threshold = 0.005;
        for sample in samples.iter_mut() {
            if sample.abs() < threshold {
                *sample *= 0.1; // Reduce, don't eliminate
            }
        }
    }

    /// Check if audio contains wake word using energy detection
    /// This is a preliminary check before transcription
    pub fn detect_wake_word_energy(&self, samples: &[f32]) -> bool {
        // In continuous mode, always listen
        if self.config.continuous_mode {
            return true;
        }
        
        // Check if audio has sufficient energy (likely speech)
        let energy: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        energy > self.vad.energy_threshold * 2.0 // Require higher energy for wake word
    }

    /// Normalize audio to -1.0 to 1.0 range
    pub fn normalize(samples: &mut [f32]) {
        let max = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if max > 0.001 {
            let factor = 0.9 / max;
            for sample in samples.iter_mut() {
                *sample *= factor;
            }
        }
    }
}

/// Voice-to-intent processor combining voice pipeline with AI
pub struct VoiceToIntent {
    pipeline: VoicePipeline,
    ai: Arc<Mutex<crate::ai::KaranaAI>>,
}

impl VoiceToIntent {
    pub fn new(ai: Arc<Mutex<crate::ai::KaranaAI>>, config: VoiceConfig) -> Self {
        Self {
            pipeline: VoicePipeline::new(config),
            ai,
        }
    }

    /// Transcribe audio recording using Whisper
    pub fn transcribe(&self, recording: &VoiceRecording) -> Result<String> {
        if recording.samples.is_empty() {
            return Err(anyhow!("Empty audio recording"));
        }

        if recording.duration_secs < 0.5 {
            return Err(anyhow!("Audio too short ({:.2}s)", recording.duration_secs));
        }

        log::info!("[VOICE] Transcribing {:.2}s of audio...", recording.duration_secs);
        
        let mut ai = self.ai.lock().unwrap();
        let text = ai.transcribe(recording.samples.clone())?;
        
        // Clean up Whisper output
        let cleaned = text
            .trim()
            .trim_start_matches("[SOT]")
            .trim_start_matches("[EN]")
            .trim_start_matches("[TRANSCRIBE]")
            .trim_end_matches("[EOT]")
            .trim()
            .to_string();
        
        log::info!("[VOICE] Transcription: \"{}\"", cleaned);
        Ok(cleaned)
    }

    /// Load WAV and transcribe
    pub fn transcribe_wav(&self, path: &str) -> Result<String> {
        let recording = self.pipeline.load_wav(path)?;
        self.transcribe(&recording)
    }

    /// Start microphone capture
    #[cfg(feature = "audio")]
    pub fn start_listening(&mut self) -> Result<()> {
        self.pipeline.start_microphone()
    }

    #[cfg(not(feature = "audio"))]
    pub fn start_listening(&mut self) -> Result<()> {
        log::warn!("[VOICE] Live audio disabled, build with --features audio");
        Ok(())
    }

    /// Start recording from microphone
    pub fn start_recording(&mut self) {
        self.pipeline.start_recording();
    }

    /// Stop recording and transcribe
    pub fn stop_and_transcribe(&mut self) -> Result<String> {
        let recording = self.pipeline.stop_recording();
        self.transcribe(&recording)
    }

    /// Process with VAD and transcribe if utterance detected
    pub fn process_vad(&mut self) -> Result<Option<String>> {
        if let Some(recording) = self.pipeline.process_with_vad() {
            let text = self.transcribe(&recording)?;
            if !text.is_empty() {
                return Ok(Some(text));
            }
        }
        Ok(None)
    }
    
    /// Process with VAD, check for wake word, and return command if found
    /// Returns: Some((command, had_wake_word)) or None
    pub fn process_with_wake_word(&mut self) -> Result<Option<(String, bool)>> {
        if let Some(recording) = self.pipeline.process_with_vad() {
            let text = self.transcribe(&recording)?;
            if !text.is_empty() {
                let wake_config = self.pipeline.config.clone();
                let (had_wake, command) = extract_command_after_wake_word(&text, &wake_config.wake_word);
                
                if had_wake {
                    log::info!("[VOICE] 🎯 Wake word detected! Command: '{}'", command);
                    return Ok(Some((command, true)));
                } else if wake_config.continuous_mode {
                    // In continuous mode, accept commands without wake word
                    return Ok(Some((text, false)));
                }
            }
        }
        Ok(None)
    }

    /// Get access to inner pipeline for direct buffer manipulation
    pub fn pipeline(&self) -> &VoicePipeline {
        &self.pipeline
    }

    /// Get mutable access to pipeline
    pub fn pipeline_mut(&mut self) -> &mut VoicePipeline {
        &mut self.pipeline
    }
}

/// Wake word detection result
#[derive(Debug, Clone)]
pub struct WakeWordResult {
    /// Was the wake word detected
    pub detected: bool,
    /// The command/text after the wake word (if detected)
    pub command: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Extract command text after wake word
/// Supports variations like:
/// - "hey karana check balance" → "check balance"
/// - "karana what time is it" → "what time is it"  
/// - "okay karana" → (detected but empty command)
fn extract_command_after_wake_word(text: &str, wake_word: &str) -> (bool, String) {
    let text_lower = text.to_lowercase();
    let wake_lower = wake_word.to_lowercase();
    
    // Try different wake word patterns
    let patterns = [
        format!("hey {}", wake_lower),
        format!("okay {}", wake_lower),
        format!("ok {}", wake_lower),
        format!("hi {}", wake_lower),
        wake_lower.clone(),
    ];
    
    for pattern in &patterns {
        if let Some(idx) = text_lower.find(pattern) {
            let command_start = idx + pattern.len();
            let command = text[command_start..].trim().to_string();
            return (true, command);
        }
    }
    
    // Check for phonetic variants (Whisper may transcribe differently)
    let phonetic_variants = ["karana", "karna", "carana", "corona", "karena"];
    for variant in phonetic_variants {
        if variant != wake_lower.as_str() {
            for prefix in ["hey ", "okay ", "ok ", "hi ", ""] {
                let pattern = format!("{}{}", prefix, variant);
                if let Some(idx) = text_lower.find(&pattern) {
                    let command_start = idx + pattern.len();
                    let command = text[command_start..].trim().to_string();
                    log::info!("[VOICE] Detected phonetic variant: {} → {}", variant, wake_word);
                    return (true, command);
                }
            }
        }
    }
    
    (false, text.to_string())
}

/// Check if text contains wake word (for pre-filtering)
pub fn contains_wake_word(text: &str, wake_word: &str) -> bool {
    let (detected, _) = extract_command_after_wake_word(text, wake_word);
    detected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_state() {
        let vad = VadState::default();
        assert!(!vad.voice_active);
        assert_eq!(vad.current_silence, 0);
    }

    #[test]
    fn test_voice_config() {
        let config = VoiceConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.wake_word, "karana");
    }

    #[test]
    fn test_resample() {
        let pipeline = VoicePipeline::new(VoiceConfig::default());
        let samples: Vec<f32> = (0..48000).map(|i| (i as f32 / 48000.0).sin()).collect();
        let resampled = pipeline.resample(&samples, 48000, 16000);
        assert_eq!(resampled.len(), 16000);
    }

    #[test]
    fn test_normalize() {
        let mut samples = vec![0.5, -0.3, 0.1, -0.8];
        VoicePipeline::normalize(&mut samples);
        let max = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!((max - 0.9).abs() < 0.01);
    }
    
    #[test]
    fn test_wake_word_detection() {
        // Test "hey karana"
        let (detected, cmd) = extract_command_after_wake_word("hey karana check my balance", "karana");
        assert!(detected);
        assert_eq!(cmd, "check my balance");
        
        // Test "okay karana"
        let (detected, cmd) = extract_command_after_wake_word("okay karana what time is it", "karana");
        assert!(detected);
        assert_eq!(cmd, "what time is it");
        
        // Test just wake word
        let (detected, cmd) = extract_command_after_wake_word("karana", "karana");
        assert!(detected);
        assert!(cmd.is_empty());
        
        // Test wake word in middle
        let (detected, cmd) = extract_command_after_wake_word("I said hey karana check balance", "karana");
        assert!(detected);
        assert_eq!(cmd, "check balance");
        
        // Test no wake word
        let (detected, cmd) = extract_command_after_wake_word("check my balance please", "karana");
        assert!(!detected);
        assert_eq!(cmd, "check my balance please");
    }
    
    #[test]
    fn test_wake_word_phonetic_variants() {
        // Test phonetic variant "karna" (common mishearing)
        let (detected, cmd) = extract_command_after_wake_word("hey karna check balance", "karana");
        assert!(detected);
        assert_eq!(cmd, "check balance");
        
        // Test phonetic variant "carana"
        let (detected, cmd) = extract_command_after_wake_word("okay carana set timer", "karana");
        assert!(detected);
        assert_eq!(cmd, "set timer");
    }
    
    #[test]
    fn test_contains_wake_word() {
        assert!(contains_wake_word("hey karana help", "karana"));
        assert!(contains_wake_word("ok karna what's up", "karana"));
        assert!(!contains_wake_word("hello world", "karana"));
    }
}
