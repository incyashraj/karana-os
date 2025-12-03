//! Haptic pattern definitions and waveforms

use std::time::Duration;

/// Waveform types for haptic patterns
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveformType {
    /// Sharp attack, quick decay
    Sharp,
    /// Smooth sine-like envelope
    Smooth,
    /// Quick click sensation
    Click,
    /// Continuous buzz
    Buzz,
    /// Pulsing pattern
    Pulse,
    /// Custom envelope
    Custom,
}

impl WaveformType {
    /// Sample waveform at position (0.0 - 1.0)
    pub fn sample(&self, position: f32, start_intensity: f32, end_intensity: f32) -> f32 {
        let t = position.clamp(0.0, 1.0);
        
        match self {
            WaveformType::Sharp => {
                // Quick attack, slow decay
                let envelope = if t < 0.1 {
                    t / 0.1 // Quick attack
                } else {
                    1.0 - ((t - 0.1) / 0.9).powf(0.5) // Slow decay
                };
                start_intensity + (end_intensity - start_intensity) * envelope
            }
            WaveformType::Smooth => {
                // Sine-based smooth envelope
                let envelope = (t * std::f32::consts::PI).sin();
                start_intensity + (end_intensity - start_intensity) * envelope
            }
            WaveformType::Click => {
                // Very short impulse
                if t < 0.2 {
                    end_intensity
                } else {
                    start_intensity
                }
            }
            WaveformType::Buzz => {
                // High frequency modulation
                let carrier = (t * 50.0 * std::f32::consts::PI).sin() * 0.3 + 0.7;
                let base = start_intensity + (end_intensity - start_intensity) * t;
                base * carrier
            }
            WaveformType::Pulse => {
                // On-off pulsing
                let pulse_freq = 8.0; // 8 Hz
                let pulse = ((t * pulse_freq * std::f32::consts::PI * 2.0).sin() + 1.0) / 2.0;
                let base = start_intensity + (end_intensity - start_intensity) * t;
                base * pulse
            }
            WaveformType::Custom => {
                // Linear interpolation for custom
                start_intensity + (end_intensity - start_intensity) * t
            }
        }
    }
}

/// Pattern segment
#[derive(Debug, Clone)]
pub struct PatternSegment {
    /// Waveform type
    pub waveform: WaveformType,
    /// Start intensity
    pub start_intensity: f32,
    /// End intensity
    pub end_intensity: f32,
    /// Segment duration
    pub duration: Duration,
}

impl PatternSegment {
    /// Create new segment
    pub fn new(
        waveform: WaveformType,
        start_intensity: f32,
        end_intensity: f32,
        duration: Duration,
    ) -> Self {
        Self {
            waveform,
            start_intensity: start_intensity.clamp(0.0, 1.0),
            end_intensity: end_intensity.clamp(0.0, 1.0),
            duration,
        }
    }
    
    /// Sample segment at position (0.0 - 1.0)
    pub fn sample(&self, position: f32) -> f32 {
        self.waveform.sample(position, self.start_intensity, self.end_intensity)
    }
}

/// Haptic pattern composed of segments
#[derive(Debug, Clone)]
pub struct HapticPattern {
    /// Pattern name
    pub name: String,
    /// Pattern segments
    segments: Vec<PatternSegment>,
    /// Total duration cache
    total_duration: Duration,
}

impl HapticPattern {
    /// Create new empty pattern
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            segments: Vec::new(),
            total_duration: Duration::ZERO,
        }
    }
    
    /// Add segment to pattern
    pub fn add_segment(
        mut self,
        waveform: WaveformType,
        start_intensity: f32,
        end_intensity: f32,
        duration: Duration,
    ) -> Self {
        let segment = PatternSegment::new(waveform, start_intensity, end_intensity, duration);
        self.total_duration += duration;
        self.segments.push(segment);
        self
    }
    
    /// Add pause (zero intensity segment)
    pub fn add_pause(mut self, duration: Duration) -> Self {
        let segment = PatternSegment::new(WaveformType::Custom, 0.0, 0.0, duration);
        self.total_duration += duration;
        self.segments.push(segment);
        self
    }
    
    /// Get total duration
    pub fn duration(&self) -> Duration {
        self.total_duration
    }
    
    /// Sample pattern at position (0.0 - 1.0 of total duration)
    pub fn sample(&self, position: f32) -> f32 {
        if self.segments.is_empty() || self.total_duration.is_zero() {
            return 0.0;
        }
        
        let t = position.clamp(0.0, 1.0);
        let target_time = self.total_duration.as_secs_f32() * t;
        
        let mut elapsed = 0.0f32;
        for segment in &self.segments {
            let segment_duration = segment.duration.as_secs_f32();
            if elapsed + segment_duration > target_time {
                // Found the segment
                let segment_position = (target_time - elapsed) / segment_duration.max(0.001);
                return segment.sample(segment_position);
            }
            elapsed += segment_duration;
        }
        
        // At end of pattern
        if let Some(last) = self.segments.last() {
            last.end_intensity
        } else {
            0.0
        }
    }
    
    /// Get segment count
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
    
    /// Create pattern from intensity curve
    pub fn from_curve(name: &str, curve: &[(f32, f32)], total_duration: Duration) -> Self {
        let mut pattern = Self::new(name);
        
        if curve.len() < 2 {
            return pattern;
        }
        
        for i in 0..curve.len() - 1 {
            let (t1, i1) = curve[i];
            let (t2, i2) = curve[i + 1];
            let segment_duration = Duration::from_secs_f32(
                total_duration.as_secs_f32() * (t2 - t1)
            );
            pattern = pattern.add_segment(WaveformType::Smooth, i1, i2, segment_duration);
        }
        
        pattern
    }
}

