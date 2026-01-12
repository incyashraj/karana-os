# Zero-Knowledge Proof System

## Overview

The Zero-Knowledge Proof (ZK) System enables privacy-preserving verification in Kāraṇa OS. It allows proving statements without revealing underlying data using Groth16 SNARKs, enabling private oracle responses, confidential transactions, and identity attestations.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   ZERO-KNOWLEDGE PROOF SYSTEM                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Circuit Compiler (Circom)                      │  │
│  │  .circom → R1CS → Witness Generator (WASM)                        │  │
│  └────────────────────┬─────────────────────────────────────────────┘  │
│                       │                                                  │
│  ┌────────────────────▼─────────────────────────────────────────────┐  │
│  │                  Trusted Setup (Powers of Tau)                    │  │
│  │  Phase 1: Universal | Phase 2: Circuit-specific                  │  │
│  └────────┬───────────────────────────────────────────┬─────────────┘  │
│           │                                            │                 │
│  ┌────────▼────────────┐                    ┌─────────▼──────────┐     │
│  │   Proof Generator   │                    │  Proof Verifier    │     │
│  │   (Prover)          │                    │  (Smart Contract)  │     │
│  │   300-800ms         │                    │  On-chain: 250K gas│     │
│  └─────────────────────┘                    └────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Circuit Compiler (Circom)

**Purpose**: Define computational statements as arithmetic circuits.

**Example Circuit** (Oracle Response Proof):
```circom
pragma circom 2.0.0;

include "circomlib/poseidon.circom";

// Prove knowledge of oracle response without revealing it
template OracleResponseProof() {
    // Private inputs
    signal input request_id;
    signal input response_data;
    signal input oracle_private_key;
    
    // Public inputs
    signal input response_hash;
    signal input oracle_public_key;
    
    // Intermediate signals
    signal computed_hash;
    signal signature;
    
    // 1. Verify response hash
    component hasher = Poseidon(2);
    hasher.inputs[0] <== request_id;
    hasher.inputs[1] <== response_data;
    computed_hash <== hasher.out;
    
    // Constrain: computed hash must equal public hash
    computed_hash === response_hash;
    
    // 2. Verify oracle signature
    // (Simplified - real implementation uses EdDSA)
    signal computed_pubkey;
    computed_pubkey <== oracle_private_key * 9; // Elliptic curve scalar mult
    
    // Constrain: private key corresponds to public key
    computed_pubkey === oracle_public_key;
    
    // 3. Generate commitment
    signal output commitment;
    component commitment_hasher = Poseidon(3);
    commitment_hasher.inputs[0] <== response_hash;
    commitment_hasher.inputs[1] <== oracle_public_key;
    commitment_hasher.inputs[2] <== timestamp;
    commitment <== commitment_hasher.out;
}

component main {public [response_hash, oracle_public_key]} = OracleResponseProof();
```

**Compilation**:
```bash
# Compile circuit
circom oracle_response_proof.circom --r1cs --wasm --sym

# Outputs:
# - oracle_response_proof.r1cs (Rank-1 Constraint System)
# - oracle_response_proof_js/oracle_response_proof.wasm (Witness generator)
# - oracle_response_proof.sym (Symbol table)
```

**R1CS Constraints**:
- **Variables**: Input/output/intermediate signals
- **Constraints**: `A * B = C` equations
- **Example**: `x * x = y` proves `y` is square of `x`

---

### 2. Trusted Setup (Powers of Tau)

**Purpose**: Generate cryptographic parameters for proof system.

**Phase 1: Universal Setup** (One-time for all circuits):
```bash
# Start ceremony
snarkjs powersoftau new bn128 12 pot12_0000.ptau -v

# Contribute randomness (multiple participants)
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau \
  --name="Contributor 1" -v

# Additional contributions...
snarkjs powersoftau contribute pot12_0001.ptau pot12_0002.ptau \
  --name="Contributor 2" -v

# Beacon (random seed from external source)
snarkjs powersoftau beacon pot12_0002.ptau pot12_beacon.ptau \
  0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -v

# Prepare phase 2
snarkjs powersoftau prepare phase2 pot12_beacon.ptau pot12_final.ptau -v
```

