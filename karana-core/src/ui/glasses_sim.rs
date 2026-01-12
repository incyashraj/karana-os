use anyhow::Result;

// Mock implementation of GlassesSim without OpenCV
pub struct GlassesSim {
    // No internal state needed for mock
}

impl GlassesSim {
    pub fn new() -> Result<Self> {
        println!("Initializing Mock GlassesSim (OpenCV disabled)");
        Ok(Self {})
    }

    pub fn detect_gaze(&mut self) -> Result<(f32, f32)> {
        // Return center gaze for now. 
        // In the full simulation, the Frontend (React) handles the mouse-based gaze simulation.
        // This backend method is kept for API compatibility.
        Ok((0.5, 0.5)) 
    }

    pub fn detect_gesture(&mut self) -> Result<String> {
        // Stub for gesture detection
        Ok("none".to_string())
    }
}

