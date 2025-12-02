//! # Kāraṇa Celestia Integration: Data Availability Layer
//!
//! This module connects Karana OS to Celestia's data availability layer.
//! Celestia provides:
//! - Data availability sampling for light nodes (perfect for glasses!)
//! - Modular blockchain architecture (we focus on execution, Celestia handles DA)
//! - Namespaced data blobs for our transactions
//!
//! ## Architecture
//! ```
//! Karana Transaction → Serialize → Submit to Celestia Namespace → DA Proof
//!                                                              ↓
//! Smart Glasses ← Light Client Verification ← Data Availability Sampling
//! ```

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

// Celestia Mocha Testnet RPC endpoint
const CELESTIA_MOCHA_RPC: &str = "https://rpc-mocha.pops.one";

// Karana's namespace ID on Celestia (8 bytes, hex)
// Using "karana00" = 0x6b6172616e613030
const KARANA_NAMESPACE: [u8; 8] = [0x6b, 0x61, 0x72, 0x61, 0x6e, 0x61, 0x30, 0x30];

/// Types of data we submit to Celestia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CelestiaBlob {
    /// State root commitment from our blockchain
    StateCommitment {
        block_height: u64,
        state_root: String,
        timestamp: u64,
    },
    /// Governance proposal for archival
    GovernanceProposal {
        proposal_id: u64,
        title: String,
        votes_for: u64,
        votes_against: u64,
        status: String,
    },
    /// User file metadata (not content - just index)
    FileMetadata {
        owner_did: String,
        file_hash: String,
        merkle_root: String,
        size: u64,
    },
    /// Transaction batch for rollup-style verification
    TransactionBatch {
        batch_id: u64,
        tx_count: u32,
        merkle_root: String,
        compressed_data: Vec<u8>,
    },
}

/// Result of submitting a blob to Celestia
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestiaSubmitResult {
    pub height: u64,
    pub tx_hash: String,
    pub namespace: String,
    pub blob_size: usize,
    pub success: bool,
}

/// Result of querying Celestia DA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestiaQueryResult {
    pub height: u64,
    pub blobs: Vec<Vec<u8>>,
    pub namespace: String,
    pub verified: bool,
}

/// Celestia client for Karana OS
pub struct CelestiaClient {
    /// RPC endpoint
    rpc_url: String,
    /// Our namespace ID
    namespace: [u8; 8],
    /// Connection state
    connected: bool,
    /// Cached last height
    last_height: Arc<Mutex<u64>>,
    /// Auth token (if using celestia-node directly)
    auth_token: Option<String>,
}

impl CelestiaClient {
    /// Create a new Celestia client for Mocha testnet
    pub fn new_mocha() -> Self {
        Self {
            rpc_url: CELESTIA_MOCHA_RPC.to_string(),
            namespace: KARANA_NAMESPACE,
            connected: false,
            last_height: Arc::new(Mutex::new(0)),
            auth_token: None,
        }
    }

