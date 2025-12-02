//! Kāraṇa Smart Glasses Visual Simulator
//!
//! A proper interactive TUI that simulates the smart glasses experience
//! with real AI responses, visual HUD, and full functionality.

use std::io::{self, Write, stdout, stdin};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const BLUE: &str = "\x1b[34m";
const WHITE: &str = "\x1b[97m";
const RED: &str = "\x1b[31m";
const BG_BLACK: &str = "\x1b[40m";
const BG_BLUE: &str = "\x1b[44m";

/// Clear screen and move cursor to top
fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    stdout().flush().unwrap();
}

/// Move cursor to position
fn goto(x: u16, y: u16) {
    print!("\x1b[{};{}H", y, x);
}

/// Current scene being viewed
#[derive(Clone, Debug)]
enum Scene {
    Home,
    Kitchen,
    Office,
    Street,
    Store,
}

impl Scene {
    fn description(&self) -> &str {
        match self {
            Scene::Home => "Living room with couch, TV, coffee table",
            Scene::Kitchen => "Kitchen with coffee maker, toaster, fruit bowl",
            Scene::Office => "Desk with monitor, keyboard, coffee mug, papers",
            Scene::Street => "City street with cars, pedestrians, shops",
            Scene::Store => "Grocery store aisle with products on shelves",
        }
    }

    fn objects(&self) -> Vec<(&str, f32)> {
        match self {
            Scene::Home => vec![
                ("couch", 0.95), ("TV", 0.93), ("coffee table", 0.91), 
                ("lamp", 0.88), ("bookshelf", 0.85)
            ],
            Scene::Kitchen => vec![
                ("coffee maker", 0.94), ("toaster", 0.92), ("apple", 0.89),
                ("banana", 0.87), ("cutting board", 0.85), ("knife", 0.82)
            ],
            Scene::Office => vec![
                ("monitor", 0.96), ("keyboard", 0.94), ("mouse", 0.92),
                ("coffee mug", 0.90), ("notebook", 0.87), ("pen", 0.84)
            ],
            Scene::Street => vec![
                ("car", 0.95), ("person", 0.93), ("traffic light", 0.91),
                ("crosswalk", 0.88), ("bus stop", 0.85), ("bicycle", 0.82)
            ],
            Scene::Store => vec![
                ("shopping cart", 0.94), ("shelf", 0.92), ("cereal box", 0.89),
                ("milk carton", 0.87), ("bread", 0.85), ("price tag", 0.83)
            ],
        }
    }
}

/// Notification to display
#[derive(Clone)]
struct Notification {
    title: String,
    body: String,
    icon: &'static str,
    created: Instant,
}

/// Timer
#[derive(Clone)]
struct Timer {
    name: String,
    duration: Duration,
    started: Instant,
}

impl Timer {
    fn remaining(&self) -> Duration {
        let elapsed = self.started.elapsed();
        if elapsed >= self.duration {
            Duration::ZERO
        } else {
            self.duration - elapsed
        }
    }

    fn is_done(&self) -> bool {
        self.started.elapsed() >= self.duration
    }

