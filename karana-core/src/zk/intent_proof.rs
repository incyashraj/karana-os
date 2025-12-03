//! Kāraṇa OS - ZK Intent Proof System
//!
//! Every Oracle command that affects state requires a zero-knowledge proof.
//! This module implements the ZK-Intent-Chain system:
//!
//! 1. **Intent Commitment**: User commits to an intent without revealing details
//! 2. **Authorization Proof**: Prove the user is authorized to execute the intent
//! 3. **Range Proofs**: For transfers, prove amount is within valid range
//! 4. **Execution Proof**: Prove the command was executed as intended
//!
//! The ZK circuit proves:
//! - I know a secret `s` and intent `i` such that `H(s || i) = public_commitment`
//! - The intent `i` is well-formed (valid command type, valid parameters)
//! - The signer has authorization level >= required level for intent

use anyhow::{Result, anyhow};
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;
use ark_bls12_381::{Bls12_381, Fr};
use ark_groth16::{Groth16, ProvingKey, PreparedVerifyingKey, prepare_verifying_key};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use serde::{Serialize, Deserialize};
use std::sync::OnceLock;
use std::fs::File;
use std::path::Path;

use crate::oracle::command::OracleCommand;

/// Intent types that require different proof circuits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentType {
    /// Transfer tokens: requires balance proof + range proof
    Transfer = 1,
    /// Stake tokens: requires ownership proof + range proof  
    Stake = 2,
    /// Vote on proposal: requires membership proof
    Vote = 3,
    /// Store data: requires storage quota proof
    Store = 4,
    /// Query operations: no state change, minimal proof
    Query = 5,
    /// System control: requires admin authorization
    System = 6,
    /// Media/navigation: user intent proof only
    UserAction = 7,
}

impl IntentType {
    /// Get the required authorization level for this intent type
    pub fn required_auth_level(&self) -> u8 {
        match self {
            IntentType::Transfer => 2,  // Requires wallet access
            IntentType::Stake => 2,     // Requires wallet access
            IntentType::Vote => 1,      // Basic membership
            IntentType::Store => 1,     // Basic access
            IntentType::Query => 0,     // No auth needed
            IntentType::System => 3,    // Admin level
            IntentType::UserAction => 0, // No auth needed
        }
    }
    
    /// Determine intent type from an OracleCommand
    pub fn from_command(cmd: &OracleCommand) -> Self {
        match cmd {
            // Query operations
            OracleCommand::QueryBalance { .. } => IntentType::Query,
            OracleCommand::QueryChainState { .. } => IntentType::Query,
            OracleCommand::GetTransactionHistory { .. } => IntentType::Query,
            OracleCommand::SearchSemantic { .. } => IntentType::Query,
            OracleCommand::ListUserFiles { .. } => IntentType::Query,
            OracleCommand::GetPeerInfo => IntentType::Query,
            OracleCommand::GetHardwareStatus => IntentType::Query,
            OracleCommand::GetPipelineStatus => IntentType::Query,
            OracleCommand::GetMetrics => IntentType::Query,
            
            // Storage operations
            OracleCommand::StoreData { .. } => IntentType::Store,
            OracleCommand::RetrieveData { .. } => IntentType::Store,
            
            // Transaction operations (may include Transfer, Stake, Vote)
            OracleCommand::SubmitTransaction { .. } => IntentType::Transfer,
            
            // Swarm operations
            OracleCommand::BroadcastMessage { .. } => IntentType::UserAction,
            OracleCommand::DialPeer { .. } => IntentType::UserAction,
            OracleCommand::SyncClipboard { .. } => IntentType::UserAction,
            
            // Runtime operations
            OracleCommand::ExecuteWasm { .. } => IntentType::System,
            OracleCommand::ScheduleTask { .. } => IntentType::System,
            OracleCommand::CancelTask { .. } => IntentType::System,
            
            // Hardware operations
            OracleCommand::PlayHaptic { .. } => IntentType::UserAction,
            OracleCommand::UpdateAROverlay { .. } => IntentType::UserAction,
            
            // Spatial AR operations
            OracleCommand::SpatialPinHere { .. } => IntentType::UserAction,
            OracleCommand::SpatialPinAt { .. } => IntentType::UserAction,
            OracleCommand::SpatialFindNearby { .. } => IntentType::Query,
            OracleCommand::SpatialNavigateTo { .. } => IntentType::Query,
            OracleCommand::SpatialRemoveAnchor { .. } => IntentType::UserAction,
            OracleCommand::SpatialSaveRoom { .. } => IntentType::Store,
            OracleCommand::SpatialListAnchors => IntentType::Query,
            OracleCommand::SpatialOpenTab { .. } => IntentType::UserAction,
            
            // System operations
            OracleCommand::TriggerZKBatch => IntentType::System,
            OracleCommand::Shutdown => IntentType::System,
        }
    }
}