/// Pattern type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// UI interaction feedback
    Interaction,
    /// Notification alert
    Notification,
    /// Navigation guidance
    Navigation,
    /// System status
    Status,
    /// Accessibility feature
    Accessibility,
    /// Custom user pattern
    Custom,
}

/// Pattern builder for complex patterns
#[derive(Debug)]
pub struct PatternBuilder {
    pattern: HapticPattern,
}

impl PatternBuilder {
    /// Create new builder
    pub fn new(name: &str) -> Self {
        Self {
            pattern: HapticPattern::new(name),
        }
    }
    
    /// Add tap
    pub fn tap(self, intensity: f32) -> Self {
        Self {
            pattern: self.pattern.add_segment(
                WaveformType::Sharp,
                0.0,
                intensity,
                Duration::from_millis(30),
            ),
        }
    }
    
    /// Add buzz
    pub fn buzz(self, intensity: f32, duration_ms: u32) -> Self {
        Self {
            pattern: self.pattern.add_segment(
                WaveformType::Buzz,
                intensity,
                intensity,
                Duration::from_millis(duration_ms as u64),
            ),
        }
    }
    
    /// Add ramp up
    pub fn ramp_up(self, target: f32, duration_ms: u32) -> Self {
        Self {
            pattern: self.pattern.add_segment(
                WaveformType::Smooth,
                0.0,
                target,
                Duration::from_millis(duration_ms as u64),
            ),
        }
    }
    
    /// Add ramp down
    pub fn ramp_down(self, from: f32, duration_ms: u32) -> Self {
        Self {
            pattern: self.pattern.add_segment(
                WaveformType::Smooth,
                from,
                0.0,
                Duration::from_millis(duration_ms as u64),
            ),
        }
    }
    
    /// Add pause
    pub fn pause(self, duration_ms: u32) -> Self {
        Self {
            pattern: self.pattern.add_pause(Duration::from_millis(duration_ms as u64)),
        }
    }
    
    /// Add pulse sequence
    pub fn pulses(mut self, intensity: f32, count: u32, on_ms: u32, off_ms: u32) -> Self {
        for _ in 0..count {
            self = Self {
                pattern: self.pattern
                    .add_segment(WaveformType::Sharp, 0.0, intensity, Duration::from_millis(on_ms as u64))
                    .add_pause(Duration::from_millis(off_ms as u64)),
            };
        }
        self
    }
    
    /// Build the pattern
    pub fn build(self) -> HapticPattern {
        self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_creation() {
        let pattern = HapticPattern::new("test")
            .add_segment(WaveformType::Sharp, 0.0, 1.0, Duration::from_millis(100));
        
        assert_eq!(pattern.name, "test");
        assert_eq!(pattern.segment_count(), 1);
        assert_eq!(pattern.duration(), Duration::from_millis(100));
    }
    
    #[test]
    fn test_pattern_sampling() {
        let pattern = HapticPattern::new("test")
            .add_segment(WaveformType::Custom, 0.0, 1.0, Duration::from_millis(100));
        
        // At start
        let start = pattern.sample(0.0);
        assert!(start < 0.1);
        
        // At middle
        let middle = pattern.sample(0.5);
        assert!((middle - 0.5).abs() < 0.1);
        
        // At end
        let end = pattern.sample(1.0);
        assert!(end > 0.9);
    }
    
    #[test]
    fn test_pattern_with_pause() {
        let pattern = HapticPattern::new("test")
            .add_segment(WaveformType::Sharp, 0.0, 1.0, Duration::from_millis(50))
            .add_pause(Duration::from_millis(50))
            .add_segment(WaveformType::Sharp, 0.0, 1.0, Duration::from_millis(50));
        
        assert_eq!(pattern.duration(), Duration::from_millis(150));
        
        // Sample during pause
        let pause_sample = pattern.sample(0.5);
        assert!(pause_sample < 0.1);
    }
    
    #[test]
    fn test_waveform_sharp() {
        let intensity = WaveformType::Sharp.sample(0.05, 0.0, 1.0);
        // Should be rising quickly at start
        assert!(intensity > 0.3);
    }
    
    #[test]
    fn test_waveform_click() {
        let early = WaveformType::Click.sample(0.1, 0.0, 1.0);
        let late = WaveformType::Click.sample(0.5, 0.0, 1.0);
        
        assert!(early > late); // Click is impulse at start
    }
    
    #[test]
    fn test_pattern_builder() {
        let pattern = PatternBuilder::new("complex")
            .tap(0.8)
            .pause(50)
            .tap(0.8)
            .build();
        
        assert!(pattern.segment_count() >= 3);
        assert!(pattern.duration() > Duration::from_millis(50));
    }
    
    #[test]
    fn test_from_curve() {
        let curve = vec![
            (0.0, 0.0),
            (0.5, 1.0),
            (1.0, 0.0),
        ];
        
        let pattern = HapticPattern::from_curve("curve", &curve, Duration::from_millis(200));
        
        assert_eq!(pattern.segment_count(), 2);
        
        // Middle should be peak
        let middle = pattern.sample(0.5);
        assert!(middle > 0.8);
    }
    
    #[test]
    fn test_pulses_builder() {
        let pattern = PatternBuilder::new("pulses")
            .pulses(0.7, 3, 30, 50)
            .build();
        
        // 3 pulses with on+off = 6 segments
        assert_eq!(pattern.segment_count(), 6);
    }
}
