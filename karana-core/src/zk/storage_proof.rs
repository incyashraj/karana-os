use ark_bls12_381::Fr;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::vec::Vec;

#[derive(Clone)]
pub struct StorageCircuit {
    pub input: Vec<u8>,
    pub expected_hash: [u8; 32],
}

impl ConstraintSynthesizer<Fr> for StorageCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // 1. Allocate Input Bytes (Witnesses - private data)
        let input_bytes: Vec<UInt8<Fr>> = self.input
            .iter()
            .map(|b| UInt8::new_witness(cs.clone(), || Ok(b)))
            .collect::<Result<Vec<_>, _>>()?;

        // 2. Compute "Hash"
        // We'll use a simpler one: XOR sum of all bytes.
        let mut xor_sum = UInt8::constant(0u8);
        for byte in &input_bytes {
            // Use BitXor trait
            xor_sum = xor_sum ^ byte;
        }

        // 3. Allocate Expected Hash (Public Input)
        let expected_bytes: Vec<UInt8<Fr>> = self.expected_hash
            .iter()
            .map(|b| UInt8::new_input(cs.clone(), || Ok(b)))
            .collect::<Result<Vec<_>, _>>()?;

        // 4. Enforce Equality
        // We only check the first byte against the XOR sum for this demo
        xor_sum.enforce_equal(&expected_bytes[0])?;
        
        // Ensure other bytes are 0
        let zero = UInt8::constant(0u8);
        for i in 1..32 {
            expected_bytes[i].enforce_equal(&zero)?;
        }

        Ok(())
    }
}

// Helper to compute the same hash outside the circuit
pub fn compute_demo_hash(input: &[u8]) -> [u8; 32] {
    let mut xor_sum = 0u8;
    for b in input {
        xor_sum ^= b;
    }
    let mut hash = [0u8; 32];
    hash[0] = xor_sum;
    hash
}
