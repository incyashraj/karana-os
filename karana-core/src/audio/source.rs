//! Audio Source
//!
//! Represents a spatialized audio source in 3D space.

use nalgebra::{Point3, Vector3};
use uuid::Uuid;
use std::time::{Duration, Instant};

use super::AudioId;

/// Audio source in 3D space
#[derive(Debug, Clone)]
pub struct AudioSource {
    /// Unique identifier
    pub id: AudioId,
    /// Source type
    pub source_type: SourceType,
    /// Position in world space
    pub position: Point3<f32>,
    /// Velocity for Doppler effect
    pub velocity: Vector3<f32>,
    /// Volume (0-1)
    pub volume: f32,
    /// Pitch multiplier
    pub pitch: f32,
    /// Playback state
    pub state: SourceState,
    /// Looping enabled
    pub looping: bool,
    /// Inner cone angle (degrees) for directional sources
    pub inner_cone_angle: f32,
    /// Outer cone angle (degrees)
    pub outer_cone_angle: f32,
    /// Outer cone gain (for directional sources)
    pub outer_cone_gain: f32,
    /// Direction (for directional sources)
    pub direction: Vector3<f32>,
    /// Minimum distance for attenuation
    pub min_distance: f32,
    /// Maximum distance for attenuation
    pub max_distance: f32,
    /// Audio data (sample buffer)
    audio_data: Option<AudioData>,
    /// Current playback position (samples)
    playback_position: usize,
    /// Created timestamp
    pub created_at: Instant,
}

impl AudioSource {
    pub fn new(source_type: SourceType) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_type,
            position: Point3::origin(),
            velocity: Vector3::zeros(),
            volume: 1.0,
            pitch: 1.0,
            state: SourceState::Stopped,
            looping: false,
            inner_cone_angle: 360.0,
            outer_cone_angle: 360.0,
            outer_cone_gain: 0.0,
            direction: Vector3::new(0.0, 0.0, -1.0),
            min_distance: 1.0,
            max_distance: 100.0,
            audio_data: None,
            playback_position: 0,
            created_at: Instant::now(),
        }
    }
    
    /// Set audio data
    pub fn set_audio_data(&mut self, data: AudioData) {
        self.audio_data = Some(data);
        self.playback_position = 0;
    }
    
    /// Start playback
    pub fn play(&mut self) {
        self.state = SourceState::Playing;
    }
    
    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == SourceState::Playing {
            self.state = SourceState::Paused;
        }
    }
    
    /// Stop playback and reset position
    pub fn stop(&mut self) {
        self.state = SourceState::Stopped;
        self.playback_position = 0;
    }
    
    /// Resume from pause
    pub fn resume(&mut self) {
        if self.state == SourceState::Paused {
            self.state = SourceState::Playing;
        }
    }
    
    /// Check if playing
    pub fn is_playing(&self) -> bool {
        self.state == SourceState::Playing
    }
    
    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.state == SourceState::Paused
    }
    
    /// Generate samples into buffer
    pub fn generate_samples(&mut self, buffer: &mut [f32], pitch_scale: f32) {
        let total_pitch = self.pitch * pitch_scale;
        
        match &self.audio_data {
            Some(data) => {
                for sample in buffer.iter_mut() {
                    // Linear interpolation with pitch scaling
                    let pos = self.playback_position as f32 * total_pitch;
                    let idx = pos as usize;
                    let frac = pos - idx as f32;
                    
                    if idx < data.samples.len() {
                        let s0 = data.samples[idx];
                        let s1 = data.samples.get(idx + 1).copied().unwrap_or(s0);
                        *sample = s0 + (s1 - s0) * frac;
                        self.playback_position += 1;
                    } else if self.looping {
                        self.playback_position = 0;
                        *sample = data.samples.first().copied().unwrap_or(0.0);
                    } else {
                        *sample = 0.0;
                        self.state = SourceState::Stopped;
                    }
                }
            }
            None => {
                // Generate test tone (440 Hz sine)
                let sample_rate = 48000.0;
                let freq = 440.0 * total_pitch;
                
                for (i, sample) in buffer.iter_mut().enumerate() {
                    let t = (self.playback_position + i) as f32 / sample_rate;
                    *sample = (t * freq * 2.0 * std::f32::consts::PI).sin() * 0.3;
                }
                self.playback_position += buffer.len();
            }
        }
    }
    
    /// Calculate directional gain for cone-based sources
    pub fn calculate_directional_gain(&self, listener_direction: Vector3<f32>) -> f32 {
        if self.source_type != SourceType::SpotSource {
            return 1.0;
        }
        
        let dot = self.direction.normalize().dot(&listener_direction.normalize());
        let angle = dot.acos().to_degrees();
        
        if angle <= self.inner_cone_angle / 2.0 {
            1.0
        } else if angle <= self.outer_cone_angle / 2.0 {
            let t = (angle - self.inner_cone_angle / 2.0) / 
                   (self.outer_cone_angle / 2.0 - self.inner_cone_angle / 2.0);
            1.0 - t * (1.0 - self.outer_cone_gain)
        } else {
            self.outer_cone_gain
        }
    }
    
    /// Set position
    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
    }
    
    /// Set direction (for directional sources)
    pub fn set_direction(&mut self, direction: Vector3<f32>) {
        self.direction = direction.normalize();
    }
    
    /// Get playback progress (0-1)
    pub fn progress(&self) -> f32 {
        match &self.audio_data {
            Some(data) if !data.samples.is_empty() => {
                self.playback_position as f32 / data.samples.len() as f32
            }
            _ => 0.0,
        }
    }
    
    /// Get remaining duration
    pub fn remaining_duration(&self) -> Duration {
        match &self.audio_data {
            Some(data) => {
                let remaining_samples = data.samples.len().saturating_sub(self.playback_position);
                let seconds = remaining_samples as f32 / data.sample_rate as f32;
                Duration::from_secs_f32(seconds)
            }
            _ => Duration::ZERO,
        }
    }
}

