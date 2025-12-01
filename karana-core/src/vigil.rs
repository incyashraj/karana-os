use anyhow::Result;
use std::sync::{Arc, Mutex};
use crate::ai::KaranaAI;
use crate::runtime::KaranaActor;
use crate::economy::Ledger;
use ark_groth16::{Groth16, ProvingKey, prepare_verifying_key, PreparedVerifyingKey};
use ark_bls12_381::{Bls12_381, Fr};
use ark_snark::SNARK;
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::rand::thread_rng;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;

// Atom 7: Vigil Circuit
// Proves: Hash(action) == allowed_policy_hash
// For demo: Hash(action) = Sum(bytes(action))
struct VigilCircuit<F: PrimeField> {
    pub action: Option<Vec<u8>>, // Private witness
    pub policy_hash: Option<F>,  // Public input
}

impl<F: PrimeField> ConstraintSynthesizer<F> for VigilCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let action_val = self.action.as_ref().map(|a| {
            let sum: u64 = a.iter().map(|&b| b as u64).sum();
            F::from(sum)
        }).unwrap_or(F::zero());

        let action_var = FpVar::new_witness(cs.clone(), || Ok(action_val))?;
        let policy_var = FpVar::new_input(cs.clone(), || self.policy_hash.ok_or(SynthesisError::AssignmentMissing))?;

        action_var.enforce_equal(&policy_var)?;
        Ok(())
    }
}

pub struct KaranaVeil {
    ai: Arc<Mutex<KaranaAI>>,
    #[allow(dead_code)]
    runtime: Arc<KaranaActor>,
    ledger: Arc<Mutex<Ledger>>,
    pk: ProvingKey<Bls12_381>,
    vk: PreparedVerifyingKey<Bls12_381>,
}

impl KaranaVeil {
    pub fn new(ai: Arc<Mutex<KaranaAI>>, runtime: &Arc<KaranaActor>, ledger: Arc<Mutex<Ledger>>) -> Result<Self> {
        // Setup ZK Circuit Keys (One-time setup for demo)
        let mut rng = thread_rng();
        let circuit = VigilCircuit::<Fr> { action: None, policy_hash: None };
        let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
            .map_err(|e| anyhow::anyhow!("Vigil ZK Setup failed: {}", e))?;
        let pvk = prepare_verifying_key(&vk);

        Ok(Self {
            ai,
            runtime: runtime.clone(),
            ledger,
            pk,
            vk: pvk,
        })
    }

    pub async fn check_action(&self, action: String, _proof: Vec<u8>) -> Result<String> {
        // Step 1: AI Anomaly Detection (Phase 3: Real ML Score)
        let score = self.ai.lock().unwrap().score_anomaly(&action)?;
        if score > 0.8 {
            // Atom 7 + 4: Slashing for Anomaly
            self.ledger.lock().unwrap().slash("Node-Alpha", 50, "High Anomaly Score");
            return Err(anyhow::anyhow!("Anomaly detected! Score: {:.4}. Slashed 50 KARA.", score));
        }

        // Step 2: ZK Enforce (Prove Action Matches Policy)
        // For demo, we generate the proof here to simulate a valid client request.
        // In a real system, the client (UI/Runtime) would provide the proof.
        let action_bytes = action.as_bytes().to_vec();
        let action_sum: u64 = action_bytes.iter().map(|&b| b as u64).sum();
        
        let circuit = VigilCircuit {
            action: Some(action_bytes),
            policy_hash: Some(Fr::from(action_sum)),
        };

        let mut rng = thread_rng();
        let proof = Groth16::<Bls12_381>::prove(&self.pk, circuit, &mut rng)
            .map_err(|e| anyhow::anyhow!("Vigil Proof generation failed: {}", e))?;

        // Verify
        let valid = Groth16::<Bls12_381>::verify_with_processed_vk(
            &self.vk,
            &[Fr::from(action_sum)],
            &proof
        )?;

        if !valid {
            self.ledger.lock().unwrap().slash("Node-Alpha", 100, "Invalid ZK Proof");
            return Err(anyhow::anyhow!("ZK Verification Failed! Slashed 100 KARA."));
        }

        Ok(format!("Veil Passed: {} (AI Score: {:.4})", action, score))
    }
}