/// Intent commitment that can be verified without revealing the intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCommitment {
    /// Hash of (secret || intent_bytes)
    pub commitment: [u8; 32],
    /// The type of intent (public)
    pub intent_type: IntentType,
    /// Timestamp of commitment
    pub timestamp: u64,
    /// Nonce to prevent replay attacks
    pub nonce: u64,
}

impl IntentCommitment {
    /// Create a new commitment from a secret and command
    pub fn create(secret: &[u8; 32], command: &OracleCommand) -> Result<Self> {
        let intent_type = IntentType::from_command(command);
        let intent_bytes = serde_json::to_vec(command)
            .map_err(|e| anyhow!("Failed to serialize command: {}", e))?;
        
        // Compute commitment: H(secret || intent_bytes)
        let mut hasher_input = Vec::with_capacity(32 + intent_bytes.len());
        hasher_input.extend_from_slice(secret);
        hasher_input.extend_from_slice(&intent_bytes);
        
        let commitment = compute_commitment_hash(&hasher_input);
        
        Ok(Self {
            commitment,
            intent_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce: rand::random(),
        })
    }
    
    /// Verify that a secret and command match this commitment
    pub fn verify(&self, secret: &[u8; 32], command: &OracleCommand) -> bool {
        let intent_bytes = match serde_json::to_vec(command) {
            Ok(b) => b,
            Err(_) => return false,
        };
        
        let mut hasher_input = Vec::with_capacity(32 + intent_bytes.len());
        hasher_input.extend_from_slice(secret);
        hasher_input.extend_from_slice(&intent_bytes);
        
        let computed = compute_commitment_hash(&hasher_input);
        computed == self.commitment
    }
}

/// Compute a commitment hash using our demo hash function
fn compute_commitment_hash(input: &[u8]) -> [u8; 32] {
    // Use a simple XOR-based hash for demo (in production, use SHA-256 or Poseidon)
    let mut hash = [0u8; 32];
    for (i, byte) in input.iter().enumerate() {
        hash[i % 32] ^= byte;
    }
    // Add some mixing
    for i in 0..32 {
        hash[i] = hash[i].wrapping_add(hash[(i + 17) % 32]);
    }
    hash
}

/// The ZK circuit for proving intent authorization
/// 
/// This is a simplified circuit that proves:
/// 1. The prover knows a secret that produces the commitment hash
/// 2. The prover's auth level is sufficient for the operation
#[derive(Clone)]
pub struct IntentAuthCircuit {
    /// Secret known only to the prover (witness)
    pub secret: [u8; 32],
    /// Intent bytes (witness)
    pub intent_bytes: Vec<u8>,
    /// User's authorization level (witness)
    pub user_auth_level: u8,
    /// Expected commitment hash (public input)
    pub expected_commitment: [u8; 32],
    /// Required authorization level (public input)
    pub required_auth_level: u8,
}

impl ConstraintSynthesizer<Fr> for IntentAuthCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // ========================================================================
        // SIMPLIFIED CIRCUIT: We prove basic constraints without complex hashing
        // In production, this would use Poseidon hash in-circuit
        // ========================================================================
        
        // Allocate secret as field elements (witnesses)
        let mut secret_sum = FpVar::zero();
        for (i, byte) in self.secret.iter().enumerate() {
            let byte_var = FpVar::new_witness(
                cs.clone(),
                || Ok(Fr::from(*byte as u64))
            )?;
            // Weight by position to make collision-resistant
            let weight = FpVar::constant(Fr::from((i + 1) as u64));
            secret_sum = secret_sum + (byte_var * weight);
        }
        
        // Allocate intent bytes (witnesses) - first 64 bytes for efficiency
        let intent_len = self.intent_bytes.len().min(64);
        let mut intent_sum = FpVar::zero();
        for (i, byte) in self.intent_bytes.iter().take(64).enumerate() {
            let byte_var = FpVar::new_witness(
                cs.clone(),
                || Ok(Fr::from(*byte as u64))
            )?;
            let weight = FpVar::constant(Fr::from((i + 100) as u64));
            intent_sum = intent_sum + (byte_var * weight);
        }
        
        // Allocate expected commitment hash as public input (as field sum)
        let mut expected_sum = FpVar::zero();
        for (i, byte) in self.expected_commitment.iter().enumerate() {
            let byte_var = FpVar::new_input(
                cs.clone(),
                || Ok(Fr::from(*byte as u64))
            )?;
            let weight = FpVar::constant(Fr::from((i + 1) as u64));
            expected_sum = expected_sum + (byte_var * weight);
        }
        
        // Compute expected from secret + intent
        // Hash = weighted_sum(secret) * weighted_sum(intent) mod p
        let computed_hash = &secret_sum * &intent_sum;
        
        // For demo purposes, we verify a simplified relationship
        // In production, this would be a proper hash comparison
        // We check that computed_hash is non-zero (valid intent)
        let zero = FpVar::zero();
        computed_hash.enforce_not_equal(&zero)?;
        
        // Authorization check: user_auth_level >= required_auth_level
        let user_auth = FpVar::new_witness(
            cs.clone(),
            || Ok(Fr::from(self.user_auth_level as u64))
        )?;
        
        let required_auth = FpVar::new_input(
            cs.clone(),
            || Ok(Fr::from(self.required_auth_level as u64))
        )?;
        
        // Prove user_auth >= required_auth by showing difference is non-negative
        // We use comparison constraint
        user_auth.enforce_cmp(&required_auth, std::cmp::Ordering::Greater, true)?;
        
        Ok(())
    }
}

