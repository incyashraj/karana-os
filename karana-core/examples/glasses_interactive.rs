//! # Kāraṇa Smart Glasses Interactive Demo
//!
//! A fully interactive simulation of the Kāraṇa smart glasses experience.
//! Features real AI responses, blockchain operations, and immersive AR HUD.
//!
//! Run with: cargo run --example glasses_interactive --release

use std::io::{self, Write, BufRead};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use std::thread;
use std::path::PathBuf;

use karana_core::hud::{GlassesHUD, HudNotification};
use karana_core::ai::KaranaAI;
use karana_core::chain::Blockchain;
use karana_core::storage::KaranaStorage;
use karana_core::economy::{Ledger, Governance};
use karana_core::oracle::KaranaOracle;
use karana_core::context::Location;
use karana_core::wallet::KaranaWallet;
use karana_core::onboarding::{OnboardingWizard, OnboardingConfig};

fn main() {
    // Initialize AI and blockchain systems
    println!("\x1b[2J\x1b[H");
    println!("Initializing Kāraṇa Smart Glasses...\n");
    
    // Create storage directories
    let storage_path = PathBuf::from("/tmp/karana_glasses_demo");
    std::fs::create_dir_all(&storage_path).ok();
    
    // ═══════════════════════════════════════════════════════════════════
    // WALLET & ONBOARDING - Real cryptographic identity
    // ═══════════════════════════════════════════════════════════════════
    
    let mut hud = GlassesHUD::new();
    
    // Check if wallet exists or run onboarding
    let wallet = if OnboardingWizard::needs_onboarding(&storage_path) {
        println!("First-time setup detected. Starting onboarding...\n");
        
        let config = OnboardingConfig {
            data_dir: storage_path.clone(),
            skip_backup_verify: true, // Skip verification for demo
            ..Default::default()
        };
        
        let mut wizard = OnboardingWizard::new(config);
        
        // Run onboarding with HUD
        hud.init();
        match wizard.run_with_hud(&mut hud) {
            Ok(result) => {
                println!("\n✓ Wallet created: {}", result.wallet.did());
                result.wallet
            }
            Err(e) => {
                hud.cleanup();
                eprintln!("Onboarding failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Load existing wallet
        println!("Loading existing wallet...");
        match KaranaWallet::load_encrypted(
            &storage_path.join("wallet.json"),
            "device_secured" // Default password from HUD onboarding
        ) {
            Ok(wallet) => {
                println!("✓ Wallet loaded: {}", wallet.did());
                wallet
            }
            Err(e) => {
                eprintln!("Failed to load wallet: {}", e);
                eprintln!("Removing corrupt wallet and starting fresh...");
                let _ = std::fs::remove_file(storage_path.join("wallet.json"));
                
                // Create new wallet
                let config = OnboardingConfig {
                    data_dir: storage_path.clone(),
                    skip_backup_verify: true,
                    ..Default::default()
                };
                let mut wizard = OnboardingWizard::new(config);
                hud.init();
                match wizard.run_with_hud(&mut hud) {
                    Ok(result) => result.wallet,
                    Err(e) => {
                        hud.cleanup();
                        eprintln!("Onboarding failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    };
    
    // Get user's DID for all operations (before moving wallet)
    let user_did = wallet.did().to_string();
    println!("\n  Your sovereign identity: {}", user_did);
    
    // ═══════════════════════════════════════════════════════════════════
    // SYSTEM INITIALIZATION
    // ═══════════════════════════════════════════════════════════════════
    
    // Initialize AI
    let ai = Arc::new(Mutex::new(KaranaAI::new().expect("Failed to initialize AI")));
    
    // Initialize ledger (required by governance and blockchain)
    let ledger = Arc::new(Mutex::new(Ledger::new(&format!("{}/ledger", storage_path.display()))));
    
    // Initialize governance (required by blockchain)
    let governance = Arc::new(Mutex::new(Governance::new(
        &format!("{}/governance", storage_path.display()),
        ledger.clone(),
        ai.clone(),
    )));
    
    // Initialize blockchain with ledger and governance
    let chain = Arc::new(Blockchain::new(ledger.clone(), governance.clone()));
    
    // Initialize storage with AI
    let storage = Arc::new(KaranaStorage::new(
        &format!("{}/storage", storage_path.display()),
        "http://localhost:26657",  // Chain RPC (simulated)
        ai.clone(),
    ).expect("Failed to create storage"));
    
    // Initialize user with tokens (using their real DID!)
    {
        let mut l = ledger.lock().unwrap();
        // Check if already initialized
        if l.get_balance(&user_did) == 0 {
            l.mint(&user_did, 10000);
            println!("  ✓ Minted 10,000 KARA to your wallet");
        } else {
            println!("  Balance: {} KARA", l.get_balance(&user_did));
        }
    }
    
    // Create Oracle WITH wallet for real transaction signing
    let oracle = Arc::new(KaranaOracle::with_wallet(
        ai.clone(),
        chain.clone(),
        storage.clone(),
        ledger.clone(),
        governance.clone(),
        wallet,  // Pass the real wallet for Ed25519 signing
    ));
    
    println!("  ✓ Oracle initialized with Ed25519 signing");
    
    // Run boot sequence (if not already done in onboarding)
    if !OnboardingWizard::needs_onboarding(&storage_path) {
        hud.boot_sequence();
    }
    
    // Welcome notification
    hud.notify(HudNotification::new(
        "👋",
        "Welcome",
        &format!("{}...", &user_did[..15])
    ));
    
    // Set initial location context
    oracle.set_location(Location::Home);
    
    // Running flag
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    }).ok();
    
    // Clone for render thread
    let hud = Arc::new(Mutex::new(hud));
    let hud_render = hud.clone();
    let running_render = running.clone();
    
    // Render thread - runs at 30 FPS
    let render_thread = thread::spawn(move || {
        let frame_duration = Duration::from_millis(33);
        
        while running_render.load(Ordering::SeqCst) {
            let start = Instant::now();
            
            {
                let mut h = hud_render.lock().unwrap();
                h.render();
            }
            
            let elapsed = start.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
    });
    
    // Initial render
    {
        let mut h = hud.lock().unwrap();
        h.render();
    }
    
    // Main input loop
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    
    println!("\x1b[999;1H"); // Move to bottom
    
    while running.load(Ordering::SeqCst) {
        // Read line
        let mut input = String::new();
        
        // Show prompt at bottom
        print!("\x1b[999;1H\x1b[2K\x1b[96m> \x1b[0m");
        io::stdout().flush().ok();
        
        match reader.read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let query = input.trim();
                
                if query.is_empty() {
                    continue;
                }
                
                // Special commands
                match query.to_lowercase().as_str() {
                    "quit" | "exit" => {
                        running.store(false, Ordering::SeqCst);
                        continue;
                    }
                    "help" => {
                        let help_text = "Commands: check balance, send <amount> to <addr>, stake <amount>, capture photo, set timer <min> <label>, show notifications, help, quit";
                        let mut h = hud.lock().unwrap();
                        h.add_assistant_message(help_text);
                        continue;
                    }
                    "clear" => {
                        let mut h = hud.lock().unwrap();
                        h.init();
                        continue;
                    }
                    _ => {}
                }
                
                // Add user message
                {
                    let mut h = hud.lock().unwrap();
                    h.add_user_message(query);
                    h.set_processing(true);
                }
                
                // Process through oracle (using real wallet DID!)
                let result = oracle.process_query(query, &user_did);
                
                {
                    let mut h = hud.lock().unwrap();
                    h.set_processing(false);
                    
                    match result {
                        Ok(response) => {
                            // Clean up response for HUD
                            let clean_response = response
                                .lines()
                                .filter(|l| !l.contains("╭") && !l.contains("╰"))
                                .map(|l| l.replace("│", "").replace("─", "").trim().to_string())
                                .filter(|l| !l.is_empty())
                                .collect::<Vec<_>>()
                                .join(" ");
                            
                            let display = if clean_response.is_empty() {
                                response.replace("╭", "").replace("╰", "").replace("│", "").replace("─", "").trim().to_string()
                            } else {
                                clean_response
                            };
                            
                            h.add_assistant_message(&display);
                            
                            // Add success notification for transactions
                            if query.to_lowercase().contains("send") || 
                               query.to_lowercase().contains("stake") ||
                               query.to_lowercase().contains("photo") {
                                h.notify(HudNotification::new("✓", "Success", &display[..50.min(display.len())]));
                            }
                        }
                        Err(e) => {
                            h.add_assistant_message(&format!("Error: {}", e));
                            h.notify(HudNotification::urgent("❌", "Error", &e.to_string()));
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }
    
    // Cleanup
    running.store(false, Ordering::SeqCst);
    render_thread.join().ok();
    
    {
        let h = hud.lock().unwrap();
        h.cleanup();
    }
    
    println!("\n\nGoodbye! 👋\n");
}