**Phase 2: Circuit-Specific Setup**:
```bash
# Generate zkey for specific circuit
snarkjs groth16 setup oracle_response_proof.r1cs pot12_final.ptau \
  oracle_response_proof_0000.zkey

# Contribute circuit-specific randomness
snarkjs zkey contribute oracle_response_proof_0000.zkey \
  oracle_response_proof_final.zkey --name="Circuit Contributor 1" -v

# Export verification key
snarkjs zkey export verificationkey oracle_response_proof_final.zkey \
  verification_key.json
```

**Security**:
- Only one honest participant needed (1-of-N trust)
- Toxic waste (randomness) must be destroyed
- Multi-party computation (MPC) ensures security

---

### 3. Proof Generator (Prover)

**Purpose**: Generate zero-knowledge proofs given witness data.

**Witness Generation**:
```typescript
import * as snarkjs from 'snarkjs';

class ProofGenerator {
  async generateProof(witness: Witness): Promise<Proof> {
    // 1. Load WASM witness calculator
    const wasm = await fetch('/circuits/oracle_response_proof.wasm');
    const wasmBuffer = await wasm.arrayBuffer();
    
    // 2. Calculate witness
    const witnessCalculator = await snarkjs.wtns.calculateWitness(
      wasmBuffer,
      witness
    );
    
    // 3. Load zkey
    const zkey = await fetch('/circuits/oracle_response_proof_final.zkey');
    const zkeyBuffer = await zkey.arrayBuffer();
    
    // 4. Generate proof (Groth16)
    const startTime = performance.now();
    const { proof, publicSignals } = await snarkjs.groth16.prove(
      zkeyBuffer,
      witnessCalculator
    );
    const proofTime = performance.now() - startTime;
    
    console.log(`Proof generated in ${proofTime}ms`);
    
    return {
      proof: this.serializeProof(proof),
      publicSignals,
      proofTime,
    };
  }
  
  private serializeProof(proof: any): string {
    // Groth16 proof consists of 3 elliptic curve points:
    // - π_A (2 field elements)
    // - π_B (4 field elements)
    // - π_C (2 field elements)
    
    return JSON.stringify({
      pi_a: proof.pi_a,
      pi_b: proof.pi_b,
      pi_c: proof.pi_c,
      protocol: 'groth16',
      curve: 'bn128',
    });
  }
}
```

**Witness Example**:
```typescript
const witness = {
  // Private inputs
  request_id: '0x1234...', // BigInt
  response_data: '0x5678...', // BigInt
  oracle_private_key: '0xabcd...', // BigInt
  
  // Public inputs
  response_hash: '0xef01...', // BigInt (Poseidon hash)
  oracle_public_key: '0x2345...', // BigInt
};
```

---

### 4. Proof Verifier (Smart Contract)

**Purpose**: Verify proofs on-chain without revealing private data.

