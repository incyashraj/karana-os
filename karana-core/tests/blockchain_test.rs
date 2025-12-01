use karana_core::chain::{Blockchain, Block, Transaction, TransactionData};
use karana_core::economy::{Ledger, Governance};
use karana_core::ai::KaranaAI;
use std::sync::{Arc, Mutex};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_blockchain_flow() {
    // Setup paths
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let ledger_path = format!("/tmp/karana-test-ledger-{}", timestamp);
    let gov_path = format!("/tmp/karana-test-gov-{}", timestamp);

    // Cleanup previous run if any (unlikely with timestamp)
    let _ = fs::remove_dir_all(&ledger_path);
    let _ = fs::remove_dir_all(&gov_path);

    // Initialize Components
    // AI is needed for Governance (mock it or use real one? Real one downloads models... that's bad for tests)
    // KaranaAI::new() downloads models. I should probably mock it or avoid calling methods that use it if possible.
    // Governance::create_proposal calls ai.predict().
    
    // I'll try to use the real AI but maybe it will fail if no internet or slow.
    // Actually, KaranaAI::new() might be heavy.
    // Let's see if I can mock it. KaranaAI is a struct, not a trait.
    // I can't easily mock it without refactoring.
    // However, `KaranaAI::new()` loads "quantized/phi-2-tiny.gguf".
    // If I don't have it, it downloads.
    
    // Let's try to run the test. If it hangs on AI, I'll know.
    // But wait, `Governance::new` takes `Arc<Mutex<KaranaAI>>`.
    
    // I'll instantiate KaranaAI.
    let ai = Arc::new(Mutex::new(KaranaAI::new().expect("Failed to init AI")));
    
    let ledger = Arc::new(Mutex::new(Ledger::new(&ledger_path)));
    let gov = Arc::new(Mutex::new(Governance::new(&gov_path, ledger.clone(), ai.clone())));
    let chain = Blockchain::new(ledger.clone(), gov.clone());

    // 1. Genesis State
    ledger.lock().unwrap().mint("Alice", 1000);
    ledger.lock().unwrap().mint("Bob", 0);

    assert_eq!(ledger.lock().unwrap().get_balance("Alice"), 1000);
    assert_eq!(ledger.lock().unwrap().get_balance("Bob"), 0);

    // 2. Create a Block with Transfer
    let tx = Transaction {
        sender: "Alice".to_string(),
        data: TransactionData::Transfer { to: "Bob".to_string(), amount: 100 },
        signature: "sig".to_string(),
        nonce: 1,
    };

    let block = Block::new(
        "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        1,
        "Validator".to_string(),
        vec![tx],
    );

    // 3. Apply Block
    chain.apply_block(&block).expect("Failed to apply block");

    // 4. Verify State Changes
    assert_eq!(ledger.lock().unwrap().get_balance("Alice"), 900);
    assert_eq!(ledger.lock().unwrap().get_balance("Bob"), 100);

    // 5. Test Governance Proposal via Block
    let prop_tx = Transaction {
        sender: "Alice".to_string(),
        data: TransactionData::Propose { 
            title: "Test Prop".to_string(), 
            description: "Should we burn tokens?".to_string() 
        },
        signature: "sig".to_string(),
        nonce: 2,
    };

    let block2 = Block::new(
        block.hash.clone(),
        2,
        "Validator".to_string(),
        vec![prop_tx],
    );

    chain.apply_block(&block2).expect("Failed to apply block 2");

    // Cleanup
    let _ = fs::remove_dir_all(&ledger_path);
    let _ = fs::remove_dir_all(&gov_path);
}
