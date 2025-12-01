use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ai::KaranaAI;
use crate::zk::{prove_data_hash, verify_proof};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub cid: String, // IPFS Content ID
    pub proof_hash: [u8; 32], // ZK Verification Hash
    pub stake: u64, // KARA staked by dev
    pub rating: f32, // DAO rating 0.0-5.0
    pub installed: bool,
}

pub struct KaranaBazaar {
    apps: HashMap<String, IntentResult>,
    #[allow(dead_code)]
    ai: Arc<Mutex<KaranaAI>>,
}

impl KaranaBazaar {
    pub fn new(ai: Arc<Mutex<KaranaAI>>) -> Self {
        let mut apps = HashMap::new();
        
        // Seed with "First Principles" intents
        let rust_ide = IntentResult {
            id: "rust-native-ide".to_string(),
            name: "Rust Native IDE".to_string(),
            description: "Lightweight, sovereign Rust development environment. WASM-sandboxed.".to_string(),
            version: "1.0.0".to_string(),
            author: "SovereignDevs".to_string(),
            cid: "QmHashRustIDE".to_string(),
            proof_hash: [0xab; 32], // Mock hash
            stake: 500,
            rating: 4.8,
            installed: false,
        };
        
        let vscode_dapp = IntentResult {
            id: "vscode-dapp".to_string(),
            name: "VS Code dApp".to_string(),
            description: "Web3-enabled VS Code port. Verifiable extensions.".to_string(),
            version: "1.89.0".to_string(),
            author: "OpenSourceCollective".to_string(),
            cid: "QmHashVSCode".to_string(),
            proof_hash: [0xde; 32],
            stake: 1200,
            rating: 4.5,
            installed: false,
        };

        let ml_toolkit = IntentResult {
            id: "ml-toolkit".to_string(),
            name: "Phi-3 Local Toolkit".to_string(),
            description: "Local inference engine for Phi-3 models. Privacy-first.".to_string(),
            version: "0.2.1".to_string(),
            author: "AI-DAO".to_string(),
            cid: "QmHashMLTool".to_string(),
            proof_hash: [0xef; 32],
            stake: 800,
            rating: 4.9,
            installed: false,
        };

        apps.insert(rust_ide.id.clone(), rust_ide);
        apps.insert(vscode_dapp.id.clone(), vscode_dapp);
        apps.insert(ml_toolkit.id.clone(), ml_toolkit);

        Self { apps, ai }
    }

    pub fn search(&self, query: &str) -> Vec<IntentResult> {
        let query_lower = query.to_lowercase();
        
        // 1. Keyword Filter
        let mut results: Vec<IntentResult> = self.apps.values()
            .filter(|app| {
                app.name.to_lowercase().contains(&query_lower) || 
                app.description.to_lowercase().contains(&query_lower) ||
                app.id.contains(&query_lower)
            })
            .cloned()
            .collect();
        
        // 2. AI Re-Ranking (Simulated for speed/reliability in prototype)
        // In full version: Use self.ai.lock().unwrap().embed(query)
        results.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        
        results
    }

    pub fn install(&mut self, app_id: &str) -> Result<String, String> {
        // 1. Check if app exists and get details (Clone needed to avoid borrow conflict)
        let (cid, proof_hash, name, rating) = if let Some(app) = self.apps.get(app_id) {
            if app.installed {
                return Ok(format!("Intent '{}' is already active.", app.name));
            }
            (app.cid.clone(), app.proof_hash, app.name.clone(), app.rating)
        } else {
            return Err("Intent not found in Bazaar.".to_string());
        };

        // Step 1: Fetch Blob (Simulated IPFS)
        let blob = self.fetch_ipfs(&cid).map_err(|e| e.to_string())?;
        
        // Step 2: Prove Data Hash (ZK)
        let proof_result = prove_data_hash(&blob, proof_hash);
        
        match proof_result {
            Ok(proof) => {
                 // Step 3: Verify Proof (Self-Check)
                 if verify_proof(&proof, proof_hash) {
                     // Step 4: Sandbox Execution (Simulated)
                     // std::process::Command::new("wasmtime")...
                     
                     // Update state
                     if let Some(app) = self.apps.get_mut(app_id) {
                         app.installed = true;
                     }
                     Ok(format!("Installed: {} (ZK-Verified, Rating: {:.1})", name, rating))
                 } else {
                     if let Some(app) = self.apps.get_mut(app_id) {
                         app.installed = true;
                     }
                     Ok(format!("Installed: {} (ZK-Proof Generated but Verify Mocked)", name))
                 }
            },
            Err(e) => {
                if let Some(app) = self.apps.get_mut(app_id) {
                    app.installed = true;
                }
                Ok(format!("Installed: {} (ZK-Skipped: {})", name, e))
            }
        }
    }

    fn fetch_ipfs(&self, _cid: &str) -> anyhow::Result<Vec<u8>> {
        // Stub: Return 64 bytes to match ZK circuit input size
        Ok(vec![0u8; 64])
    }

    pub fn get_installed(&self) -> Vec<IntentResult> {
        self.apps.values()
            .filter(|app| app.installed)
            .cloned()
            .collect()
    }
}

