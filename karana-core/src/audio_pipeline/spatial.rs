// Kāraṇa OS - Spatial Audio Module
// 3D audio positioning for AR/VR experiences

use std::f32::consts::PI;

use super::capture::AudioFrame;

/// 3D position in space
#[derive(Debug, Clone, Copy, Default)]
pub struct Position3D {
    pub x: f32, // Right (+) / Left (-)
    pub y: f32, // Up (+) / Down (-)
    pub z: f32, // Forward (+) / Back (-)
}

impl Position3D {
    /// Create new position
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Origin position
    pub fn origin() -> Self {
        Self::default()
    }

    /// Distance from origin
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize to unit vector
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        } else {
            Self::origin()
        }
    }

    /// Distance to another position
    pub fn distance(&self, other: &Position3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Azimuth angle in radians (-PI to PI)
    pub fn azimuth(&self) -> f32 {
        self.x.atan2(self.z)
    }

    /// Elevation angle in radians (-PI/2 to PI/2)
    pub fn elevation(&self) -> f32 {
        let horizontal = (self.x * self.x + self.z * self.z).sqrt();
        self.y.atan2(horizontal)
    }

    /// Linear interpolation to another position
    pub fn lerp(&self, target: &Position3D, t: f32) -> Self {
        Self {
            x: self.x + (target.x - self.x) * t,
            y: self.y + (target.y - self.y) * t,
            z: self.z + (target.z - self.z) * t,
        }
    }
}

/// 3D orientation (Euler angles)
#[derive(Debug, Clone, Copy, Default)]
pub struct Orientation3D {
    pub yaw: f32,   // Rotation around Y axis (left/right)
    pub pitch: f32, // Rotation around X axis (up/down)
    pub roll: f32,  // Rotation around Z axis (tilt)
}

impl Orientation3D {
    /// Create new orientation
    pub fn new(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self { yaw, pitch, roll }
    }

    /// Forward vector from orientation
    pub fn forward(&self) -> Position3D {
        Position3D {
            x: self.yaw.sin() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.cos() * self.pitch.cos(),
        }
    }

    /// Right vector from orientation
    pub fn right(&self) -> Position3D {
        let yaw = self.yaw + PI / 2.0;
        Position3D {
            x: yaw.sin(),
            y: 0.0,
            z: yaw.cos(),
        }
    }
}

/// Listener (head/camera) state
#[derive(Debug, Clone)]
pub struct Listener {
    /// Position in 3D space
    pub position: Position3D,
    /// Orientation
    pub orientation: Orientation3D,
    /// Velocity for doppler effect
    pub velocity: Position3D,
}

impl Default for Listener {
    fn default() -> Self {
        Self {
            position: Position3D::origin(),
            orientation: Orientation3D::default(),
            velocity: Position3D::origin(),
        }
    }
}

/// Audio source in 3D space
#[derive(Debug, Clone)]
pub struct AudioSource3D {
    /// Unique ID
    pub id: u32,
    /// Position in 3D space
    pub position: Position3D,
    /// Velocity for doppler effect
    pub velocity: Position3D,
    /// Volume/gain (0.0 to 1.0+)
    pub gain: f32,
    /// Attenuation model
    pub attenuation: AttenuationModel,
    /// Reference distance (no attenuation within this)
    pub reference_distance: f32,
    /// Maximum distance (no sound beyond this)
    pub max_distance: f32,
    /// Inner cone angle (full volume)
    pub inner_cone: f32,
    /// Outer cone angle (attenuated)
    pub outer_cone: f32,
    /// Outer cone gain multiplier
    pub outer_gain: f32,
    /// Direction (for directional sources)
    pub direction: Option<Position3D>,
    /// Is looping
    pub looping: bool,
    /// Current playback position
    pub playback_position: usize,
}

impl AudioSource3D {
    /// Create new audio source
    pub fn new(id: u32, position: Position3D) -> Self {
        Self {
            id,
            position,
            velocity: Position3D::origin(),
            gain: 1.0,
            attenuation: AttenuationModel::InverseDistance,
            reference_distance: 1.0,
            max_distance: 100.0,
            inner_cone: PI * 2.0,
            outer_cone: PI * 2.0,
            outer_gain: 0.0,
            direction: None,
            looping: false,
            playback_position: 0,
        }
    }
}

