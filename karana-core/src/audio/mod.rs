//! Spatial Audio System
//!
//! 3D positional audio for immersive AR experiences.
//! Provides HRTF-based spatialization, reverb, and audio anchoring.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use nalgebra::{Point3, Vector3, UnitQuaternion};
use uuid::Uuid;

pub mod source;
pub mod listener;
pub mod hrtf;
pub mod reverb;
pub mod mixer;

pub use source::{AudioSource, SourceState, SourceType};
pub use listener::{AudioListener, ListenerState};
pub use hrtf::{HrtfProcessor, HrtfData};
pub use reverb::{ReverbProcessor, ReverbPreset, RoomAcoustics};
pub use mixer::{AudioMixer, MixerChannel, MasterBus};

/// Unique identifier for audio elements
pub type AudioId = Uuid;

/// Spatial audio engine configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Buffer size (samples)
    pub buffer_size: usize,
    /// Maximum simultaneous sources
    pub max_sources: usize,
    /// Enable HRTF processing
    pub hrtf_enabled: bool,
    /// Enable reverb processing
    pub reverb_enabled: bool,
    /// Distance attenuation model
    pub attenuation_model: AttenuationModel,
    /// Reference distance for attenuation
    pub reference_distance: f32,
    /// Maximum audible distance
    pub max_distance: f32,
    /// Rolloff factor for distance attenuation
    pub rolloff_factor: f32,
    /// Doppler effect enabled
    pub doppler_enabled: bool,
    /// Doppler factor
    pub doppler_factor: f32,
    /// Speed of sound (m/s)
    pub speed_of_sound: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 512,
            max_sources: 32,
            hrtf_enabled: true,
            reverb_enabled: true,
            attenuation_model: AttenuationModel::InverseDistance,
            reference_distance: 1.0,
            max_distance: 100.0,
            rolloff_factor: 1.0,
            doppler_enabled: true,
            doppler_factor: 1.0,
            speed_of_sound: 343.0, // m/s at 20°C
        }
    }
}

/// Distance attenuation models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttenuationModel {
    /// No distance attenuation
    None,
    /// Inverse distance: 1 / distance
    InverseDistance,
    /// Inverse distance clamped
    InverseDistanceClamped,
    /// Linear attenuation
    Linear,
    /// Linear clamped
    LinearClamped,
    /// Exponential falloff
    Exponential,
}

/// Main spatial audio engine
#[derive(Debug)]
pub struct SpatialAudioEngine {
    config: AudioConfig,
    listener: AudioListener,
    sources: HashMap<AudioId, AudioSource>,
    hrtf_processor: HrtfProcessor,
    reverb_processor: ReverbProcessor,
    mixer: AudioMixer,
    room_acoustics: Option<RoomAcoustics>,
    is_running: bool,
    last_update: Instant,
}

impl SpatialAudioEngine {
    pub fn new(config: AudioConfig) -> Self {
        let sample_rate = config.sample_rate;
        let buffer_size = config.buffer_size;
        
        Self {
            listener: AudioListener::new(),
            sources: HashMap::new(),
            hrtf_processor: HrtfProcessor::new(sample_rate),
            reverb_processor: ReverbProcessor::new(sample_rate, buffer_size),
            mixer: AudioMixer::new(sample_rate, buffer_size),
            room_acoustics: None,
            is_running: false,
            last_update: Instant::now(),
            config,
        }
    }
    
    /// Start the audio engine
    pub fn start(&mut self) -> Result<(), AudioError> {
        if self.is_running {
            return Ok(());
        }
        
        self.is_running = true;
        self.last_update = Instant::now();
        Ok(())
    }
    
    /// Stop the audio engine
    pub fn stop(&mut self) {
        self.is_running = false;
    }
    
    /// Check if engine is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    
    /// Create a new audio source
    pub fn create_source(&mut self, source_type: SourceType) -> AudioId {
        let source = AudioSource::new(source_type);
        let id = source.id;
        self.sources.insert(id, source);
        id
    }
    
    /// Create source at position
    pub fn create_source_at(&mut self, source_type: SourceType, position: Point3<f32>) -> AudioId {
        let mut source = AudioSource::new(source_type);
        source.position = position;
        let id = source.id;
        self.sources.insert(id, source);
        id
    }
    
