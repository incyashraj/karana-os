//! Smart Glasses Simulator TUI
//!
//! An interactive terminal interface that simulates the smart glasses experience.
//! This allows testing the full Kāraṇa OS experience without hardware.

use super::*;
use super::device::{VirtualGlasses, DeviceStatus, PowerState};
use super::display::VirtualDisplay;
use super::sensors::{VirtualSensors, GestureType};
use super::input::{VirtualCamera, VirtualMicrophone, TestScene, VoiceCommand};
use super::scenarios::{ScenarioEngine, ScenarioLibrary, EventType};
use std::io::{self, Write, stdout};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// Commands available in the simulator
#[derive(Debug, Clone)]
pub enum SimCommand {
    // Device control
    Boot,
    Shutdown,
    Sleep,
    Wake,
    
    // Voice commands
    Voice(String),
    
    // Scene control
    SetScene(String),
    
    // Gestures
    Tap,
    SwipeLeft,
    SwipeRight,
    NodYes,
    ShakeNo,
    LookLeft,
    LookRight,
    
    // Scenarios
    LoadScenario(String),
    StartScenario,
    PauseScenario,
    StopScenario,
    ListScenarios,
    
    // Display
    Brightness(f32),
    ShowNotification(String),
    ClearDisplay,
    
    // Sensors
    SetLocation(f64, f64),
    SetLight(f32),
    SimulateWalking,
    SimulateStationary,
    
    // Info
    Status,
    Help,
    Quit,
}

/// Parse a command string
fn parse_command(input: &str) -> Option<SimCommand> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0].to_lowercase().as_str() {
        "boot" => Some(SimCommand::Boot),
        "shutdown" | "off" => Some(SimCommand::Shutdown),
        "sleep" => Some(SimCommand::Sleep),
        "wake" => Some(SimCommand::Wake),
        
        "voice" | "say" | "v" => {
            let text = parts[1..].join(" ");
            if text.is_empty() {
                None
            } else {
                Some(SimCommand::Voice(text))
            }
        }
        
        "scene" => {
            parts.get(1).map(|s| SimCommand::SetScene(s.to_string()))
        }
        
        "tap" => Some(SimCommand::Tap),
        "swipe-left" | "sl" => Some(SimCommand::SwipeLeft),
        "swipe-right" | "sr" => Some(SimCommand::SwipeRight),
        "nod" => Some(SimCommand::NodYes),
        "shake" => Some(SimCommand::ShakeNo),
        "look-left" | "ll" => Some(SimCommand::LookLeft),
        "look-right" | "lr" => Some(SimCommand::LookRight),
        
        "scenario" | "sc" => {
            parts.get(1).map(|s| SimCommand::LoadScenario(s.to_string()))
        }
        "start" => Some(SimCommand::StartScenario),
        "pause" => Some(SimCommand::PauseScenario),
        "stop" => Some(SimCommand::StopScenario),
        "scenarios" | "list" => Some(SimCommand::ListScenarios),
        
        "brightness" | "bright" => {
            parts.get(1).and_then(|s| s.parse().ok()).map(SimCommand::Brightness)
        }
        "notify" => {
            let text = parts[1..].join(" ");
            if text.is_empty() {
                None
            } else {
                Some(SimCommand::ShowNotification(text))
            }
        }
        "clear" => Some(SimCommand::ClearDisplay),
        
        "location" | "loc" => {
            if parts.len() >= 3 {
                let lat = parts[1].parse().ok()?;
                let lon = parts[2].parse().ok()?;
                Some(SimCommand::SetLocation(lat, lon))
            } else {
                None
            }
        }
        "light" => {
            parts.get(1).and_then(|s| s.parse().ok()).map(SimCommand::SetLight)
        }
        "walk" | "walking" => Some(SimCommand::SimulateWalking),
        "stand" | "stationary" => Some(SimCommand::SimulateStationary),
        
        "status" | "stat" => Some(SimCommand::Status),
        "help" | "h" | "?" => Some(SimCommand::Help),
        "quit" | "exit" | "q" => Some(SimCommand::Quit),
        
        _ => None,
    }
}

/// The interactive simulator TUI
pub struct SimulatorTUI {
    device: Arc<Mutex<VirtualGlasses>>,
    scenario_engine: Arc<Mutex<ScenarioEngine>>,
    last_tick: Instant,
    notification_id: u32,
}

impl SimulatorTUI {
    pub fn new(profile: DeviceProfile) -> Self {
        let config = profile.config();
        Self {
            device: Arc::new(Mutex::new(VirtualGlasses::new(config))),
            scenario_engine: Arc::new(Mutex::new(ScenarioEngine::new())),
            last_tick: Instant::now(),
            notification_id: 0,
        }
    }

