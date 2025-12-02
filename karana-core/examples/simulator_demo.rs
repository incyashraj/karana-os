//! Smart Glasses Simulator Demo
//! 
//! Interactive demo to test the Kāraṇa OS experience without hardware.
//! Run with: cargo run --example simulator_demo

use karana_core::simulator::{
    Simulator, DeviceProfile, SimulatorTUI,
    scenarios::ScenarioLibrary,
};

fn main() {
    println!();
    println!("  ╭─────────────────────────────────────────────────────╮");
    println!("  │                                                     │");
    println!("  │   🕶️  KĀRAṆA SMART GLASSES SIMULATOR               │");
    println!("  │                                                     │");
    println!("  │   Test the full Kāraṇa OS experience without        │");
    println!("  │   physical hardware. Simulates display, sensors,    │");
    println!("  │   camera, voice input, and more.                    │");
    println!("  │                                                     │");
    println!("  ╰─────────────────────────────────────────────────────╯");
    println!();
    
    // Select device profile
    println!("  Select device profile:");
    println!("    1. XREAL Air (1080p, 120fps, tethered)");
    println!("    2. Rokid Max (1080p, 120fps, tethered)");
    println!("    3. Meta Ray-Ban (camera-focused, battery)");
    println!("    4. Enterprise AR (HoloLens-like, high-end)");
    println!();
    print!("  Choice [1-4, default=1]: ");
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let profile = match input.trim() {
        "2" => DeviceProfile::RokidMax,
        "3" => DeviceProfile::MetaRayBan,
        "4" => DeviceProfile::EnterpriseAR,
        _ => DeviceProfile::XrealAir,
    };
    
    println!();
    println!("  Selected: {:?}", profile);
    println!("  Config: {:?}", profile.config());
    println!();
    
    // Create and run simulator
    let mut tui = SimulatorTUI::new(profile);
    tui.run();
}