    /// Get audio source by ID
    pub fn get_source(&self, id: AudioId) -> Option<&AudioSource> {
        self.sources.get(&id)
    }
    
    /// Get mutable audio source
    pub fn get_source_mut(&mut self, id: AudioId) -> Option<&mut AudioSource> {
        self.sources.get_mut(&id)
    }
    
    /// Remove an audio source
    pub fn remove_source(&mut self, id: AudioId) -> bool {
        self.sources.remove(&id).is_some()
    }
    
    /// Play a source
    pub fn play_source(&mut self, id: AudioId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.play();
        }
    }
    
    /// Pause a source
    pub fn pause_source(&mut self, id: AudioId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.pause();
        }
    }
    
    /// Stop a source
    pub fn stop_source(&mut self, id: AudioId) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.stop();
        }
    }
    
    /// Update listener position and orientation
    pub fn update_listener(&mut self, position: Point3<f32>, orientation: UnitQuaternion<f32>) {
        self.listener.set_position(position);
        self.listener.set_orientation(orientation);
    }
    
    /// Set listener position
    pub fn set_listener_position(&mut self, position: Point3<f32>) {
        self.listener.set_position(position);
    }
    
    /// Set listener orientation
    pub fn set_listener_orientation(&mut self, orientation: UnitQuaternion<f32>) {
        self.listener.set_orientation(orientation);
    }
    
    /// Get listener reference
    pub fn listener(&self) -> &AudioListener {
        &self.listener
    }
    
    /// Update source position
    pub fn set_source_position(&mut self, id: AudioId, position: Point3<f32>) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.position = position;
        }
    }
    
    /// Set source volume
    pub fn set_source_volume(&mut self, id: AudioId, volume: f32) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.volume = volume.clamp(0.0, 1.0);
        }
    }
    
    /// Set room acoustics for reverb
    pub fn set_room_acoustics(&mut self, acoustics: RoomAcoustics) {
        self.reverb_processor.set_room_acoustics(&acoustics);
        self.room_acoustics = Some(acoustics);
    }
    
    /// Clear room acoustics
    pub fn clear_room_acoustics(&mut self) {
        self.room_acoustics = None;
    }
    
    /// Process audio frame
    pub fn process(&mut self, output_buffer: &mut [f32]) {
        if !self.is_running {
            output_buffer.fill(0.0);
            return;
        }
        
        // Clear output buffer
        output_buffer.fill(0.0);
        
        // Cache config values to avoid borrow issues
        let listener_pos = self.listener.position;
        let listener_orient = self.listener.orientation;
        let listener_velocity = self.listener.velocity;
        let hrtf_enabled = self.config.hrtf_enabled;
        let doppler_enabled = self.config.doppler_enabled;
        let doppler_factor = self.config.doppler_factor;
        let speed_of_sound = self.config.speed_of_sound;
        let attenuation_model = self.config.attenuation_model;
        let reference_distance = self.config.reference_distance;
        let max_distance = self.config.max_distance;
        let rolloff_factor = self.config.rolloff_factor;
        
        // Collect source data to process
        let source_data: Vec<_> = self.sources.values_mut()
            .filter(|s| s.state == SourceState::Playing)
            .map(|source| {
                let to_source = source.position - listener_pos;
                let distance = to_source.norm();
                let source_pos = source.position;
                let source_vel = source.velocity;
                let volume = source.volume;
                (source, to_source, distance, source_pos, source_vel, volume)
            })
            .collect();
        
        for (source, to_source, distance, source_pos, source_vel, volume) in source_data {
            // Apply distance attenuation
            let gain = Self::calculate_attenuation_static(
                distance, attenuation_model, reference_distance, max_distance, rolloff_factor
            ) * volume;
            
            if gain < 0.001 {
                continue; // Too quiet to hear
            }
            
            // Calculate direction in listener space
            let listener_to_source = listener_orient.inverse() * to_source;
            let direction = if listener_to_source.norm() > 0.001 {
                listener_to_source.normalize()
            } else {
                Vector3::new(0.0, 0.0, -1.0) // Default to front
            };
            
            // Calculate azimuth and elevation for HRTF
            let azimuth = direction.x.atan2(-direction.z).to_degrees();
            let elevation = direction.y.asin().to_degrees();
            
            // Calculate Doppler shift if enabled
            let pitch = if doppler_enabled {
                Self::calculate_doppler_static(
                    listener_pos, listener_velocity, source_pos, source_vel,
                    speed_of_sound, doppler_factor
                )
            } else {
                1.0
            };
            
            // Generate source audio (simplified - real impl would use audio data)
            let mut source_buffer = vec![0.0f32; output_buffer.len()];
            source.generate_samples(&mut source_buffer, pitch);
            
            // Apply spatialization
            if hrtf_enabled {
                let mut spatialized = vec![0.0f32; output_buffer.len()];
                self.hrtf_processor.process(
                    &source_buffer,
                    &mut spatialized,
                    azimuth,
                    elevation,
                    gain,
                );
                
                // Mix into output
                for (out, src) in output_buffer.iter_mut().zip(spatialized.iter()) {
                    *out += src;
                }
            } else {
                // Simple stereo panning
                let pan = (azimuth / 90.0).clamp(-1.0, 1.0);
                let left_gain = gain * (1.0 - pan.max(0.0));
                let right_gain = gain * (1.0 + pan.min(0.0));
                
                for (i, sample) in source_buffer.iter().enumerate() {
                    let stereo_idx = i * 2;
                    if stereo_idx + 1 < output_buffer.len() {
                        output_buffer[stereo_idx] += sample * left_gain;
                        output_buffer[stereo_idx + 1] += sample * right_gain;
                    }
                }
            }
        }
        
        // Apply reverb if enabled
        if self.config.reverb_enabled && self.room_acoustics.is_some() {
            self.reverb_processor.process(output_buffer);
        }
        
        // Limit output
        for sample in output_buffer.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
        
        self.last_update = Instant::now();
    }
    
    fn calculate_attenuation(&self, distance: f32) -> f32 {
        Self::calculate_attenuation_static(
            distance,
            self.config.attenuation_model,
            self.config.reference_distance,
            self.config.max_distance,
            self.config.rolloff_factor,
        )
    }
    
    fn calculate_attenuation_static(
        distance: f32,
        model: AttenuationModel,
        reference_distance: f32,
        max_distance: f32,
        rolloff_factor: f32,
    ) -> f32 {
        let clamped_distance = distance.max(reference_distance);
        
        match model {
            AttenuationModel::None => 1.0,
            
            AttenuationModel::InverseDistance => {
                reference_distance / 
                (reference_distance + 
                 rolloff_factor * (clamped_distance - reference_distance))
            }
            
            AttenuationModel::InverseDistanceClamped => {
                let d = clamped_distance.min(max_distance);
                reference_distance / 
                (reference_distance + 
                 rolloff_factor * (d - reference_distance))
            }
            
            AttenuationModel::Linear => {
                1.0 - rolloff_factor * 
                (clamped_distance - reference_distance) /
                (max_distance - reference_distance)
            }
            
            AttenuationModel::LinearClamped => {
                let factor = (clamped_distance - reference_distance) /
                    (max_distance - reference_distance);
                (1.0 - rolloff_factor * factor).max(0.0)
            }
            
            AttenuationModel::Exponential => {
                (clamped_distance / reference_distance)
                    .powf(-rolloff_factor)
            }
        }
    }
    
    fn calculate_doppler(
        &self,
        listener_pos: Point3<f32>,
        listener_velocity: Vector3<f32>,
        source_pos: Point3<f32>,
        source_velocity: Vector3<f32>,
    ) -> f32 {
        Self::calculate_doppler_static(
            listener_pos, listener_velocity,
            source_pos, source_velocity,
            self.config.speed_of_sound,
            self.config.doppler_factor,
        )
    }
    
    fn calculate_doppler_static(
        listener_pos: Point3<f32>,
        listener_velocity: Vector3<f32>,
        source_pos: Point3<f32>,
        source_velocity: Vector3<f32>,
        speed_of_sound: f32,
        doppler_factor: f32,
    ) -> f32 {
        let to_source = (source_pos - listener_pos).normalize();
        
        let v_listener = listener_velocity.dot(&to_source);
        let v_source = source_velocity.dot(&to_source);
        
        let c = speed_of_sound;
        
        // Doppler equation: f' = f * (c + v_listener) / (c + v_source)
        let factor = (c + v_listener * doppler_factor) / 
                     (c + v_source * doppler_factor);
        
        factor.clamp(0.5, 2.0) // Limit extreme pitch shifts
    }
    
    /// Get number of active sources
    pub fn active_source_count(&self) -> usize {
        self.sources.values().filter(|s| s.state == SourceState::Playing).count()
    }
    
    /// Get total source count
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
    
    /// Get configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }
}