    fn format_remaining(&self) -> String {
        let rem = self.remaining();
        let mins = rem.as_secs() / 60;
        let secs = rem.as_secs() % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

/// Memory item
#[derive(Clone)]
struct Memory {
    item: String,
    location: String,
    scene: Scene,
}

/// Glasses state
struct GlassesState {
    scene: Scene,
    notifications: Vec<Notification>,
    timers: Vec<Timer>,
    memories: Vec<Memory>,
    battery: f32,
    time: String,
    is_listening: bool,
    last_ai_response: String,
    conversation_history: Vec<(String, String)>, // (user, ai)
    navigation_active: bool,
    navigation_dest: String,
    navigation_steps: Vec<String>,
    nav_step_idx: usize,
}

impl GlassesState {
    fn new() -> Self {
        Self {
            scene: Scene::Home,
            notifications: vec![],
            timers: vec![],
            memories: vec![],
            battery: 100.0,
            time: "10:30 AM".to_string(),
            is_listening: false,
            last_ai_response: String::new(),
            conversation_history: vec![],
            navigation_active: false,
            navigation_dest: String::new(),
            navigation_steps: vec![],
            nav_step_idx: 0,
        }
    }

    fn add_notification(&mut self, title: &str, body: &str, icon: &'static str) {
        self.notifications.push(Notification {
            title: title.to_string(),
            body: body.to_string(),
            icon,
            created: Instant::now(),
        });
    }

    fn add_timer(&mut self, name: &str, seconds: u64) {
        self.timers.push(Timer {
            name: name.to_string(),
            duration: Duration::from_secs(seconds),
            started: Instant::now(),
        });
    }

    fn remember(&mut self, item: &str, location: &str) {
        self.memories.push(Memory {
            item: item.to_string(),
            location: location.to_string(),
            scene: self.scene.clone(),
        });
    }

    fn find_memory(&self, query: &str) -> Option<&Memory> {
        let query_lower = query.to_lowercase();
        self.memories.iter().find(|m| m.item.to_lowercase().contains(&query_lower))
    }

    fn start_navigation(&mut self, destination: &str) {
        self.navigation_active = true;
        self.navigation_dest = destination.to_string();
        self.navigation_steps = vec![
            format!("Head north on Main St for 200m"),
            format!("Turn right onto Oak Ave"),
            format!("Continue for 150m"),
            format!("Turn left onto {}", destination),
            format!("Destination on your right"),
        ];
        self.nav_step_idx = 0;
    }
}

/// Process AI command and return response
fn process_ai_command(input: &str, state: &mut GlassesState) -> String {
    let input_lower = input.to_lowercase();
    
    // Timer commands
    if input_lower.contains("timer") || input_lower.contains("remind me in") {
        if let Some(mins) = extract_minutes(&input_lower) {
            let name = if input_lower.contains("coffee") {
                "Coffee"
            } else if input_lower.contains("tea") {
                "Tea"
            } else if input_lower.contains("cook") || input_lower.contains("food") {
                "Cooking"
            } else if input_lower.contains("break") {
                "Break"
            } else {
                "Timer"
            };
            state.add_timer(name, mins * 60);
            return format!("✓ {} timer set for {} minutes", name, mins);
        }
        return "I'll set a timer. How many minutes?".to_string();
    }
    
    // What am I looking at?
    if input_lower.contains("looking at") || input_lower.contains("what do you see") 
        || input_lower.contains("what's in front") || input_lower.contains("identify") {
        let objects = state.scene.objects();
        let top_objects: Vec<_> = objects.iter().take(4).collect();
        let obj_list = top_objects.iter()
            .map(|(name, conf)| format!("{} ({:.0}%)", name, conf * 100.0))
            .collect::<Vec<_>>()
            .join(", ");
        return format!("I can see: {}. {}", obj_list, state.scene.description());
    }
    
    // Remember location
    if input_lower.contains("remember") && (input_lower.contains("where") || input_lower.contains("location")) {
        if let Some(item) = extract_item_to_remember(&input_lower) {
            let location = state.scene.description().to_string();
            state.remember(&item, &location);
            return format!("✓ I'll remember your {} is in the {:?} area", item, state.scene);
        }
        return "What would you like me to remember the location of?".to_string();
    }
    
    // Find something
    if input_lower.contains("where") && (input_lower.contains("my") || input_lower.contains("find")) {
        let query = extract_search_query(&input_lower);
        if let Some(memory) = state.find_memory(&query) {
            return format!("Your {} was last seen in: {}", memory.item, memory.location);
        }
        return format!("I don't have a saved location for '{}'. Say 'remember where my {}' when you see it.", query, query);
    }
    
    // Navigation
    if input_lower.contains("navigate") || input_lower.contains("directions") 
        || input_lower.contains("how do i get to") || input_lower.contains("take me to") {
        let dest = extract_destination(&input_lower);
        state.start_navigation(&dest);
        return format!("🧭 Starting navigation to {}. {}", dest, state.navigation_steps[0]);
    }
    
    // Next navigation step
    if input_lower.contains("next") && state.navigation_active {
        if state.nav_step_idx < state.navigation_steps.len() - 1 {
            state.nav_step_idx += 1;
            return format!("🧭 {}", state.navigation_steps[state.nav_step_idx]);
        } else {
            state.navigation_active = false;
            return "✓ You have arrived at your destination!".to_string();
        }
    }
    
    // Weather
    if input_lower.contains("weather") {
        return "☀️ Currently 72°F (22°C), sunny with light clouds. High of 78°F expected.".to_string();
    }
    
    // Time
    if input_lower.contains("time") && !input_lower.contains("timer") {
        return format!("🕐 The current time is {}", state.time);
    }
    
    // Calendar/schedule
    if input_lower.contains("calendar") || input_lower.contains("schedule") || input_lower.contains("meeting") {
        return "📅 You have 2 meetings today:\n  • 2:00 PM - Team standup\n  • 4:30 PM - Product review".to_string();
    }
    
    // Messages
    if input_lower.contains("message") || input_lower.contains("notification") {
        state.add_notification("New Message", "Mom: Don't forget dinner tonight!", "💬");
        return "💬 You have 1 new message from Mom: 'Don't forget dinner tonight!'".to_string();
    }
    
    // Take photo
    if input_lower.contains("photo") || input_lower.contains("picture") || input_lower.contains("capture") {
        state.add_notification("Photo Captured", "Saved to gallery", "📸");
        return "📸 Photo captured and saved!".to_string();
    }
    
    // Battery
    if input_lower.contains("battery") {
        return format!("🔋 Battery at {:.0}%", state.battery);
    }
    
    // Help
    if input_lower.contains("help") || input_lower.contains("what can you") {
        return "I can help you with:\n  • \"What am I looking at?\" - Identify objects\n  • \"Set timer for X minutes\"\n  • \"Remember where my keys are\"\n  • \"Where are my keys?\"\n  • \"Navigate to [place]\"\n  • \"What's the weather?\"\n  • \"Check my calendar\"".to_string();
    }
    
    // Scene change commands (for demo)
    if input_lower.contains("go to kitchen") || input_lower.contains("enter kitchen") {
        state.scene = Scene::Kitchen;
        return "📍 You are now in the kitchen.".to_string();
    }
    if input_lower.contains("go to office") || input_lower.contains("enter office") {
        state.scene = Scene::Office;
        return "📍 You are now in the office.".to_string();
    }
    if input_lower.contains("go outside") || input_lower.contains("go to street") {
        state.scene = Scene::Street;
        return "📍 You are now outside on the street.".to_string();
    }
    if input_lower.contains("go home") || input_lower.contains("go to living") {
        state.scene = Scene::Home;
        return "📍 You are now in the living room.".to_string();
    }
    if input_lower.contains("go to store") || input_lower.contains("enter store") {
        state.scene = Scene::Store;
        return "📍 You are now in the grocery store.".to_string();
    }
    
    // General AI response
    "I'm Kāraṇa, your AI assistant. Try asking 'What am I looking at?' or 'Set a timer for 5 minutes'. Say 'help' for more options.".to_string()
}

fn extract_minutes(input: &str) -> Option<u64> {
    for word in input.split_whitespace() {
        if let Ok(n) = word.parse::<u64>() {
            return Some(n);
        }
    }
    // Handle written numbers
    if input.contains("one") { return Some(1); }
    if input.contains("two") { return Some(2); }
    if input.contains("three") { return Some(3); }
    if input.contains("five") { return Some(5); }
    if input.contains("ten") { return Some(10); }
    if input.contains("fifteen") { return Some(15); }
    if input.contains("twenty") { return Some(20); }
    if input.contains("thirty") { return Some(30); }
    None
}

fn extract_item_to_remember(input: &str) -> Option<String> {
    let keywords = ["keys", "wallet", "phone", "glasses", "bag", "laptop", "book", "remote"];
    for kw in keywords {
        if input.contains(kw) {
            return Some(kw.to_string());
        }
    }
    // Try to extract after "my"
    if let Some(pos) = input.find("my ") {
        let after = &input[pos + 3..];
        let word = after.split_whitespace().next()?;
        return Some(word.to_string());
    }
    None
}

fn extract_search_query(input: &str) -> String {
    let keywords = ["keys", "wallet", "phone", "glasses", "bag", "laptop", "book", "remote"];
    for kw in keywords {
        if input.contains(kw) {
            return kw.to_string();
        }
    }
    "item".to_string()
}

fn extract_destination(input: &str) -> String {
    let places = ["coffee shop", "store", "home", "office", "restaurant", "gym", "park", "station"];
    for place in places {
        if input.contains(place) {
            return place.to_string();
        }
    }
    // Extract after "to"
    if let Some(pos) = input.rfind(" to ") {
        return input[pos + 4..].trim().to_string();
    }
    "destination".to_string()
}

/// Render the glasses HUD display
fn render_display(state: &GlassesState, width: usize, height: usize) {
    clear_screen();
    
    // Top border with gradient effect
    println!("{}{}", BG_BLACK, CYAN);
    println!("  ╔{}╗", "═".repeat(width - 4));
    
    // Status bar
    let status = format!(
        "{}  {}  🔋 {:.0}%  📶 ●●●  {}",
        state.time,
        if state.is_listening { "🎤" } else { "  " },
        state.battery,
        match &state.scene {
            Scene::Home => "🏠 Home",
            Scene::Kitchen => "🍳 Kitchen", 
            Scene::Office => "💼 Office",
            Scene::Street => "🛣️ Street",
            Scene::Store => "🛒 Store",
        }
    );
    let padding = width - 6 - strip_ansi_len(&status);
    println!("  ║ {}{}{} ║", WHITE, status, " ".repeat(padding.max(0)));
    println!("  ╟{}╢", "─".repeat(width - 4));
    
    // Main content area
    let content_height = height - 12;
    
    // Scene view (camera feed representation)
    println!("  ║{}{}  ┌─── 📷 CAMERA VIEW ───┐{}", RESET, DIM, " ".repeat(width - 28));
    println!("  ║{}  │ {} │{}", DIM, truncate(state.scene.description(), 22), " ".repeat(width - 28));
    println!("  ║{}  └──────────────────────┘{}{}", DIM, RESET, " ".repeat(width - 28));
    
    // Detected objects overlay
    let objects = state.scene.objects();
    println!("  ║{}", " ".repeat(width - 4));
    println!("  ║  {}{}DETECTED OBJECTS:{}", CYAN, BOLD, RESET);
    for (i, (obj, conf)) in objects.iter().take(4).enumerate() {
        let bar_len = (*conf * 15.0) as usize;
        let bar = format!("{}{}",
            GREEN.to_string() + &"█".repeat(bar_len),
            DIM.to_string() + &"░".repeat(15 - bar_len) + RESET
        );
        let line = format!("    {} {:15} [{}] {:.0}%", 
            ["▸", "▸", "▸", "▸"][i], obj, bar, conf * 100.0);
        println!("  ║  {}{}", line, " ".repeat((width - 8).saturating_sub(50)));
    }
    
    // Active timer display
    if !state.timers.is_empty() {
        println!("  ║{}", " ".repeat(width - 4));
        println!("  ║  {}{}⏱️  ACTIVE TIMERS:{}", YELLOW, BOLD, RESET);
        for timer in &state.timers {
            if !timer.is_done() {
                println!("  ║    {} {} - {}{}", 
                    YELLOW, timer.name, timer.format_remaining(), 
                    " ".repeat(width - 30));
            }
        }
    }
    
    // Navigation overlay
    if state.navigation_active {
        println!("  ║{}", " ".repeat(width - 4));
        println!("  ║  {}{}🧭 NAVIGATION:{}", BLUE, BOLD, RESET);
        println!("  ║    {} To: {}{}", BLUE, state.navigation_dest, " ".repeat(width - 20));
        if state.nav_step_idx < state.navigation_steps.len() {
            println!("  ║    {} {}{}", 
                CYAN, state.navigation_steps[state.nav_step_idx], 
                " ".repeat((width - 8).saturating_sub(state.navigation_steps[state.nav_step_idx].len())));
        }
    }
    
    // Notifications
    let recent_notifs: Vec<_> = state.notifications.iter()
        .filter(|n| n.created.elapsed() < Duration::from_secs(10))
        .collect();
    if !recent_notifs.is_empty() {
        println!("  ║{}", " ".repeat(width - 4));
        for notif in recent_notifs.iter().take(2) {
            println!("  ║  ┌{}┐", "─".repeat(width - 8));
            println!("  ║  │ {} {}{}: {}{} │", 
                notif.icon, BOLD, notif.title, RESET, 
                truncate(&notif.body, width - 20));
            println!("  ║  └{}┘", "─".repeat(width - 8));
        }
    }
    
    // AI Response area
    println!("  ║{}", " ".repeat(width - 4));
    println!("  ╟{}╢", "─".repeat(width - 4));
    println!("  ║  {}{}🤖 KĀRAṆA AI:{}", MAGENTA, BOLD, RESET);
    
    // Wrap AI response
    let response_lines = wrap_text(&state.last_ai_response, width - 8);
    for line in response_lines.iter().take(4) {
        println!("  ║    {}{}{}", WHITE, line, " ".repeat((width - 8).saturating_sub(line.len())));
    }
    
    // Fill remaining space
    let used_lines = 15 + state.timers.len().min(2) + recent_notifs.len().min(2) * 3 
        + if state.navigation_active { 3 } else { 0 }
        + response_lines.len().min(4);
    for _ in used_lines..content_height {
        println!("  ║{}", " ".repeat(width - 4));
    }
    
    // Input area
    println!("  ╟{}╢", "─".repeat(width - 4));
    println!("  ║  {}💬 Say something (or type command):{}", CYAN, " ".repeat(width - 42));
    println!("  ╚{}╝{}", "═".repeat(width - 4), RESET);
    
    // Command hint
    println!("{}  Commands: [scene] kitchen/office/street | [quit] exit | Or speak naturally{}", DIM, RESET);
    print!("  {}>>{} ", GREEN, RESET);
    stdout().flush().unwrap();
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}

fn strip_ansi_len(s: &str) -> usize {
    // Simple estimate - just count visible chars
    s.chars().filter(|c| !c.is_ascii_control()).count()
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > width {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn main() {
    // Welcome screen
    clear_screen();
    println!("\n\n");
    println!("{}{}    ╔═══════════════════════════════════════════════════════════╗", CYAN, BOLD);
    println!("    ║                                                           ║");
    println!("    ║   🕶️   KĀRAṆA SMART GLASSES SIMULATOR                     ║");
    println!("    ║                                                           ║");
    println!("    ║   Experience the AI-native OS for augmented reality       ║");
    println!("    ║                                                           ║");
    println!("    ╚═══════════════════════════════════════════════════════════╝{}\n", RESET);
    
    println!("{}    This simulator lets you experience Kāraṇa OS features:{}", WHITE, RESET);
    println!("    {}• Object identification{} - \"What am I looking at?\"", GREEN, RESET);
    println!("    {}• Voice timers{} - \"Set a timer for 5 minutes\"", GREEN, RESET);
    println!("    {}• Memory assistance{} - \"Remember where my keys are\"", GREEN, RESET);
    println!("    {}• Navigation{} - \"Navigate to coffee shop\"", GREEN, RESET);
    println!("    {}• Context awareness{} - AI understands your environment", GREEN, RESET);
    println!();
    println!("{}    Change scenes with: go to kitchen / office / street / store{}", DIM, RESET);
    println!();
    print!("{}    Press ENTER to start...{}", YELLOW, RESET);
    stdout().flush().unwrap();
    
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    
    // Initialize state
    let mut state = GlassesState::new();
    state.last_ai_response = "Hello! I'm Kāraṇa, your AI assistant. Try asking 'What am I looking at?' or say 'help' for more options.".to_string();
    
    // Add initial notification
    state.add_notification("Welcome", "Kāraṇa OS initialized", "✨");
    
    // Main loop
    loop {
        // Check for completed timers
        let completed: Vec<String> = state.timers.iter()
            .filter(|t| t.is_done())
            .map(|t| t.name.clone())
            .collect();
        
        for name in &completed {
            state.add_notification(&format!("Timer: {}", name), "Time's up!", "⏰");
        }
        state.timers.retain(|t| !t.is_done());
        
        // Remove old notifications
        state.notifications.retain(|n| n.created.elapsed() < Duration::from_secs(15));
        
        // Render
        render_display(&state, 70, 30);
        
        // Read input
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        // Check for quit
        if input == "quit" || input == "exit" || input == "q" {
            clear_screen();
            println!("\n{}{}  Thanks for trying Kāraṇa OS! 👋{}\n", CYAN, BOLD, RESET);
            break;
        }
        
        // Process command
        state.is_listening = true;
        let response = process_ai_command(input, &mut state);
        state.is_listening = false;
        state.last_ai_response = response.clone();
        state.conversation_history.push((input.to_string(), response));
        
        // Drain battery slightly
        state.battery = (state.battery - 0.1).max(0.0);
    }
}
