//! Automated Scenario Runner
//! 
//! Runs through scenarios automatically for testing and demo purposes.
//! Run with: cargo run --example scenario_runner

use karana_core::simulator::{
    Simulator, DeviceProfile,
    device::VirtualGlasses,
    scenarios::{ScenarioEngine, ScenarioLibrary, EventType},
    input::TestScene,
};
use std::time::{Duration, Instant};
use std::thread;

fn main() {
    println!("\n");
    println!("  ╔═══════════════════════════════════════════════════════════╗");
    println!("  ║           KĀRAṆA SCENARIO RUNNER                          ║");
    println!("  ║                                                           ║");
    println!("  ║   Automated testing of smart glasses scenarios            ║");
    println!("  ╚═══════════════════════════════════════════════════════════╝\n");

    // List all scenarios
    let scenarios = ScenarioLibrary::all();
    println!("  Available Scenarios:");
    println!("  ────────────────────────────────────────────────────────────");
    for (i, scenario) in scenarios.iter().enumerate() {
        println!("    {}. {} ({} events, {}s duration)", 
            i + 1, 
            scenario.name,
            scenario.events.len(),
            scenario.duration.as_secs()
        );
        println!("       └─ {}", scenario.description);
    }
    println!();

    // Run a demo scenario
    println!("  Running: Morning Routine Scenario (Accelerated)\n");
    println!("  ════════════════════════════════════════════════════════════\n");

    let scenario = ScenarioLibrary::morning_routine();
    let mut engine = ScenarioEngine::new();
    let mut device = VirtualGlasses::new(DeviceProfile::XrealAir.config());
    
    // Boot device
    device.boot().unwrap();
    device.display.show_status_bar("7:00 AM", "100%", "●●●");
    
    // Load and start scenario
    engine.load(scenario);
    engine.start();
    
    let start = Instant::now();
    let acceleration = 10.0; // Run 10x faster
    
    println!("  Time │ Event");
    println!("  ─────┼────────────────────────────────────────────────────────");
    
    loop {
        // Calculate accelerated time
        let real_elapsed = start.elapsed();
        let sim_elapsed = Duration::from_secs_f64(real_elapsed.as_secs_f64() * acceleration);
        
        // Tick with simulated time
        let delta = Duration::from_millis(100);
        device.tick(delta);
        
        // Skip engine to current time
        engine.skip_to(sim_elapsed);
        let events = engine.tick(Duration::ZERO);
        
        // Print events
        for event in &events {
            let time_str = format!("{:>5.1}s", event.time.as_secs_f32());
            
            let event_str = match &event.event_type {
                EventType::VoiceInput(text) => format!("🎤 Voice: \"{}\"", text),
                EventType::Notification { title, body } => format!("🔔 {}: {}", title, body),
                EventType::SetScene(scene) => format!("📷 Scene change: {:?}", 
                    match scene {
                        TestScene::Kitchen => "Kitchen",
                        TestScene::Office => "Office",
                        TestScene::Street => "Street",
                        _ => "Other",
                    }),
                EventType::ActivityChange(activity) => format!("🚶 Activity: {:?}", activity),
                EventType::TimerAlarm(name) => format!("⏰ Timer: {} complete!", name),
                EventType::LocationChange(loc) => format!("📍 Location: ({:.2}, {:.2})", 
                    loc.latitude, loc.longitude),
                EventType::AmbientLight(lux) => format!("💡 Light: {} lux", lux),
                EventType::NavigationStep { instruction, distance } => 
                    format!("🧭 Nav: {} ({})", instruction, distance),
                _ => format!("📌 {}", event.description),
            };
            
            println!("  {} │ {}", time_str, event_str);
        }
        
        // Check if done
        if !engine.is_running {
            break;
        }
        
        // Don't run forever
        if real_elapsed > Duration::from_secs(30) {
            println!("  ─────┴────────────────────────────────────────────────────────");
            println!("  [Scenario complete or timed out]\n");
            break;
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    // Print summary
    println!();
    println!("  ╔═══════════════════════════════════════════════════════════╗");
    println!("  ║  SCENARIO SUMMARY                                         ║");
    println!("  ╠═══════════════════════════════════════════════════════════╣");
    println!("  ║  Total events triggered: {}                               ", 
        engine.triggered_events.len());
    println!("  ║  Device status: {:?}                            ", 
        device.power_state);
    println!("  ║  Battery: {:.0}%                                          ", 
        device.battery.percentage());
    println!("  ║  Temperature: {:.1}°C ({:?})                     ", 
        device.temperature_c, device.thermal_state);
    println!("  ╚═══════════════════════════════════════════════════════════╝\n");
    
    // Show object detection demo
    println!("  ────────────────────────────────────────────────────────────");
    println!("  OBJECT DETECTION DEMO");
    println!("  ────────────────────────────────────────────────────────────\n");
    
    device.camera.set_scene(TestScene::Kitchen);
    device.camera.start_capture();
    
    println!("  Scene: {}\n", device.camera.get_scene_description());
    println!("  Detected Objects:");
    for obj in device.camera.get_detected_objects() {
        let bar_len = (obj.confidence * 20.0) as usize;
        let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
        println!("    {:20} [{}] {:.0}%", obj.label, bar, obj.confidence * 100.0);
    }
    
    println!("\n  ════════════════════════════════════════════════════════════");
    println!("  Demo complete! Run 'cargo run --example simulator_demo' for");
    println!("  interactive mode.\n");
}
