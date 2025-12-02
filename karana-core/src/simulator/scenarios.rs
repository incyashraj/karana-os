//! Scenario System
//!
//! Pre-built scenarios for testing the smart glasses experience.
//! Each scenario simulates a real-world use case with timed events.

use super::input::{TestScene, VoiceCommand};
use super::sensors::{GestureType, Location, ActivityState};
use std::time::Duration;

/// An event that occurs during a scenario
#[derive(Debug, Clone)]
pub struct ScenarioEvent {
    pub time: Duration,
    pub event_type: EventType,
    pub description: String,
}

/// Types of events in a scenario
#[derive(Debug, Clone)]
pub enum EventType {
    /// Change the camera scene
    SetScene(TestScene),
    /// Inject a voice command
    VoiceInput(String),
    /// Show a notification
    Notification { title: String, body: String },
    /// Change location
    LocationChange(Location),
    /// Change activity state
    ActivityChange(ActivityState),
    /// Simulate a gesture
    Gesture(GestureType),
    /// Change ambient light
    AmbientLight(f32),
    /// Trigger timer alarm
    TimerAlarm(String),
    /// Incoming call
    IncomingCall { caller: String },
    /// Message received
    MessageReceived { from: String, text: String },
    /// Navigation instruction
    NavigationStep { instruction: String, distance: String },
    /// Custom event
    Custom(String),
}

/// A complete test scenario
#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub duration: Duration,
    pub events: Vec<ScenarioEvent>,
    pub initial_scene: TestScene,
    pub initial_location: Option<Location>,
}

impl Scenario {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            duration: Duration::from_secs(60),
            events: Vec::new(),
            initial_scene: TestScene::EmptyRoom,
            initial_location: None,
        }
    }

    /// Add an event at a specific time
    pub fn add_event(mut self, time_secs: u64, event: EventType, description: &str) -> Self {
        self.events.push(ScenarioEvent {
            time: Duration::from_secs(time_secs),
            event_type: event,
            description: description.to_string(),
        });
        self
    }

    /// Set the scenario duration
    pub fn with_duration(mut self, secs: u64) -> Self {
        self.duration = Duration::from_secs(secs);
        self
    }

    /// Set initial scene
    pub fn with_initial_scene(mut self, scene: TestScene) -> Self {
        self.initial_scene = scene;
        self
    }

    /// Set initial location
    pub fn with_initial_location(mut self, location: Location) -> Self {
        self.initial_location = Some(location);
        self
    }
}

/// Predefined scenarios for common use cases
pub struct ScenarioLibrary;

impl ScenarioLibrary {
    /// Morning routine scenario
    pub fn morning_routine() -> Scenario {
        Scenario::new("Morning Routine", "Wake up, check notifications, make coffee")
            .with_duration(300)
            .with_initial_scene(TestScene::EmptyRoom)
            .add_event(0, EventType::Notification {
                title: "Good Morning!".to_string(),
                body: "It's 7:00 AM. You have 3 meetings today.".to_string(),
            }, "System greeting")
            .add_event(5, EventType::VoiceInput("what's on my calendar today".to_string()), "User checks calendar")
            .add_event(15, EventType::SetScene(TestScene::Kitchen), "User walks to kitchen")
            .add_event(20, EventType::ActivityChange(ActivityState::Walking), "Walking detected")
            .add_event(25, EventType::ActivityChange(ActivityState::Stationary), "Stopped in kitchen")
            .add_event(30, EventType::VoiceInput("set a timer for 4 minutes for coffee".to_string()), "User sets coffee timer")
            .add_event(270, EventType::TimerAlarm("Coffee".to_string()), "Coffee timer goes off")
    }

    /// Office work scenario
    pub fn office_work() -> Scenario {
        Scenario::new("Office Work", "Working at desk with notifications and reminders")
            .with_duration(600)
            .with_initial_scene(TestScene::Office)
            .add_event(0, EventType::AmbientLight(500.0), "Indoor office lighting")
            .add_event(30, EventType::MessageReceived {
                from: "Boss".to_string(),
                text: "Can we sync at 2pm?".to_string(),
            }, "Message from boss")
            .add_event(60, EventType::VoiceInput("reply yes sounds good".to_string()), "User replies")
            .add_event(120, EventType::VoiceInput("remind me to send the report in 30 minutes".to_string()), "User sets reminder")
            .add_event(180, EventType::IncomingCall { caller: "Mom".to_string() }, "Incoming call")
            .add_event(300, EventType::Notification {
                title: "Reminder".to_string(),
                body: "Send the report".to_string(),
            }, "Reminder fires")
    }

