use ark_groth16::{Groth16, prepare_verifying_key};
use ark_bls12_381::{Bls12_381, Fr};
use ark_snark::SNARK;
use ark_ff::PrimeField;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::rand::thread_rng;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::fields::fp::FpVar;

struct BootCircuit<F: PrimeField> {
    pub path: Option<Vec<u8>>,
    pub hash: Option<F>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for BootCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let path_val = self.path.as_ref().map(|p| {
            let sum: u64 = p.iter().map(|&b| b as u64).sum();
            F::from(sum)
        }).unwrap_or(F::zero());

        println!("DEBUG: Generating Constraints. Path Sum (Witness): {:?}", path_val);

        let path_var = FpVar::new_witness(cs.clone(), || Ok(path_val))?;
        
        let hash_val = self.hash.unwrap_or(F::zero());
        println!("DEBUG: Generating Constraints. Hash (Public Input): {:?}", hash_val);
        
        let hash_var = FpVar::new_input(cs.clone(), || self.hash.ok_or(SynthesisError::AssignmentMissing))?;

        path_var.enforce_equal(&hash_var)?;

        println!("DEBUG: CS Satisfied? {:?}", cs.is_satisfied());

        Ok(())
    }
}

fn main() {
    println!("Starting ZK Test...");
    let mut rng = thread_rng();

    // Setup
    println!("Setup...");
    let circuit = BootCircuit::<Fr> { 
        path: Some(vec![0u8]), 
        hash: Some(Fr::from(0u64)) 
    };
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng).unwrap();
    let pvk = prepare_verifying_key(&vk);

    // Prove
    println!("Prove...");
    let safe_path = vec![1u8; 32];
    let safe_sum = 32u64;
    
    let circuit = BootCircuit {
        path: Some(safe_path),
        hash: Some(Fr::from(safe_sum)),
    };

    let proof = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng).unwrap();
    println!("Proof generated!");

    // Verify
    println!("Verify...");
    let valid = Groth16::<Bls12_381>::verify_with_processed_vk(
        &pvk,
        &[Fr::from(safe_sum)],
        &proof
    ).unwrap();

    println!("Verification result: {}", valid);
}
