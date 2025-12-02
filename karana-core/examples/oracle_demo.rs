//! Oracle Demo: Shows the AI ↔ Blockchain pipeline working
//!
//! Run with: cargo run --example oracle_demo

use karana_core::ai::KaranaAI;
use karana_core::chain::Blockchain;
use karana_core::economy::{Ledger, Governance};
use karana_core::storage::KaranaStorage;
use karana_core::oracle::KaranaOracle;
use std::sync::{Arc, Mutex};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_millis()
        .init();
    
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   KARANA ORACLE: REAL AI ↔ BLOCKCHAIN INTEGRATION         ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Setup REAL persistent components
    println!("🔧 Initializing REAL persistent state...\n");
    
    let ai = Arc::new(Mutex::new(KaranaAI::new()?));
    let ledger = Arc::new(Mutex::new(Ledger::new("/tmp/oracle_demo_ledger")));
    let gov = Arc::new(Mutex::new(Governance::new("/tmp/oracle_demo_gov", ledger.clone(), ai.clone())));
    let chain = Arc::new(Blockchain::new(ledger.clone(), gov.clone()));
    let storage = Arc::new(KaranaStorage::new("/tmp/oracle_demo_storage", "http://localhost:26657", ai.clone())?);
    
    // Bootstrap: Give user initial balance
    {
        let mut l = ledger.lock().unwrap();
        l.mint("Node-Alpha", 1000);  // Start with 1000 KARA
    }
    
    // Create the Oracle with REAL ledger & governance
    let oracle = KaranaOracle::new(
        ai.clone(), 
        chain.clone(), 
        storage.clone(),
        ledger.clone(),
        gov.clone(),
    );
    
    println!("✓ Oracle initialized with REAL persistent state!\n");
    println!("═══════════════════════════════════════════════════════════════");
    
    // Demo queries - these use REAL persistent ledger
    let demo_queries = vec![
        ("check my balance", "Node-Alpha"),
        ("stake 200 tokens", "Node-Alpha"),
        ("check my balance", "Node-Alpha"),  // Should show 800 + 200 staked
        ("propose Enable AR Gesture Control", "Node-Alpha"),
        ("show governance proposals", "Node-Alpha"),
        ("vote yes on proposal 1", "Node-Alpha"),
        ("show governance proposals", "Node-Alpha"),  // Should show vote
        ("send 50 tokens to Node-Beta", "Node-Alpha"),
        ("check my balance", "Node-Alpha"),  // Should show 750
        ("store note: Meeting at 3pm", "Node-Alpha"),
        ("show my files", "Node-Alpha"),
    ];
    
    for (query, did) in demo_queries {
        println!("\n╭─────────────────────────────────────────────────────────────╮");
        println!("│ 🗣️  USER: \"{}\"", query);
        println!("│ 🆔  DID:  {}", did);
        println!("╰─────────────────────────────────────────────────────────────╯");
        
        match oracle.process_query(query, did) {
            Ok(result) => {
                println!("\n{}", result);
            },
            Err(e) => {
                println!("\n❌ Error: {}", e);
            }
        }
        
        println!("\n───────────────────────────────────────────────────────────────");
    }
    
    // Cleanup
    let _ = std::fs::remove_dir_all("/tmp/oracle_demo_ledger");
    let _ = std::fs::remove_dir_all("/tmp/oracle_demo_gov");
    let _ = std::fs::remove_dir_all("/tmp/oracle_demo_storage");
    
    println!("\n\n╔═══════════════════════════════════════════════════════════╗");
    println!("║                    DEMO COMPLETE                           ║");
    println!("║                                                             ║");
    println!("║  ✓ REAL persistent RocksDB ledger                          ║");
    println!("║  ✓ REAL balance tracking (transfers deduct)                ║");
    println!("║  ✓ REAL staking (locks tokens)                             ║");
    println!("║  ✓ REAL governance proposals (AI-analyzed)                 ║");
    println!("║  ✓ REAL voting (weighted by stake)                         ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    Ok(())
}