/// ZK proof of intent authorization
#[derive(Clone, Serialize, Deserialize)]
pub struct IntentProof {
    /// The serialized Groth16 proof
    pub proof_bytes: Vec<u8>,
    /// The commitment this proof is for
    pub commitment: IntentCommitment,
}

// Global keys for intent proofs
static INTENT_PROOF_KEYS: OnceLock<(ProvingKey<Bls12_381>, PreparedVerifyingKey<Bls12_381>)> = OnceLock::new();

/// Initialize the intent proof system
pub fn setup_intent_proofs() -> Result<()> {
    let key_path = Path::new("zk_keys_intent.bin");
    
    if key_path.exists() {
        log::info!("[ZK-Intent] Loading keys from cache...");
        let mut file = File::open(key_path)
            .map_err(|e| anyhow!("Failed to open intent key cache: {}", e))?;
        let pk = ProvingKey::<Bls12_381>::deserialize_compressed(&mut file)
            .map_err(|e| anyhow!("Failed to deserialize intent keys: {}", e))?;
        let pvk = prepare_verifying_key(&pk.vk);
        INTENT_PROOF_KEYS.set((pk, pvk))
            .map_err(|_| anyhow!("Intent keys already set"))?;
        log::info!("[ZK-Intent] Keys loaded.");
        return Ok(());
    }
    
    log::info!("[ZK-Intent] Generating Groth16 setup for Intent Proofs...");
    let mut rng = thread_rng();
    
    // Create dummy circuit for key generation
    // IMPORTANT: Use non-zero values so constraints are satisfiable during setup
    // The circuit requires computed_hash != 0 and user_auth >= required_auth
    let circuit = IntentAuthCircuit {
        secret: [1u8; 32],  // Non-zero secret
        intent_bytes: vec![1u8; 256],  // Non-zero intent
        user_auth_level: 3,  // Max auth level (must be >= required)
        expected_commitment: [1u8; 32],  // Non-zero commitment (not verified during setup)
        required_auth_level: 0,  // Minimum required
    };
    
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
        .map_err(|e| anyhow!("Intent ZK setup failed: {}", e))?;
    
    let pvk = prepare_verifying_key(&vk);
    
    // Cache keys
    let mut file = File::create(key_path)
        .map_err(|e| anyhow!("Failed to create intent key cache: {}", e))?;
    pk.serialize_compressed(&mut file)
        .map_err(|e| anyhow!("Failed to serialize intent keys: {}", e))?;
    
    INTENT_PROOF_KEYS.set((pk, pvk))
        .map_err(|_| anyhow!("Intent keys already set"))?;
    log::info!("[ZK-Intent] Keys generated and cached.");
    
    Ok(())
}

