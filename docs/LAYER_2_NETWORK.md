# Layer 2: P2P Network Layer

## Overview

The P2P Network Layer enables decentralized communication between Kāraṇa OS devices without relying on centralized servers. Using libp2p, it provides peer discovery, encrypted messaging, blockchain synchronization, and distributed file sharing with automatic NAT traversal and connection resilience.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      LAYER 2: P2P NETWORK                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────────────────────────────────────────────────────┐    │
│  │               NetworkManager (libp2p node)                      │    │
│  │  - PeerId: 12D3KooW... (Ed25519 keypair)                       │    │
│  │  - Multiaddr: /ip4/192.168.1.42/tcp/4001                       │    │
│  │  - Transport: TCP + QUIC + WebRTC                              │    │
│  └────┬───────────────────────────────────────────────────────────┘    │
│       │                                                                  │
│  ┌────▼──────────┬─────────────┬─────────────┬──────────────┬─────────┐
│  │ Peer          │ Connection  │ Message     │ Block Sync   │ DHT     │
│  │ Discovery     │ Manager     │ Routing     │ Protocol     │ Service │
│  └───────────────┴─────────────┴─────────────┴──────────────┴─────────┘
│       │                │              │              │            │     │
│  ┌────▼────────────────▼──────────────▼──────────────▼────────────▼──┐
│  │                    GossipSub Message Layer                         │
│  │  Topics: /karana/blocks/v1, /karana/transactions/v1               │
│  └────────────────────────────────────────────────────────────────────┘
│       │                                                                  │
│  ┌────▼──────────────────────────────────────────────────────────────┐
│  │                 Transport Security (Noise Protocol)                │
│  │  Encryption: ChaCha20-Poly1305 | Auth: Ed25519 signatures         │
│  └────────────────────────────────────────────────────────────────────┘
└───────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Peer Discovery

**Purpose**: Find other Kāraṇa OS devices on local network and internet.

**Discovery Methods**:
1. **mDNS (Local)**: Broadcast on LAN for instant peer detection
2. **DHT (Internet)**: Kademlia distributed hash table for global peers
3. **Bootstrap Nodes**: Hardcoded initial peers for network join
4. **PubSub Peering**: Discover via GossipSub message propagation

**Implementation**:
```rust
// karana-core/src/network/discovery.rs
pub struct PeerDiscovery {
    mdns: Mdns,
    dht: Kademlia,
    bootstrap_peers: Vec<Multiaddr>,
    discovered_peers: HashMap<PeerId, PeerInfo>,
}

impl PeerDiscovery {
    pub async fn discover(&mut self) -> Result<Vec<PeerInfo>> {
        // 1. mDNS broadcast
        self.mdns.broadcast("_karana._tcp.local").await?;
        
        // 2. Listen for responses
        let local_peers = self.mdns.receive_peers(Duration::from_secs(2)).await?;
        
        // 3. Query DHT for global peers
        let global_peers = self.dht.get_closest_peers(self.local_peer_id).await?;
        
        // 4. Validate and store
        for peer in local_peers.chain(global_peers) {
            if self.validate_peer(&peer).await? {
                self.discovered_peers.insert(peer.id, peer);
            }
        }
        
        Ok(self.discovered_peers.values().cloned().collect())
    }
}
```

**Peer Information Structure**:
```rust
pub struct PeerInfo {
    pub id: PeerId,                     // Ed25519 public key hash
    pub addrs: Vec<Multiaddr>,          // Network addresses
    pub protocols: Vec<String>,         // Supported protocols
    pub agent_version: String,          // "Karana/0.1.0"
    pub last_seen: Instant,
    pub reputation: i32,                // -100 to +100
}
```

**Discovery Timeline**:
```
t=0ms    : mDNS broadcast "_karana._tcp.local"
t=100ms  : Receive mDNS responses (2-5 local peers)
t=500ms  : DHT query for /karana/peers
t=1000ms : DHT returns 20 closest peers
t=1500ms : Validate peer identities (signature check)
t=2000ms : Add validated peers to routing table
```

