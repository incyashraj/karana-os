//! Audio Mixer
//!
//! Multi-channel mixing and master output processing.

use std::collections::HashMap;
use uuid::Uuid;

use super::AudioId;

/// Audio mixer for combining multiple channels
#[derive(Debug)]
pub struct AudioMixer {
    sample_rate: u32,
    buffer_size: usize,
    channels: HashMap<AudioId, MixerChannel>,
    master: MasterBus,
    /// Temporary mixing buffer
    mix_buffer: Vec<f32>,
}

impl AudioMixer {
    pub fn new(sample_rate: u32, buffer_size: usize) -> Self {
        Self {
            sample_rate,
            buffer_size,
            channels: HashMap::new(),
            master: MasterBus::new(),
            mix_buffer: vec![0.0; buffer_size * 2], // Stereo
        }
    }
    
    /// Create a new mixer channel
    pub fn create_channel(&mut self, name: &str) -> AudioId {
        let channel = MixerChannel::new(name);
        let id = channel.id;
        self.channels.insert(id, channel);
        id
    }
    
    /// Get channel by ID
    pub fn get_channel(&self, id: AudioId) -> Option<&MixerChannel> {
        self.channels.get(&id)
    }
    
    /// Get mutable channel
    pub fn get_channel_mut(&mut self, id: AudioId) -> Option<&mut MixerChannel> {
        self.channels.get_mut(&id)
    }
    
    /// Remove channel
    pub fn remove_channel(&mut self, id: AudioId) -> bool {
        self.channels.remove(&id).is_some()
    }
    
    /// Set channel volume
    pub fn set_channel_volume(&mut self, id: AudioId, volume: f32) {
        if let Some(channel) = self.channels.get_mut(&id) {
            channel.volume = volume.clamp(0.0, 2.0);
        }
    }
    
    /// Set channel pan (-1 = left, 0 = center, 1 = right)
    pub fn set_channel_pan(&mut self, id: AudioId, pan: f32) {
        if let Some(channel) = self.channels.get_mut(&id) {
            channel.pan = pan.clamp(-1.0, 1.0);
        }
    }
    
    /// Mute channel
    pub fn mute_channel(&mut self, id: AudioId) {
        if let Some(channel) = self.channels.get_mut(&id) {
            channel.mute = true;
        }
    }
    
    /// Unmute channel
    pub fn unmute_channel(&mut self, id: AudioId) {
        if let Some(channel) = self.channels.get_mut(&id) {
            channel.mute = false;
        }
    }
    
    /// Solo channel (mute all others)
    pub fn solo_channel(&mut self, id: AudioId) {
        for (cid, channel) in &mut self.channels {
            channel.solo = *cid == id;
        }
    }
    
    /// Clear solo
    pub fn clear_solo(&mut self) {
        for channel in self.channels.values_mut() {
            channel.solo = false;
        }
    }
    
    /// Add audio to a channel's buffer
    pub fn add_to_channel(&mut self, id: AudioId, samples: &[f32]) {
        if let Some(channel) = self.channels.get_mut(&id) {
            let len = samples.len().min(channel.buffer.len());
            for i in 0..len {
                channel.buffer[i] += samples[i];
            }
        }
    }
    
    /// Mix all channels to output
    pub fn mix(&mut self, output: &mut [f32]) {
        output.fill(0.0);
        
        // Check for any soloed channels
        let has_solo = self.channels.values().any(|c| c.solo);
        
        for channel in self.channels.values_mut() {
            // Skip if muted or not soloed (when solo exists)
            if channel.mute || (has_solo && !channel.solo) {
                continue;
            }
            
            let volume = channel.volume;
            let pan = channel.pan;
            
            // Calculate pan gains
            let left_gain = volume * (1.0 - pan.max(0.0));
            let right_gain = volume * (1.0 + pan.min(0.0));
            
            // Mix channel buffer to output
            let samples = channel.buffer.len().min(output.len() / 2);
            for i in 0..samples {
                let stereo_idx = i * 2;
                if stereo_idx + 1 < output.len() {
                    output[stereo_idx] += channel.buffer[i] * left_gain;
                    output[stereo_idx + 1] += channel.buffer[i] * right_gain;
                }
            }
            
            // Update peak meter
            channel.update_peak();
            
            // Clear channel buffer for next frame
            channel.buffer.fill(0.0);
        }
        
        // Apply master processing
        self.master.process(output);
    }
    