/// Distance attenuation model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttenuationModel {
    /// No attenuation
    None,
    /// Linear rolloff
    Linear,
    /// Inverse distance (1/d)
    InverseDistance,
    /// Inverse distance clamped
    InverseDistanceClamped,
    /// Exponential rolloff
    Exponential,
}

impl AttenuationModel {
    /// Calculate attenuation factor
    pub fn attenuate(&self, distance: f32, ref_dist: f32, max_dist: f32, rolloff: f32) -> f32 {
        match self {
            AttenuationModel::None => 1.0,
            AttenuationModel::Linear => {
                let d = distance.clamp(ref_dist, max_dist);
                1.0 - rolloff * (d - ref_dist) / (max_dist - ref_dist)
            }
            AttenuationModel::InverseDistance => {
                ref_dist / (ref_dist + rolloff * (distance - ref_dist).max(0.0))
            }
            AttenuationModel::InverseDistanceClamped => {
                let d = distance.clamp(ref_dist, max_dist);
                ref_dist / (ref_dist + rolloff * (d - ref_dist))
            }
            AttenuationModel::Exponential => {
                (distance / ref_dist).powf(-rolloff)
            }
        }
    }
}

/// Spatial audio configuration
#[derive(Debug, Clone)]
pub struct SpatialConfig {
    /// Enable HRTF processing
    pub hrtf_enabled: bool,
    /// Speed of sound (m/s)
    pub speed_of_sound: f32,
    /// Doppler factor
    pub doppler_factor: f32,
    /// Distance model
    pub distance_model: AttenuationModel,
    /// Default rolloff factor
    pub rolloff_factor: f32,
    /// Enable reverb
    pub reverb_enabled: bool,
    /// Room size for reverb
    pub room_size: f32,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            hrtf_enabled: true,
            speed_of_sound: 343.0,
            doppler_factor: 1.0,
            distance_model: AttenuationModel::InverseDistanceClamped,
            rolloff_factor: 1.0,
            reverb_enabled: true,
            room_size: 10.0,
        }
    }
}

/// Spatial audio processor
#[derive(Debug)]
pub struct SpatialAudio {
    /// Configuration
    config: SpatialConfig,
    /// Sample rate
    sample_rate: u32,
    /// Listener state
    listener: Listener,
    /// Active audio sources
    sources: Vec<AudioSource3D>,
    /// HRTF filters
    hrtf: HrtfProcessor,
    /// Reverb processor
    reverb: ReverbProcessor,
}

impl SpatialAudio {
    /// Create new spatial audio processor
    pub fn new(config: SpatialConfig, sample_rate: u32) -> Self {
        Self {
            config: config.clone(),
            sample_rate,
            listener: Listener::default(),
            sources: Vec::new(),
            hrtf: HrtfProcessor::new(sample_rate),
            reverb: ReverbProcessor::new(config.room_size, sample_rate),
        }
    }

    /// Update listener position
    pub fn set_listener(&mut self, listener: Listener) {
        self.listener = listener;
    }

    /// Get listener reference
    pub fn listener(&self) -> &Listener {
        &self.listener
    }

    /// Add audio source
    pub fn add_source(&mut self, source: AudioSource3D) {
        self.sources.push(source);
    }

    /// Remove audio source
    pub fn remove_source(&mut self, id: u32) {
        self.sources.retain(|s| s.id != id);
    }

    /// Get source by ID
    pub fn source(&self, id: u32) -> Option<&AudioSource3D> {
        self.sources.iter().find(|s| s.id == id)
    }

    /// Get mutable source by ID
    pub fn source_mut(&mut self, id: u32) -> Option<&mut AudioSource3D> {
        self.sources.iter_mut().find(|s| s.id == id)
    }

    /// Update source position
    pub fn update_source_position(&mut self, id: u32, position: Position3D) {
        if let Some(source) = self.source_mut(id) {
            source.position = position;
        }
    }