**Solidity Verifier**:
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Groth16Verifier {
    using Pairing for *;
    
    struct VerifyingKey {
        Pairing.G1Point alpha;
        Pairing.G2Point beta;
        Pairing.G2Point gamma;
        Pairing.G2Point delta;
        Pairing.G1Point[] gamma_abc;
    }
    
    struct Proof {
        Pairing.G1Point a;
        Pairing.G2Point b;
        Pairing.G1Point c;
    }
    
    function verifyingKey() pure internal returns (VerifyingKey memory vk) {
        vk.alpha = Pairing.G1Point(/* ... */);
        vk.beta = Pairing.G2Point(/* ... */);
        vk.gamma = Pairing.G2Point(/* ... */);
        vk.delta = Pairing.G2Point(/* ... */);
        vk.gamma_abc = new Pairing.G1Point[](3);
        vk.gamma_abc[0] = Pairing.G1Point(/* ... */);
        vk.gamma_abc[1] = Pairing.G1Point(/* ... */);
        vk.gamma_abc[2] = Pairing.G1Point(/* ... */);
    }
    
    function verify(
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[2] memory input
    ) public view returns (bool) {
        Proof memory proof;
        proof.a = Pairing.G1Point(a[0], a[1]);
        proof.b = Pairing.G2Point([b[0][0], b[0][1]], [b[1][0], b[1][1]]);
        proof.c = Pairing.G1Point(c[0], c[1]);
        
        VerifyingKey memory vk = verifyingKey();
        
        // Validate public inputs
        require(input.length + 1 == vk.gamma_abc.length, "Invalid input length");
        
        // Compute linear combination
        Pairing.G1Point memory vk_x = vk.gamma_abc[0];
        for (uint i = 0; i < input.length; i++) {
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.gamma_abc[i + 1], input[i]));
        }
        
        // Verify pairing equation:
        // e(A, B) = e(alpha, beta) * e(vk_x, gamma) * e(C, delta)
        return Pairing.pairingProd4(
            proof.a, proof.b,
            Pairing.negate(vk.alpha), vk.beta,
            Pairing.negate(vk_x), vk.gamma,
            Pairing.negate(proof.c), vk.delta
        );
    }
}

