use anyhow::Result;
use ark_groth16::{Groth16, ProvingKey, prepare_verifying_key, PreparedVerifyingKey};
use ark_bls12_381::{Bls12_381, Fr};
use ark_snark::SNARK;
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::rand::thread_rng;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use ark_std::Zero;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};

// Atom 8: Identity Circuit (ZK-Biometrics)
// Proves: I know the biometric data that hashes to the public commitment.
// Commitment = PolynomialHash(biometric_data)
// We use a simple polynomial rolling hash: H = sum(b_i * R^i)
const HASH_R: u64 = 12345;

struct BiometricCircuit<F: PrimeField> {
    pub biometric_data: Option<Vec<u8>>, // Private witness
    pub commitment: Option<F>,           // Public input
}

impl<F: PrimeField> ConstraintSynthesizer<F> for BiometricCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // 1. Allocate Witness (Biometric Data)
        // We expect a fixed size for the circuit, say 32 bytes (standard hash size)
        let data_len = 32;
        let mut data_vars = Vec::new();
        
        let data = self.biometric_data.unwrap_or(vec![0u8; data_len]);
        
        for i in 0..data_len {
            let val = if i < data.len() { F::from(data[i] as u64) } else { F::zero() };
            let var = FpVar::new_witness(cs.clone(), || Ok(val))?;
            data_vars.push(var);
        }
        
        // 2. Compute Polynomial Hash inside Circuit
        // H = (...((b_0 * R + b_1) * R + b_2) ... ) * R + b_n
        let r_const = FpVar::new_constant(cs.clone(), F::from(HASH_R))?;
        let mut computed_hash = FpVar::new_constant(cs.clone(), F::zero())?;
        
        for var in data_vars {
            computed_hash = computed_hash * &r_const + &var;
        }

        // 3. Allocate Public Input (Commitment)
        let commitment_var = FpVar::new_input(cs.clone(), || self.commitment.ok_or(SynthesisError::AssignmentMissing))?;

        // 4. Enforce Equality
        computed_hash.enforce_equal(&commitment_var)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub id: String, // did:karana:<hash>
    pub public_key: String,
    pub biometric_commitment: String, // Decimal string of the field element
}

pub struct KaranaIdentity {
    pk: ProvingKey<Bls12_381>,
    vk: PreparedVerifyingKey<Bls12_381>,
    active_did: Option<DID>,
}

impl KaranaIdentity {
    pub fn new() -> Result<Self> {
        // Setup ZK Circuit Keys
        let mut rng = thread_rng();
        // Provide dummy values for setup
        let circuit = BiometricCircuit::<Fr> { 
            biometric_data: Some(vec![0u8; 32]), 
            commitment: Some(Fr::zero()) 
        };
        let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
            .map_err(|e| anyhow::anyhow!("Identity ZK Setup failed: {}", e))?;
        let pvk = prepare_verifying_key(&vk);

        Ok(Self {
            pk,
            vk: pvk,
            active_did: None,
        })
    }

    fn compute_commitment(data: &[u8]) -> Fr {
        let mut hash = Fr::zero();
        let r = Fr::from(HASH_R);
        // Pad or truncate to 32 bytes to match circuit
        let mut fixed_data = vec![0u8; 32];
        for (i, &b) in data.iter().take(32).enumerate() {
            fixed_data[i] = b;
        }
        
        for &b in fixed_data.iter() {
            hash = hash * r + Fr::from(b as u64);
        }
        hash
    }

    pub fn create_did(&mut self, public_key: &str, biometric_sample: &[u8]) -> Result<DID> {
        // 1. Compute Commitment
        let commitment_fr = Self::compute_commitment(biometric_sample);
        
        // 2. Generate DID String
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let did_hash = hex::encode(hasher.finalize());
        let did_str = format!("did:karana:{}", &did_hash[0..16]);

        // Convert Fr to string for storage
        let mut bytes = Vec::new();
        commitment_fr.serialize_compressed(&mut bytes)?;
        let commitment_str = hex::encode(bytes);

        let did = DID {
            id: did_str,
            public_key: public_key.to_string(),
            biometric_commitment: commitment_str,
        };

        self.active_did = Some(did.clone());
        Ok(did)
    }

    pub fn authenticate(&self, biometric_sample: &[u8]) -> Result<Vec<u8>> {
        if self.active_did.is_none() {
            return Err(anyhow::anyhow!("No active DID found. Create one first."));
        }
        let did = self.active_did.as_ref().unwrap();

        // 1. Verify Sample matches Commitment (Local Check)
        let commitment_fr = Self::compute_commitment(biometric_sample);
        
        let expected_bytes = hex::decode(&did.biometric_commitment)?;
        let expected_fr = Fr::deserialize_compressed(&mut &expected_bytes[..])?;
        
        if commitment_fr != expected_fr {
             return Err(anyhow::anyhow!("Biometric mismatch! Authentication failed."));
        }

        // 2. Generate ZK Proof
        // Ensure data is 32 bytes for circuit
        let mut fixed_data = vec![0u8; 32];
        for (i, &b) in biometric_sample.iter().take(32).enumerate() {
            fixed_data[i] = b;
        }

        let circuit = BiometricCircuit {
            biometric_data: Some(fixed_data),
            commitment: Some(commitment_fr),
        };

        let mut rng = thread_rng();
        let proof = Groth16::<Bls12_381>::prove(&self.pk, circuit, &mut rng)
            .map_err(|e| anyhow::anyhow!("Auth Proof generation failed: {}", e))?;

        // 3. Serialize Proof
        use ark_serialize::CanonicalSerialize;
        let mut proof_bytes = Vec::new();
        proof.serialize_compressed(&mut proof_bytes)?;

        Ok(proof_bytes)
    }

    pub fn verify_auth(&self, proof_bytes: &[u8], commitment_hex: &str) -> bool {
        use ark_serialize::CanonicalDeserialize;
        
        let proof = match ark_groth16::Proof::<Bls12_381>::deserialize_compressed(proof_bytes) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let commitment_bytes = match hex::decode(commitment_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };
        
        let commitment_val = match Fr::deserialize_compressed(&mut &commitment_bytes[..]) {
            Ok(v) => v,
            Err(_) => return false,
        };
        
        Groth16::<Bls12_381>::verify_with_processed_vk(
            &self.vk,
            &[commitment_val],
            &proof
        ).unwrap_or(false)
    }
    
    pub fn get_active_did(&self) -> Option<String> {
        self.active_did.as_ref().map(|d| d.id.clone())
    }
}