    /// Print the help message
    fn print_help(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║            KĀRAṆA SMART GLASSES SIMULATOR                        ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║ DEVICE CONTROL                                                   ║");
        println!("║   boot          - Power on the glasses                           ║");
        println!("║   shutdown      - Power off                                      ║");
        println!("║   sleep/wake    - Sleep/wake mode                                ║");
        println!("║                                                                  ║");
        println!("║ VOICE COMMANDS                                                   ║");
        println!("║   voice <text>  - Simulate voice input                           ║");
        println!("║   say <text>    - Alias for voice                                ║");
        println!("║                                                                  ║");
        println!("║ GESTURES                                                         ║");
        println!("║   tap           - Tap gesture                                    ║");
        println!("║   swipe-left/sl - Swipe left                                     ║");
        println!("║   swipe-right/sr- Swipe right                                    ║");
        println!("║   nod/shake     - Head nod yes / shake no                        ║");
        println!("║   look-left/ll  - Look left                                      ║");
        println!("║   look-right/lr - Look right                                     ║");
        println!("║                                                                  ║");
        println!("║ SCENES                                                           ║");
        println!("║   scene <name>  - Set scene (office/kitchen/street/empty)        ║");
        println!("║                                                                  ║");
        println!("║ SCENARIOS                                                        ║");
        println!("║   scenarios     - List available scenarios                       ║");
        println!("║   scenario <n>  - Load scenario by number                        ║");
        println!("║   start/pause   - Start/pause scenario                           ║");
        println!("║   stop          - Stop scenario                                  ║");
        println!("║                                                                  ║");
        println!("║ DISPLAY                                                          ║");
        println!("║   brightness <n>- Set brightness (0.0-1.0)                       ║");
        println!("║   notify <text> - Show notification                              ║");
        println!("║   clear         - Clear display                                  ║");
        println!("║                                                                  ║");
        println!("║ SENSORS                                                          ║");
        println!("║   location <lat> <lon> - Set GPS location                        ║");
        println!("║   light <lux>   - Set ambient light                              ║");
        println!("║   walk/stand    - Simulate walking/stationary                    ║");
        println!("║                                                                  ║");
        println!("║ OTHER                                                            ║");
        println!("║   status        - Show device status                             ║");
        println!("║   help          - Show this help                                 ║");
        println!("║   quit          - Exit simulator                                 ║");
        println!("╚══════════════════════════════════════════════════════════════════╝\n");
    }

    /// Print the current display
    fn render_display(&self) {
        let device = self.device.lock().unwrap();
        
        if !device.is_operational() {
            println!("\n  [Device is {:?}]\n", device.power_state);
            return;
        }

        let lines = device.display.render_to_text(80, 20);
        println!();
        for line in lines {
            println!("  {}", line);
        }
        println!();
    }

    /// Print device status
    fn print_status(&self) {
        let device = self.device.lock().unwrap();
        let status = device.status_summary();
        let sensors = device.sensors.status_summary();
        
        println!("\n┌─────────────── DEVICE STATUS ───────────────┐");
        println!("│ {}", status);
        println!("│ {}", sensors);
        println!("│ Display: {} | Camera: {} | Scene: {:?}", 
            if device.display.is_on { "ON" } else { "OFF" },
            if device.camera.is_capturing { "Recording" } else { "Idle" },
            "Current");
        
        // Scenario status
        let scenario = self.scenario_engine.lock().unwrap();
        if let Some(name) = scenario.current_name() {
            println!("│ Scenario: {} ({:.0}%) {}", 
                name,
                scenario.progress(),
                if scenario.is_running { "▶ Running" } else { "⏸ Paused" }
            );
        }
        println!("└──────────────────────────────────────────────┘\n");
    }

    /// List available scenarios
    fn list_scenarios(&self) {
        println!("\n┌─────────── AVAILABLE SCENARIOS ───────────┐");
        for (i, scenario) in ScenarioLibrary::all().iter().enumerate() {
            println!("│ {}. {} - {}", i + 1, scenario.name, scenario.description);
        }
        println!("│                                            │");
        println!("│ Usage: scenario <number>                   │");
        println!("└────────────────────────────────────────────┘\n");
    }

