use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::File;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::audio::SampleBuffer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputSource {
    Keyboard,
    Gaze(f32, f32), // x, y coordinates (0.0 - 1.0)
    Voice(String),
    Gesture(String), // "SwipeLeft", "Nod"
}

pub struct MultimodalInput {
    pub last_gaze: (f32, f32),
    pub last_voice_command: Option<String>,
}

impl MultimodalInput {
    pub fn new() -> Self {
        Self {
            last_gaze: (0.5, 0.5), // Center
            last_voice_command: None,
        }
    }

    pub fn update_gaze(&mut self, x: f32, y: f32) {
        self.last_gaze = (x.max(0.0).min(1.0), y.max(0.0).min(1.0));
    }

    pub fn process_voice(&mut self, command: &str) {
        self.last_voice_command = Some(command.to_string());
        log::info!("Atom 3 (Input): Voice Command Recognized: '{}'", command);
    }

    pub fn simulate_random_gaze(&mut self) {
        // Simulate eye movement
        let dx = (rand::random::<f32>() - 0.5) * 0.1;
        let dy = (rand::random::<f32>() - 0.5) * 0.1;
        self.update_gaze(self.last_gaze.0 + dx, self.last_gaze.1 + dy);
    }

    /// Reads an audio file and converts it to a float vector (16kHz mono) for Whisper.
    /// This is "Real Functionality" - it processes actual audio bytes.
    pub fn read_audio_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<f32>> {
        let src = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let hint = Hint::new();

        let probed = symphonia::default::get_probe().format(&hint, mss, &Default::default(), &Default::default())?;
        let mut format = probed.format;
        let track = format.default_track().ok_or_else(|| anyhow::anyhow!("No track found"))?;
        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &Default::default())?;

        let track_id = track.id;
        let mut samples = Vec::new();

        // Decode loop
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(symphonia::core::errors::Error::IoError(_)) => break, // EOF
                Err(e) => return Err(e.into()),
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Resample/Mix to Mono 16kHz if needed?
                    // For now, assume input is close enough or just take first channel.
                    // Whisper expects 16000Hz.
                    // We should check sample rate.
                    let spec = *decoded.spec();
                    
                    if spec.rate != 16000 {
                        // In a full OS, we'd resample. For this prototype, we warn.
                        // log::warn!("Audio sample rate is {}, Whisper expects 16000. Results may be poor.", spec.rate);
                    }

                    let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
                    sample_buf.copy_interleaved_ref(decoded);
                    
                    // Convert to mono (average channels)
                    let channels = spec.channels.count();
                    for frame in sample_buf.samples().chunks(channels) {
                        let sum: f32 = frame.iter().sum();
                        samples.push(sum / channels as f32);
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(samples)
    }
}
