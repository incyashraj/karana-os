# Layer 3: Blockchain Layer

## Overview

The Blockchain Layer provides the decentralized ledger, wallet, governance, and data availability systems for Kāraṇa OS. It implements a custom proof-of-stake blockchain optimized for micropayments, oracle settlements, and DAO governance, with Celestia DA integration for scalability.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      LAYER 3: BLOCKCHAIN                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │               BlockchainManager (Consensus Coordinator)         │    │
│  │  - Current Height: 42,891 blocks                                │    │
│  │  - Block Time: 12 seconds                                       │    │
│  │  - Validator Set: 21 active validators                          │    │
│  └────┬───────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│  ┌────▼──────────┬─────────────┬─────────────┬──────────────┬─────────┐
│  │ Chain         │ Wallet      │ Governance  │ Celestia DA  │ Ledger  │
│  │ (Blocks/TXs)  │ (Accounts)  │ (DAO)       │ Integration  │ (State) │
│  └───────────────┴─────────────┴─────────────┴──────────────┴─────────┘
│       │                │              │              │            │     │
│  ┌────▼────────────────▼──────────────▼──────────────▼────────────▼──┐
│  │                    Transaction Pool (Mempool)                      │
│  │  Pending: 234 txs | Gas Price: 0.001 KARA | Throughput: 100 TPS  │
│  └────────────────────────────────────────────────────────────────────┘
└───────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Chain (Blockchain Core)

**Block Structure**:
```rust
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub state_root: Hash,           // Merkle root of state
    pub receipts_root: Hash,        // Merkle root of receipts
    pub validator_signature: Signature,
}

pub struct BlockHeader {
    pub height: u64,
    pub timestamp: i64,
    pub parent_hash: Hash,
    pub transactions_root: Hash,
    pub validator: PublicKey,
    pub gas_used: u64,
    pub gas_limit: u64,
}
```

**Transaction Types**:
```rust
pub enum Transaction {
    Transfer {
        from: Address,
        to: Address,
        amount: u128,         // in KARA (18 decimals)
        nonce: u64,
        signature: Signature,
    },
    Stake {
        validator: Address,
        amount: u128,
    },
    Unstake {
        validator: Address,
        amount: u128,
    },
    GovernanceVote {
        proposal_id: u64,
        vote: Vote,          // Yes/No/Abstain
        weight: u128,        // Voting power
    },
    OracleResponse {
        request_id: Hash,
        response: Vec<u8>,
        proof: ZKProof,
    },
    ContractCall {
        contract: Address,
        method: String,
        args: Vec<u8>,
        gas_limit: u64,
    },
}
```

**Consensus**: Proof-of-Stake (Tendermint-inspired)
- **Block Time**: 12 seconds
- **Finality**: 1 block (~12 seconds)
- **Validators**: 21 active (top stakers)
- **Rewards**: 2 KARA per block
- **Slashing**: 5% for downtime, 20% for double-sign

**State Transition**:
```rust
impl Blockchain {
    pub fn apply_block(&mut self, block: Block) -> Result<StateRoot> {
        // 1. Validate block
        self.validate_block(&block)?;
        
        // 2. Execute transactions
        let mut state_changes = Vec::new();
        for tx in &block.transactions {
            let receipt = self.execute_transaction(tx)?;
            state_changes.push(receipt.state_delta);
        }
        
        // 3. Update state tree
        for change in state_changes {
            self.state.apply_delta(change)?;
        }
        
        // 4. Calculate new state root
        let new_root = self.state.merkle_root();
        
        // 5. Persist block
        self.db.put_block(&block)?;
        self.height += 1;
        
        Ok(new_root)
    }
}
```

**Integration Points**:
- **→ Layer 2 (Network)**: Block propagation
- **→ Layer 4 (Oracle)**: Oracle response settlement
- **→ Layer 8 (Apps)**: Wallet balance queries

---

### 2. Wallet (Account Management)