    /// Handle a command
    fn handle_command(&mut self, cmd: SimCommand) -> bool {
        match cmd {
            SimCommand::Boot => {
                let mut device = self.device.lock().unwrap();
                match device.boot() {
                    Ok(_) => {
                        println!("✓ Device booted successfully");
                        device.display.show_status_bar("7:00 AM", "100%", "●●●");
                    }
                    Err(e) => println!("✗ Boot failed: {}", e),
                }
            }
            
            SimCommand::Shutdown => {
                let mut device = self.device.lock().unwrap();
                device.shutdown();
                println!("✓ Device shutdown");
            }
            
            SimCommand::Sleep => {
                let mut device = self.device.lock().unwrap();
                device.sleep();
                println!("✓ Device sleeping");
            }
            
            SimCommand::Wake => {
                let mut device = self.device.lock().unwrap();
                device.wake();
                println!("✓ Device awake");
            }
            
            SimCommand::Voice(text) => {
                let mut device = self.device.lock().unwrap();
                device.microphone.inject_text(&text);
                device.display.show_voice_indicator(true);
                println!("🎤 Voice input: \"{}\"", text);
                
                // Simulate processing
                std::thread::sleep(Duration::from_millis(100));
                device.display.show_voice_indicator(false);
                
                // Show a response notification
                self.notification_id += 1;
                device.display.show_notification(
                    &format!("response_{}", self.notification_id),
                    &format!("Processing: {}", text),
                    3000
                );
            }
            
            SimCommand::SetScene(scene_name) => {
                let mut device = self.device.lock().unwrap();
                let scene = match scene_name.to_lowercase().as_str() {
                    "office" => TestScene::Office,
                    "kitchen" => TestScene::Kitchen,
                    "street" => TestScene::Street,
                    "empty" | "room" => TestScene::EmptyRoom,
                    _ => {
                        println!("Unknown scene. Available: office, kitchen, street, empty");
                        return true;
                    }
                };
                device.camera.set_scene(scene.clone());
                println!("✓ Scene set to: {:?}", scene_name);
                
                // Show detected objects
                let objects = device.camera.get_detected_objects();
                if !objects.is_empty() {
                    println!("  Detected objects:");
                    for obj in objects {
                        println!("    - {} ({:.0}%)", obj.label, obj.confidence * 100.0);
                    }
                }
            }
            
            SimCommand::Tap => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_tap();
                println!("👆 Tap gesture");
            }
            
            SimCommand::SwipeLeft => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_swipe(GestureType::SwipeLeft);
                println!("👈 Swipe left");
            }
            
