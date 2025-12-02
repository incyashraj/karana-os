//! Intelligence System Demo
//! 
//! Demonstrates the adaptive intelligence features:
//! - Context awareness (time, location)
//! - User behavior learning
//! - Conversation memory with anaphora resolution
//! - Proactive suggestions
//! - Pattern recognition and predictions

use std::collections::HashMap;

// Import intelligence modules
use karana_core::context::{ContextEngine, Location, Activity, TimeOfDay};
use karana_core::learning::{LearningSystem, UserProfile};
use karana_core::memory::MemorySystem;
use karana_core::proactive::{IntelligentAssistant, ProactiveConfig, SuggestionPriority};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  🧠 KARANA-OS INTELLIGENCE SYSTEM DEMO");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    
    // Create the intelligent assistant
    let mut assistant = IntelligentAssistant::new();
    
    // ═══════════════════════════════════════════════════════════════════
    // DEMO 1: CONTEXT AWARENESS
    // ═══════════════════════════════════════════════════════════════════
    println!("📍 DEMO 1: Context Awareness");
    println!("────────────────────────────────────────────────────────────────────");
    
    // Get current context
    let snapshot = assistant.context.snapshot();
    println!("  Current Time: {:?}", snapshot.time_of_day);
    println!("  Day Type: {:?}", snapshot.day_type);
    println!("  Location: {:?}", snapshot.location);
    println!("  Session Duration: {:?}", snapshot.session_duration);
    
    // Simulate location changes
    println!("\n  📍 Simulating location change to Office...");
    assistant.context.set_location(Location::Office);
    assistant.context.set_activity(Activity::Working);
    
    let greeting = assistant.context.get_contextual_greeting();
    println!("  Contextual Greeting: \"{}\"", greeting);
    
    println!();
    
    // ═══════════════════════════════════════════════════════════════════
    // DEMO 2: USER BEHAVIOR LEARNING
    // ═══════════════════════════════════════════════════════════════════
    println!("📚 DEMO 2: User Behavior Learning");
    println!("────────────────────────────────────────────────────────────────────");
    
    // Simulate user interactions
    let actions = vec![
        "check_balance",
        "stake_tokens",
        "check_balance",
        "send_tokens",
        "check_balance",
        "stake_tokens",
        "check_balance",
    ];
    
    println!("  Simulating user actions: {:?}", actions);
    
    for action in &actions {
        assistant.context.record_action(action);
        assistant.learning.record_command(action, action, true);
    }
    
    // Check what patterns emerged
    let predictions = assistant.context.predict_next_actions(3);
    println!("\n  🔮 Predicted next actions (based on patterns):");
    for (action, score) in &predictions {
        println!("      {:20} (score: {:.1})", action, score);
    }
    
    // Teach it a custom phrase
    println!("\n  📝 Teaching custom phrase: 'yo show me the money' → check_balance");
    assistant.learning.profile_mut().learn_phrase("yo show me the money", "check_balance", 0.9);
    
    // Test the learned phrase
    let enhanced = assistant.learning.enhance_input("yo show me the money");
    println!("  Enhanced input: {:?}", enhanced);
    
    // Record some common amounts
    println!("\n  💰 Recording common transaction amounts...");
    assistant.learning.profile_mut().record_amount(100);
    assistant.learning.profile_mut().record_amount(100);
    assistant.learning.profile_mut().record_amount(50);
    assistant.learning.profile_mut().record_amount(100);
    
    let common = assistant.learning.profile().common_amounts(3);
    println!("  Most common amounts: {:?}", common);
    
    println!();
    
    // ═══════════════════════════════════════════════════════════════════
    // DEMO 3: CONVERSATION MEMORY & ANAPHORA
    // ═══════════════════════════════════════════════════════════════════
    println!("💭 DEMO 3: Conversation Memory & Anaphora Resolution");
    println!("────────────────────────────────────────────────────────────────────");
    
    // Simulate a conversation with entities
    let mut entities = HashMap::new();
    entities.insert("amount".to_string(), "500".to_string());
    
    assistant.memory.record_turn(
        "send 500 KARA to Node-Beta",
        "send_tokens",
        entities.clone(),
        "Sent 500 KARA to Node-Beta",
        true
    );
    
    println!("  Recorded turn: 'send 500 KARA to Node-Beta'");
    println!("  Active entities: amount=500");
    
    // Now test anaphora resolution
    let ambiguous = "send it to them again";
    let resolved = assistant.memory.working.resolve_anaphora(ambiguous);
    println!("\n  🔗 Anaphora Resolution:");
    println!("    Input:    \"{}\"", ambiguous);
    println!("    Resolved: \"{}\"", resolved);
    
    // Test fact extraction
    println!("\n  📝 Testing fact extraction...");
    let processed = assistant.memory.process_input("My name is Alice and I prefer dark mode");
    println!("    Input: \"My name is Alice and I prefer dark mode\"");
    println!("    Extracted facts: {:?}", processed.extracted_facts);
    
    // Now greetings should be personalized
    let greeting = assistant.memory.personalized_greeting();
    println!("    Personalized greeting: \"{}\"", greeting);
    
    println!();
    
    // ═══════════════════════════════════════════════════════════════════
    // DEMO 4: PROACTIVE SUGGESTIONS
    // ═══════════════════════════════════════════════════════════════════
    println!("🤖 DEMO 4: Proactive Suggestions");
    println!("────────────────────────────────────────────────────────────────────");
    
    // Get suggestions based on current context and patterns
    let suggestions = assistant.get_suggestions();
    
    if suggestions.is_empty() {
        println!("  No suggestions yet (need more interaction patterns)");
        println!("  Simulating more patterns...");
        
        // Simulate more patterns to trigger suggestions
        for _ in 0..10 {
            assistant.context.record_action("check_balance");
            assistant.context.record_action("stake_tokens");
        }
        
        // Force cooldown reset for demo
        assistant.proactive.set_enabled(true);
    }
    
    // Show proactive stats
    let stats = assistant.proactive.stats();
    println!("\n  📊 Proactive Statistics:");
    println!("    Total suggestions generated: {}", stats.total_suggestions);
    println!("    Declined patterns: {}", stats.declined_actions);
    println!("    Enabled: {}", stats.enabled);
    
    println!();
    
    // ═══════════════════════════════════════════════════════════════════
    // DEMO 5: FULL INTELLIGENCE PROCESSING
    // ═══════════════════════════════════════════════════════════════════
    println!("🧠 DEMO 5: Full Intelligence Processing Pipeline");
    println!("────────────────────────────────────────────────────────────────────");
    
    // Process a query through the full pipeline
    let test_queries = vec![
        "check my balance",
        "send 100 to Node-Beta",
        "what about staking?",
        "My name is Bob",
        "yo show me the money",  // Learned phrase!
    ];
    
    for query in test_queries {
        let processed = assistant.process(query);
        println!("\n  Query: \"{}\"", query);
        println!("    → Resolved: \"{}\"", processed.resolved_input);
        if let Some(ref learned) = processed.learned_action {
            println!("    → Learned action: {}", learned);
        }
        if !processed.extracted_facts.is_empty() {
            println!("    → Extracted facts: {:?}", processed.extracted_facts);
        }
    }
    
    println!();
    
    // ═══════════════════════════════════════════════════════════════════
    // DEMO 6: INTELLIGENCE REPORT
    // ═══════════════════════════════════════════════════════════════════
    println!("📊 DEMO 6: Intelligence Report");
    println!("────────────────────────────────────────────────────────────────────");
    
    let report = assistant.describe_intelligence();
    println!("{}", report);
    
    println!();
    
    // ═══════════════════════════════════════════════════════════════════
    // DEMO 7: CORRECTION LEARNING
    // ═══════════════════════════════════════════════════════════════════
    println!("🔧 DEMO 7: Learning from Corrections");
    println!("────────────────────────────────────────────────────────────────────");
    
    // Simulate a misunderstanding and correction
    println!("  User said: 'gimme my coins'");
    println!("  System understood: unknown");
    println!("  User corrected: 'no, I meant check balance'");
    
    assistant.learning.profile_mut().record_correction(
        "gimme my coins",
        "unknown",
        "no, check balance",
        "check_balance"
    );
    
    // Now it should recognize the phrase
    let lookup = assistant.learning.profile().lookup_phrase("gimme my coins");
    if let Some(learned) = lookup {
        println!("\n  ✓ Learned new phrase!");
        println!("    '{}' → {} (confidence: {:.0}%)", 
            learned.phrase, learned.action, learned.confidence * 100.0);
    }
    
    println!();
    
    // ═══════════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  ✅ INTELLIGENCE SYSTEM DEMO COMPLETE");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("  Features demonstrated:");
    println!("  ✓ Context awareness (time, location, activity)");
    println!("  ✓ User behavior pattern learning");
    println!("  ✓ Action prediction based on patterns");
    println!("  ✓ Custom phrase learning");
    println!("  ✓ Conversation memory");
    println!("  ✓ Anaphora resolution ('it', 'them')");
    println!("  ✓ Fact extraction from conversation");
    println!("  ✓ Personalized greetings");
    println!("  ✓ Proactive suggestion system");
    println!("  ✓ Learning from corrections");
    println!();
    println!("  The system gets smarter with every interaction!");
    println!();
}
