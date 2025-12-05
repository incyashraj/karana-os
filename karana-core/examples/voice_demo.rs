//! # Voice-to-Blockchain Demo
//!
//! Demonstrates the complete voice pipeline:
//! 1. Load audio (WAV file or simulated)
//! 2. Transcribe with Whisper
//! 3. Process through Oracle
//! 4. Execute on blockchain
//! 5. Display AR result
//!
//! ## Usage
//! ```bash
//! # With real WAV file:
//! cargo run --release --example voice_demo -- path/to/audio.wav
//!
//! # With simulated audio (text input):
//! cargo run --release --example voice_demo
//! ```

use karana_core::{
    ai::KaranaAI,
    chain::Blockchain,
    storage::KaranaStorage,
    economy::{Ledger, Governance},
    oracle::KaranaOracle,
    voice_pipeline::{VoiceConfig, VoiceToIntent},
};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║   KARANA VOICE-TO-BLOCKCHAIN DEMO                              ║");
    println!("║   Voice → Whisper → AI → Oracle → Blockchain → AR             ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Initialize components
    println!("🔧 Initializing Karana OS components...\n");

    let ai = Arc::new(Mutex::new(KaranaAI::new()?));
    println!("  ✓ AI Engine: Whisper + MiniLM loaded");

    let ledger = Arc::new(Mutex::new(Ledger::new("/tmp/voice_demo_ledger")));
    let governance = Arc::new(Mutex::new(Governance::new("/tmp/voice_demo_gov", ledger.clone(), ai.clone())));
    let chain = Arc::new(Blockchain::new(ledger.clone(), governance.clone()));
    let storage = Arc::new(KaranaStorage::new("/tmp/voice_demo_storage", "http://localhost:26657", ai.clone())?);
    
    println!("  ✓ Blockchain: Persistent state ready");

    // Bootstrap user
    let user_did = "did:karana:voice_user";
    {
        let mut l = ledger.lock().unwrap();
        l.mint(user_did, 10000);
    }
    println!("  ✓ Wallet: 10,000 KARA minted");

    // Create Oracle
    let oracle = KaranaOracle::new(
        ai.clone(),
        chain.clone(),
        storage.clone(),
        ledger.clone(),
        governance.clone(),
    );
    
    println!("  ✓ Oracle: AI ↔ Blockchain bridge ready");

    // Create Voice Pipeline
    let voice_config = VoiceConfig {
        sample_rate: 16000,
        wake_word: "karana".to_string(),
        continuous_mode: true,
        max_duration_secs: 30,
        vad_threshold: 0.01,
        noise_reduction: true,
    };
    
    let voice_to_intent = VoiceToIntent::new(ai.clone(), voice_config);
    println!("  ✓ Voice Pipeline: 16kHz Whisper ready\n");

    // Check for WAV file argument
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        // WAV file mode
        let wav_path = &args[1];
        println!("════════════════════════════════════════════════════════════════");
        println!("  PROCESSING WAV FILE: {}", wav_path);
        println!("════════════════════════════════════════════════════════════════\n");
        
        // Load and transcribe
        match voice_to_intent.transcribe_wav(wav_path) {
            Ok(transcript) => {
                println!("┌────────────────────────────────────────────────────────────────");
                println!("│ 🎤 Transcription: \"{}\"", transcript);
                println!("└────────────────────────────────────────────────────────────────\n");
                
                // Process through Oracle
                let result = oracle.process_query(&transcript, user_did)?;
                
                println!("┌────────────────────────────────────────────────────────────────");
                println!("│ 📱 AR Display:");
                for line in result.lines() {
                    println!("│   {}", line);
                }
                println!("└────────────────────────────────────────────────────────────────\n");
            },
            Err(e) => {
                println!("❌ Failed to transcribe: {}", e);
            }
        }
    } else {
        // Interactive text mode (simulating voice input)
        println!("════════════════════════════════════════════════════════════════");
        println!("  INTERACTIVE MODE (simulating voice commands)");
        println!("  Type commands as if you spoke them. Type 'quit' to exit.");
        println!("════════════════════════════════════════════════════════════════\n");

        // Demo some voice commands
        let demo_commands = vec![
            "check my wallet balance",
            "stake 500 tokens for voting",
            "create a proposal for AR gesture controls",
            "vote yes on proposal 1",
            "send 100 tokens to alice",
            "show my files",
            "take a photo",
            "open VS Code",  // Should be detected as infeasible
        ];

        println!("📋 Demo voice commands:\n");
        
        for (i, cmd) in demo_commands.iter().enumerate() {
            println!("┌────────────────────────────────────────────────────────────────");
            println!("│ 🎤 Voice #{}: \"{}\"", i + 1, cmd);
            println!("├────────────────────────────────────────────────────────────────");
            
            let result = oracle.process_query(cmd, user_did)?;
            
            println!("│ 📱 Response:");
            for line in result.lines() {
                println!("│   {}", line);
            }
            println!("└────────────────────────────────────────────────────────────────\n");
            
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        // Interactive loop
        println!("\n════════════════════════════════════════════════════════════════");
        println!("  YOUR TURN - Type voice commands:");
        println!("════════════════════════════════════════════════════════════════\n");

        loop {
            print!("🎤 > ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            if input == "quit" || input == "exit" {
                break;
            }
            
            let result = oracle.process_query(input, user_did)?;
            println!("\n{}\n", result);
        }
    }

    // Show final balance
    let final_balance = {
        let l = ledger.lock().unwrap();
        l.get_balance(user_did)
    };

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  SESSION COMPLETE                                              ║");
    println!("║  Final Balance: {} KARA                                   ║", final_balance);
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