    /// Navigation/walking scenario
    pub fn city_navigation() -> Scenario {
        let start_location = Location {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: 10.0,
            accuracy: 5.0,
            speed: 1.4,
            heading: 45.0,
        };

        Scenario::new("City Navigation", "Walking to a coffee shop with turn-by-turn directions")
            .with_duration(300)
            .with_initial_scene(TestScene::Street)
            .with_initial_location(start_location)
            .add_event(0, EventType::VoiceInput("navigate to Blue Bottle Coffee".to_string()), "User requests navigation")
            .add_event(5, EventType::NavigationStep {
                instruction: "Head north on Market St".to_string(),
                distance: "500 ft".to_string(),
            }, "First navigation step")
            .add_event(10, EventType::ActivityChange(ActivityState::Walking), "User starts walking")
            .add_event(60, EventType::NavigationStep {
                instruction: "Turn right on 3rd St".to_string(),
                distance: "200 ft".to_string(),
            }, "Turn instruction")
            .add_event(65, EventType::Gesture(GestureType::LookRight), "User looks right")
            .add_event(120, EventType::NavigationStep {
                instruction: "Destination on your left".to_string(),
                distance: "50 ft".to_string(),
            }, "Arriving")
            .add_event(150, EventType::SetScene(TestScene::Objects { 
                labels: vec!["coffee shop".to_string(), "door".to_string(), "menu".to_string()]
            }), "Arrived at coffee shop")
    }

    /// Cooking assistance scenario
    pub fn cooking_helper() -> Scenario {
        Scenario::new("Cooking Helper", "Following a recipe with timers")
            .with_duration(600)
            .with_initial_scene(TestScene::Kitchen)
            .add_event(0, EventType::VoiceInput("show me the pasta recipe".to_string()), "User requests recipe")
            .add_event(10, EventType::Notification {
                title: "Recipe: Pasta Carbonara".to_string(),
                body: "Step 1: Boil water".to_string(),
            }, "Recipe displayed")
            .add_event(30, EventType::VoiceInput("set a timer for 10 minutes for boiling water".to_string()), "Timer for water")
            .add_event(60, EventType::VoiceInput("what's the next step".to_string()), "User asks for next step")
            .add_event(65, EventType::Notification {
                title: "Recipe".to_string(),
                body: "Step 2: Cook bacon until crispy".to_string(),
            }, "Next step")
            .add_event(120, EventType::VoiceInput("set a timer for 5 minutes for bacon".to_string()), "Timer for bacon")
            .add_event(420, EventType::TimerAlarm("Bacon".to_string()), "Bacon timer")
            .add_event(630, EventType::TimerAlarm("Boiling water".to_string()), "Water timer")
    }

    /// Object identification scenario
    pub fn object_explorer() -> Scenario {
        Scenario::new("Object Explorer", "Walking around identifying objects with AI")
            .with_duration(180)
            .with_initial_scene(TestScene::EmptyRoom)
            .add_event(0, EventType::SetScene(TestScene::Office), "Start in office")
            .add_event(5, EventType::VoiceInput("what am I looking at".to_string()), "User asks about scene")
            .add_event(30, EventType::Gesture(GestureType::LookLeft), "User looks left")
            .add_event(35, EventType::VoiceInput("what's that".to_string()), "User asks about object")
            .add_event(60, EventType::SetScene(TestScene::Kitchen), "Move to kitchen")
            .add_event(65, EventType::VoiceInput("identify all objects".to_string()), "User wants full scan")
            .add_event(120, EventType::SetScene(TestScene::Objects {
                labels: vec!["keys".to_string(), "wallet".to_string(), "phone".to_string()],
            }), "Looking at personal items")
            .add_event(125, EventType::VoiceInput("remember where my keys are".to_string()), "User saves location")
    }

    /// Low light scenario
    pub fn night_mode() -> Scenario {
        Scenario::new("Night Mode", "Testing display adaptation in low light")
            .with_duration(120)
            .with_initial_scene(TestScene::EmptyRoom)
            .add_event(0, EventType::AmbientLight(500.0), "Normal indoor lighting")
            .add_event(30, EventType::AmbientLight(50.0), "Lights dimmed")
            .add_event(35, EventType::Notification {
                title: "Display".to_string(),
                body: "Adjusting brightness for low light".to_string(),
            }, "System adapts")
            .add_event(60, EventType::AmbientLight(5.0), "Near darkness")
            .add_event(90, EventType::VoiceInput("turn on flashlight".to_string()), "User needs light")
    }