/// Audio source types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    /// Omnidirectional point source
    PointSource,
    /// Directional spotlight source
    SpotSource,
    /// Ambient (non-spatial) source
    Ambient,
    /// Line source (e.g., for roads, rivers)
    LineSource,
    /// Area source
    AreaSource,
}

impl SourceType {
    pub fn is_spatial(&self) -> bool {
        !matches!(self, SourceType::Ambient)
    }
    
    pub fn is_directional(&self) -> bool {
        matches!(self, SourceType::SpotSource)
    }
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceState {
    Playing,
    Paused,
    Stopped,
}

/// Audio data container
#[derive(Debug, Clone)]
pub struct AudioData {
    /// Audio samples (mono)
    pub samples: Vec<f32>,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels in original data
    pub channels: u16,
    /// Duration in seconds
    pub duration: f32,
}

impl AudioData {
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        let duration = samples.len() as f32 / sample_rate as f32;
        Self {
            samples,
            sample_rate,
            channels: 1,
            duration,
        }
    }
    
    /// Generate a sine wave test tone
    pub fn sine_wave(frequency: f32, duration: f32, sample_rate: u32) -> Self {
        let sample_count = (duration * sample_rate as f32) as usize;
        let samples: Vec<f32> = (0..sample_count)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (t * frequency * 2.0 * std::f32::consts::PI).sin()
            })
            .collect();
        
        Self::new(samples, sample_rate)
    }
    
    /// Generate white noise
    pub fn white_noise(duration: f32, sample_rate: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let sample_count = (duration * sample_rate as f32) as usize;
        let samples: Vec<f32> = (0..sample_count)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();
        
        Self::new(samples, sample_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_source_creation() {
        let source = AudioSource::new(SourceType::PointSource);
        assert_eq!(source.source_type, SourceType::PointSource);
        assert_eq!(source.state, SourceState::Stopped);
        assert!((source.volume - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_source_playback() {
        let mut source = AudioSource::new(SourceType::PointSource);
        
        source.play();
        assert!(source.is_playing());
        
        source.pause();
        assert!(source.is_paused());
        
        source.resume();
        assert!(source.is_playing());
        
        source.stop();
        assert_eq!(source.state, SourceState::Stopped);
    }
    
    #[test]
    fn test_source_type() {
        assert!(SourceType::PointSource.is_spatial());
        assert!(!SourceType::Ambient.is_spatial());
        assert!(SourceType::SpotSource.is_directional());
        assert!(!SourceType::PointSource.is_directional());
    }
    
    #[test]
    fn test_audio_data_sine() {
        let data = AudioData::sine_wave(440.0, 1.0, 48000);
        assert_eq!(data.samples.len(), 48000);
        assert!((data.duration - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_generate_samples() {
        let mut source = AudioSource::new(SourceType::PointSource);
        source.play();
        
        let mut buffer = vec![0.0f32; 512];
        source.generate_samples(&mut buffer, 1.0);
        
        // Should have generated non-zero samples (test tone)
        assert!(buffer.iter().any(|&s| s != 0.0));
    }
    
    #[test]
    fn test_directional_gain() {
        let mut source = AudioSource::new(SourceType::SpotSource);
        source.direction = Vector3::new(0.0, 0.0, -1.0);
        source.inner_cone_angle = 30.0;
        source.outer_cone_angle = 60.0;
        source.outer_cone_gain = 0.2;
        
        // Direct: full gain (listener in the direction the source is pointing)
        let gain = source.calculate_directional_gain(Vector3::new(0.0, 0.0, -1.0));
        assert!((gain - 1.0).abs() < 0.001);
        
        // Outside outer cone: outer gain (perpendicular to source direction)
        let gain_outside = source.calculate_directional_gain(Vector3::new(1.0, 0.0, 0.0));
        assert!(gain_outside <= source.outer_cone_gain + 0.1);
    }
}