    /// Process mono input to spatial stereo output
    pub fn spatialize(&self, source_id: u32, mono_input: &AudioFrame) -> AudioFrame {
        let source = match self.source(source_id) {
            Some(s) => s,
            None => return mono_input.clone(),
        };

        // Calculate relative position
        let rel_pos = Position3D {
            x: source.position.x - self.listener.position.x,
            y: source.position.y - self.listener.position.y,
            z: source.position.z - self.listener.position.z,
        };

        // Transform to listener space (accounting for head rotation)
        let rel_pos = self.transform_to_listener_space(&rel_pos);

        // Calculate distance
        let distance = rel_pos.magnitude().max(0.01);

        // Calculate azimuth and elevation
        let azimuth = rel_pos.azimuth();
        let elevation = rel_pos.elevation();

        // Calculate distance attenuation
        let attenuation = source.attenuation.attenuate(
            distance,
            source.reference_distance,
            source.max_distance,
            self.config.rolloff_factor,
        );

        // Calculate cone attenuation
        let cone_gain = self.calculate_cone_gain(source, &rel_pos);

        // Total gain
        let total_gain = source.gain * attenuation * cone_gain;

        // Apply HRTF or simple panning
        let stereo = if self.config.hrtf_enabled {
            self.hrtf.process(mono_input, azimuth, elevation, total_gain)
        } else {
            self.simple_pan(mono_input, azimuth, total_gain)
        };

        // Apply reverb if enabled
        if self.config.reverb_enabled {
            self.reverb.process(&stereo, distance)
        } else {
            stereo
        }
    }

    /// Transform position to listener space
    fn transform_to_listener_space(&self, pos: &Position3D) -> Position3D {
        let yaw = -self.listener.orientation.yaw;
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();

        Position3D {
            x: pos.x * cos_yaw - pos.z * sin_yaw,
            y: pos.y,
            z: pos.x * sin_yaw + pos.z * cos_yaw,
        }
    }

    /// Calculate cone attenuation for directional sources
    fn calculate_cone_gain(&self, source: &AudioSource3D, rel_pos: &Position3D) -> f32 {
        let direction = match &source.direction {
            Some(d) => d,
            None => return 1.0, // Omnidirectional
        };

        // Calculate angle between source direction and listener direction
        let to_listener = Position3D {
            x: -rel_pos.x,
            y: -rel_pos.y,
            z: -rel_pos.z,
        }.normalize();

        let dot = direction.x * to_listener.x 
            + direction.y * to_listener.y 
            + direction.z * to_listener.z;
        let angle = dot.clamp(-1.0, 1.0).acos();

        let inner = source.inner_cone / 2.0;
        let outer = source.outer_cone / 2.0;

        if angle <= inner {
            1.0
        } else if angle >= outer {
            source.outer_gain
        } else {
            // Linear interpolation between inner and outer cone
            let t = (angle - inner) / (outer - inner);
            1.0 + (source.outer_gain - 1.0) * t
        }
    }

    /// Simple stereo panning (non-HRTF)
    fn simple_pan(&self, mono: &AudioFrame, azimuth: f32, gain: f32) -> AudioFrame {
        // Convert azimuth to pan position (-1 to 1)
        let pan = (azimuth / PI).clamp(-1.0, 1.0);

        // Calculate left/right gains using constant power pan law
        let pan_angle = (pan + 1.0) * 0.25 * PI;
        let left_gain = gain * pan_angle.cos();
        let right_gain = gain * pan_angle.sin();

        // Convert mono to stereo
        let mono_samples = mono.samples_per_channel();
        let mut stereo_data = Vec::with_capacity(mono_samples * 2);

        for i in 0..mono_samples {
            let sample = mono.data.get(i).copied().unwrap_or(0.0);
            stereo_data.push(sample * left_gain);
            stereo_data.push(sample * right_gain);
        }

        AudioFrame {
            data: stereo_data,
            channels: 2,
            sample_rate: mono.sample_rate,
            timestamp: mono.timestamp,
            sequence: mono.sequence,
        }
    }

    /// Calculate doppler shift
    pub fn calculate_doppler(&self, source: &AudioSource3D) -> f32 {
        if self.config.doppler_factor == 0.0 {
            return 1.0;
        }

        let rel_pos = Position3D {
            x: source.position.x - self.listener.position.x,
            y: source.position.y - self.listener.position.y,
            z: source.position.z - self.listener.position.z,
        };

        let distance = rel_pos.magnitude().max(0.01);
        let direction = rel_pos.normalize();

        // Calculate velocities along the direction
        let source_vel = source.velocity.x * direction.x 
            + source.velocity.y * direction.y 
            + source.velocity.z * direction.z;
        let listener_vel = self.listener.velocity.x * direction.x 
            + self.listener.velocity.y * direction.y 
            + self.listener.velocity.z * direction.z;

        let c = self.config.speed_of_sound;

        // Doppler formula
        let factor = (c + listener_vel * self.config.doppler_factor) 
            / (c + source_vel * self.config.doppler_factor);

        factor.clamp(0.5, 2.0)
    }