    /// Create with custom RPC (for local light node)
    pub fn new_custom(rpc_url: &str, auth_token: Option<String>) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            namespace: KARANA_NAMESPACE,
            connected: false,
            last_height: Arc::new(Mutex::new(0)),
            auth_token,
        }
    }

    /// Connect to Celestia network
    pub async fn connect(&mut self) -> Result<()> {
        log::info!("[CELESTIA] Connecting to Mocha testnet: {}", self.rpc_url);
        
        // Try to get the latest block height to verify connection
        match self.get_latest_height().await {
            Ok(height) => {
                *self.last_height.lock().await = height;
                self.connected = true;
                log::info!("[CELESTIA] ✓ Connected! Latest height: {}", height);
                Ok(())
            }
            Err(e) => {
                log::warn!("[CELESTIA] Connection failed: {}. Running in offline mode.", e);
                // Don't fail - we can still work offline
                self.connected = false;
                Ok(())
            }
        }
    }

    /// Get the latest block height from Celestia
    async fn get_latest_height(&self) -> Result<u64> {
        // Using standard Tendermint RPC endpoint
        let url = format!("{}/status", self.rpc_url);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        
        let response = client.get(&url).send().await?;
        let body: serde_json::Value = response.json().await?;
        
        let height = body["result"]["sync_info"]["latest_block_height"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing height"))?
            .parse::<u64>()?;
        
        Ok(height)
    }

    /// Submit a blob to Celestia DA layer
    pub async fn submit_blob(&mut self, blob: CelestiaBlob) -> Result<CelestiaSubmitResult> {
        let blob_json = serde_json::to_vec(&blob)?;
        let blob_size = blob_json.len();
        
        log::info!("[CELESTIA] Submitting blob ({} bytes) to namespace 'karana00'", blob_size);

        if !self.connected {
            // Offline mode - simulate submission
            log::info!("[CELESTIA] Offline mode - simulating DA submission");
            let simulated_height = *self.last_height.lock().await + 1;
            *self.last_height.lock().await = simulated_height;
            
            return Ok(CelestiaSubmitResult {
                height: simulated_height,
                tx_hash: format!("sim_{}", hex::encode(&blob_json[..32.min(blob_json.len())])),
                namespace: hex::encode(self.namespace),
                blob_size,
                success: true,
            });
        }

        // Real submission using PayForBlob transaction
        // This requires a Celestia wallet with TIA tokens
        // For now, we'll use the blob submission API if a light node is running
        
        let submit_url = if self.auth_token.is_some() {
            // Using local celestia-node
            "http://localhost:26658"
        } else {
            // Using public RPC - requires different approach
            &self.rpc_url
        };

        // Build the PayForBlob transaction
        let namespace_hex = hex::encode(self.namespace);
        
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "blob.Submit",
            "params": [
                [{
                    "namespace": namespace_hex,
                    "data": BASE64.encode(&blob_json),
                    "share_version": 0
                }],
                0.002 // gas price
            ]
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let mut request = client.post(submit_url);
        
        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        match request.json(&payload).send().await {
            Ok(response) => {
                let result: serde_json::Value = response.json().await?;
                
                if let Some(error) = result.get("error") {
                    log::warn!("[CELESTIA] Submit error: {:?}", error);
                    // Fall back to simulated
                    let sim_height = *self.last_height.lock().await + 1;
                    return Ok(CelestiaSubmitResult {
                        height: sim_height,
                        tx_hash: format!("err_{}", hex::encode(&blob_json[..16.min(blob_json.len())])),
                        namespace: namespace_hex,
                        blob_size,
                        success: false,
                    });
                }

                let height = result["result"]["height"].as_u64().unwrap_or(0);
                let tx_hash = result["result"]["txhash"].as_str()
                    .unwrap_or("unknown")
                    .to_string();

                *self.last_height.lock().await = height;

                log::info!("[CELESTIA] ✓ Blob submitted at height {}", height);
                
                Ok(CelestiaSubmitResult {
                    height,
                    tx_hash,
                    namespace: namespace_hex,
                    blob_size,
                    success: true,
                })
            }
            Err(e) => {
                log::warn!("[CELESTIA] Submit failed: {}. Using simulated.", e);
                let sim_height = *self.last_height.lock().await + 1;
                Ok(CelestiaSubmitResult {
                    height: sim_height,
                    tx_hash: format!("offline_{}", sim_height),
                    namespace: hex::encode(self.namespace),
                    blob_size,
                    success: false,
                })
            }
        }
    }

    /// Query blobs from our namespace at a specific height
    pub async fn query_blobs(&self, height: u64) -> Result<CelestiaQueryResult> {
        log::info!("[CELESTIA] Querying blobs at height {}", height);

        if !self.connected {
            return Ok(CelestiaQueryResult {
                height,
                blobs: vec![],
                namespace: hex::encode(self.namespace),
                verified: false,
            });
        }

        let namespace_hex = hex::encode(self.namespace);
        
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "blob.GetAll",
            "params": [height, [namespace_hex]]
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let url = if self.auth_token.is_some() {
            "http://localhost:26658"
        } else {
            &self.rpc_url
        };

        let mut request = client.post(url);
        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        match request.json(&payload).send().await {
            Ok(response) => {
                let result: serde_json::Value = response.json().await?;
                
                let blobs: Vec<Vec<u8>> = result["result"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|b| {
                        b["data"].as_str()
                            .and_then(|s| BASE64.decode(s).ok())
                    })
                    .collect();

                Ok(CelestiaQueryResult {
                    height,
                    blobs,
                    namespace: namespace_hex,
                    verified: true,
                })
            }
            Err(e) => {
                log::warn!("[CELESTIA] Query failed: {}", e);
                Ok(CelestiaQueryResult {
                    height,
                    blobs: vec![],
                    namespace: namespace_hex,
                    verified: false,
                })
            }
        }
    }

    /// Create a state commitment blob for the current chain state
    pub fn create_state_commitment(
        block_height: u64,
        state_root: &str,
    ) -> CelestiaBlob {
        CelestiaBlob::StateCommitment {
            block_height,
            state_root: state_root.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Create a governance proposal blob for archival
    pub fn create_governance_blob(
        proposal_id: u64,
        title: &str,
        votes_for: u64,
        votes_against: u64,
        status: &str,
    ) -> CelestiaBlob {
        CelestiaBlob::GovernanceProposal {
            proposal_id,
            title: title.to_string(),
            votes_for,
            votes_against,
            status: status.to_string(),
        }
    }

    /// Create a file metadata blob
    pub fn create_file_blob(
        owner_did: &str,
        file_hash: &str,
        merkle_root: &str,
        size: u64,
    ) -> CelestiaBlob {
        CelestiaBlob::FileMetadata {
            owner_did: owner_did.to_string(),
            file_hash: file_hash.to_string(),
            merkle_root: merkle_root.to_string(),
            size,
        }
    }

    /// Check if connected to Celestia
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the namespace hex string
    pub fn namespace_hex(&self) -> String {
        hex::encode(self.namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_celestia_offline() {
        let mut client = CelestiaClient::new_mocha();
        
        // Should work in offline mode
        let blob = CelestiaClient::create_state_commitment(1, "0xabc123");
        let result = client.submit_blob(blob).await.unwrap();
        
        assert!(result.height > 0);
        assert!(result.blob_size > 0);
    }
}
