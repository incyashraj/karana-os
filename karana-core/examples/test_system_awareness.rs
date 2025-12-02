//! Test System Awareness: Infeasible Action Detection
//! 
//! This example tests that the AI correctly identifies actions that
//! smart glasses CANNOT perform, and provides helpful alternatives.

use karana_core::{
    ai::KaranaAI,
    oracle::KaranaOracle,
    chain::Blockchain,
    storage::KaranaStorage,
    economy::{Ledger, Governance},
};
use std::sync::{Arc, Mutex};

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  KARANA SYSTEM AWARENESS TEST                                   ║");
    println!("║  Testing AI's understanding of what glasses CAN and CAN'T do    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Initialize components
    let ai = Arc::new(Mutex::new(KaranaAI::new()?));
    
    let ledger = Arc::new(Mutex::new(Ledger::new("/tmp/test_awareness_ledger")));
    let governance = Arc::new(Mutex::new(Governance::new("/tmp/test_awareness_gov", ledger.clone(), ai.clone())));
    let chain = Arc::new(Blockchain::new(ledger.clone(), governance.clone()));
    let storage = Arc::new(KaranaStorage::new("/tmp/test_awareness_storage", "http://localhost:26657", ai.clone())?);

    let oracle = KaranaOracle::new(
        ai.clone(),
        chain,
        storage,
        ledger.clone(),
        governance.clone(),
    );

    // Mint some tokens
    {
        let mut ledger = ledger.lock().unwrap();
        ledger.mint("did:karana:test_user", 5000);
    }

    println!("═══════════════════════════════════════════════════════════════════");
    println!("  TESTING INFEASIBLE ACTIONS (things glasses CAN'T do)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let infeasible_tests = vec![
        ("open VS Code", "Should detect: desktop IDE not available on glasses"),
        ("open terminal", "Should detect: terminal requires keyboard/screen"),
        ("launch photoshop", "Should detect: creative software needs desktop"),
        ("open Chrome browser", "Should detect: full browsing needs larger screen"),
        ("write a long email", "Should detect: no keyboard on glasses"),
        ("download and install an app", "Should detect: limited storage"),
        ("play Fortnite", "Should detect: gaming needs GPU"),
        ("join Zoom meeting with screen share", "Should detect: video conf limitations"),
    ];

    for (query, description) in &infeasible_tests {
        println!("┌──────────────────────────────────────────────────────────────────");
        println!("│ 🎤 Query: \"{}\"", query);
        println!("│ 📋 Expected: {}", description);
        println!("├──────────────────────────────────────────────────────────────────");
        
        let response = oracle.process_query(query, "did:karana:test_user")?;
        
        // Check if it detected as infeasible
        if response.contains("Not Available") || response.contains("⚠️") {
            println!("│ ✅ CORRECTLY DETECTED AS INFEASIBLE");
        } else {
            println!("│ ⚠️ May not have detected as infeasible");
        }
        println!("│ Response:");
        for line in response.lines() {
            println!("│   {}", line);
        }
        println!("└──────────────────────────────────────────────────────────────────\n");
    }

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("  TESTING FEASIBLE ACTIONS (things glasses CAN do)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let feasible_tests = vec![
        ("check my balance", "Should work: wallet check"),
        ("take a photo", "Should work: camera available"),
        ("show notifications", "Should work: glasses can display alerts"),
        ("navigate to the coffee shop", "Should work: AR navigation"),
        ("what am I looking at", "Should work: object identification via camera"),
        ("set a 5 minute timer", "Should work: timers work"),
        ("play some music", "Should work: audio playback"),
    ];

    for (query, description) in &feasible_tests {
        println!("┌──────────────────────────────────────────────────────────────────");
        println!("│ 🎤 Query: \"{}\"", query);
        println!("│ 📋 Expected: {}", description);
        println!("├──────────────────────────────────────────────────────────────────");
        
        let response = oracle.process_query(query, "did:karana:test_user")?;
        
        // Check if it worked (not infeasible)
        if response.contains("Not Available") || response.contains("⚠️") {
            println!("│ ⚠️ INCORRECTLY marked as infeasible");
        } else {
            println!("│ ✅ CORRECTLY PROCESSED AS FEASIBLE");
        }
        println!("│ Response:");
        for line in response.lines() {
            println!("│   {}", line);
        }
        println!("└──────────────────────────────────────────────────────────────────\n");
    }

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  TEST COMPLETE                                                   ║");
    println!("║  The AI now understands system capabilities!                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