**Integration Points**:
- **→ Connection Manager**: Dial discovered peers
- **→ DHT Service**: Bootstrap Kademlia routing table
- **→ Layer 3 (Blockchain)**: Find block producers

---

### 2. Connection Manager

**Purpose**: Maintain stable connections to peers with automatic reconnection.

**Connection States**:
```
NotConnected ──dial──► Dialing ──success──► Connected
     ▲                    │                     │
     │                    │                     │
     └────────────────────┴──failed─────────────┘
                          │
                      Disconnected
```

**Implementation**:
```rust
pub struct ConnectionManager {
    swarm: Swarm<KaranaBehaviour>,
    connections: HashMap<PeerId, Connection>,
    target_peer_count: usize,  // 8-12 peers
    max_peer_count: usize,     // 50 peers
}

impl ConnectionManager {
    pub async fn maintain(&mut self) {
        // 1. Remove stale connections
        self.remove_stale_peers().await;
        
        // 2. Check if we need more peers
        if self.connections.len() < self.target_peer_count {
            let needed = self.target_peer_count - self.connections.len();
            let candidates = self.get_connection_candidates(needed);
            
            for peer in candidates {
                self.dial_peer(peer).await?;
            }
        }
        
        // 3. Ping existing peers
        for (peer_id, conn) in &self.connections {
            if conn.last_ping.elapsed() > Duration::from_secs(30) {
                self.ping(peer_id).await?;
            }
        }
        
        // 4. Update routing table
        self.update_routing_table().await?;
    }
    
    async fn dial_peer(&mut self, peer: PeerInfo) -> Result<()> {
        // Try each address in order
        for addr in peer.addrs {
            match self.swarm.dial(addr.clone()) {
                Ok(_) => {
                    self.connections.insert(peer.id, Connection {
                        state: ConnectionState::Dialing,
                        addr,
                        started: Instant::now(),
                    });
                    return Ok(());
                }
                Err(e) => continue,
            }
        }
        Err(anyhow!("Failed to dial peer"))
    }
}
```

**Connection Prioritization**:
1. **Low Latency**: <50ms RTT preferred
2. **High Bandwidth**: >1 Mbps upload
3. **Good Reputation**: Score > 50
4. **Protocol Support**: Supports required features
5. **Geographic Diversity**: Mix of local/remote

**NAT Traversal**:
- **STUN**: Discover external IP/port
- **Hole Punching**: Direct connections through NAT
- **Relay**: Fall back to relay servers if direct fails
- **UPnP**: Automatic port forwarding if router supports

**Integration Points**:
- **→ GossipSub**: Establish message channels
- **→ Block Sync**: Request missing blocks
- **→ Layer 3**: Transaction propagation

---

### 3. GossipSub (Message Routing)

**Purpose**: Efficiently broadcast messages to all interested peers using epidemic protocols.

**Topics**:
```rust
pub enum KaranaTopic {
    Blocks = "/karana/blocks/v1",
    Transactions = "/karana/txs/v1",
    Governance = "/karana/governance/v1",
    Oracle = "/karana/oracle/v1",
    Discovery = "/karana/discovery/v1",
}
```

**Message Flow**:
```
Sender ──► GossipSub ──┬──► Peer 1 (subscribed)
                       ├──► Peer 2 (subscribed)
                       ├──► Peer 3 (not subscribed) ✗
                       ├──► Peer 4 (subscribed)
                       └──► Peer 5 (subscribed)
                              │
                        (each re-gossips to their peers)
```

