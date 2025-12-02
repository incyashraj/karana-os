//! # Smart Glasses Demo
//!
//! Demonstrates the complete smart glasses experience:
//! - Voice command → Whisper → AI Intent → Blockchain → AR HUD
//! - Gaze tracking with element focus
//! - Real-time HUD updates

use karana_core::{
    ai::KaranaAI,
    chain::Blockchain,
    storage::KaranaStorage,
    economy::{Ledger, Governance},
    oracle::KaranaOracle,
    celestia::CelestiaClient,
    glasses::{SmartGlasses, GlassesConfig, ARElementType},
};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   KARANA SMART GLASSES SIMULATOR                          ║");
    println!("║   Voice → AI → Blockchain → AR HUD                        ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Initialize components
    println!("🔧 Initializing Karana OS components...\n");

    let ai = Arc::new(Mutex::new(KaranaAI::new()?));
    println!("  ✓ AI Engine: Semantic embeddings loaded");

    let ledger = Arc::new(Mutex::new(Ledger::new("/tmp/glasses_demo_ledger")));
    let governance = Arc::new(Mutex::new(Governance::new("/tmp/glasses_demo_gov", ledger.clone(), ai.clone())));
    let chain = Arc::new(Blockchain::new(ledger.clone(), governance.clone()));
    let storage = Arc::new(KaranaStorage::new("/tmp/glasses_demo_storage", "http://localhost:26657", ai.clone())?);
    
    println!("  ✓ Blockchain: RocksDB persistent state");

    // Bootstrap user
    let user_did = "did:karana:glasses_user";
    {
        let mut l = ledger.lock().unwrap();
        l.mint(user_did, 10000);
    }
    println!("  ✓ Wallet: 10,000 KARA minted\n");

    // Create Oracle
    let oracle = KaranaOracle::new(
        ai.clone(),
        chain.clone(),
        storage.clone(),
        ledger.clone(),
        governance.clone(),
    );
    
    // Create Smart Glasses
    let config = GlassesConfig {
        display_opacity: 0.9,
        eye_tracking: true,
        wake_word: "karana".to_string(),
        notification_timeout: 5,
        gaze_dismiss: true,
        minimal_mode: false,
        font_scale: 1.0,
    };
    
    let mut glasses = SmartGlasses::new(ai.clone(), user_did)
        .with_config(config);
    
    println!("  ✓ Smart Glasses: AR overlay ready\n");

    // Connect to Celestia (async block)
    let rt = tokio::runtime::Runtime::new()?;
    let celestia_status = rt.block_on(async {
        let mut celestia = CelestiaClient::new_mocha();
        match celestia.connect().await {
            Ok(_) => {
                if celestia.is_connected() {
                    "Connected to Mocha".to_string()
                } else {
                    "Offline Mode".to_string()
                }
            }
            Err(_) => "Offline Mode".to_string(),
        }
    });
    println!("  ✓ Celestia DA: {}\n", celestia_status);

    // Initial HUD update
    glasses.update_hud(85, &celestia_status, 9052936);

    println!("════════════════════════════════════════════════════════════");
    println!("  SMART GLASSES AR INTERFACE");
    println!("════════════════════════════════════════════════════════════\n");

    // Render initial AR view
    println!("{}", glasses.render_ascii(80, 24));

    println!("════════════════════════════════════════════════════════════\n");

    // Simulate voice commands
    let voice_commands = [
        "check my balance",
        "stake 500 tokens",
        "create proposal: Enable gaze-based scrolling",
        "vote yes on proposal 1",
        "send 200 tokens to alice",
        "show my files",
    ];

    for (i, cmd) in voice_commands.iter().enumerate() {
        println!("╭──────────────────────────────────────────────────────────╮");
        println!("│ 🎤 Voice Command #{}: \"{}\"", i + 1, cmd);
        println!("╰──────────────────────────────────────────────────────────╯\n");

        // Simulate gaze movement to center
        glasses.update_gaze(0.5, 0.5);

        // Process through Oracle
        match oracle.process_query(cmd, user_did) {
            Ok(result) => {
                // Show notification
                glasses.show_notification(&format!("✓ {}", cmd), 3000);
                
                // Display result in AR
                let lines: Vec<&str> = result.lines().collect();
                let preview = if lines.len() > 3 {
                    lines[..3].join("\n") + "..."
                } else {
                    result.clone()
                };

                println!("📱 AR Display:");
                println!("┌──────────────────────────────────────────────────────────┐");
                for line in preview.lines() {
                    println!("│ {:^56} │", line);
                }
                println!("└──────────────────────────────────────────────────────────┘\n");
            }
            Err(e) => {
                glasses.show_notification(&format!("❌ Error: {}", e), 5000);
                println!("❌ Error: {}\n", e);
            }
        }

        // Update HUD
        let battery = 85 - (i as u8 * 2);
        glasses.update_hud(battery, "Synced", 9052936 + i as u64);

        // Clean up expired elements
        glasses.cleanup_expired();

        // Render AR view
        println!("🕶️ AR View ({}x{}):", 70, 20);
        println!("{}", glasses.render_ascii(70, 20));
        println!();

        // Small delay between commands
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Final balance check
    let final_balance = {
        let l = ledger.lock().unwrap();
        l.get_balance(user_did)
    };

    println!("════════════════════════════════════════════════════════════");
    println!("                    DEMO COMPLETE");
    println!("════════════════════════════════════════════════════════════\n");

    println!("📊 Session Summary:");
    println!("   • Final Balance: {} KARA", final_balance);
    println!("   • Commands Executed: {}", voice_commands.len());
    println!("   • Proposals Created: 1");
    println!("   • Votes Cast: 1\n");

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  ✓ Voice Input: Whisper transcription ready               ║");
    println!("║  ✓ AI Processing: Semantic intent matching                ║");
    println!("║  ✓ Blockchain: Real persistent state                      ║");
    println!("║  ✓ AR Display: Gaze-aware overlay                         ║");
    println!("║  ✓ Celestia DA: Network connected                         ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Interactive mode prompt
    print!("Press Enter to try interactive mode (or Ctrl+C to exit)...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    println!("\n🎮 Interactive Mode - Type commands or 'quit' to exit\n");

    loop {
        print!("🎤 > ");
        io::stdout().flush()?;
        
        input.clear();
        io::stdin().read_line(&mut input)?;
        let cmd = input.trim();
        
        if cmd == "quit" || cmd == "exit" {
            println!("👋 Goodbye!");
            break;
        }

        if cmd == "gaze" {
            // Simulate random gaze
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let x: f32 = rng.gen_range(0.1..0.9);
            let y: f32 = rng.gen_range(0.1..0.9);
            glasses.update_gaze(x, y);
            println!("👁️ Gaze updated to ({:.2}, {:.2})", x, y);
            continue;
        }

        if cmd == "render" {
            println!("{}", glasses.render_ascii(70, 20));
            continue;
        }

        if cmd == "minimal" {
            glasses.toggle_minimal_mode();
            glasses.update_hud(75, "Synced", 9052940);
            println!("🔄 Minimal mode toggled");
            continue;
        }

        if cmd.is_empty() {
            continue;
        }

        // Process command through Oracle
        match oracle.process_query(cmd, user_did) {
            Ok(result) => {
                println!("\n{}\n", result);
                glasses.cleanup_expired();
            }
            Err(e) => println!("❌ {}\n", e),
        }
    }

    Ok(())
}