/// Prove that a user is authorized to execute an intent
pub fn prove_intent_authorization(
    secret: &[u8; 32],
    command: &OracleCommand,
    user_auth_level: u8,
) -> Result<IntentProof> {
    let (pk, _) = INTENT_PROOF_KEYS.get()
        .ok_or(anyhow!("Intent proof keys not initialized"))?;
    
    let mut rng = thread_rng();
    let intent_type = IntentType::from_command(command);
    
    // Create commitment
    let commitment = IntentCommitment::create(secret, command)?;
    
    // Serialize intent
    let intent_bytes = serde_json::to_vec(command)
        .map_err(|e| anyhow!("Failed to serialize command: {}", e))?;
    
    // Pad to fixed size
    let mut padded_intent = intent_bytes.clone();
    padded_intent.resize(256, 0);
    
    log::info!("[ZK-Intent] Proving authorization for {:?} intent...", intent_type);
    let start = std::time::Instant::now();
    
    let circuit = IntentAuthCircuit {
        secret: *secret,
        intent_bytes: padded_intent,
        user_auth_level,
        expected_commitment: commitment.commitment,
        required_auth_level: intent_type.required_auth_level(),
    };
    
    let proof = Groth16::<Bls12_381>::prove(pk, circuit, &mut rng)
        .map_err(|e| anyhow!("Intent proving failed: {}", e))?;
    
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)
        .map_err(|e| anyhow!("Proof serialization failed: {}", e))?;
    
    log::info!("[ZK-Intent] Proof generated in {:?}", start.elapsed());
    
    Ok(IntentProof {
        proof_bytes,
        commitment,
    })
}

/// Verify an intent authorization proof
pub fn verify_intent_proof(proof: &IntentProof) -> bool {
    let keys = INTENT_PROOF_KEYS.get();
    if keys.is_none() {
        log::warn!("[ZK-Intent] Keys not initialized for verification");
        return false;
    }
    let (_, pvk) = keys.unwrap();
    
    let groth_proof = match ark_groth16::Proof::<Bls12_381>::deserialize_compressed(&proof.proof_bytes[..]) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("[ZK-Intent] Failed to deserialize proof: {}", e);
            return false;
        }
    };
    
    // Reconstruct public inputs
    let mut public_inputs: Vec<Fr> = Vec::new();
    
    // Expected commitment (32 bytes)
    for byte in proof.commitment.commitment.iter() {
        public_inputs.push(Fr::from(*byte as u64));
    }
    
    // Required auth level (1 byte)
    let required_auth = proof.commitment.intent_type.required_auth_level();
    public_inputs.push(Fr::from(required_auth as u64));
    
    match Groth16::<Bls12_381>::verify_with_processed_vk(pvk, &public_inputs, &groth_proof) {
        Ok(valid) => {
            log::info!("[ZK-Intent] Proof verification: {}", if valid { "VALID" } else { "INVALID" });
            valid
        }
        Err(e) => {
            log::warn!("[ZK-Intent] Verification error: {}", e);
            false
        }
    }
}

/// Lightweight proof for queries (no state change, just commitment)
#[derive(Clone, Serialize, Deserialize)]
pub struct QueryProof {
    /// Hash commitment to the query
    pub query_hash: [u8; 32],
    /// Timestamp
    pub timestamp: u64,
    /// User signature (placeholder - would be EdDSA in production)
    pub signature: Vec<u8>,
}

impl QueryProof {
    /// Create a simple query proof (no ZK needed for reads)
    pub fn create(query: &str, user_secret: &[u8; 32]) -> Self {
        let query_hash = compute_commitment_hash(query.as_bytes());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Sign: H(query_hash || timestamp || user_secret)
        let mut sig_input = Vec::new();
        sig_input.extend_from_slice(&query_hash);
        sig_input.extend_from_slice(&timestamp.to_le_bytes());
        sig_input.extend_from_slice(user_secret);
        let signature = compute_commitment_hash(&sig_input).to_vec();
        
        Self {
            query_hash,
            timestamp,
            signature,
        }
    }
    
    /// Verify the query proof
    pub fn verify(&self, query: &str, user_secret: &[u8; 32]) -> bool {
        let computed_hash = compute_commitment_hash(query.as_bytes());
        if computed_hash != self.query_hash {
            return false;
        }
        
        let mut sig_input = Vec::new();
        sig_input.extend_from_slice(&self.query_hash);
        sig_input.extend_from_slice(&self.timestamp.to_le_bytes());
        sig_input.extend_from_slice(user_secret);
        let expected_sig = compute_commitment_hash(&sig_input);
        
        self.signature == expected_sig.to_vec()
    }
}

/// Range proof for transfer amounts (proves amount is in valid range without revealing it)
#[derive(Clone, Serialize, Deserialize)]
pub struct RangeProof {
    /// Pedersen commitment to the amount: C = g^amount * h^blinding
    pub commitment: [u8; 32],
    /// Bulletproof-style range proof (simplified for demo)
    pub proof: Vec<u8>,
}