    /// Get master bus
    pub fn master(&self) -> &MasterBus {
        &self.master
    }
    
    /// Get mutable master bus
    pub fn master_mut(&mut self) -> &mut MasterBus {
        &mut self.master
    }
    
    /// Get channel count
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}

/// Individual mixer channel
#[derive(Debug)]
pub struct MixerChannel {
    /// Channel ID
    pub id: AudioId,
    /// Channel name
    pub name: String,
    /// Volume (0-2)
    pub volume: f32,
    /// Pan (-1 to 1)
    pub pan: f32,
    /// Muted
    pub mute: bool,
    /// Solo
    pub solo: bool,
    /// Audio buffer
    buffer: Vec<f32>,
    /// Peak level (for metering)
    peak: f32,
    /// Peak hold counter
    peak_hold: u32,
}

impl MixerChannel {
    pub fn new(name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            volume: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
            buffer: vec![0.0; 4096],
            peak: 0.0,
            peak_hold: 0,
        }
    }
    
    /// Get current peak level
    pub fn peak(&self) -> f32 {
        self.peak
    }
    
    /// Get peak in dB
    pub fn peak_db(&self) -> f32 {
        if self.peak > 0.0001 {
            20.0 * self.peak.log10()
        } else {
            -60.0
        }
    }
    
    /// Update peak meter
    fn update_peak(&mut self) {
        let current_peak = self.buffer.iter()
            .map(|s| s.abs())
            .fold(0.0f32, |a, b| a.max(b));
        
        if current_peak > self.peak {
            self.peak = current_peak;
            self.peak_hold = 30; // Hold for ~30 frames
        } else if self.peak_hold > 0 {
            self.peak_hold -= 1;
        } else {
            // Decay peak
            self.peak *= 0.95;
        }
    }
    
    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
    }
}

/// Master output bus
#[derive(Debug)]
pub struct MasterBus {
    /// Master volume
    pub volume: f32,
    /// Limiter enabled
    pub limiter_enabled: bool,
    /// Limiter threshold
    pub limiter_threshold: f32,
    /// Current peak level
    peak_left: f32,
    peak_right: f32,
    /// Limiter gain reduction
    gain_reduction: f32,
}

impl MasterBus {
    pub fn new() -> Self {
        Self {
            volume: 1.0,
            limiter_enabled: true,
            limiter_threshold: 0.95,
            peak_left: 0.0,
            peak_right: 0.0,
            gain_reduction: 0.0,
        }
    }
    
    /// Process master output
    pub fn process(&mut self, buffer: &mut [f32]) {
        if buffer.is_empty() {
            return;
        }
        
        // Apply master volume
        for sample in buffer.iter_mut() {
            *sample *= self.volume;
        }
        
        // Update peak meters
        let samples = buffer.len() / 2;
        let mut max_left = 0.0f32;
        let mut max_right = 0.0f32;
        
        for i in 0..samples {
            let left_idx = i * 2;
            let right_idx = i * 2 + 1;
            
            if right_idx < buffer.len() {
                max_left = max_left.max(buffer[left_idx].abs());
                max_right = max_right.max(buffer[right_idx].abs());
            }
        }
        
        self.peak_left = max_left;
        self.peak_right = max_right;
        
        // Apply limiter
        if self.limiter_enabled {
            self.apply_limiter(buffer);
        }
    }
    
    fn apply_limiter(&mut self, buffer: &mut [f32]) {
        let threshold = self.limiter_threshold;
        let mut max_gain_reduction = 0.0f32;
        
        for sample in buffer.iter_mut() {
            let abs_sample = sample.abs();
            
            if abs_sample > threshold {
                let reduction = threshold / abs_sample;
                max_gain_reduction = max_gain_reduction.max(1.0 - reduction);
                *sample *= reduction;
            }
        }
        
        // Smooth gain reduction for metering
        self.gain_reduction = self.gain_reduction * 0.9 + max_gain_reduction * 0.1;
    }
    
    /// Get left peak level
    pub fn peak_left(&self) -> f32 {
        self.peak_left
    }
    
    /// Get right peak level
    pub fn peak_right(&self) -> f32 {
        self.peak_right
    }
    
    /// Get left peak in dB
    pub fn peak_left_db(&self) -> f32 {
        if self.peak_left > 0.0001 {
            20.0 * self.peak_left.log10()
        } else {
            -60.0
        }
    }
    