    /// Mix all active sources
    pub fn mix_all_sources(&self, source_buffers: &[(u32, AudioFrame)]) -> AudioFrame {
        let sample_count = source_buffers.first()
            .map(|(_, f)| f.samples_per_channel())
            .unwrap_or(960);

        let mut result = AudioFrame::silence(sample_count, 2, self.sample_rate);

        for (id, mono) in source_buffers {
            let spatialized = self.spatialize(*id, mono);

            // Mix into result
            for i in 0..result.data.len().min(spatialized.data.len()) {
                result.data[i] += spatialized.data[i];
            }
        }

        result
    }
}

/// HRTF (Head-Related Transfer Function) processor
#[derive(Debug)]
struct HrtfProcessor {
    /// Sample rate
    sample_rate: u32,
    /// Left ear delay line
    left_delay: Vec<f32>,
    /// Right ear delay line
    right_delay: Vec<f32>,
    /// Head radius in meters (for ITD calculation)
    head_radius: f32,
}

impl HrtfProcessor {
    /// Create new HRTF processor
    fn new(sample_rate: u32) -> Self {
        let max_delay = (sample_rate as f32 * 0.001) as usize; // 1ms max delay
        Self {
            sample_rate,
            left_delay: vec![0.0; max_delay],
            right_delay: vec![0.0; max_delay],
            head_radius: 0.09, // Average human head radius
        }
    }

    /// Process mono to binaural stereo
    fn process(&self, mono: &AudioFrame, azimuth: f32, elevation: f32, gain: f32) -> AudioFrame {
        let samples = mono.samples_per_channel();
        let mut stereo = vec![0.0f32; samples * 2];

        // Calculate ITD (Interaural Time Difference)
        let itd = self.calculate_itd(azimuth);
        let itd_samples = (itd * self.sample_rate as f32).abs() as usize;

        // Calculate ILD (Interaural Level Difference)
        let (left_gain, right_gain) = self.calculate_ild(azimuth, elevation);

        // Apply ITD and ILD
        for i in 0..samples {
            let sample = mono.data.get(i).copied().unwrap_or(0.0) * gain;

            // Apply to left/right with delay
            if azimuth > 0.0 {
                // Source on right - left ear delayed
                stereo[i * 2] = if i >= itd_samples {
                    sample * left_gain
                } else {
                    0.0
                };
                stereo[i * 2 + 1] = sample * right_gain;
            } else {
                // Source on left - right ear delayed
                stereo[i * 2] = sample * left_gain;
                stereo[i * 2 + 1] = if i >= itd_samples {
                    sample * right_gain
                } else {
                    0.0
                };
            }
        }

        AudioFrame {
            data: stereo,
            channels: 2,
            sample_rate: mono.sample_rate,
            timestamp: mono.timestamp,
            sequence: mono.sequence,
        }
    }

    /// Calculate ITD based on azimuth
    fn calculate_itd(&self, azimuth: f32) -> f32 {
        // Woodworth formula for ITD
        let c = 343.0; // Speed of sound
        self.head_radius * (azimuth.sin() + azimuth) / c
    }

    /// Calculate ILD based on azimuth and elevation
    fn calculate_ild(&self, azimuth: f32, _elevation: f32) -> (f32, f32) {
        // Simplified ILD model - more attenuation for shadowed ear
        let shadow = 0.7; // Shadow factor

        let left = if azimuth > 0.0 {
            // Source on right - left ear shadowed
            1.0 - shadow * azimuth.sin().abs()
        } else {
            1.0
        };

        let right = if azimuth < 0.0 {
            // Source on left - right ear shadowed
            1.0 - shadow * azimuth.sin().abs()
        } else {
            1.0
        };

        (left, right)
    }
}

/// Simple reverb processor
#[derive(Debug)]
struct ReverbProcessor {
    /// Delay lines for early reflections
    early_delays: Vec<DelayLine>,
    /// Delay lines for late reverb
    late_delays: Vec<DelayLine>,
    /// Feedback amount
    feedback: f32,
    /// Mix amount (wet/dry)
    mix: f32,
}