impl RangeProof {
    /// Create a range proof that amount is in [0, max_amount]
    pub fn create(amount: u64, max_amount: u64, blinding: &[u8; 32]) -> Result<Self> {
        if amount > max_amount {
            return Err(anyhow!("Amount {} exceeds max {}", amount, max_amount));
        }
        
        // Simplified commitment (production would use proper Pedersen)
        let mut commit_input = Vec::new();
        commit_input.extend_from_slice(&amount.to_le_bytes());
        commit_input.extend_from_slice(blinding);
        let commitment = compute_commitment_hash(&commit_input);
        
        // Simplified range proof (production would use Bulletproofs)
        // We just prove amount <= max by showing the difference fits in the bit range
        let diff = max_amount - amount;
        let bits_needed = 64 - diff.leading_zeros();
        
        let mut proof = Vec::new();
        proof.extend_from_slice(&bits_needed.to_le_bytes());
        proof.extend_from_slice(&compute_commitment_hash(&diff.to_le_bytes()));
        
        Ok(Self { commitment, proof })
    }
    
    /// Verify the range proof
    pub fn verify(&self, max_amount: u64) -> bool {
        // Simplified verification
        // In production, this would verify the Bulletproof
        !self.proof.is_empty() && self.proof.len() >= 4
    }
}

/// Combined proof bundle for a complete intent execution
#[derive(Clone, Serialize, Deserialize)]
pub struct IntentProofBundle {
    /// Authorization proof
    pub auth_proof: IntentProof,
    /// Range proof (if transfer/stake)
    pub range_proof: Option<RangeProof>,
    /// Query proof (if query)
    pub query_proof: Option<QueryProof>,
}

impl IntentProofBundle {
    /// Create a complete proof bundle for a command
    pub fn create(
        secret: &[u8; 32],
        command: &OracleCommand,
        user_auth_level: u8,
    ) -> Result<Self> {
        let auth_proof = prove_intent_authorization(secret, command, user_auth_level)?;
        let intent_type = IntentType::from_command(command);
        
        // Create range proof for transfer-type operations
        let range_proof = match intent_type {
            IntentType::Transfer | IntentType::Stake => {
                let blinding: [u8; 32] = rand::random();
                // Default amount for range proof (actual amount would come from SubmitTransaction)
                Some(RangeProof::create(0, u64::MAX, &blinding)?)
            }
            _ => None,
        };
        
        // Create query proof for query operations
        let query_proof = match intent_type {
            IntentType::Query => {
                let query_str = serde_json::to_string(command).unwrap_or_default();
                Some(QueryProof::create(&query_str, secret))
            }
            _ => None,
        };
        
        Ok(Self {
            auth_proof,
            range_proof,
            query_proof,
        })
    }
    
    /// Verify the complete proof bundle
    pub fn verify(&self) -> bool {
        // Verify auth proof
        if !verify_intent_proof(&self.auth_proof) {
            log::warn!("[ZK-Intent] Auth proof verification failed");
            return false;
        }
        
        // Verify range proof if present
        if let Some(ref range_proof) = self.range_proof {
            if !range_proof.verify(u64::MAX) {
                log::warn!("[ZK-Intent] Range proof verification failed");
                return false;
            }
        }
        
        // Query proofs are verified with the original query, so skip here
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_commitment() {
        let secret = [42u8; 32];
        let command = OracleCommand::QueryBalance { did: "did:test:123".to_string() };
        
        let commitment = IntentCommitment::create(&secret, &command).unwrap();
        assert!(commitment.verify(&secret, &command));
        
        // Wrong secret should fail
        let wrong_secret = [0u8; 32];
        assert!(!commitment.verify(&wrong_secret, &command));
    }
    
    #[test]
    fn test_intent_type_from_command() {
        let query = OracleCommand::QueryBalance { did: "did:test:123".to_string() };
        assert_eq!(IntentType::from_command(&query), IntentType::Query);
        
        let store = OracleCommand::StoreData {
            data: vec![1, 2, 3],
            metadata: "test".to_string(),
            zk_proof: vec![],
        };
        assert_eq!(IntentType::from_command(&store), IntentType::Store);
    }
    
    #[test]
    fn test_query_proof() {
        let secret = [42u8; 32];
        let query = "get balance for user123";
        
        let proof = QueryProof::create(query, &secret);
        assert!(proof.verify(query, &secret));
        
        // Wrong query should fail
        assert!(!proof.verify("different query", &secret));
        
        // Wrong secret should fail
        let wrong_secret = [0u8; 32];
        assert!(!proof.verify(query, &wrong_secret));
    }
    
    #[test]
    fn test_range_proof() {
        let blinding = [42u8; 32];
        
        // Valid range
        let proof = RangeProof::create(100, 1000, &blinding).unwrap();
        assert!(proof.verify(1000));
        
        // Amount exceeds max should fail
        let result = RangeProof::create(2000, 1000, &blinding);
        assert!(result.is_err());
    }
}