    /// Get right peak in dB
    pub fn peak_right_db(&self) -> f32 {
        if self.peak_right > 0.0001 {
            20.0 * self.peak_right.log10()
        } else {
            -60.0
        }
    }
    
    /// Get current gain reduction (0-1)
    pub fn gain_reduction(&self) -> f32 {
        self.gain_reduction
    }
    
    /// Get gain reduction in dB
    pub fn gain_reduction_db(&self) -> f32 {
        if self.gain_reduction > 0.0001 {
            20.0 * (1.0 - self.gain_reduction).log10()
        } else {
            0.0
        }
    }
    
    /// Set master volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 2.0);
    }
}

impl Default for MasterBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mixer_creation() {
        let mixer = AudioMixer::new(48000, 512);
        assert_eq!(mixer.channel_count(), 0);
    }
    
    #[test]
    fn test_channel_creation() {
        let mut mixer = AudioMixer::new(48000, 512);
        
        let id = mixer.create_channel("Main");
        assert_eq!(mixer.channel_count(), 1);
        
        let channel = mixer.get_channel(id).unwrap();
        assert_eq!(channel.name, "Main");
    }
    
    #[test]
    fn test_channel_volume() {
        let mut mixer = AudioMixer::new(48000, 512);
        let id = mixer.create_channel("Test");
        
        mixer.set_channel_volume(id, 0.5);
        
        let channel = mixer.get_channel(id).unwrap();
        assert!((channel.volume - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_channel_pan() {
        let mut mixer = AudioMixer::new(48000, 512);
        let id = mixer.create_channel("Test");
        
        mixer.set_channel_pan(id, -0.7);
        
        let channel = mixer.get_channel(id).unwrap();
        assert!((channel.pan + 0.7).abs() < 0.001);
    }
    
    #[test]
    fn test_channel_mute() {
        let mut mixer = AudioMixer::new(48000, 512);
        let id = mixer.create_channel("Test");
        
        mixer.mute_channel(id);
        assert!(mixer.get_channel(id).unwrap().mute);
        
        mixer.unmute_channel(id);
        assert!(!mixer.get_channel(id).unwrap().mute);
    }
    
    #[test]
    fn test_mix_output() {
        let mut mixer = AudioMixer::new(48000, 512);
        let id = mixer.create_channel("Test");
        
        // Add some audio
        let samples: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        mixer.add_to_channel(id, &samples);
        
        let mut output = vec![0.0f32; 1024];
        mixer.mix(&mut output);
        
        // Should have non-zero output
        assert!(output.iter().any(|&s| s != 0.0));
    }
    
    #[test]
    fn test_muted_channel_silent() {
        let mut mixer = AudioMixer::new(48000, 512);
        let id = mixer.create_channel("Test");
        
        mixer.mute_channel(id);
        
        let samples: Vec<f32> = (0..512).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        mixer.add_to_channel(id, &samples);
        
        let mut output = vec![0.0f32; 1024];
        mixer.mix(&mut output);
        
        // Muted channel should be silent
        assert!(output.iter().all(|&s| s == 0.0));
    }
    
    #[test]
    fn test_master_limiter() {
        let mut master = MasterBus::new();
        master.limiter_threshold = 0.5;
        
        let mut buffer = vec![1.0f32; 100]; // All samples at 1.0
        master.process(&mut buffer);
        
        // All samples should be limited to threshold
        assert!(buffer.iter().all(|&s| s.abs() <= 0.51));
    }
    
    #[test]
    fn test_master_volume() {
        let mut master = MasterBus::new();
        master.limiter_enabled = false;
        master.set_volume(0.5);
        
        let mut buffer = vec![1.0f32; 100];
        master.process(&mut buffer);
        
        assert!(buffer.iter().all(|&s| (s - 0.5).abs() < 0.001));
    }
    
    #[test]
    fn test_peak_db() {
        let channel = MixerChannel::new("Test");
        
        // Empty buffer should be very low dB
        let db = channel.peak_db();
        assert!(db < -50.0);
    }
    
    #[test]
    fn test_solo() {
        let mut mixer = AudioMixer::new(48000, 512);
        let id1 = mixer.create_channel("Ch1");
        let id2 = mixer.create_channel("Ch2");
        
        mixer.solo_channel(id1);
        
        assert!(mixer.get_channel(id1).unwrap().solo);
        assert!(!mixer.get_channel(id2).unwrap().solo);
        
        mixer.clear_solo();
        
        assert!(!mixer.get_channel(id1).unwrap().solo);
    }
}
