//! Kāraṇa OS Smart Glasses Visual Demo
//! 
//! This runs an automated visual demonstration showing the glasses UI
//! and AI features in action - no interaction needed.

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const WHITE: &str = "\x1b[37m";
const BG_BLACK: &str = "\x1b[40m";
const CLEAR: &str = "\x1b[2J\x1b[H";

fn main() {
    // Clear screen and show intro
    print!("{}", CLEAR);
    println!("\n");
    println!("{}{}╔════════════════════════════════════════════════════════════════════╗{}", BOLD, CYAN, RESET);
    println!("{}{}║         🕶️  KĀRAṆA OS - SMART GLASSES SIMULATOR 🕶️                 ║{}", BOLD, CYAN, RESET);
    println!("{}{}║                                                                    ║{}", BOLD, CYAN, RESET);
    println!("{}{}║      Decentralized AI Operating System for AR Glasses             ║{}", BOLD, CYAN, RESET);
    println!("{}{}║                                                                    ║{}", BOLD, CYAN, RESET);
    println!("{}{}╚════════════════════════════════════════════════════════════════════╝{}", BOLD, CYAN, RESET);
    println!("\n");
    println!("{}Starting visual demonstration...{}", DIM, RESET);
    println!("{}This demo shows Kāraṇa OS features automatically{}", DIM, RESET);
    thread::sleep(Duration::from_secs(2));

    // Demo 1: Boot sequence
    demo_boot_sequence();
    
    // Demo 2: Object recognition
    demo_object_recognition();
    
    // Demo 3: AI Assistant
    demo_ai_assistant();
    
    // Demo 4: Memory assist
    demo_memory_assist();
    
    // Demo 5: Navigation
    demo_navigation();
    
    // Demo 6: Notifications
    demo_notifications();
    
    // Final summary
    demo_summary();
}

fn demo_boot_sequence() {
    print!("{}", CLEAR);
    println!("\n{}{}=== DEMO 1: DEVICE BOOT SEQUENCE ==={}\n", BOLD, YELLOW, RESET);
    
    let boot_steps = [
        ("Initializing hardware...", 300),
        ("Loading Kāraṇa OS v0.2.0...", 400),
        ("Starting AI models...", 500),
        ("Calibrating sensors...", 300),
        ("Connecting to network...", 200),
        ("Loading user profile...", 200),
        ("System ready!", 0),
    ];
    
    println!("{}┌────────────────────────────────────────┐{}", CYAN, RESET);
    println!("{}│      XREAL Air - Powering On...        │{}", CYAN, RESET);
    println!("{}└────────────────────────────────────────┘{}", CYAN, RESET);
    println!();
    
    for (step, delay) in boot_steps {
        print!("  {}▸{} {}", GREEN, RESET, step);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(delay));
        println!(" {}✓{}", GREEN, RESET);
    }
    
    println!("\n{}Device booted successfully!{}\n", GREEN, RESET);
    thread::sleep(Duration::from_secs(2));
}

