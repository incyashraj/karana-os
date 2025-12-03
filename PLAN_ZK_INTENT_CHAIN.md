# Plan: ZK Intent Chain Implementation

## Overview
Every user intent must be ZK-proven before execution. This establishes sovereignty - cryptographic proof that "this intent is mine."

---

## Target Architecture

```
User Voice → Oracle Parse → ZK-Prove Intent → Monad Execute → Chain Attest
                              ↓
                         [Groth16 Proof]
                              ↓
                    "I (DID) commanded this action"
```

---

## Current State

### What Exists:
- `zk/mod.rs` - Groth16 setup and proving for storage
- `zk/storage_proof.rs` - Circuit for data hash proofs
- Batch proving infrastructure

### What's Missing:
- Intent-specific circuit
- Intent commitment structure
- ZK-signing in Oracle flow
- Proof verification at Monad entry

---

## Implementation Plan

### File: `karana-core/src/zk/intent_proof.rs`

### Step 1: Intent Circuit

```rust
use ark_bls12_381::Fr;
use ark_r1cs_std::{
    prelude::*,
    uint8::UInt8,
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use sha2::{Sha256, Digest};

/// Circuit that proves: "I know the preimage of this intent commitment"
/// Public inputs: commitment hash (32 bytes = 256 bits)
/// Private inputs: intent_data, did, nonce
pub struct IntentCircuit {
    /// The intent data (action + params JSON)
    pub intent_data: Vec<u8>,
    
    /// User's DID (proves ownership)
    pub did: Vec<u8>,
    
    /// Nonce for replay protection
    pub nonce: u64,
    
    /// Expected commitment: H(intent_data || did || nonce)
    pub expected_commitment: [u8; 32],
}

impl ConstraintSynthesizer<Fr> for IntentCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate private inputs
        let intent_bits: Vec<UInt8<Fr>> = self.intent_data
            .iter()
            .enumerate()
            .map(|(i, &byte)| UInt8::new_witness(cs.clone(), || Ok(byte)))
            .collect::<Result<Vec<_>, _>>()?;
        
        let did_bits: Vec<UInt8<Fr>> = self.did
            .iter()
            .enumerate()
            .map(|(i, &byte)| UInt8::new_witness(cs.clone(), || Ok(byte)))
            .collect::<Result<Vec<_>, _>>()?;
        
        let nonce_bytes = self.nonce.to_le_bytes();
        let nonce_bits: Vec<UInt8<Fr>> = nonce_bytes
            .iter()
            .enumerate()
            .map(|(i, &byte)| UInt8::new_witness(cs.clone(), || Ok(byte)))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Allocate public output (commitment)
        let commitment_bits: Vec<Boolean<Fr>> = self.expected_commitment
            .iter()
            .flat_map(|byte| {
                (0..8).map(move |i| {
                    Boolean::new_input(cs.clone(), || Ok((byte >> i) & 1 == 1))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        // Compute hash inside circuit
        // For simplicity, use XOR-based hash (real impl would use Poseidon)
        let mut hash_result = vec![UInt8::constant(0); 32];
        
        // XOR all intent bytes
        for (i, byte) in intent_bits.iter().enumerate() {
            let idx = i % 32;
            hash_result[idx] = hash_result[idx].xor(byte)?;
        }
        
        // XOR all DID bytes
        for (i, byte) in did_bits.iter().enumerate() {
            let idx = i % 32;
            hash_result[idx] = hash_result[idx].xor(byte)?;
        }
        
        // XOR nonce bytes
        for (i, byte) in nonce_bits.iter().enumerate() {
            let idx = i % 32;
            hash_result[idx] = hash_result[idx].xor(byte)?;
        }
        
        // Verify hash matches commitment
        for (i, byte) in hash_result.iter().enumerate() {
            let byte_bits = byte.to_bits_le()?;
            for (j, bit) in byte_bits.iter().enumerate() {
                let expected_bit = &commitment_bits[i * 8 + j];
                bit.enforce_equal(expected_bit)?;
            }
        }
        
        Ok(())
    }
}
```

### Step 2: Intent Proof Functions

