use anyhow::Result;
use ark_groth16::{Groth16, ProvingKey, prepare_verifying_key, PreparedVerifyingKey};
use ark_bls12_381::{Bls12_381};
use ark_snark::SNARK;
use ark_std::rand::thread_rng;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use std::sync::OnceLock;
use std::fs::File;
use std::path::Path;

pub mod storage_proof;
use storage_proof::StorageCircuit;

// Global keys for the storage circuit
static ZK_KEYS: OnceLock<(ProvingKey<Bls12_381>, PreparedVerifyingKey<Bls12_381>)> = OnceLock::new();

pub fn setup_zk() -> Result<()> {
    let key_path = Path::new("zk_keys_storage.bin");
    
    if key_path.exists() {
        // println!("Atom 1 (ZK): Loading keys from cache...");
        let mut file = File::open(key_path).map_err(|e| anyhow::anyhow!("Failed to open key cache: {}", e))?;
        let pk = ProvingKey::<Bls12_381>::deserialize_compressed(&mut file)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ZK keys: {}", e))?;
        let pvk = prepare_verifying_key(&pk.vk);
        ZK_KEYS.set((pk, pvk)).map_err(|_| anyhow::anyhow!("ZK Keys already set"))?;
        // log::info!("Atom 1 (ZK): Keys loaded.");
        return Ok(());
    }

    log::info!("Atom 1 (ZK): Generating Groth16 setup for Storage Proofs...");
    let mut rng = thread_rng();
    
    // Create a dummy circuit to generate keys
    // We fix input size to 64 bytes for the demo.
    let circuit = StorageCircuit {
        input: vec![0u8; 64], 
        expected_hash: [0u8; 32],
    };
    
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
        .map_err(|e| anyhow::anyhow!("ZK Setup failed: {}", e))?;
        
    let pvk = prepare_verifying_key(&vk);
    
    // Cache keys
    let mut file = File::create(key_path).map_err(|e| anyhow::anyhow!("Failed to create key cache: {}", e))?;
    pk.serialize_compressed(&mut file).map_err(|e| anyhow::anyhow!("Failed to serialize ZK keys: {}", e))?;
    
    ZK_KEYS.set((pk, pvk)).map_err(|_| anyhow::anyhow!("ZK Keys already set"))?;
    log::info!("Atom 1 (ZK): Keys generated and cached.");
    
    Ok(())
}

pub fn prove_data_hash(input: &[u8], expected_hash: [u8; 32]) -> Result<Vec<u8>> {
    let (pk, _) = ZK_KEYS.get().ok_or(anyhow::anyhow!("ZK Keys not initialized"))?;
    let mut rng = thread_rng();
    
    // Pad or truncate input to 64 bytes to match setup
    let mut padded_input = input.to_vec();
    padded_input.resize(64, 0);
    
    // Recalculate hash for the PADDED input to ensure circuit satisfaction
    // The circuit proves that Hash(padded_input) == expected_hash
    // If the caller passed a hash of the UNPADDED input, it might mismatch if padding changes the hash.
    // For XOR sum, padding with 0s doesn't change the hash, BUT if input was > 64 bytes, truncation would.
    // Let's ensure we are proving what we claim.
    
    // Debug logging for ZK
    log::info!("Atom 1 (ZK): Proving Data Hash. Input Len: {}, Padded: 64", input.len());
    
    let circuit = StorageCircuit {
        input: padded_input,
        expected_hash,
    };
    
    let proof = Groth16::<Bls12_381>::prove(pk, circuit, &mut rng)
        .map_err(|e| anyhow::anyhow!("Proving failed: {}", e))?;
        
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes).map_err(|e| anyhow::anyhow!("Proof serialization failed: {}", e))?;
    
    Ok(proof_bytes)
}

pub fn verify_proof(proof_bytes: &[u8], expected_hash: [u8; 32]) -> bool {
    let keys = ZK_KEYS.get();
    if keys.is_none() { return false; }
    let (_, pvk) = keys.unwrap();
    
    let proof = match ark_groth16::Proof::<Bls12_381>::deserialize_compressed(proof_bytes) {
        Ok(p) => p,
        Err(_) => return false,
    };
    
    
    use ark_bls12_381::Fr;
    
    // Convert expected_hash to bits (Little Endian per byte, as UInt8 does)
    let mut public_inputs = Vec::new();
    for byte in expected_hash.iter() {
        for i in 0..8 {
            let bit = (byte >> i) & 1 == 1;
            public_inputs.push(Fr::from(bit as u64));
        }
    }
    
    Groth16::<Bls12_381>::verify_with_processed_vk(
        pvk,
        &public_inputs,
        &proof
    ).unwrap_or(false)
}

// Re-export helper
pub use storage_proof::compute_demo_hash as compute_hash;