/// Audio engine errors
#[derive(Debug, Clone)]
pub enum AudioError {
    DeviceNotFound,
    InitializationFailed(String),
    BufferError(String),
    FormatNotSupported,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::DeviceNotFound => write!(f, "Audio device not found"),
            AudioError::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            AudioError::BufferError(msg) => write!(f, "Buffer error: {}", msg),
            AudioError::FormatNotSupported => write!(f, "Audio format not supported"),
        }
    }
}

impl std::error::Error for AudioError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 512);
        assert!(config.hrtf_enabled);
    }
    
    #[test]
    fn test_engine_creation() {
        let engine = SpatialAudioEngine::new(AudioConfig::default());
        assert!(!engine.is_running());
        assert_eq!(engine.source_count(), 0);
    }
    
    #[test]
    fn test_engine_start_stop() {
        let mut engine = SpatialAudioEngine::new(AudioConfig::default());
        
        engine.start().unwrap();
        assert!(engine.is_running());
        
        engine.stop();
        assert!(!engine.is_running());
    }
    
    #[test]
    fn test_source_creation() {
        let mut engine = SpatialAudioEngine::new(AudioConfig::default());
        
        let id = engine.create_source(SourceType::PointSource);
        assert_eq!(engine.source_count(), 1);
        assert!(engine.get_source(id).is_some());
    }
    
    #[test]
    fn test_source_at_position() {
        let mut engine = SpatialAudioEngine::new(AudioConfig::default());
        
        let pos = Point3::new(5.0, 0.0, 3.0);
        let id = engine.create_source_at(SourceType::PointSource, pos);
        
        let source = engine.get_source(id).unwrap();
        assert!((source.position.x - 5.0).abs() < 0.001);
    }
    
    #[test]
    fn test_source_removal() {
        let mut engine = SpatialAudioEngine::new(AudioConfig::default());
        
        let id = engine.create_source(SourceType::PointSource);
        assert!(engine.remove_source(id));
        assert!(!engine.remove_source(id)); // Already removed
        assert_eq!(engine.source_count(), 0);
    }
    
    #[test]
    fn test_listener_update() {
        let mut engine = SpatialAudioEngine::new(AudioConfig::default());
        
        let pos = Point3::new(1.0, 2.0, 3.0);
        engine.set_listener_position(pos);
        
        assert!((engine.listener().position.x - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_attenuation_none() {
        let mut config = AudioConfig::default();
        config.attenuation_model = AttenuationModel::None;
        
        let engine = SpatialAudioEngine::new(config);
        
        let gain = engine.calculate_attenuation(100.0);
        assert!((gain - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_attenuation_inverse() {
        let config = AudioConfig::default();
        let engine = SpatialAudioEngine::new(config);
        
        // At reference distance, gain should be 1.0
        let gain_ref = engine.calculate_attenuation(1.0);
        assert!((gain_ref - 1.0).abs() < 0.001);
        
        // At further distances, gain should decrease
        let gain_far = engine.calculate_attenuation(10.0);
        assert!(gain_far < gain_ref);
    }
    
    #[test]
    fn test_process_silent_when_stopped() {
        let mut engine = SpatialAudioEngine::new(AudioConfig::default());
        
        let mut buffer = vec![1.0f32; 1024];
        engine.process(&mut buffer);
        
        // Should be silent when not running
        assert!(buffer.iter().all(|&s| s == 0.0));
    }
}