            SimCommand::SwipeRight => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_swipe(GestureType::SwipeRight);
                println!("👉 Swipe right");
            }
            
            SimCommand::NodYes => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_nod_yes();
                println!("✓ Nod yes gesture");
            }
            
            SimCommand::ShakeNo => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_shake_no();
                println!("✗ Shake no gesture");
            }
            
            SimCommand::LookLeft => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_look_left();
                println!("◀ Look left");
            }
            
            SimCommand::LookRight => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_look_right();
                println!("▶ Look right");
            }
            
            SimCommand::LoadScenario(idx) => {
                if let Ok(n) = idx.parse::<usize>() {
                    let scenarios = ScenarioLibrary::all();
                    if n > 0 && n <= scenarios.len() {
                        let scenario = scenarios[n - 1].clone();
                        let mut engine = self.scenario_engine.lock().unwrap();
                        engine.load(scenario.clone());
                        println!("✓ Loaded scenario: {}", scenario.name);
                    } else {
                        println!("Invalid scenario number. Use 'scenarios' to list available.");
                    }
                }
            }
            
            SimCommand::StartScenario => {
                let mut engine = self.scenario_engine.lock().unwrap();
                if engine.current_scenario.is_some() {
                    engine.start();
                    println!("▶ Scenario started");
                } else {
                    println!("No scenario loaded. Use 'scenario <n>' first.");
                }
            }
            
            SimCommand::PauseScenario => {
                let mut engine = self.scenario_engine.lock().unwrap();
                engine.pause();
                println!("⏸ Scenario paused");
            }
            
            SimCommand::StopScenario => {
                let mut engine = self.scenario_engine.lock().unwrap();
                engine.stop();
                println!("⏹ Scenario stopped");
            }
            
            SimCommand::ListScenarios => {
                self.list_scenarios();
            }
            
            SimCommand::Brightness(level) => {
                let mut device = self.device.lock().unwrap();
                device.display.dim(level);
                println!("✓ Brightness set to {:.0}%", level * 100.0);
            }
            
            SimCommand::ShowNotification(text) => {
                self.notification_id += 1;
                let mut device = self.device.lock().unwrap();
                device.display.show_notification(
                    &format!("notif_{}", self.notification_id),
                    &text,
                    5000
                );
                println!("🔔 Notification shown");
            }
            
            SimCommand::ClearDisplay => {
                let mut device = self.device.lock().unwrap();
                device.display.clear();
                println!("✓ Display cleared");
            }
            
            SimCommand::SetLocation(lat, lon) => {
                let mut device = self.device.lock().unwrap();
                device.sensors.set_location(lat, lon);
                println!("📍 Location set to ({:.4}, {:.4})", lat, lon);
            }
            
            SimCommand::SetLight(lux) => {
                let mut device = self.device.lock().unwrap();
                device.sensors.set_ambient_light(lux);
                println!("💡 Ambient light set to {} lux", lux);
            }
            
            SimCommand::SimulateWalking => {
                let mut device = self.device.lock().unwrap();
                device.sensors.simulate_walking();
                println!("🚶 Simulating walking");
            }
            
            SimCommand::SimulateStationary => {
                let device = self.device.lock().unwrap();
                // Reset to stationary
                println!("🧍 Simulating stationary");
            }
            
            SimCommand::Status => {
                self.print_status();
            }
            
            SimCommand::Help => {
                self.print_help();
            }
            
            SimCommand::Quit => {
                println!("Goodbye! 👋");
                return false;
            }
        }
        
        true
    }

    /// Tick the simulation
    fn tick(&mut self) {
        let delta = self.last_tick.elapsed();
        self.last_tick = Instant::now();

        // Update device
        {
            let mut device = self.device.lock().unwrap();
            device.tick(delta);
        }

        // Process scenario events
        let events = {
            let mut engine = self.scenario_engine.lock().unwrap();
            engine.tick(delta)
        };

        // Handle scenario events
        for event in events {
            println!("📌 Scenario event: {}", event.description);
            
            match event.event_type {
                EventType::VoiceInput(text) => {
                    let mut device = self.device.lock().unwrap();
                    device.microphone.inject_text(&text);
                    println!("   🎤 Voice: \"{}\"", text);
                }
                EventType::Notification { title, body } => {
                    self.notification_id += 1;
                    let mut device = self.device.lock().unwrap();
                    device.display.show_notification(
                        &format!("scenario_{}", self.notification_id),
                        &format!("{}: {}", title, body),
                        5000
                    );
                }
                EventType::SetScene(scene) => {
                    let mut device = self.device.lock().unwrap();
                    device.camera.set_scene(scene);
                }
                EventType::LocationChange(loc) => {
                    let mut device = self.device.lock().unwrap();
                    device.sensors.set_location(loc.latitude, loc.longitude);
                }
                EventType::AmbientLight(lux) => {
                    let mut device = self.device.lock().unwrap();
                    device.sensors.set_ambient_light(lux);
                }
                EventType::NavigationStep { instruction, distance } => {
                    let mut device = self.device.lock().unwrap();
                    device.display.show_navigation(&instruction, &distance);
                }
                EventType::TimerAlarm(name) => {
                    self.notification_id += 1;
                    let mut device = self.device.lock().unwrap();
                    device.display.show_notification(
                        &format!("alarm_{}", self.notification_id),
                        &format!("⏰ Timer: {} complete!", name),
                        10000
                    );
                }
                _ => {}
            }
        }
    }

    /// Run the interactive simulator
    pub fn run(&mut self) {
        // Print welcome
        println!("\n");
        println!("  ╔═══════════════════════════════════════════════════╗");
        println!("  ║      KĀRAṆA SMART GLASSES SIMULATOR v1.0          ║");
        println!("  ║                                                   ║");
        println!("  ║  Type 'help' for commands, 'quit' to exit         ║");
        println!("  ║  Type 'boot' to power on the virtual device       ║");
        println!("  ╚═══════════════════════════════════════════════════╝");
        println!();

        let mut running = true;
        
        while running {
            // Update simulation
            self.tick();

            // Show prompt
            let scenario = self.scenario_engine.lock().unwrap();
            let scenario_indicator = if scenario.is_running {
                format!(" [{}:{:.0}%]", 
                    scenario.current_name().unwrap_or(""), 
                    scenario.progress())
            } else {
                String::new()
            };
            drop(scenario);

            print!("glasses{} > ", scenario_indicator);
            stdout().flush().unwrap();

            // Read input
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                continue;
            }

            // Parse and handle command
            if let Some(cmd) = parse_command(&input) {
                running = self.handle_command(cmd);
                
                // Render display after command
                let device = self.device.lock().unwrap();
                if device.is_operational() {
                    drop(device);
                    self.render_display();
                }
            } else if !input.trim().is_empty() {
                println!("Unknown command. Type 'help' for available commands.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parsing() {
        assert!(matches!(parse_command("boot"), Some(SimCommand::Boot)));
        assert!(matches!(parse_command("voice hello"), Some(SimCommand::Voice(_))));
        assert!(matches!(parse_command("quit"), Some(SimCommand::Quit)));
    }

    #[test]
    fn test_tui_creation() {
        let tui = SimulatorTUI::new(DeviceProfile::XrealAir);
        let device = tui.device.lock().unwrap();
        assert_eq!(device.power_state, PowerState::Off);
    }
}