**Account Structure**:
```rust
pub struct Account {
    pub address: Address,        // Derived from public key
    pub balance: u128,           // KARA tokens
    pub nonce: u64,              // Transaction counter
    pub code_hash: Option<Hash>, // Smart contract code (if any)
    pub storage_root: Hash,      // Contract storage
}
```

**Wallet Operations**:
```rust
pub struct Wallet {
    keypair: Ed25519KeyPair,
    address: Address,
    blockchain: Arc<RwLock<Blockchain>>,
}

impl Wallet {
    pub async fn transfer(&self, to: Address, amount: u128) -> Result<TxHash> {
        // 1. Create transaction
        let tx = Transaction::Transfer {
            from: self.address,
            to,
            amount,
            nonce: self.get_nonce().await?,
            signature: Signature::default(), // Placeholder
        };
        
        // 2. Sign transaction
        let signed_tx = self.sign_transaction(tx)?;
        
        // 3. Submit to mempool
        let tx_hash = self.blockchain.write().await
            .submit_transaction(signed_tx)?;
        
        Ok(tx_hash)
    }
    
    pub async fn get_balance(&self) -> Result<u128> {
        self.blockchain.read().await
            .get_account_balance(&self.address)
    }
    
    fn sign_transaction(&self, mut tx: Transaction) -> Result<Transaction> {
        let msg = tx.signing_bytes();
        let signature = self.keypair.sign(&msg);
        
        match &mut tx {
            Transaction::Transfer { signature: sig, .. } => *sig = signature,
            _ => {}
        }
        
        Ok(tx)
    }
}
```

**Hierarchical Deterministic Wallets** (BIP-44):
```
Mnemonic (12 words)
  ↓ PBKDF2
Seed (512 bits)
  ↓ HMAC-SHA512
Master Key
  ↓ Derivation Path: m/44'/710'/0'/0/0
Account Keys (unlimited)
```

**Integration Points**:
- **← Layer 4 (Oracle)**: Payment for AI queries
- **← Layer 8 (Apps)**: Transaction signing
- **→ Layer 7 (Interface)**: Balance display

---

### 3. Governance (DAO)

**Proposal System**:
```rust
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub votes_yes: u128,
    pub votes_no: u128,
    pub votes_abstain: u128,
    pub status: ProposalStatus,
    pub created_at: i64,
    pub voting_ends_at: i64,
}

pub enum ProposalType {
    ParameterChange { key: String, value: String },
    TreasurySpend { recipient: Address, amount: u128 },
    ValidatorUpdate { add: Vec<Address>, remove: Vec<Address> },
    OracleModelUpdate { model_hash: Hash, weight_url: String },
    FeatureToggle { feature: String, enabled: bool },
}

pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}
```

**Voting Mechanism**:
```rust
impl Governance {
    pub fn vote(&mut self, proposal_id: u64, voter: Address, vote: Vote) -> Result<()> {
        let proposal = self.proposals.get_mut(&proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found"))?;
        
        // Check voting period
        if Instant::now() > proposal.voting_ends_at {
            return Err(anyhow!("Voting period ended"));
        }
        
        // Get voting power (1 KARA staked = 1 vote)
        let voting_power = self.blockchain.get_staked_amount(&voter)?;
        
        // Record vote
        match vote {
            Vote::Yes => proposal.votes_yes += voting_power,
            Vote::No => proposal.votes_no += voting_power,
            Vote::Abstain => proposal.votes_abstain += voting_power,
        }
        
        self.voters.insert((proposal_id, voter), vote);
        
        Ok(())
    }
    
    pub fn tally_proposal(&mut self, proposal_id: u64) -> Result<ProposalStatus> {
        let proposal = self.proposals.get_mut(&proposal_id)?;
        
        let total_votes = proposal.votes_yes + proposal.votes_no + proposal.votes_abstain;
        let quorum = self.config.quorum_threshold; // 40% of total stake
        
        if total_votes < quorum {
            proposal.status = ProposalStatus::Rejected;
            return Ok(ProposalStatus::Rejected);
        }
        
        let approval_rate = proposal.votes_yes as f64 / (proposal.votes_yes + proposal.votes_no) as f64;
        
        if approval_rate > 0.66 {
            proposal.status = ProposalStatus::Passed;
            self.execute_proposal(proposal)?;
            Ok(ProposalStatus::Passed)
        } else {
            proposal.status = ProposalStatus::Rejected;
            Ok(ProposalStatus::Rejected)
        }
    }
}
```

