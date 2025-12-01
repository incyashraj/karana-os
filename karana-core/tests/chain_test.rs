use karana_core::chain::{ChainState, Transaction, Block};
use alloy_primitives::U256;

#[test]
fn test_chain_state_transition() {
    let mut state = ChainState::new();
    
    // Genesis
    state.balances.insert("Alice".to_string(), U256::from(1000u64));
    
    // Tx 1: Transfer
    let tx1 = Transaction::Transfer {
        to: "Bob".to_string(),
        amount: U256::from(100u64),
    };
    
    state.apply(&tx1, "Alice").expect("Tx1 failed");
    
    assert_eq!(state.balances.get("Alice"), Some(&U256::from(900u64)));
    assert_eq!(state.balances.get("Bob"), Some(&U256::from(100u64)));
    
    // Tx 2: Stake
    let tx2 = Transaction::Stake {
        amount: U256::from(500u64),
    };
    
    state.apply(&tx2, "Alice").expect("Tx2 failed");
    
    assert_eq!(state.balances.get("Alice"), Some(&U256::from(400u64)));
    assert_eq!(state.staked.get("Alice"), Some(&U256::from(500u64)));
    
    // Tx 3: Insufficient Funds
    let tx3 = Transaction::Transfer {
        to: "Charlie".to_string(),
        amount: U256::from(1000u64),
    };
    
    let res = state.apply(&tx3, "Alice");
    assert!(res.is_err());
}

#[test]
fn test_block_hashing() {
    let txs = vec![
        Transaction::Transfer { to: "Bob".to_string(), amount: U256::from(10u64) }
    ];
    
    let block = Block::new(
        "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        1,
        "Alice".to_string(),
        txs
    );
    
    assert_eq!(block.header.height, 1);
    assert_eq!(block.transactions.len(), 1);
    assert_ne!(block.hash, "");
}