**Implementation**:
```rust
pub struct GossipSubRouter {
    topics: HashMap<String, HashSet<PeerId>>,
    seen_messages: LruCache<MessageId, ()>,  // Duplicate detection
    fanout: usize,  // 6 peers per gossip
    heartbeat_interval: Duration,  // 1 second
}

impl GossipSubRouter {
    pub async fn publish(&mut self, topic: &str, data: Vec<u8>) -> Result<()> {
        // 1. Create message
        let msg = GossipsubMessage {
            source: self.local_peer_id,
            data,
            sequence_number: self.next_sequence(),
            topic: topic.into(),
            signature: self.sign_message(&data),
        };
        
        // 2. Check not already seen
        let msg_id = self.compute_message_id(&msg);
        if self.seen_messages.contains(&msg_id) {
            return Ok(()); // Duplicate
        }
        self.seen_messages.put(msg_id, ());
        
        // 3. Select peers (fanout)
        let subscribers = self.topics.get(topic).ok_or_else(|| anyhow!("No subscribers"))?;
        let targets = self.select_gossip_peers(subscribers, self.fanout);
        
        // 4. Send to selected peers
        for peer in targets {
            self.send_message(peer, msg.clone()).await?;
        }
        
        Ok(())
    }
    
    pub async fn handle_received(&mut self, msg: GossipsubMessage) -> Result<()> {
        // 1. Validate signature
        if !self.verify_signature(&msg) {
            return Err(anyhow!("Invalid signature"));
        }
        
        // 2. Check duplicate
        let msg_id = self.compute_message_id(&msg);
        if self.seen_messages.contains(&msg_id) {
            return Ok(()); // Already processed
        }
        self.seen_messages.put(msg_id, ());
        
        // 3. Deliver to local subscribers
        if let Some(handlers) = self.local_handlers.get(&msg.topic) {
            for handler in handlers {
                handler.handle(&msg.data).await?;
            }
        }
        
        // 4. Forward to other peers (gossip propagation)
        let subscribers = self.topics.get(&msg.topic).unwrap_or(&HashSet::new());
        let targets = self.select_gossip_peers(subscribers, self.fanout);
        for peer in targets {
            if peer != msg.source { // Don't send back to source
                self.send_message(peer, msg.clone()).await?;
            }
        }
        
        Ok(())
    }
}
```

**Message Structure**:
```rust
pub struct GossipsubMessage {
    pub source: PeerId,
    pub data: Vec<u8>,
    pub sequence_number: u64,
    pub topic: String,
    pub signature: Vec<u8>,  // Ed25519
    pub timestamp: i64,
}
```

**Performance Characteristics**:
- **Latency**: 50-200ms to reach all peers (depends on network size)
- **Bandwidth**: O(√n) messages per node (vs O(n²) for flood)
- **Reliability**: 99.9% delivery with redundant paths
- **Scalability**: Tested up to 10,000 nodes

**Integration Points**:
- **→ Layer 3 (Blockchain)**: Block/transaction propagation
- **→ Layer 4 (Oracle)**: AI response sharing
- **→ Layer 8 (Apps)**: App-to-app messaging

---

### 4. Block Sync Protocol

**Purpose**: Synchronize blockchain state with peers efficiently.

**Sync Strategies**:
1. **Full Sync**: Download entire chain from genesis (first boot)
2. **Incremental Sync**: Request missing blocks only
3. **Fast Sync**: Download state snapshot + recent blocks
4. **Warp Sync**: Checkpoint-based sync (future)

**Implementation**:
```rust
pub struct BlockSync {
    local_chain: Arc<RwLock<Blockchain>>,
    peers: HashMap<PeerId, PeerSyncState>,
    sync_mode: SyncMode,
}

impl BlockSync {
    pub async fn sync(&mut self) -> Result<()> {
        // 1. Determine our current state
        let local_height = self.local_chain.read().await.height();
        
        // 2. Query peers for their heights
        let peer_heights = self.query_peer_heights().await?;
        let max_peer_height = peer_heights.values().max().unwrap_or(&0);
        
        if local_height >= *max_peer_height {
            return Ok(()); // Already synced
        }
        
        // 3. Select best peer (lowest latency, highest reputation)
        let sync_peer = self.select_sync_peer(&peer_heights)?;
        
        // 4. Request missing blocks in batches
        let missing_range = (local_height + 1)..=*max_peer_height;
        let batch_size = 100;
        
        for chunk in missing_range.collect::<Vec<_>>().chunks(batch_size) {
            let blocks = self.request_blocks(sync_peer, chunk).await?;
            
            // 5. Validate and apply blocks
            for block in blocks {
                self.validate_block(&block)?;
                self.local_chain.write().await.add_block(block)?;
            }
            
            // 6. Publish sync progress event
            self.publish_event(Event::SyncProgress {
                current: chunk.last().unwrap(),
                total: max_peer_height,
            });
        }
        
        Ok(())
    }
    
    async fn request_blocks(&self, peer: PeerId, range: &[u64]) -> Result<Vec<Block>> {
        let request = BlockRequest {
            start_height: *range.first().unwrap(),
            end_height: *range.last().unwrap(),
        };
        
        // Send request with timeout
        let response = timeout(
            Duration::from_secs(30),
            self.send_request(peer, request)
        ).await??;
        
        Ok(response.blocks)
    }
}
```