    /// Get all available scenarios
    pub fn all() -> Vec<Scenario> {
        vec![
            Self::morning_routine(),
            Self::office_work(),
            Self::city_navigation(),
            Self::cooking_helper(),
            Self::object_explorer(),
            Self::night_mode(),
        ]
    }
}

/// Engine that runs scenarios
pub struct ScenarioEngine {
    pub current_scenario: Option<Scenario>,
    pub elapsed: Duration,
    pub is_running: bool,
    pub event_index: usize,
    pub triggered_events: Vec<ScenarioEvent>,
}

impl ScenarioEngine {
    pub fn new() -> Self {
        Self {
            current_scenario: None,
            elapsed: Duration::ZERO,
            is_running: false,
            event_index: 0,
            triggered_events: Vec::new(),
        }
    }

    /// Load a scenario
    pub fn load(&mut self, scenario: Scenario) {
        self.current_scenario = Some(scenario);
        self.elapsed = Duration::ZERO;
        self.is_running = false;
        self.event_index = 0;
        self.triggered_events.clear();
    }

    /// Start the scenario
    pub fn start(&mut self) {
        self.is_running = true;
        self.elapsed = Duration::ZERO;
        self.event_index = 0;
    }

    /// Pause the scenario
    pub fn pause(&mut self) {
        self.is_running = false;
    }

    /// Resume the scenario
    pub fn resume(&mut self) {
        self.is_running = true;
    }

    /// Stop and reset
    pub fn stop(&mut self) {
        self.is_running = false;
        self.elapsed = Duration::ZERO;
        self.event_index = 0;
        self.triggered_events.clear();
    }

    /// Advance time and get triggered events
    pub fn tick(&mut self, delta: Duration) -> Vec<ScenarioEvent> {
        if !self.is_running {
            return vec![];
        }

        self.elapsed += delta;

        let mut triggered = Vec::new();

        if let Some(ref scenario) = self.current_scenario {
            // Check for events that should trigger
            while self.event_index < scenario.events.len() {
                let event = &scenario.events[self.event_index];
                if event.time <= self.elapsed {
                    triggered.push(event.clone());
                    self.triggered_events.push(event.clone());
                    self.event_index += 1;
                } else {
                    break;
                }
            }

            // Check if scenario is complete
            if self.elapsed >= scenario.duration {
                self.is_running = false;
            }
        }

        triggered
    }

    /// Get progress as percentage
    pub fn progress(&self) -> f32 {
        if let Some(ref scenario) = self.current_scenario {
            (self.elapsed.as_secs_f32() / scenario.duration.as_secs_f32() * 100.0).min(100.0)
        } else {
            0.0
        }
    }

    /// Get scenario name
    pub fn current_name(&self) -> Option<&str> {
        self.current_scenario.as_ref().map(|s| s.name.as_str())
    }

    /// Skip to a specific time
    pub fn skip_to(&mut self, time: Duration) {
        self.elapsed = time;
        
        // Update event index
        if let Some(ref scenario) = self.current_scenario {
            self.event_index = scenario.events
                .iter()
                .position(|e| e.time > time)
                .unwrap_or(scenario.events.len());
        }
    }
}

impl Default for ScenarioEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_creation() {
        let scenario = ScenarioLibrary::morning_routine();
        assert_eq!(scenario.name, "Morning Routine");
        assert!(!scenario.events.is_empty());
    }

    #[test]
    fn test_scenario_engine() {
        let mut engine = ScenarioEngine::new();
        engine.load(ScenarioLibrary::morning_routine());
        engine.start();
        
        // Advance 10 seconds
        let events = engine.tick(Duration::from_secs(10));
        assert!(!events.is_empty());
    }

    #[test]
    fn test_all_scenarios() {
        let scenarios = ScenarioLibrary::all();
        assert!(scenarios.len() >= 5);
    }

    #[test]
    fn test_progress() {
        let mut engine = ScenarioEngine::new();
        let scenario = Scenario::new("Test", "Test")
            .with_duration(100);
        
        engine.load(scenario);
        engine.start();
        engine.tick(Duration::from_secs(50));
        
        assert!((engine.progress() - 50.0).abs() < 1.0);
    }
}
