//! Shared Application State for the API server

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::wallet::KaranaWallet;
use crate::api::types::{Transaction, OsMode};

/// Shared state accessible across all API handlers
pub struct AppState {
    /// The active wallet (if one exists)
    pub wallet: RwLock<Option<KaranaWallet>>,
    
    /// Token balance (in a real system, this would come from the chain)
    pub balance: RwLock<u64>,
    
    /// Transaction history
    pub transactions: RwLock<Vec<Transaction>>,
    
    /// Transaction nonce counter
    pub nonce: RwLock<u64>,
    
    /// Current OS mode
    pub mode: RwLock<OsMode>,
    
    /// WebSocket subscribers by channel
    pub ws_subscribers: RwLock<HashMap<String, Vec<tokio::sync::mpsc::Sender<String>>>>,
    
    /// Celestia RPC endpoint
    pub celestia_endpoint: Option<String>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            wallet: RwLock::new(None),
            balance: RwLock::new(1000), // Start with 1000 KARA for testing
            transactions: RwLock::new(Vec::new()),
            nonce: RwLock::new(0),
            mode: RwLock::new(OsMode::Idle),
            ws_subscribers: RwLock::new(HashMap::new()),
            celestia_endpoint: std::env::var("CELESTIA_RPC_URL").ok(),
        })
    }
    
    /// Get the next nonce
    pub async fn next_nonce(&self) -> u64 {
        let mut nonce = self.nonce.write().await;
        let current = *nonce;
        *nonce += 1;
        current
    }
    
    /// Update balance
    pub async fn update_balance(&self, new_balance: u64) {
        *self.balance.write().await = new_balance;
    }
    
    /// Subtract from balance
    pub async fn debit(&self, amount: u64) -> bool {
        let mut balance = self.balance.write().await;
        if *balance >= amount {
            *balance -= amount;
            true
        } else {
            false
        }
    }
    
    /// Add to balance
    pub async fn credit(&self, amount: u64) {
        let mut balance = self.balance.write().await;
        *balance += amount;
    }
    
    /// Add a transaction to history
    pub async fn add_transaction(&self, tx: Transaction) {
        let mut transactions = self.transactions.write().await;
        transactions.insert(0, tx); // Most recent first
        // Keep only last 100 transactions
        if transactions.len() > 100 {
            transactions.pop();
        }
    }
    
    /// Set OS mode
    pub async fn set_mode(&self, mode: OsMode) {
        *self.mode.write().await = mode;
    }
    
    /// Broadcast message to WebSocket subscribers
    pub async fn broadcast(&self, channel: &str, message: &str) {
        let subscribers = self.ws_subscribers.read().await;
        if let Some(subs) = subscribers.get(channel) {
            for tx in subs {
                let _ = tx.send(message.to_string()).await;
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            wallet: RwLock::new(None),
            balance: RwLock::new(1000),
            transactions: RwLock::new(Vec::new()),
            nonce: RwLock::new(0),
            mode: RwLock::new(OsMode::Idle),
            ws_subscribers: RwLock::new(HashMap::new()),
            celestia_endpoint: None,
        }
    }
}