**Protocol Messages**:
```rust
enum BlockSyncMessage {
    GetHeight,                          // Query peer's chain height
    HeightResponse { height: u64 },
    GetBlocks { start: u64, end: u64 },
    BlocksResponse { blocks: Vec<Block> },
    GetState { root_hash: Hash },       // Fast sync
    StateResponse { state: StateTree },
}
```

**Sync Performance**:
- **Full Sync**: ~1 hour for 1M blocks (10KB avg)
- **Incremental**: <5 seconds for 100 blocks
- **Fast Sync**: ~5 minutes (state snapshot + 1000 recent blocks)

**Integration Points**:
- **→ Layer 3 (Blockchain)**: Apply validated blocks
- **→ Connection Manager**: Peer selection
- **→ Layer 9**: Progress notifications

---

### 5. DHT (Distributed Hash Table)

**Purpose**: Decentralized key-value store for peer discovery and data routing.

**DHT Operations**:
```rust
pub trait DHTProvider {
    async fn put(&self, key: Key, value: Vec<u8>) -> Result<()>;
    async fn get(&self, key: Key) -> Result<Vec<u8>>;
    async fn get_closest_peers(&self, key: Key) -> Result<Vec<PeerInfo>>;
}
```

**Kademlia Distance Metric**:
```
distance(peer1, peer2) = XOR(hash(peer1_id), hash(peer2_id))
```

**Use Cases**:
1. **Peer Discovery**: Find peers by content hash
2. **Content Routing**: Locate who has specific data
3. **Rendezvous**: Meet at known keys for coordination
4. **DHT Records**: Store small metadata (< 1KB)

**Implementation Highlights**:
- **k-bucket size**: 20 peers per bucket
- **Replication factor**: 3 copies
- **Refresh interval**: 10 minutes
- **Record TTL**: 24 hours

**Integration Points**:
- **→ Peer Discovery**: Find new peers
- **→ Layer 3**: Locate transaction originators
- **→ Layer 8**: App discovery

---

## Security & Privacy

### 1. Transport Security (Noise Protocol)

**Handshake**:
```
Peer A                              Peer B
  │                                    │
  ├─► Noise_XX_25519_ChaChaPoly       │
  │    (ephemeral key A)               │
  │                                    │
  │                     ◄──────────────┤
  │    (ephemeral key B + signature)   │
  │                                    │
  ├─► (payload encrypted)              │
  │                                    │
  └──► Secure channel established      │
```

**Encryption**:
- **Algorithm**: ChaCha20-Poly1305
- **Key Exchange**: X25519 (ECDH)
- **Authentication**: Ed25519 signatures
- **Forward Secrecy**: Yes (ephemeral keys)

### 2. Message Validation

```rust
fn validate_message(&self, msg: &GossipsubMessage) -> Result<()> {
    // 1. Signature check
    let pubkey = self.get_peer_pubkey(&msg.source)?;
    if !pubkey.verify(&msg.data, &msg.signature) {
        return Err(anyhow!("Invalid signature"));
    }
    
    // 2. Timestamp check (prevent replay)
    let age = Instant::now().duration_since(msg.timestamp);
    if age > Duration::from_secs(300) { // 5 min max
        return Err(anyhow!("Message too old"));
    }
    
    // 3. Size check
    if msg.data.len() > 1_000_000 { // 1MB max
        return Err(anyhow!("Message too large"));
    }
    
    Ok(())
}
```

### 3. Peer Reputation System