**Governance Parameters**:
- **Quorum**: 40% of staked tokens must vote
- **Approval**: 66% majority required
- **Voting Period**: 7 days
- **Execution Delay**: 24 hours (timelock)

**Integration Points**:
- **← Layer 4 (Oracle)**: Model update proposals
- **← Layer 8 (Apps)**: Feature flag votes
- **→ Layer 9 (System)**: Parameter changes

---

### 4. Celestia DA Integration

**Purpose**: Offload transaction data to Celestia for scalability while keeping state on Kāraṇa chain.

**Data Submission**:
```rust
pub struct CelestiaDA {
    namespace: Namespace,
    client: CelestiaClient,
}

impl CelestiaDA {
    pub async fn submit_block_data(&self, block: &Block) -> Result<DACommitment> {
        // 1. Serialize transactions
        let data = bincode::serialize(&block.transactions)?;
        
        // 2. Submit to Celestia
        let blob = Blob {
            namespace: self.namespace,
            data,
            share_version: 0,
        };
        
        let commitment = self.client.submit_blob(blob).await?;
        
        // 3. Store commitment on-chain
        Ok(DACommitment {
            height: block.header.height,
            data_root: commitment.data_root,
            proof: commitment.proof,
        })
    }
    
    pub async fn retrieve_block_data(&self, height: u64) -> Result<Vec<Transaction>> {
        // 1. Get commitment from chain
        let commitment = self.get_commitment(height)?;
        
        // 2. Download from Celestia
        let blob = self.client.get_blob(
            commitment.height,
            self.namespace,
            commitment.data_root
        ).await?;
        
        // 3. Deserialize
        let transactions = bincode::deserialize(&blob.data)?;
        
        Ok(transactions)
    }
}
```

**Benefits**:
- **Scalability**: 1000+ TPS (vs 100 TPS without DA)
- **Cost**: 95% cheaper storage
- **Censorship Resistance**: Data always retrievable from Celestia
- **Light Clients**: Verify state without downloading full blocks

**Integration Points**:
- **↔ Chain**: Store block commitments
- **→ Layer 2 (Network)**: Sync via Celestia instead of P2P
- **← Layer 9 (System)**: Archive old data

---

### 5. Ledger (State Management)

**State Structure**:
```rust
pub struct StateLedger {
    accounts: MerkleTree<Address, Account>,
    contracts: MerkleTree<Address, Contract>,
    storage: HashMap<Address, MerkleTree<Hash, Vec<u8>>>,
    db: RocksDB,
}
```

**Merkle Tree for Efficient Proofs**:
```
                  State Root
                  /         \
          Account Tree    Contract Tree
           /      \          /       \
      Alice     Bob      Oracle    Wallet
      (50 KARA) (100)   (code)    (code)
```

**State Queries**:
```rust
impl StateLedger {
    pub fn get_account(&self, address: &Address) -> Result<Option<Account>> {
        self.accounts.get(address)
    }
    
    pub fn get_proof(&self, address: &Address) -> Result<MerkleProof> {
        self.accounts.get_proof(address)
    }
    
    pub fn verify_proof(&self, proof: &MerkleProof, root: &Hash) -> bool {
        proof.verify(root)
    }
}
```

**Integration Points**:
- **↔ Chain**: Read/write state during block execution
- **→ Layer 4 (Oracle)**: Verify account states with ZK proofs
- **← Wallet**: Query balances

---