```rust
use ark_groth16::{Groth16, ProvingKey, PreparedVerifyingKey, prepare_verifying_key};
use ark_bls12_381::Bls12_381;
use ark_snark::SNARK;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use std::sync::OnceLock;

// Global keys for intent circuit
static INTENT_ZK_KEYS: OnceLock<(ProvingKey<Bls12_381>, PreparedVerifyingKey<Bls12_381>)> = OnceLock::new();

/// Setup ZK keys for intent proofs
pub fn setup_intent_zk() -> anyhow::Result<()> {
    let key_path = std::path::Path::new("zk_keys_intent.bin");
    
    if key_path.exists() {
        log::info!("[ZK-INTENT] Loading keys from cache...");
        let mut file = std::fs::File::open(key_path)?;
        let pk = ProvingKey::<Bls12_381>::deserialize_compressed(&mut file)?;
        let pvk = prepare_verifying_key(&pk.vk);
        INTENT_ZK_KEYS.set((pk, pvk)).map_err(|_| anyhow::anyhow!("Keys already set"))?;
        return Ok(());
    }
    
    log::info!("[ZK-INTENT] Generating Groth16 setup for Intent Proofs...");
    let mut rng = ark_std::rand::thread_rng();
    
    // Dummy circuit for setup (max 256 bytes intent + 64 bytes DID)
    let circuit = IntentCircuit {
        intent_data: vec![0u8; 256],
        did: vec![0u8; 64],
        nonce: 0,
        expected_commitment: [0u8; 32],
    };
    
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)?;
    let pvk = prepare_verifying_key(&vk);
    
    // Cache keys
    let mut file = std::fs::File::create(key_path)?;
    pk.serialize_compressed(&mut file)?;
    
    INTENT_ZK_KEYS.set((pk, pvk)).map_err(|_| anyhow::anyhow!("Keys already set"))?;
    log::info!("[ZK-INTENT] Keys generated and cached.");
    
    Ok(())
}

/// Compute intent commitment: H(intent || did || nonce)
pub fn compute_intent_commitment(intent_data: &[u8], did: &[u8], nonce: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(intent_data);
    hasher.update(did);
    hasher.update(&nonce.to_le_bytes());
    hasher.finalize().into()
}

/// Prove an intent is yours (ZK-sign)
pub fn prove_intent(
    intent_data: &[u8],
    did: &[u8],
    nonce: u64,
) -> anyhow::Result<IntentProof> {
    let (pk, _) = INTENT_ZK_KEYS.get()
        .ok_or_else(|| anyhow::anyhow!("Intent ZK keys not initialized"))?;
    
    let mut rng = ark_std::rand::thread_rng();
    
    // Compute commitment
    let commitment = compute_intent_commitment(intent_data, did, nonce);
    
    // Pad inputs to match circuit size
    let mut padded_intent = intent_data.to_vec();
    padded_intent.resize(256, 0);
    
    let mut padded_did = did.to_vec();
    padded_did.resize(64, 0);
    
    let circuit = IntentCircuit {
        intent_data: padded_intent,
        did: padded_did,
        nonce,
        expected_commitment: commitment,
    };
    
    let start = std::time::Instant::now();
    let proof = Groth16::<Bls12_381>::prove(pk, circuit, &mut rng)?;
    log::info!("[ZK-INTENT] Proof generated in {:?}", start.elapsed());
    
    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)?;
    
    Ok(IntentProof {
        commitment,
        proof_bytes,
        nonce,
    })
}

/// Verify an intent proof
pub fn verify_intent_proof(proof: &IntentProof) -> bool {
    let keys = INTENT_ZK_KEYS.get();
    if keys.is_none() {
        return false;
    }
    let (_, pvk) = keys.unwrap();
    
    let groth_proof = match ark_groth16::Proof::<Bls12_381>::deserialize_compressed(&proof.proof_bytes[..]) {
        Ok(p) => p,
        Err(_) => return false,
    };
    
    // Convert commitment to public inputs (bits)
    let mut public_inputs = Vec::new();
    for byte in proof.commitment.iter() {
        for i in 0..8 {
            let bit = (byte >> i) & 1 == 1;
            public_inputs.push(Fr::from(bit as u64));
        }
    }
    
    Groth16::<Bls12_381>::verify_with_processed_vk(pvk, &public_inputs, &groth_proof)
        .unwrap_or(false)
}

/// Intent proof structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntentProof {
    /// H(intent || did || nonce)
    pub commitment: [u8; 32],
    
    /// Groth16 proof bytes
    pub proof_bytes: Vec<u8>,
    
    /// Nonce used (for replay protection)
    pub nonce: u64,
}
```

### Step 3: Update `zk/mod.rs`

```rust
// Add to existing mod.rs

pub mod intent_proof;
pub use intent_proof::{
    setup_intent_zk,
    prove_intent,
    verify_intent_proof,
    compute_intent_commitment,
    IntentProof,
};

// Update setup_zk to include intent circuit
pub fn setup_zk() -> Result<()> {
    // Existing storage proof setup
    setup_storage_zk()?;
    
    // NEW: Intent proof setup
    setup_intent_zk()?;
    
    Ok(())
}
```

---

## Integration with Oracle

### In `oracle/veil.rs`:

```rust
impl OracleVeil {
    pub async fn mediate(&mut self, input: MultimodalInput) -> Result<Manifest> {
        // Step 1: Parse intent
        let intent = self.parse_intent(&input).await?;
        
        // Step 2: ZK-Sign the intent
        let intent_bytes = serde_json::to_vec(&intent)?;
        let did_bytes = self.user_did.as_bytes();
        let nonce = self.get_next_nonce();
        
        let proof = crate::zk::prove_intent(&intent_bytes, did_bytes, nonce)?;
        log::info!("[ORACLE] Intent ZK-signed: commitment={}", hex::encode(&proof.commitment[..8]));
        
        // Step 3: Send to Monad with proof
        let command = OracleCommand::ExecuteIntent {
            intent: intent.clone(),
            proof: proof.clone(),
        };
        self.monad_tx.send(command).await?;
        
        // ... rest of mediate
    }
}
```