fn demo_object_recognition() {
    print!("{}", CLEAR);
    println!("\n{}{}=== DEMO 2: AI OBJECT RECOGNITION ==={}\n", BOLD, YELLOW, RESET);
    
    // Simulated glasses view
    println!("{}{}╔══════════════════════════════════════════════════════════════╗{}", BOLD, CYAN, RESET);
    println!("{}{}║ 10:30 AM    🔋 95%  📶 ●●●  📍 Kitchen                       ║{}", WHITE, BG_BLACK, RESET);
    println!("{}{}╟──────────────────────────────────────────────────────────────╢{}", BOLD, CYAN, RESET);
    
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║      ┌─────────────────────────────────────────┐             ║{}", DIM, CYAN, RESET);
    println!("{}{}║      │                                         │             ║{}", DIM, CYAN, RESET);
    println!("{}{}║      │    📷 LIVE CAMERA VIEW                  │             ║{}", DIM, CYAN, RESET);
    println!("{}{}║      │                                         │             ║{}", DIM, CYAN, RESET);
    println!("{}{}║      │    Looking at: Kitchen counter          │             ║{}", DIM, CYAN, RESET);
    println!("{}{}║      │                                         │             ║{}", DIM, CYAN, RESET);
    println!("{}{}║      └─────────────────────────────────────────┘             ║{}", DIM, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    
    thread::sleep(Duration::from_millis(800));
    
    println!("{}{}║  {}🔍 AI VISION - Objects Detected:{}                          ║{}", DIM, CYAN, BOLD, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    
    let objects = [
        ("🍎 Apple", 97),
        ("🍳 Frying pan", 94),
        ("🧀 Cheese block", 91),
        ("🔪 Knife", 89),
        ("📦 Recipe book", 85),
    ];
    
    for (obj, conf) in objects {
        let bar_len = conf / 5;
        let bar: String = "█".repeat(bar_len as usize);
        let empty: String = "░".repeat((20 - bar_len) as usize);
        println!("{}{}║     {} {} [{}{}{}{}] {}%{}                   ║{}", 
            DIM, CYAN, 
            obj, 
            if obj.len() < 20 { " ".repeat(20 - obj.len()) } else { String::new() },
            GREEN, bar, DIM, empty, 
            conf, RESET, RESET);
        thread::sleep(Duration::from_millis(300));
    }
    
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}╚══════════════════════════════════════════════════════════════╝{}", BOLD, CYAN, RESET);
    
    println!("\n{}AI Vision processed 5 objects in 0.8 seconds{}", GREEN, RESET);
    thread::sleep(Duration::from_secs(3));
}

fn demo_ai_assistant() {
    print!("{}", CLEAR);
    println!("\n{}{}=== DEMO 3: AI VOICE ASSISTANT ==={}\n", BOLD, YELLOW, RESET);
    
    println!("{}{}╔══════════════════════════════════════════════════════════════╗{}", BOLD, CYAN, RESET);
    println!("{}{}║ 10:32 AM    🔋 94%  📶 ●●●  📍 Kitchen                       ║{}", WHITE, BG_BLACK, RESET);
    println!("{}{}╟──────────────────────────────────────────────────────────────╢{}", BOLD, CYAN, RESET);
    
    // User speaks
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║   {}🎤 USER SPEAKING:{} \"Hey Kāraṇa, what can I cook with       ║{}", DIM, CYAN, YELLOW, RESET, RESET);
    println!("{}{}║       these ingredients?\"                                    ║{}", DIM, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    
    thread::sleep(Duration::from_secs(1));
    
    // AI processing
    println!("{}{}║   {}🤖 AI PROCESSING...{}                                        ║{}", DIM, CYAN, MAGENTA, RESET, RESET);
    thread::sleep(Duration::from_millis(800));
    
    // AI Response
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║   {}🧠 KĀRAṆA AI:{}                                              ║{}", DIM, CYAN, GREEN, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║     \"Based on what I see - apples, cheese, and a pan -      ║{}", DIM, CYAN, RESET);
    println!("{}{}║      here are some options:                                  ║{}", DIM, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║      {}1. Apple & Cheese Quesadilla{} (10 min)                   ║{}", DIM, CYAN, BOLD, RESET, RESET);
    println!("{}{}║      {}2. Caramelized Apple Grilled Cheese{} (15 min)            ║{}", DIM, CYAN, BOLD, RESET, RESET);
    println!("{}{}║      {}3. Apple Cheese Toast{} (5 min)                           ║{}", DIM, CYAN, BOLD, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║      Say a number to see the recipe!\"                        ║{}", DIM, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}╚══════════════════════════════════════════════════════════════╝{}", BOLD, CYAN, RESET);
    
    println!("\n{}AI processed voice + vision to provide contextual help{}", GREEN, RESET);
    thread::sleep(Duration::from_secs(3));
}

fn demo_memory_assist() {
    print!("{}", CLEAR);
    println!("\n{}{}=== DEMO 4: MEMORY ASSISTANCE ==={}\n", BOLD, YELLOW, RESET);
    
    println!("{}Scenario: You placed your keys somewhere and forgot where...{}", DIM, RESET);
    println!();
    
    thread::sleep(Duration::from_secs(1));
    
    println!("{}{}╔══════════════════════════════════════════════════════════════╗{}", BOLD, CYAN, RESET);
    println!("{}{}║ 10:35 AM    🔋 93%  📶 ●●●  📍 Living Room                   ║{}", WHITE, BG_BLACK, RESET);
    println!("{}{}╟──────────────────────────────────────────────────────────────╢{}", BOLD, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║   {}🎤 USER:{} \"Kāraṇa, where did I put my keys?\"               ║{}", DIM, CYAN, YELLOW, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    
    thread::sleep(Duration::from_secs(1));
    
    println!("{}{}║   {}🧠 SEARCHING MEMORY...{}                                     ║{}", DIM, CYAN, MAGENTA, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    
    thread::sleep(Duration::from_millis(600));
    
    println!("{}{}║   {}📍 MEMORY FOUND:{} 2 hours ago                               ║{}", DIM, CYAN, GREEN, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║     {}🔑 Keys detected{} at:                                     ║{}", DIM, CYAN, BOLD, RESET, RESET);
    println!("{}{}║        {}Location:{} Kitchen counter, near the fruit bowl       ║{}", DIM, CYAN, WHITE, RESET, RESET);
    println!("{}{}║        {}Time:{} 8:35 AM this morning                           ║{}", DIM, CYAN, WHITE, RESET, RESET);
    println!("{}{}║        {}Context:{} After you came in from jogging              ║{}", DIM, CYAN, WHITE, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║     ┌─────────────────────────────────────┐                  ║{}", DIM, CYAN, RESET);
    println!("{}{}║     │ 📸 Memory snapshot attached        │                  ║{}", DIM, CYAN, RESET);
    println!("{}{}║     └─────────────────────────────────────┘                  ║{}", DIM, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}╚══════════════════════════════════════════════════════════════╝{}", BOLD, CYAN, RESET);
    
    println!("\n{}Privacy: All memories stored locally on YOUR device only{}", GREEN, RESET);
    thread::sleep(Duration::from_secs(3));
}

fn demo_navigation() {
    print!("{}", CLEAR);
    println!("\n{}{}=== DEMO 5: AR NAVIGATION ==={}\n", BOLD, YELLOW, RESET);
    
    println!("{}{}╔══════════════════════════════════════════════════════════════╗{}", BOLD, CYAN, RESET);
    println!("{}{}║ 10:40 AM    🔋 91%  📶 ●●●  📍 Street View                   ║{}", WHITE, BG_BLACK, RESET);
    println!("{}{}╟──────────────────────────────────────────────────────────────╢{}", BOLD, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║   {}🎤 USER:{} \"Navigate to the nearest coffee shop\"            ║{}", DIM, CYAN, YELLOW, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    
    thread::sleep(Duration::from_secs(1));
    
    println!("{}{}║   {}🗺️  NAVIGATION ACTIVE{}                                      ║{}", DIM, CYAN, GREEN, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║     Destination: {}Blue Bottle Coffee{}                          ║{}", DIM, CYAN, BOLD, RESET, RESET);
    println!("{}{}║     Distance: {}350m{} • ETA: {}4 min walking{}                      ║{}", DIM, CYAN, WHITE, RESET, WHITE, RESET, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║   ┌─────────────────── AR OVERLAY ───────────────────┐       ║{}", DIM, CYAN, RESET);
    println!("{}{}║   │                                                   │       ║{}", DIM, CYAN, RESET);
    println!("{}{}║   │        {}↑{}                                          │       ║{}", DIM, CYAN, GREEN, RESET, RESET);
    println!("{}{}║   │       {}╱ ╲{}   Walk straight for 200m              │       ║{}", DIM, CYAN, GREEN, RESET, RESET);
    println!("{}{}║   │      {}╱   ╲{}                                       │       ║{}", DIM, CYAN, GREEN, RESET, RESET);
    println!("{}{}║   │     ╔═════╗                                       │       ║{}", DIM, CYAN, RESET);
    println!("{}{}║   │     ║ ☕  ║   {}Blue Bottle Coffee{}                 │       ║{}", DIM, CYAN, YELLOW, RESET, RESET);
    println!("{}{}║   │     ╚═════╝   {}Rating: ★★★★☆ 4.2{}                  │       ║{}", DIM, CYAN, DIM, RESET, RESET);
    println!("{}{}║   │               {}Open until 8 PM{}                    │       ║{}", DIM, CYAN, DIM, RESET, RESET);
    println!("{}{}║   │                                                   │       ║{}", DIM, CYAN, RESET);
    println!("{}{}║   └───────────────────────────────────────────────────┘       ║{}", DIM, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}╚══════════════════════════════════════════════════════════════╝{}", BOLD, CYAN, RESET);
    
    println!("\n{}AR arrows overlay directly on your field of view{}", GREEN, RESET);
    thread::sleep(Duration::from_secs(3));
}

fn demo_notifications() {
    print!("{}", CLEAR);
    println!("\n{}{}=== DEMO 6: SMART NOTIFICATIONS ==={}\n", BOLD, YELLOW, RESET);
    
    println!("{}{}╔══════════════════════════════════════════════════════════════╗{}", BOLD, CYAN, RESET);
    println!("{}{}║ 10:45 AM    🔋 90%  📶 ●●●  📍 Office                        ║{}", WHITE, BG_BLACK, RESET);
    println!("{}{}╟──────────────────────────────────────────────────────────────╢{}", BOLD, CYAN, RESET);
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    
    // Notification 1
    println!("{}{}║  ╭──────────────────────────────────────────────╮            ║{}", DIM, CYAN, RESET);
    println!("{}{}║  │ {}📧 Email from: Sarah Chen{}                   │            ║{}", DIM, CYAN, YELLOW, RESET, RESET);
    println!("{}{}║  │ \"Meeting moved to 2 PM. New room: 304\"       │            ║{}", DIM, CYAN, RESET);
    println!("{}{}║  │ {}[Reply] [Snooze] [Dismiss]{}                   │            ║{}", DIM, CYAN, DIM, RESET, RESET);
    println!("{}{}║  ╰──────────────────────────────────────────────╯            ║{}", DIM, CYAN, RESET);
    
    thread::sleep(Duration::from_secs(1));
    
    // Notification 2
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║  ╭──────────────────────────────────────────────╮            ║{}", DIM, CYAN, RESET);
    println!("{}{}║  │ {}⏰ Reminder{}                                   │            ║{}", DIM, CYAN, MAGENTA, RESET, RESET);
    println!("{}{}║  │ \"Take medication\" in 15 minutes              │            ║{}", DIM, CYAN, RESET);
    println!("{}{}║  │ {}[Acknowledge] [Snooze 30m]{}                   │            ║{}", DIM, CYAN, DIM, RESET, RESET);
    println!("{}{}║  ╰──────────────────────────────────────────────╯            ║{}", DIM, CYAN, RESET);
    
    thread::sleep(Duration::from_secs(1));
    
    // Notification 3 - person recognition
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}║  ╭──────────────────────────────────────────────╮            ║{}", DIM, CYAN, RESET);
    println!("{}{}║  │ {}👤 Person Identified{}                         │            ║{}", DIM, CYAN, GREEN, RESET, RESET);
    println!("{}{}║  │ \"Alex from Engineering\"                      │            ║{}", DIM, CYAN, RESET);
    println!("{}{}║  │ {}Last met: Project sync, 3 days ago{}          │            ║{}", DIM, CYAN, DIM, RESET, RESET);
    println!("{}{}║  │ {}They mentioned: Launch deadline Friday{}      │            ║{}", DIM, CYAN, DIM, RESET, RESET);
    println!("{}{}║  ╰──────────────────────────────────────────────╯            ║{}", DIM, CYAN, RESET);
    
    println!("{}{}║                                                              ║{}", DIM, CYAN, RESET);
    println!("{}{}╚══════════════════════════════════════════════════════════════╝{}", BOLD, CYAN, RESET);
    
    println!("\n{}Context-aware notifications with privacy controls{}", GREEN, RESET);
    thread::sleep(Duration::from_secs(3));
}

fn demo_summary() {
    print!("{}", CLEAR);
    println!("\n");
    println!("{}{}╔════════════════════════════════════════════════════════════════════╗{}", BOLD, GREEN, RESET);
    println!("{}{}║              ✨ KĀRAṆA OS FEATURE SUMMARY ✨                       ║{}", BOLD, GREEN, RESET);
    println!("{}{}╚════════════════════════════════════════════════════════════════════╝{}", BOLD, GREEN, RESET);
    println!();
    
    println!("  {}Core Features Demonstrated:{}", BOLD, RESET);
    println!();
    println!("    {}✓{} {}AI Object Recognition{}   - Identify 20+ object types in real-time", GREEN, RESET, BOLD, RESET);
    println!("    {}✓{} {}Voice AI Assistant{}      - Natural language interaction", GREEN, RESET, BOLD, RESET);
    println!("    {}✓{} {}Memory Assistance{}       - Never forget where you put things", GREEN, RESET, BOLD, RESET);
    println!("    {}✓{} {}AR Navigation{}           - Overlay directions on real world", GREEN, RESET, BOLD, RESET);
    println!("    {}✓{} {}Smart Notifications{}     - Context-aware, non-intrusive", GREEN, RESET, BOLD, RESET);
    println!("    {}✓{} {}Person Recognition{}      - Remember names and conversations", GREEN, RESET, BOLD, RESET);
    println!();
    
    println!("  {}Privacy & Security:{}", BOLD, RESET);
    println!();
    println!("    {}🔒{} All data processed {}locally on device{}", BLUE, RESET, BOLD, RESET);
    println!("    {}🔐{} Cryptographic signatures for all operations", BLUE, RESET);
    println!("    {}🌐{} Decentralized - {}no central servers{}", BLUE, RESET, BOLD, RESET);
    println!("    {}👤{} {}You own your data{} - export anytime", BLUE, RESET, BOLD, RESET);
    println!();
    
    println!("  {}Supported Devices:{}", BOLD, RESET);
    println!();
    println!("    • XREAL Air / Air 2 Pro");
    println!("    • Rokid Max");
    println!("    • Meta Ray-Ban Smart Glasses");
    println!("    • Enterprise AR headsets");
    println!();
    
    println!("{}{}════════════════════════════════════════════════════════════════════{}", BOLD, CYAN, RESET);
    println!();
    println!("  {}Learn more:{} https://github.com/anthropics/karana-os", BOLD, RESET);
    println!("  {}Documentation:{} See ARCHITECTURE.md and SIMPLE_GUIDE.md", BOLD, RESET);
    println!();
    println!("{}{}════════════════════════════════════════════════════════════════════{}", BOLD, CYAN, RESET);
    println!();
    println!("{}Demo complete! Thank you for exploring Kāraṇa OS.{}", GREEN, RESET);
    println!();
}