```rust
pub struct ReputationTracker {
    scores: HashMap<PeerId, i32>,  // -100 to +100
}

impl ReputationTracker {
    fn update_score(&mut self, peer: PeerId, event: PeerEvent) {
        let delta = match event {
            PeerEvent::ValidBlock => +5,
            PeerEvent::InvalidBlock => -20,
            PeerEvent::Timeout => -10,
            PeerEvent::FastResponse => +2,
            PeerEvent::HelpedSync => +10,
        };
        
        let score = self.scores.entry(peer).or_insert(0);
        *score = (*score + delta).clamp(-100, 100);
        
        // Ban if score too low
        if *score < -50 {
            self.ban_peer(peer);
        }
    }
}
```

---

## Performance & Scalability

### Metrics

**Connection Metrics**:
- Active Peers: 8-12 (target)
- Max Peers: 50
- Connection Latency: <100ms (local), <500ms (global)
- Bandwidth: 1-5 Mbps (depends on activity)

**Message Metrics**:
- GossipSub Fanout: 6 peers
- Message TTL: 120 seconds
- Duplicate Detection: 10,000 messages cached
- Propagation Time: 50-200ms to all peers

**Sync Metrics**:
- Block Download: 1000 blocks/minute
- Full Sync: 1 hour (1M blocks)
- Incremental: <5 seconds

### Optimization Techniques

1. **Connection Pooling**: Reuse TCP connections
2. **Message Batching**: Combine multiple small messages
3. **Compression**: Zstd for large payloads
4. **Bloom Filters**: Efficient duplicate detection
5. **Adaptive Fanout**: Reduce redundancy in large networks

---

## Cross-Layer Integration

### Event Bus Integration

```rust
pub enum NetworkEvent {
    PeerDiscovered(PeerInfo),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    MessageReceived { topic: String, data: Vec<u8> },
    BlockReceived(Block),
    SyncComplete,
}
```

**Event Flow**:
```
Network receives block ──► Event::BlockReceived
                          │
                          └──► Layer 3: Validate and add to chain
                          │
                          └──► Layer 4: Check for oracle responses
                          │
                          └──► Layer 8: Update app state
```

---

## Testing & Debugging

### Network Simulator

```bash
# Spawn 10 local nodes
cargo run --bin karana-net-sim -- --nodes 10

# Inject network partition
karana-net-sim partition --nodes 1,2,3 --duration 30s

# Measure propagation time
karana-net-sim benchmark --message-size 1kb --iterations 100
```

### Metrics Dashboard

```
┌─ Network Status ────────────────────┐
│ Peers: 9 connected, 23 discovered   │
│ Latency: 67ms avg, 234ms max        │
│ Bandwidth: ↓ 1.2 Mbps ↑ 0.8 Mbps    │
│ Messages: 142/min (95% delivered)   │
│ Sync: Block 42,891 / 42,891 ✓       │
└──────────────────────────────────────┘
```

---

## Future Development

### Phase 1: WebRTC Transport (Q1 2026)
- Browser-based peers (no install needed)
- Better NAT traversal
- Lower latency (UDP-based)

### Phase 2: Swarm Intelligence (Q2 2026)
- Distributed AI model inference
- Task coordination across peers
- Incentivized compute sharing

### Phase 3: Mesh Networking (Q3 2026)
- Offline mesh for disasters
- Bluetooth Low Energy peering
- LoRa long-range fallback

### Phase 4: Privacy Enhancements (Q4 2026)
- Tor integration for anonymity
- Onion routing for messages
- Mix networks for metadata privacy

---

## Code References

- `karana-core/src/network/mod.rs`: Network manager
- `karana-core/src/network/discovery.rs`: Peer discovery
- `karana-core/src/network/gossip.rs`: GossipSub implementation
- `karana-core/src/network/sync.rs`: Block synchronization

---

## Summary

Layer 2 provides:
- **Decentralized Communication**: No servers required
- **Automatic Peer Discovery**: mDNS + DHT
- **Encrypted Messaging**: Noise protocol security
- **Efficient Propagation**: GossipSub for O(√n) scaling
- **Blockchain Sync**: Fast synchronization with peers
- **Resilient Connections**: Auto-reconnect with NAT traversal

This layer enables Kāraṇa OS to operate as a true peer-to-peer network, forming a decentralized infrastructure for smart glasses communication.