library Pairing {
    struct G1Point {
        uint X;
        uint Y;
    }
    
    struct G2Point {
        uint[2] X;
        uint[2] Y;
    }
    
    // Pairing check (uses precompiled contract at 0x08)
    function pairingProd4(
        G1Point memory a1, G2Point memory a2,
        G1Point memory b1, G2Point memory b2,
        G1Point memory c1, G2Point memory c2,
        G1Point memory d1, G2Point memory d2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](4);
        G2Point[] memory p2 = new G2Point[](4);
        
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p1[3] = d1;
        
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        p2[3] = d2;
        
        return pairing(p1, p2);
    }
    
    function pairing(G1Point[] memory p1, G2Point[] memory p2) internal view returns (bool) {
        require(p1.length == p2.length, "Length mismatch");
        uint elements = p1.length;
        uint inputSize = elements * 6;
        uint[] memory input = new uint[](inputSize);
        
        for (uint i = 0; i < elements; i++) {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[0];
            input[i * 6 + 3] = p2[i].X[1];
            input[i * 6 + 4] = p2[i].Y[0];
            input[i * 6 + 5] = p2[i].Y[1];
        }
        
        uint[1] memory out;
        bool success;
        
        assembly {
            success := staticcall(sub(gas(), 2000), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
        }
        
        require(success, "Pairing check failed");
        return out[0] != 0;
    }
}
```

**Gas Costs**:
- Proof verification: ~250,000 gas
- Pairing precompile: ~180,000 gas (4 pairings)
- Storage: ~20,000 gas per proof (if storing)

---

## Use Cases

### 1. Private Oracle Responses

**Scenario**: Prove oracle fetched weather data without revealing exact temperature.

**Circuit**:
```circom
template WeatherProof() {
    signal input temperature; // Private: 15°C
    signal input threshold;   // Private: 10°C
    signal output is_cold;    // Public: true/false
    
    is_cold <== temperature < threshold;
}
```

**Application**:
- Smart contract asks: "Is it cold outside?"
- Oracle proves: "Yes, but I won't tell you the exact temperature"
- Use case: Parametric insurance (pays if temperature < threshold)

---

### 2. Identity Attestation

**Scenario**: Prove age >18 without revealing birthdate.

**Circuit**:
```circom
template AgeProof() {
    signal input birthdate;   // Private: 2000-01-01
    signal input current_date; // Private: 2025-12-08
    signal input min_age;     // Public: 18
    
    signal age;
    age <== (current_date - birthdate) / 365;
    
    signal output is_adult;
    is_adult <== age >= min_age;
}
```

**Application**:
- Prove eligibility without revealing identity
- Access age-restricted content
- Decentralized KYC

---

### 3. Confidential Transactions

**Scenario**: Transfer tokens without revealing amount.

**Circuit**:
```circom
template ConfidentialTransfer() {
    signal input sender_balance;    // Private: 100 tokens
    signal input recipient_balance; // Private: 50 tokens
    signal input transfer_amount;   // Private: 20 tokens
    
    signal new_sender_balance;
    signal new_recipient_balance;
    
    new_sender_balance <== sender_balance - transfer_amount;
    new_recipient_balance <== recipient_balance + transfer_amount;
    
    // Constrain: no negative balances
    component range_check = RangeCheck(64);
    range_check.in <== new_sender_balance;
    
    signal output commitment_sender;
    signal output commitment_recipient;
    
    // Public commitments (hide actual balances)
    commitment_sender <== Poseidon([new_sender_balance, salt1]);
    commitment_recipient <== Poseidon([new_recipient_balance, salt2]);
}
```

**Application**:
- Private payments on public blockchain
- Confidential DeFi
- Enterprise blockchain privacy

---

## Performance Characteristics

```
┌─ ZK Proof Performance ──────────────────┐
│ Circuit Compilation: 1-10s              │
│ Trusted Setup (Phase 1): Hours (one-time)│
│ Trusted Setup (Phase 2): Minutes        │
│ Witness Generation: 10-50ms             │
│ Proof Generation: 300-800ms             │
│ Proof Verification: 5-15ms (off-chain)  │
│ Proof Verification: 250K gas (on-chain) │
│                                          │
│ Proof Size: 256 bytes (Groth16)         │
│ Public Inputs: 32 bytes each             │
│                                          │
│ Circuit Constraints:                     │
│   Small (100 constraints): 100ms prove  │
│   Medium (10K constraints): 300ms prove │
│   Large (1M constraints): 10s prove     │
└──────────────────────────────────────────┘
```

---

## Security Considerations

**Trusted Setup**:
- Multi-party ceremony (1-of-N honest assumption)
- Publicly verifiable
- Use Powers of Tau from large ceremonies (Perpetual Powers of Tau)

**Circuit Bugs**:
- Under-constrained circuits allow invalid proofs
- Use circuit auditing tools (circom-witness-checker)
- Formal verification

**Side Channels**:
- Timing attacks on prover
- Constant-time implementations
- Blinding factors

---

## Future Development

### Phase 1: Recursive Proofs (Q1 2026)
- Proof aggregation (N proofs → 1)
- Reduces on-chain verification cost
- Halo2/PLONK integration

### Phase 2: Universal SNARKs (Q2 2026)
- No trusted setup (transparent zkSNARKs)
- STARKs for post-quantum security
- Nova for folding schemes

### Phase 3: Hardware Acceleration (Q3 2026)
- GPU proving (10x speedup)
- ASIC provers (100x speedup)
- <100ms proof generation

### Phase 4: ZK Machine Learning (Q4 2026)
- Prove ML inference correctness
- Private model inference
- Verifiable AI outputs

---

## Code References

- `karana-core/src/zk/circuits/`: Circom circuits
- `karana-core/src/zk/prover.rs`: Proof generator
- `karana-contracts/Verifier.sol`: On-chain verifier

---

## Summary

The ZK Proof System provides:
- **Privacy**: Prove statements without revealing data
- **Groth16**: Efficient SNARKs (256-byte proofs)
- **Trusted Setup**: Multi-party ceremony for security
- **On-Chain Verification**: 250K gas per proof
- **Use Cases**: Private oracles, identity, confidential transactions

This enables privacy-preserving verifiable computation on Kāraṇa OS.