### In `monad.rs`:

```rust
impl KaranaMonad {
    pub async fn handle_oracle_command(&mut self, cmd: OracleCommand) -> MonadResponse {
        match cmd {
            OracleCommand::ExecuteIntent { intent, proof } => {
                // VERIFY PROOF FIRST
                if !crate::zk::verify_intent_proof(&proof) {
                    log::warn!("[MONAD] Invalid intent proof rejected!");
                    return MonadResponse {
                        success: false,
                        data: "Invalid ZK proof".into(),
                        proof_hash: None,
                        chain_tx: None,
                    };
                }
                
                log::info!("[MONAD] Intent proof verified ✓");
                
                // Now execute the intent
                let result = self.execute_verified_intent(&intent).await?;
                
                // Attest to chain
                let tx = self.chain.attest_intent(&self.user_did, &intent, &proof, &result);
                
                MonadResponse {
                    success: true,
                    data: result,
                    proof_hash: Some(proof.commitment.to_vec()),
                    chain_tx: Some(tx.hash),
                }
            },
            // ... other commands
        }
    }
}
```

---

## Nonce Management

To prevent replay attacks, each intent needs a unique nonce:

```rust
// In OracleVeil:
pub struct OracleVeil {
    // ...
    nonce_counter: AtomicU64,
}

impl OracleVeil {
    fn get_next_nonce(&self) -> u64 {
        self.nonce_counter.fetch_add(1, Ordering::SeqCst)
    }
}

// In Monad, track seen nonces:
pub struct NonceTracker {
    seen: HashSet<(String, u64)>,  // (DID, nonce)
}

impl NonceTracker {
    pub fn check_and_mark(&mut self, did: &str, nonce: u64) -> bool {
        let key = (did.to_string(), nonce);
        if self.seen.contains(&key) {
            return false;  // Replay!
        }
        self.seen.insert(key);
        true
    }
}
```

---

## Performance Optimization

### Batch Intent Proving

For multiple rapid intents, batch them:

```rust
pub struct IntentBatch {
    intents: Vec<(Vec<u8>, Vec<u8>, u64)>,  // (intent, did, nonce)
    max_size: usize,
}

impl IntentBatch {
    pub fn add(&mut self, intent: &[u8], did: &[u8], nonce: u64) -> bool {
        if self.intents.len() >= self.max_size {
            return false;
        }
        self.intents.push((intent.to_vec(), did.to_vec(), nonce));
        true
    }
    
    pub fn prove_all(&mut self) -> Vec<IntentProof> {
        let items = std::mem::take(&mut self.intents);
        items.into_iter()
            .filter_map(|(i, d, n)| prove_intent(&i, &d, n).ok())
            .collect()
    }
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_commitment() {
        let intent = b"transfer 50 to alice";
        let did = b"did:karana:abc123";
        let nonce = 42;
        
        let commitment = compute_intent_commitment(intent, did, nonce);
        assert_eq!(commitment.len(), 32);
        
        // Same inputs = same commitment
        let commitment2 = compute_intent_commitment(intent, did, nonce);
        assert_eq!(commitment, commitment2);
        
        // Different nonce = different commitment
        let commitment3 = compute_intent_commitment(intent, did, 43);
        assert_ne!(commitment, commitment3);
    }
    
    #[test]
    fn test_intent_proof_roundtrip() {
        setup_intent_zk().unwrap();
        
        let intent = b"tune battery";
        let did = b"did:karana:user1";
        let nonce = 1;
        
        let proof = prove_intent(intent, did, nonce).unwrap();
        assert!(verify_intent_proof(&proof));
    }
    
    #[test]
    fn test_invalid_proof_rejected() {
        setup_intent_zk().unwrap();
        
        let proof = IntentProof {
            commitment: [0u8; 32],
            proof_bytes: vec![0u8; 100],  // Garbage proof
            nonce: 0,
        };
        
        assert!(!verify_intent_proof(&proof));
    }
}
```

---

## Timeline

| Task | Duration |
|------|----------|
| Create `intent_proof.rs` | 3 hours |
| Update `zk/mod.rs` | 1 hour |
| Integrate with OracleVeil | 2 hours |
| Integrate with Monad | 2 hours |
| Nonce management | 1 hour |
| Testing | 2 hours |
| **Total** | **11 hours** |

---

## Success Criteria

- [ ] `prove_intent()` generates valid proofs
- [ ] `verify_intent_proof()` correctly validates
- [ ] All Oracle intents are ZK-signed
- [ ] Invalid proofs rejected by Monad
- [ ] Proof generation < 200ms
- [ ] Replay attacks prevented via nonces

---

*ZK Intent Chain Plan v1.0 - December 3, 2025*