## Transaction Flow

```
User: "Send 50 KARA to Alice"
         │
         ▼
┌─────────────────────────────────┐
│   Layer 6 (AI Engine)            │
│   Intent: Transfer               │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Layer 4 (Oracle Bridge)        │
│   Create TX, request signature   │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Layer 3: Wallet.sign_tx()     │
│   Ed25519 signature              │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Mempool: Add pending TX        │
└────────────┬────────────────────┘
             │
             ▼ (next block)
┌─────────────────────────────────┐
│   Validator: Include TX in block │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Chain: Execute & commit        │
│   Alice balance: 50 + 50 = 100   │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│   Layer 2: Broadcast block       │
│   Propagate to all peers         │
└─────────────────────────────────┘
```

**Timing**:
- TX submission to mempool: <100ms
- Mempool to block inclusion: 0-12s (next block)
- Block execution: <1s
- Network propagation: 50-200ms
- **Total**: 1-13 seconds (12s average)

---

## Economics

### Token (KARA)
- **Total Supply**: 1 billion KARA
- **Decimals**: 18
- **Distribution**:
  - 40% Community (DAO-controlled)
  - 25% Staking Rewards
  - 20% Team (4-year vest)
  - 10% Investors (2-year vest)
  - 5% Oracle AI Fund

### Fee Structure
- **Transfer**: 0.001 KARA
- **Stake/Unstake**: 0.01 KARA
- **Oracle Query**: 0.1-1.0 KARA (depends on complexity)
- **Contract Call**: 0.001 KARA per 1000 gas

### Staking Rewards
- **Base APY**: 8%
- **Validator Commission**: 10%
- **Slashing Risk**: 5% (downtime), 20% (malicious)

---

## Security

### Cryptographic Primitives
- **Signing**: Ed25519 (64-byte signatures)
- **Hashing**: SHA-256
- **Merkle Trees**: SHA-256 + binary tree
- **Key Derivation**: BIP-39 (mnemonic) + BIP-44 (HD wallets)

### Attack Mitigations
1. **51% Attack**: Requires >50% stake ($500M+ at current prices)
2. **Long-Range Attack**: Checkpointing every 10,000 blocks
3. **DDoS**: Rate limiting (100 TX/sec per address)
4. **Replay Attack**: Nonces + chain ID
5. **Front-Running**: Priority gas auction

---

## Performance Metrics

- **TPS**: 100 (without DA), 1000+ (with Celestia)
- **Block Time**: 12 seconds
- **Finality**: 1 block (~12 seconds)
- **State Size**: ~10 GB (at 1M accounts)
- **Sync Time**: 1 hour (full sync), 5 min (fast sync)

---

## Future Development

### Phase 1: EVM Compatibility (Q1 2026)
- Run Solidity smart contracts
- Bridge to Ethereum
- Support existing DeFi apps

### Phase 2: zkRollup (Q2 2026)
- 10,000+ TPS
- Sub-second finality
- Privacy-preserving transactions

### Phase 3: Cross-Chain Bridges (Q3 2026)
- IBC (Cosmos)
- XCMP (Polkadot)
- Wormhole (multichain)

### Phase 4: Governance v2 (Q4 2026)
- Quadratic voting
- Delegation
- Futarchy (prediction markets)

---

## Code References

- `karana-core/src/blockchain/chain.rs`: Blockchain core
- `karana-core/src/blockchain/wallet.rs`: Wallet management
- `karana-core/src/blockchain/governance.rs`: DAO
- `karana-core/src/blockchain/celestia.rs`: DA integration

---

## Summary

Layer 3 provides:
- **Decentralized Ledger**: Immutable transaction history
- **Wallet**: Self-custody with HD key derivation
- **Governance**: DAO for protocol upgrades
- **Data Availability**: Celestia integration for scale
- **State Management**: Merkle trees for efficient proofs

This layer enables economic incentives, governance, and provable state for the entire Kāraṇa OS ecosystem.