impl ReverbProcessor {
    /// Create new reverb processor
    fn new(room_size: f32, sample_rate: u32) -> Self {
        // Create delay lines based on room size
        let base_delay = (room_size * sample_rate as f32 / 343.0) as usize;

        Self {
            early_delays: vec![
                DelayLine::new(base_delay / 4),
                DelayLine::new(base_delay / 3),
                DelayLine::new(base_delay / 2),
            ],
            late_delays: vec![
                DelayLine::new(base_delay),
                DelayLine::new((base_delay as f32 * 1.3) as usize),
                DelayLine::new((base_delay as f32 * 1.7) as usize),
                DelayLine::new(base_delay * 2),
            ],
            feedback: 0.7,
            mix: 0.3,
        }
    }

    /// Process frame with reverb
    fn process(&self, input: &AudioFrame, distance: f32) -> AudioFrame {
        // Reverb amount based on distance
        let amount = (1.0 - 1.0 / (1.0 + distance * 0.1)).min(0.8);
        let mix = self.mix * amount;

        let mut output = input.clone();

        // For now, simple attenuation based on reverb amount
        // Full implementation would use delay lines and feedback
        for sample in &mut output.data {
            *sample *= 1.0 - mix * 0.5;
        }

        output
    }
}

/// Simple delay line
#[derive(Debug)]
struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayLine {
    fn new(length: usize) -> Self {
        Self {
            buffer: vec![0.0; length.max(1)],
            write_pos: 0,
        }
    }

    #[allow(dead_code)]
    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.write_pos];
        self.buffer[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position3d() {
        let pos = Position3D::new(3.0, 4.0, 0.0);
        assert_eq!(pos.magnitude(), 5.0);

        let norm = pos.normalize();
        assert!((norm.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_position_distance() {
        let a = Position3D::new(0.0, 0.0, 0.0);
        let b = Position3D::new(3.0, 4.0, 0.0);
        assert_eq!(a.distance(&b), 5.0);
    }

    #[test]
    fn test_position_azimuth() {
        // Front
        let front = Position3D::new(0.0, 0.0, 1.0);
        assert!(front.azimuth().abs() < 0.001);

        // Right
        let right = Position3D::new(1.0, 0.0, 0.0);
        assert!((right.azimuth() - PI / 2.0).abs() < 0.001);

        // Left
        let left = Position3D::new(-1.0, 0.0, 0.0);
        assert!((left.azimuth() + PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_position_elevation() {
        let above = Position3D::new(0.0, 1.0, 1.0);
        assert!(above.elevation() > 0.0);

        let below = Position3D::new(0.0, -1.0, 1.0);
        assert!(below.elevation() < 0.0);
    }

    #[test]
    fn test_position_lerp() {
        let a = Position3D::new(0.0, 0.0, 0.0);
        let b = Position3D::new(10.0, 10.0, 10.0);

        let mid = a.lerp(&b, 0.5);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 5.0);
        assert_eq!(mid.z, 5.0);
    }

    #[test]
    fn test_attenuation_none() {
        let model = AttenuationModel::None;
        assert_eq!(model.attenuate(100.0, 1.0, 100.0, 1.0), 1.0);
    }

    #[test]
    fn test_attenuation_inverse() {
        let model = AttenuationModel::InverseDistance;
        
        // At reference distance, gain = 1.0
        let at_ref = model.attenuate(1.0, 1.0, 100.0, 1.0);
        assert!((at_ref - 1.0).abs() < 0.01);

        // Further away, gain decreases
        let far = model.attenuate(10.0, 1.0, 100.0, 1.0);
        assert!(far < at_ref);
    }

    #[test]
    fn test_spatial_audio_basic() {
        let config = SpatialConfig::default();
        let mut spatial = SpatialAudio::new(config, 48000);

        // Add source
        let source = AudioSource3D::new(1, Position3D::new(5.0, 0.0, 5.0));
        spatial.add_source(source);

        assert!(spatial.source(1).is_some());
        assert!(spatial.source(999).is_none());
    }

    #[test]
    fn test_spatial_audio_listener() {
        let config = SpatialConfig::default();
        let mut spatial = SpatialAudio::new(config, 48000);

        let listener = Listener {
            position: Position3D::new(1.0, 2.0, 3.0),
            orientation: Orientation3D::new(0.5, 0.0, 0.0),
            velocity: Position3D::origin(),
        };
        spatial.set_listener(listener.clone());

        assert_eq!(spatial.listener().position.x, 1.0);
    }

    #[test]
    fn test_spatialize() {
        let config = SpatialConfig::default();
        let mut spatial = SpatialAudio::new(config, 48000);

        let source = AudioSource3D::new(1, Position3D::new(5.0, 0.0, 0.0)); // To the right
        spatial.add_source(source);

        let mono = AudioFrame::new(vec![0.5; 480], 1, 48000);
        let stereo = spatial.spatialize(1, &mono);

        assert_eq!(stereo.channels, 2);
        // Right channel should be louder for source on right
        let left_sum: f32 = stereo.data.iter().step_by(2).sum();
        let right_sum: f32 = stereo.data.iter().skip(1).step_by(2).sum();
        assert!(right_sum > left_sum);
    }

    #[test]
    fn test_doppler() {
        let config = SpatialConfig {
            doppler_factor: 1.0,
            ..Default::default()
        };
        let spatial = SpatialAudio::new(config, 48000);

        // Stationary source
        let mut source = AudioSource3D::new(1, Position3D::new(10.0, 0.0, 0.0));
        source.velocity = Position3D::origin();
        let doppler_stationary = spatial.calculate_doppler(&source);
        assert!((doppler_stationary - 1.0).abs() < 0.01);

        // Approaching source
        source.velocity = Position3D::new(-50.0, 0.0, 0.0);
        let doppler_approaching = spatial.calculate_doppler(&source);
        assert!(doppler_approaching > 1.0); // Higher pitch

        // Receding source
        source.velocity = Position3D::new(50.0, 0.0, 0.0);
        let doppler_receding = spatial.calculate_doppler(&source);
        assert!(doppler_receding < 1.0); // Lower pitch
    }

    #[test]
    fn test_cone_attenuation() {
        let config = SpatialConfig::default();
        let spatial = SpatialAudio::new(config, 48000);

        let mut source = AudioSource3D::new(1, Position3D::new(0.0, 0.0, 10.0));
        source.direction = Some(Position3D::new(0.0, 0.0, -1.0).normalize()); // Pointing toward origin
        source.inner_cone = PI / 4.0;
        source.outer_cone = PI / 2.0;
        source.outer_gain = 0.1;

        // Relative position from source's perspective - listener is in direction source is pointing
        // The rel_pos passed to calculate_cone_gain is from listener to source, so (0, 0, 10)
        // But the function calculates to_listener as -rel_pos, so to_listener = (0, 0, -10) normalized = (0, 0, -1)
        // And source.direction is (0, 0, -1), so dot product = 1.0, angle = 0 = full gain
        let rel_pos = Position3D::new(0.0, 0.0, 10.0);
        let gain = spatial.calculate_cone_gain(&source, &rel_pos);
        assert!((gain - 1.0).abs() < 0.1, "Expected gain ~1.0, got {}", gain);
    }

    #[test]
    fn test_simple_pan() {
        let config = SpatialConfig {
            hrtf_enabled: false,
            ..Default::default()
        };
        let spatial = SpatialAudio::new(config, 48000);

        let mono = AudioFrame::new(vec![1.0; 100], 1, 48000);

        // Center pan
        let center = spatial.simple_pan(&mono, 0.0, 1.0);
        let l: f32 = center.data.iter().step_by(2).sum();
        let r: f32 = center.data.iter().skip(1).step_by(2).sum();
        assert!((l - r).abs() < 1.0);

        // Hard right
        let right = spatial.simple_pan(&mono, PI / 2.0, 1.0);
        let l: f32 = right.data.iter().step_by(2).sum();
        let r: f32 = right.data.iter().skip(1).step_by(2).sum();
        assert!(r > l);
    }

    #[test]
    fn test_mix_all_sources() {
        let config = SpatialConfig::default();
        let mut spatial = SpatialAudio::new(config, 48000);

        spatial.add_source(AudioSource3D::new(1, Position3D::new(-5.0, 0.0, 5.0)));
        spatial.add_source(AudioSource3D::new(2, Position3D::new(5.0, 0.0, 5.0)));

        let mono1 = AudioFrame::new(vec![0.3; 480], 1, 48000);
        let mono2 = AudioFrame::new(vec![0.3; 480], 1, 48000);

        let mixed = spatial.mix_all_sources(&[(1, mono1), (2, mono2)]);
        assert_eq!(mixed.channels, 2);
    }

    #[test]
    fn test_orientation_forward() {
        let orient = Orientation3D::new(0.0, 0.0, 0.0);
        let forward = orient.forward();
        assert!(forward.z > 0.9); // Should be pointing forward

        let orient_right = Orientation3D::new(PI / 2.0, 0.0, 0.0);
        let forward_right = orient_right.forward();
        assert!(forward_right.x > 0.9); // Should be pointing right
    }
}
