use anyhow::Result;
use std::sync::{Arc, Mutex};
use crate::ai::KaranaAI;
use sha2::{Sha256, Digest};
use rocksdb::{DB, Options, ColumnFamilyDescriptor};
use lru::LruCache;
use std::num::NonZeroUsize;
use crate::chain::Block as ChainBlock;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

/// Real Output Directory for Intent Actions
const REAL_OUTPUT_DIR: &str = "/tmp/karana";

pub struct MerkleTree {
    pub root: Vec<u8>,
    pub leaves: Vec<Vec<u8>>,
}

impl MerkleTree {
    pub fn new(data: &[u8]) -> Self {
        // Atom 1: Chunk data into 256-byte segments (simulating 256KB)
        let chunk_size = 256; 
        let chunks: Vec<Vec<u8>> = data.chunks(chunk_size).map(|c| c.to_vec()).collect();
        
        // Hash leaves
        let mut current_level: Vec<Vec<u8>> = chunks.iter()
            .map(|c| Sha256::digest(c).to_vec())
            .collect();
            
        // Build tree up (Merkleization)
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i+1]
                } else {
                    left // Duplicate last if odd
                };
                
                let mut hasher = Sha256::new();
                hasher.update(left);
                hasher.update(right);
                next_level.push(hasher.finalize().to_vec());
            }
            current_level = next_level;
        }
        
        let root = current_level.first().cloned().unwrap_or_default();
        
        Self {
            root,
            leaves: chunks,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageBlob {
    pub merkle_root: Vec<u8>,
    pub data_chunks: usize,
    pub raw_data: Vec<u8>,
    pub zk_proof: Vec<u8>,
}

pub struct KaranaStorage {
    db: DB,
    // Atom 5: Hot Tier (LRU Cache)
    cache: Arc<Mutex<LruCache<Vec<u8>, Vec<u8>>>>,
    // Simple in-memory vector store: (ID, Vector)
    vector_index: Arc<Mutex<Vec<(u64, Vec<f32>)>>>,
    next_id: Arc<Mutex<u64>>,
    #[allow(dead_code)]
    chain_rpc: String,
    ai: Arc<Mutex<KaranaAI>>,
}

impl KaranaStorage {
    pub fn new(local_cache: &str, chain_rpc: &str, ai: Arc<Mutex<KaranaAI>>) -> Result<Self> {
        // Atom 5: Scalability - Sharding via Column Families
        let shards = vec!["shard_0", "shard_1", "shard_2", "shard_3", "vector_index", "blocks"];
        
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let cfs = shards.iter().map(|name| ColumnFamilyDescriptor::new(*name, Options::default()));
        
        // Open DB with Column Families (Shards)
        let db = DB::open_cf_descriptors(&opts, local_cache, cfs).map_err(|e| anyhow::anyhow!("RocksDB open failed: {}", e))?;

        // Atom 5: Initialize Hot Tier Cache (Capacity: 1000 chunks)
        let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

        // Atom 3: Initialize Vector DB (Load from RocksDB)
        let mut index = Vec::new();
        let mut max_id = 0;
        
        if let Some(cf) = db.cf_handle("vector_index") {
            log::info!("Atom 5: Loading Vector Index from Disk...");
            let iter = db.iterator_cf(cf, rocksdb::IteratorMode::Start);
            for item in iter {
                if let Ok((key, value)) = item {
                    // Key is u64 (8 bytes big endian)
                    if key.len() == 8 {
                        let id = u64::from_be_bytes(key[..8].try_into().unwrap());
                        if id > max_id { max_id = id; }
                        
                        // Value is Vec<f32> serialized as JSON
                        if let Ok(vec) = serde_json::from_slice::<Vec<f32>>(&value) {
                            index.push((id, vec));
                        }
                    }
                }
            }
            log::info!("Atom 5: Loaded {} vectors from disk.", index.len());
        }

        Ok(Self {
            db,
            cache,
            vector_index: Arc::new(Mutex::new(index)),
            next_id: Arc::new(Mutex::new(max_id + 1)), // Start ID at max + 1
            chain_rpc: chain_rpc.to_string(),
            ai,
        })
    }

    pub fn write(&self, data: &[u8], context: &str) -> Result<StorageBlob> {
        let data_str = String::from_utf8_lossy(data).to_string();

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 7.1: REAL FILE OUTPUT
        // Write to /tmp/karana/<context>.conf for observable results
        // ═══════════════════════════════════════════════════════════════════
        fs::create_dir_all(REAL_OUTPUT_DIR)?;
        let safe_context = context.replace(" ", "_").replace("/", "_");
        let output_path = Path::new(REAL_OUTPUT_DIR).join(format!("{}.conf", safe_context));
        fs::write(&output_path, data)?;
        log::info!("[STORAGE] ✓ Written: {} ({} bytes)", output_path.display(), data.len());

        // Atom 3: Use AI to summarize AND Embed
        let prompt = format!("Summarize this data for storage indexing: '{}'", data_str);
        let summary = self.ai.lock().unwrap().predict(&prompt, 20)?;
        log::info!("AI Storage Index: {}", summary);

        // Atom 3: AI Oracle for Storage Tuning (Phase 3)
        let tune_prompt = format!("Tune/compress this data summary: '{}'", summary);
        let tuning_advice = self.ai.lock().unwrap().predict(&tune_prompt, 20)?;
        log::info!("Atom 3 (AI Oracle): Storage Tuning Advice: {}", tuning_advice);
        
        // Atom 3: Generate Embedding
        let embedding = self.ai.lock().unwrap().embed(&data_str).unwrap_or_default();
        
        if !embedding.is_empty() {
            let mut index = self.vector_index.lock().unwrap();
            let mut id_guard = self.next_id.lock().unwrap();
            let id = *id_guard;
            *id_guard += 1;
            
            log::info!("Atom 3: Adding to Vector DB. ID: {}, Vector Size: {}", id, embedding.len());
            
            // Persist to RocksDB
            if let Some(cf) = self.db.cf_handle("vector_index") {
                let key = id.to_be_bytes();
                let value = serde_json::to_vec(&embedding).unwrap();
                self.db.put_cf(&cf, key, value).map_err(|e| anyhow::anyhow!("DB write vector failed: {}", e))?;
            }

            index.push((id, embedding.clone()));
            log::info!("Atom 3 (Semantic Mind): Indexed document ID {} with vector dim {}.", id, embedding.len());
        }

        // Atom 1: Create Merkle Tree
        let tree = MerkleTree::new(data);
        let root_hex = hex::encode(&tree.root);
        
        log::info!("Atom 1 (Immutable Truth): Merkle Root generated: {}", root_hex);
        log::info!("Atom 1: Data chunked into {} leaves.", tree.leaves.len());

        // Atom 7: ZK-Attested Storage
        // Real ZK: Prove data hashes to commitment (Phase 2)
        let data_to_prove = tree.leaves.first().map(|c| c.as_slice()).unwrap_or(b"");
        let commitment = crate::zk::compute_hash(data_to_prove);
        
        log::info!("Atom 7 (ZK): Generating Attestation for Storage Write...");
        let zk_proof = crate::zk::prove_data_hash(data_to_prove, commitment)
            .map_err(|e| anyhow::anyhow!("ZK Proof failed: {}", e))?;
        log::info!("Atom 7 (ZK): Storage Attested. Proof Size: {} bytes", zk_proof.len());
        
        // Atom 5: Persist to RocksDB (Scalability)
        // Store Root -> Metadata (e.g., "ROOT")
        self.db.put(&tree.root, b"ROOT").map_err(|e| anyhow::anyhow!("DB write failed: {}", e))?;
        
        // Store Hash -> Chunk (Sharded)
        for chunk in &tree.leaves {
             let chunk_hash = Sha256::digest(chunk).to_vec();
             
             // 1. Hot Tier: Write to LRU
             self.cache.lock().unwrap().put(chunk_hash.clone(), chunk.clone());

             // 2. Cold Tier: Determine Shard
             // Simple sharding: First byte % 4
             let shard_id = (chunk_hash[0] % 4) as usize;
             let cf_name = format!("shard_{}", shard_id);
             
             if let Some(cf) = self.db.cf_handle(&cf_name) {
                 self.db.put_cf(&cf, &chunk_hash, chunk).map_err(|e| anyhow::anyhow!("DB write chunk failed: {}", e))?;
             } else {
                 // Fallback to default if shard missing (should not happen)
                 self.db.put(&chunk_hash, chunk).map_err(|e| anyhow::anyhow!("DB write chunk failed: {}", e))?;
             }
        }
        log::info!("Atom 5 (Scalability): Persisted {} chunks across 4 shards (Hot/Cold Tiering Active).", tree.leaves.len());
        
        Ok(StorageBlob {
            merkle_root: tree.root,
            data_chunks: tree.leaves.len(),
            raw_data: data.to_vec(),
            zk_proof,
        })
    }

    /// Atom 5: Retrieve chunk with Tiering Logic
    pub fn read_chunk(&self, hash: &[u8]) -> Result<Option<Vec<u8>>> {
        // 1. Check Hot Tier (LRU)
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(data) = cache.get(hash) {
                // log::info!("Atom 5: Cache Hit for chunk {:?}", hex::encode(&hash[0..4]));
                return Ok(Some(data.clone()));
            }
        }

        // 2. Check Cold Tier (Sharded RocksDB)
        let shard_id = (hash[0] % 4) as usize;
        let cf_name = format!("shard_{}", shard_id);
        
        let data = if let Some(cf) = self.db.cf_handle(&cf_name) {
            self.db.get_cf(&cf, hash)?
        } else {
            self.db.get(hash)?
        };

        // 3. Promote to Hot Tier if found
        if let Some(ref d) = data {
            self.cache.lock().unwrap().put(hash.to_vec(), d.clone());
        }

        Ok(data)
    }

    pub fn persist_block(&self, block: &ChainBlock) -> Result<()> {
        if let Some(cf) = self.db.cf_handle("blocks") {
            let key = block.hash.as_bytes();
            let value = serde_json::to_vec(block)?;
            self.db.put_cf(&cf, key, value).map_err(|e| anyhow::anyhow!("DB write block failed: {}", e))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Blocks column family not found"))
        }
    }

    pub fn get_block(&self, hash: &str) -> Result<Option<ChainBlock>> {
        if let Some(cf) = self.db.cf_handle("blocks") {
            let key = hash.as_bytes();
            if let Some(value) = self.db.get_cf(&cf, key)? {
                let block: ChainBlock = serde_json::from_slice(&value)?;
                Ok(Some(block))
            } else {
                Ok(None)
            }
        } else {
            Err(anyhow::anyhow!("Blocks column family not found"))
        }
    }

    pub fn search(&self, query: &str) -> Result<Vec<String>> {
        log::info!("Atom 3: Semantic Search for '{}'...", query);
        let embedding = self.ai.lock().unwrap().embed(query)?;
        if embedding.is_empty() {
            return Ok(vec![]);
        }

        let index = self.vector_index.lock().unwrap();
        
        // Linear Scan for Cosine Similarity
        let mut scores: Vec<(u64, f32)> = index.iter()
            .map(|(id, vec)| (*id, cosine_similarity(&embedding, vec)))
            .collect();
            
        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top 5
        let top_k = scores.into_iter().take(5).collect::<Vec<_>>();
        
        let ids: Vec<String> = top_k.iter().map(|(id, score)| format!("DocID: {} (Score: {:.4})", id, score)).collect();
        log::info!("Atom 3: Found {} semantic matches.", ids.len());
        
        Ok(ids)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
