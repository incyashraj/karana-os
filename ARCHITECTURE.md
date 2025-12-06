# Kāraṇa OS - Technical Architecture

> The operating system is not a tool. It is a partner.

**Status: 2,225+ tests | 180,000+ LOC Rust | Phases 1-52 Complete**

---

## Layered Architecture Stack (9 Layers)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Layer 9: System Services (OTA, Security, Diagnostics, Recovery)         │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 8: Applications (Timer, Navigation, Social, Settings, Wellness)   │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 7: Interface (Voice, HUD, Gestures, Gaze, AR Rendering)           │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 6: AI Engine (NLU, Dialogue, Reasoning, Action Execution)         │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 5: Intelligence (Multimodal Fusion, Scene Understanding, Memory)  │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 4: Oracle Bridge (Intent Processing, Manifest Rendering, ZK)      │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 3: Blockchain (Chain, Ledger, Governance, Wallet, Celestia DA)    │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 2: P2P Network (libp2p, mDNS, Gossip, Sync)                       │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 1: Hardware (Camera, Sensors, Audio, Display, Power)              │
└─────────────────────────────────────────────────────────────────────────┘

         Cross-Cutting Systems (Phases 46-52)
┌─────────────────────────────────────────────────────────────────────────┐
│ • Resource Management (Adaptive Ledger, AI Profiles, Monitor)           │
│ • Capability Architecture (Layer Discovery, Requirements, Registry)     │
│ • Event Bus (Async Pub/Sub, Priority Routing, Filtering)                │
│ • Resilience (Minimal Mode, Health Monitoring, Feature Gates, Chaos)    │
│ • Progressive UX (Simple Intents, Smart Defaults, Tutorials, Personas)  │
│ • Privacy Management (Retention, Ephemeral, Permissions, Zones)         │
│ • App Ecosystem (Intent Protocol, Android Container, Native Apps)       │
│ • Distributed Compute (Node Discovery, Model Partitioning, Pooling)     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Master System Flow: Complete Layer Interaction

This diagram shows how all 9 layers + cross-cutting systems interact during a typical user interaction:

```
USER: "Hey Kāraṇa, send 50 tokens to Mom"
         │
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 7: INTERFACE                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ VoiceManager │  │   HUD        │  │ GestureTrack │             │
│  └──────┬───────┘  └──────▲───────┘  └──────────────┘             │
│         │ audio            │ render                                 │
│         │ stream           │ manifest                               │
└─────────┼──────────────────┼────────────────────────────────────────┘
          │                  │
          ▼                  │
┌─────────────────────────────────────────────────────────────────────┐
│              CROSS-CUTTING: UX COORDINATOR                           │
│  ┌──────────────────┐  ┌──────────────────┐                        │
│  │ SimpleIntents    │  │ SmartDefaults    │                        │
│  │ "send to Mom" →  │  │ Resolve "Mom" →  │                        │
│  │ Intent::Transfer │  │ mom_address      │                        │
│  └────────┬─────────┘  └────────┬─────────┘                        │
└───────────┼────────────────────┼──────────────────────────────────┘
            │                    │
            ▼                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 6: AI ENGINE                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ NLU Engine   │→ │ Dialogue Mgr │→ │ Action Exec  │             │
│  │ classify()   │  │ fill_slots() │  │ validate()   │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │ Intent           │ context         │ ExecutionPlan       │
└─────────┼──────────────────┼─────────────────┼───────────────────┘
          │                  │                 │
          ▼                  ▼                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│              CROSS-CUTTING: EVENT BUS                                │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │ Event { category: AI, payload: IntentClassified }        │      │
│  │ Event { category: User, payload: TransactionRequested }  │      │
│  └─────────────────┬────────────────────────────────────────┘      │
└────────────────────┼───────────────────────────────────────────────┘
                     │ route_to_subscribers()
                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 4: ORACLE BRIDGE                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ OracleVeil   │→ │ IntentProver │→ │ Manifest Gen │             │
│  │ process()    │  │ generate_zk()│  │ create_ui()  │             │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘             │
│         │ Intent           │ ZKProof         │ UIManifest          │
└─────────┼──────────────────┼─────────────────┼───────────────────┘
          │                  │                 │
          ▼                  ▼                 │
┌─────────────────────────────────────────────┼───────────────────────┐
│              CROSS-CUTTING: PRIVACY MANAGER  │                       │
│  ┌──────────────────┐  ┌──────────────────┐│                       │
│  │ PermissionTrack  │  │ DataRetention    ││                       │
│  │ check_wallet()   │  │ log_transaction()││                       │
│  └────────┬─────────┘  └────────┬─────────┘│                       │
└───────────┼────────────────────┼───────────┼───────────────────────┘
            │ ✓ Allowed          │           │
            ▼                    ▼           │
┌─────────────────────────────────────────────┼───────────────────────┐
│                    LAYER 3: BLOCKCHAIN       │                       │
│  ┌──────────────┐  ┌──────────────┐  ┌─────▼─────┐                │
│  │ Wallet       │→ │ Chain        │→ │ Celestia  │                │
│  │ sign_tx()    │  │ add_block()  │  │ submit()  │                │
│  └──────┬───────┘  └──────┬───────┘  └───────────┘                │
│         │ Transaction      │ Block                                  │
└─────────┼──────────────────┼────────────────────────────────────────┘
          │                  │
          ▼                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│              CROSS-CUTTING: RESOURCE COORDINATOR                     │
│  ┌──────────────────┐  ┌──────────────────┐                        │
│  │ AdaptiveLedger   │  │ ResourceMonitor  │                        │
│  │ check_mode()     │  │ track_usage()    │                        │
│  │ → Full Mode ✓    │  │ Battery: 65%     │                        │
│  └────────┬─────────┘  └──────────────────┘                        │
└───────────┼──────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 2: P2P NETWORK                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ libp2p Node  │→ │ Gossipsub    │→ │ BlockSync    │             │
│  │ broadcast()  │  │ propagate()  │  │ sync_peers() │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
│         │ Block propagated to 5 peers                               │
└─────────┼──────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│              CROSS-CUTTING: RESILIENCE COORDINATOR                   │
│  ┌──────────────────┐  ┌──────────────────┐                        │
│  │ HealthMonitor    │  │ FeatureGates     │                        │
│  │ Layer3: Healthy  │  │ Blockchain: ✓    │                        │
│  │ Layer2: Healthy  │  │ Network: ✓       │                        │
│  └──────────────────┘  └──────────────────┘                        │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼ (render result)
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 7: INTERFACE (Display)                      │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │               HUD DISPLAYS:                               │      │
│  │  ┌─────────────────────────────────────────────────┐     │      │
│  │  │  ✓ Transaction Sent                             │     │      │
│  │  │  50 KARA → Mom                                  │     │      │
│  │  │  Block #42,891                                  │     │      │
│  │  │  5 peers confirmed                              │     │      │
│  │  └─────────────────────────────────────────────────┘     │      │
│  └──────────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 1: HARDWARE                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ Display      │  │ Speaker      │  │ Haptic       │             │
│  │ render()     │  │ beep_success │  │ vibrate()    │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
│         │ Visual + Audio + Haptic Feedback                          │
└─────────┼──────────────────────────────────────────────────────────┘
          │
          ▼
        USER: Sees confirmation, hears beep, feels vibration
        
⏱️ Total Time: ~150ms (voice→display)
✓ All layers coordinated via Event Bus
✓ Cross-cutting systems monitored health, resources, privacy
✓ ZK proof generated (privacy preserved)
✓ Transaction recorded on blockchain
✓ Data retention policy applied
```

---

## Layer-by-Layer Internal Flow Diagrams

### Layer 1: Hardware Layer - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                       LAYER 1: HARDWARE                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │              HardwareManager::tick()                        │    │
│  │           (Called every 16ms - 60 FPS)                      │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► CameraManager::capture_frame()                           │
│       │    ├─► read_v4l2_device() / simulate_frame()                │
│       │    ├─► apply_auto_exposure()                                │
│       │    ├─► apply_white_balance()                                │
│       │    └─► publish_event(Event::CameraFrameReady)               │
│       │                                                              │
│       ├──► SensorFusion::update()                                   │
│       │    ├─► read_imu_data()                                      │
│       │    ├─► read_gps_data()                                      │
│       │    ├─► fuse_position() → WorldCoord                         │
│       │    ├─► calculate_orientation() → Quaternion                 │
│       │    └─► publish_event(Event::PoseUpdated)                    │
│       │                                                              │
│       ├──► AudioCapture::process()                                  │
│       │    ├─► read_microphone_buffer()                             │
│       │    ├─► apply_noise_reduction()                              │
│       │    ├─► detect_voice_activity() → VAD result                 │
│       │    └─► publish_event(Event::AudioReady)                     │
│       │                                                              │
│       ├──► PowerManager::monitor()                                  │
│       │    ├─► read_battery_level() → 0.0-1.0                       │
│       │    ├─► read_temperature() → Celsius                         │
│       │    ├─► calculate_remaining_time()                           │
│       │    ├─► check_thermal_throttle()                             │
│       │    └─► publish_event(Event::PowerStateChanged)              │
│       │                                                              │
│       └──► DisplayManager::render(framebuffer)                      │
│            ├─► composite_layers() → final_image                     │
│            ├─► apply_color_correction()                             │
│            ├─► send_to_waveguide_display()                          │
│            └─► update_refresh_rate()                                │
│                                                                      │
│  Event Flow:                                                         │
│  Hardware → Event Bus → [Subscribed Layers]                         │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 2: P2P Network Layer - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 2: P2P NETWORK                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │              P2PNetwork::tick()                             │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► PeerDiscovery::discover()                                │
│       │    ├─► mDNS::broadcast("_karana._tcp")                      │
│       │    ├─► listen_for_responses()                               │
│       │    ├─► validate_peer_identity()                             │
│       │    ├─► add_to_peer_table(peer_id, multiaddr)                │
│       │    └─► publish_event(Event::PeerDiscovered)                 │
│       │                                                              │
│       ├──► ConnectionManager::maintain()                            │
│       │    ├─► check_peer_health() → ping_timeout?                  │
│       │    ├─► remove_stale_peers()                                 │
│       │    ├─► dial_new_peers()                                     │
│       │    └─► update_routing_table()                               │
│       │                                                              │
│       ├──► GossipSub::process_messages()                            │
│       │    ├─► receive_from_network()                               │
│       │    ├─► validate_message_signature()                         │
│       │    ├─► check_duplicate() → seen_cache                       │
│       │    ├─► route_to_subscribers(topic)                          │
│       │    ├─► forward_to_peers() → propagation                     │
│       │    └─► publish_event(Event::MessageReceived)                │
│       │                                                              │
│       ├──► BlockSync::sync()                                        │
│       │    ├─► request_missing_blocks(peer_id, range)               │
│       │    ├─► receive_block_response()                             │
│       │    ├─► validate_block_signatures()                          │
│       │    ├─► apply_to_local_chain()                               │
│       │    └─► publish_event(Event::ChainSynced)                    │
│       │                                                              │
│       └──► MessageBroadcast::send(message)                          │
│            ├─► serialize_message()                                  │
│            ├─► sign_with_local_key()                                │
│            ├─► select_peers(fanout=6)                               │
│            ├─► send_via_gossipsub()                                 │
│            └─► track_delivery_status()                              │
│                                                                      │
│  Protocols: libp2p + mDNS + Gossipsub + Kademlia DHT               │
│  Security: Ed25519 signatures, encrypted channels                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 3: Blockchain Layer - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 3: BLOCKCHAIN                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │           Blockchain::produce_block()                       │    │
│  │           (Every 30 seconds)                                │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► TxPool::collect_transactions()                           │
│       │    ├─► get_pending_transactions()                           │
│       │    ├─► sort_by_priority() → high_value_first                │
│       │    ├─► validate_each_tx()                                   │
│       │    │    ├─► verify_signature()                              │
│       │    │    ├─► check_balance()                                 │
│       │    │    └─► validate_nonce()                                │
│       │    └─► select_top_N(max_block_size)                         │
│       │                                                              │
│       ├──► BlockBuilder::build()                                    │
│       │    ├─► create_block_header()                                │
│       │    │    ├─► prev_block_hash                                 │
│       │    │    ├─► timestamp = now()                               │
│       │    │    ├─► height = prev_height + 1                        │
│       │    │    └─► merkle_root(transactions)                       │
│       │    ├─► add_transactions(selected_txs)                       │
│       │    ├─► calculate_state_root()                               │
│       │    └─► return unsigned_block                                │
│       │                                                              │
│       ├──► Wallet::sign_block(block)                                │
│       │    ├─► load_keypair()                                       │
│       │    ├─► serialize_block()                                    │
│       │    ├─► sign_with_ed25519()                                  │
│       │    └─► attach_signature(block)                              │
│       │                                                              │
│       ├──► Chain::add_block(signed_block)                           │
│       │    ├─► validate_block()                                     │
│       │    │    ├─► verify_signature()                              │
│       │    │    ├─► check_prev_hash()                               │
│       │    │    ├─► validate_transactions()                         │
│       │    │    └─► check_state_root()                              │
│       │    ├─► apply_state_changes()                                │
│       │    ├─► update_balances()                                    │
│       │    ├─► persist_to_rocksdb()                                 │
│       │    └─► publish_event(Event::BlockAdded)                     │
│       │                                                              │
│       ├──► CelestiaDA::submit(block)                                │
│       │    ├─► encode_block_to_blob()                               │
│       │    ├─► connect_to_mocha_testnet()                           │
│       │    ├─► submit_blob_tx()                                     │
│       │    ├─► wait_for_commitment()                                │
│       │    └─► store_commitment_proof()                             │
│       │                                                              │
│       └──► P2PNetwork::broadcast(block)                             │
│            └─► gossipsub.publish("blocks", block)                   │
│                                                                      │
│  Storage: RocksDB (karana-ledger/)                                  │
│  Consensus: Single validator (dev), future: PoS                     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 4: Oracle Bridge - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 4: ORACLE BRIDGE                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │         OracleVeil::process_intent(intent)                  │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► IntentClassifier::classify()                             │
│       │    ├─► parse_intent_type() → Transfer/Query/Action          │
│       │    ├─► extract_parameters()                                 │
│       │    ├─► validate_completeness()                              │
│       │    └─► return classified_intent                             │
│       │                                                              │
│       ├──► ZKIntentProver::generate_proof()                         │
│       │    ├─► create_intent_commitment()                           │
│       │    ├─► generate_witness()                                   │
│       │    ├─► prove_with_groth16()                                 │
│       │    │    ├─► load_proving_key()                              │
│       │    │    ├─► compute_proof()                                 │
│       │    │    └─► serialize_proof()                               │
│       │    └─► return zk_proof                                      │
│       │                                                              │
│       ├──► BlockchainInterface::execute()                           │
│       │    ├─► create_transaction(intent)                           │
│       │    │    ├─► encode_intent_to_tx_data()                      │
│       │    │    ├─► set_gas_limit()                                 │
│       │    │    └─► set_nonce()                                     │
│       │    ├─► wallet.sign_transaction(tx)                          │
│       │    ├─► chain.submit_transaction(signed_tx)                  │
│       │    └─► wait_for_confirmation()                              │
│       │                                                              │
│       ├──► ManifestGenerator::create_ui()                           │
│       │    ├─► determine_output_type(intent)                        │
│       │    ├─► create_ar_overlays()                                 │
│       │    │    ├─► spatial_position → WorldCoord                   │
│       │    │    ├─► content → text/image                            │
│       │    │    └─► interaction → gaze/gesture                      │
│       │    ├─► create_haptic_pattern()                              │
│       │    │    ├─► intensity → 0.0-1.0                             │
│       │    │    ├─► duration → milliseconds                         │
│       │    │    └─► rhythm → pattern_array                          │
│       │    ├─► create_audio_feedback()                              │
│       │    └─► return UIManifest                                    │
│       │                                                              │
│       └──► SenseOracle::query_external()                            │
│            ├─► query_price_feed(symbol)                             │
│            ├─► query_weather(location)                              │
│            ├─► verify_sensor_data()                                 │
│            └─► return oracle_response                               │
│                                                                      │
│  Bridge: AI decisions → Blockchain actions                          │
│  Privacy: ZK proofs hide intent details on-chain                    │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 5: Intelligence Layer - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 5: INTELLIGENCE                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │         MultimodalFusion::fuse_inputs()                     │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► InputCollector::gather()                                 │
│       │    ├─► get_voice_input() → audio_buffer                     │
│       │    ├─► get_gaze_target() → (x, y, z)                        │
│       │    ├─► get_gesture_state() → hand_pose                      │
│       │    ├─► get_camera_frame() → image                           │
│       │    └─► get_context() → location, time, history              │
│       │                                                              │
│       ├──► ModalityAligner::align()                                 │
│       │    ├─► timestamp_sync() → align_to_same_moment              │
│       │    ├─► spatial_transform() → common_coordinate_frame        │
│       │    ├─► semantic_align() → match_referring_expressions       │
│       │    └─► return aligned_inputs                                │
│       │                                                              │
│       ├──► SceneUnderstanding::analyze()                            │
│       │    ├─► run_blip_model(image)                                │
│       │    │    ├─► image_encoding()                                │
│       │    │    ├─► vision_transformer()                            │
│       │    │    └─► caption_generation()                            │
│       │    ├─► detect_objects(image)                                │
│       │    │    ├─► yolo_detection()                                │
│       │    │    ├─► bounding_boxes()                                │
│       │    │    └─► class_labels()                                  │
│       │    ├─► semantic_segmentation()                              │
│       │    ├─► spatial_relationships()                              │
│       │    │    ├─► "cup on table"                                  │
│       │    │    ├─► "person next to door"                           │
│       │    │    └─► relative_positions()                            │
│       │    └─► return scene_graph                                   │
│       │                                                              │
│       ├──► ContextMemory::retrieve()                                │
│       │    ├─► query_recent_history(limit=10)                       │
│       │    ├─► query_relevant_facts(embedding)                      │
│       │    ├─► query_spatial_anchors(location)                      │
│       │    └─► return context_state                                 │
│       │                                                              │
│       ├──► IntentPredictor::predict()                               │
│       │    ├─► analyze_user_patterns()                              │
│       │    ├─► check_time_of_day()                                  │
│       │    ├─► check_location()                                     │
│       │    ├─► predict_next_action()                                │
│       │    │    ├─► "User usually checks time at 9am"               │
│       │    │    ├─► "User at home → likely to relax"                │
│       │    │    └─► confidence_score                                │
│       │    └─► return predicted_intents                             │
│       │                                                              │
│       └──► FusionEngine::combine()                                  │
│            ├─► weight_modalities() → voice=0.7, gaze=0.2, etc       │
│            ├─► resolve_conflicts() → voice overrides gesture        │
│            ├─► create_unified_intent()                              │
│            └─► publish_event(Event::IntentFused)                    │
│                                                                      │
│  Models: BLIP (vision), MiniLM (embeddings), custom fusion          │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 6: AI Engine - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 6: AI ENGINE                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │         NLUEngine::process_utterance(text)                  │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► IntentClassifier::classify()                             │
│       │    ├─► tokenize_text()                                      │
│       │    ├─► run_minilm_embedding()                               │
│       │    ├─► cosine_similarity(embeddings, intent_patterns)       │
│       │    ├─► select_top_intent(threshold=0.7)                     │
│       │    │    ├─► SendMessage (confidence: 0.92)                  │
│       │    │    ├─► MakePayment (confidence: 0.15)                  │
│       │    │    └─► CheckBalance (confidence: 0.08)                 │
│       │    └─► return Intent::SendMessage                           │
│       │                                                              │
│       ├──► EntityExtractor::extract()                               │
│       │    ├─► regex_patterns() → phone, email, amounts             │
│       │    ├─► named_entity_recognition()                           │
│       │    │    ├─► PERSON: "Mom"                                   │
│       │    │    ├─► NUMBER: "50"                                    │
│       │    │    ├─► TIME: "tomorrow"                                │
│       │    │    └─► LOCATION: "coffee shop"                         │
│       │    ├─► resolve_references() → "Mom" → contact_id            │
│       │    └─► return extracted_entities                            │
│       │                                                              │
│       ├──► DialogueManager::track_conversation()                    │
│       │    ├─► update_dialogue_state()                              │
│       │    │    ├─► current_topic = "payment"                       │
│       │    │    ├─► turn_count = 3                                  │
│       │    │    └─► last_utterance_time                             │
│       │    ├─► manage_context_stack()                               │
│       │    │    ├─► push_new_topic()                                │
│       │    │    ├─► pop_completed_topic()                           │
│       │    │    └─► maintain_history(max=10)                        │
│       │    ├─► slot_filling()                                       │
│       │    │    ├─► required: [recipient, amount]                   │
│       │    │    ├─► filled: [recipient="Mom"]                       │
│       │    │    ├─► missing: [amount]                               │
│       │    │    └─► prompt_for_missing()                            │
│       │    └─► return dialogue_state                                │
│       │                                                              │
│       ├──► ReasoningEngine::reason()                                │
│       │    ├─► load_context(dialogue_state, scene, memory)          │
│       │    ├─► check_feasibility()                                  │
│       │    │    ├─► has_sufficient_balance?                         │
│       │    │    ├─► recipient_valid?                                │
│       │    │    ├─► amount_reasonable?                              │
│       │    │    └─► return feasibility_check                        │
│       │    ├─► predict_consequences()                               │
│       │    ├─► suggest_alternatives()                               │
│       │    └─► return reasoning_result                              │
│       │                                                              │
│       ├──► ActionExecutor::execute()                                │
│       │    ├─► validate_action(intent, reasoning)                   │
│       │    ├─► check_permissions()                                  │
│       │    ├─► create_execution_plan()                              │
│       │    │    ├─► Step 1: Get wallet balance                      │
│       │    │    ├─► Step 2: Create transaction                      │
│       │    │    ├─► Step 3: Sign with key                           │
│       │    │    ├─► Step 4: Submit to chain                         │
│       │    │    └─► Step 5: Wait confirmation                       │
│       │    ├─► execute_steps()                                      │
│       │    ├─► handle_errors() → retry/fallback                     │
│       │    └─► return execution_result                              │
│       │                                                              │
│       └──► ResponseGenerator::generate()                            │
│            ├─► select_response_template(intent)                     │
│            ├─► fill_template_slots(entities)                        │
│            ├─► personalize(user_preferences)                        │
│            └─► return "Sent 50 KARA to Mom. Block #42,891"          │
│                                                                      │
│  Models: MiniLM-L6 (NLU), TinyLlama (dialogue), rule-based logic    │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 7: Interface Layer - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 7: INTERFACE                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │         VoiceCommandManager::process_audio()                │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► WakeWordDetector::detect()                               │
│       │    ├─► run_porcupine("Hey Karana")                          │
│       │    ├─► check_threshold(sensitivity=0.7)                     │
│       │    ├─► if detected → set_listening_mode()                   │
│       │    └─► publish_event(Event::WakeWordDetected)               │
│       │                                                              │
│       ├──► VAD::detect_voice()                                      │
│       │    ├─► analyze_energy_levels()                              │
│       │    ├─► check_spectral_features()                            │
│       │    ├─► duration_check(min=300ms)                            │
│       │    └─► return is_speech_present                             │
│       │                                                              │
│       ├──► WhisperSTT::transcribe()                                 │
│       │    ├─► load_whisper_tiny_model()                            │
│       │    ├─► preprocess_audio()                                   │
│       │    │    ├─► resample_to_16khz()                             │
│       │    │    ├─► normalize_volume()                              │
│       │    │    └─► mel_spectrogram()                               │
│       │    ├─► run_inference()                                      │
│       │    ├─► decode_tokens_to_text()                              │
│       │    └─► return transcript                                    │
│       │                                                              │
│       ├──► GestureRecognizer::track_hands()                         │
│       │    ├─► detect_hands_in_frame(camera)                        │
│       │    ├─► mediapipe_hand_tracking()                            │
│       │    │    ├─► 21 landmarks per hand                           │
│       │    │    ├─► 3D positions + confidence                       │
│       │    │    └─► hand_edness(left/right)                         │
│       │    ├─► classify_gesture()                                   │
│       │    │    ├─► pinch: thumb+index < 2cm                        │
│       │    │    ├─► grab: all_fingers_closed                        │
│       │    │    ├─► swipe: hand_velocity > threshold                │
│       │    │    └─► point: index_extended, others_closed            │
│       │    └─► publish_event(Event::GestureDetected)                │
│       │                                                              │
│       ├──► GazeTracker::track_eyes()                                │
│       │    ├─► detect_eyes_in_frame()                               │
│       │    ├─► estimate_gaze_direction()                            │
│       │    │    ├─► pupil_center()                                  │
│       │    │    ├─► eye_corner_landmarks()                          │
│       │    │    └─► calculate_gaze_vector()                         │
│       │    ├─► project_to_display_space()                           │
│       │    ├─► detect_fixation(dwell_time > 500ms)                  │
│       │    └─► publish_event(Event::GazeUpdated)                    │
│       │                                                              │
│       ├──► HUDManager::render()                                     │
│       │    ├─► collect_render_data()                                │
│       │    │    ├─► notifications_queue                             │
│       │    │    ├─► active_widgets                                  │
│       │    │    ├─► ar_overlays                                     │
│       │    │    └─► system_indicators                               │
│       │    ├─► layout_elements()                                    │
│       │    │    ├─► position_in_viewport()                          │
│       │    │    ├─► z_order_sorting()                               │
│       │    │    └─► occlusion_handling()                            │
│       │    ├─► composite_layers()                                   │
│       │    ├─► apply_transparency()                                 │
│       │    └─► send_to_display(framebuffer)                         │
│       │                                                              │
│       └──► ARRenderer::render_scene()                               │
│            ├─► load_spatial_anchors()                               │
│            ├─► transform_to_camera_space(pose)                      │
│            ├─► render_ar_tabs()                                     │
│            │    ├─► for each tab:                                   │
│            │    ├─► calculate_billboard_matrix()                    │
│            │    ├─► render_content(tab.texture)                     │
│            │    └─► apply_lighting()                                │
│            ├─► render_virtual_objects()                             │
│            └─► blend_with_passthrough(camera)                       │
│                                                                      │
│  Input: Voice, Gestures, Gaze  │  Output: HUD, AR, Haptics         │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 8: Applications Layer - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 8: APPLICATIONS                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │         ApplicationManager::dispatch_intent()               │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► TimerManager::handle()                                   │
│       │    ├─► parse_duration("5 minutes")                          │
│       │    ├─► create_timer(id, duration, callback)                 │
│       │    ├─► start_countdown()                                    │
│       │    ├─► tick() → update_remaining_time()                     │
│       │    ├─► on_complete() → trigger_alert()                      │
│       │    └─► publish_event(Event::TimerFinished)                  │
│       │                                                              │
│       ├──► NavigationEngine::route()                                │
│       │    ├─► resolve_destination("coffee shop")                   │
│       │    ├─► query_location_db()                                  │
│       │    ├─► calculate_route(current_pos, dest)                   │
│       │    │    ├─► fetch_map_data()                                │
│       │    │    ├─► pathfinding_algorithm(A*)                       │
│       │    │    ├─► consider_traffic()                              │
│       │    │    └─► optimize_for_walking()                          │
│       │    ├─► generate_turn_instructions()                         │
│       │    ├─► create_ar_waypoints()                                │
│       │    │    ├─► place_arrow_at(next_turn)                       │
│       │    │    ├─► distance_indicator()                            │
│       │    │    └─► eta_display()                                   │
│       │    └─► return navigation_session                            │
│       │                                                              │
│       ├──► NotificationManager::display()                           │
│       │    ├─► receive_notification(source, message)                │
│       │    ├─► classify_priority()                                  │
│       │    │    ├─► Critical: Show immediately                      │
│       │    │    ├─► High: Queue with sound                          │
│       │    │    ├─► Normal: Queue silently                          │
│       │    │    └─► Low: Batch for later                            │
│       │    ├─► apply_whisper_mode()                                 │
│       │    │    ├─► if privacy_zone == Public:                      │
│       │    │    ├─► hide_sensitive_content                          │
│       │    │    └─► reduce_sound                                    │
│       │    ├─► render_in_hud(position, duration)                    │
│       │    ├─► trigger_haptic(pattern)                              │
│       │    └─► log_to_history()                                     │
│       │                                                              │
│       ├──► SocialManager::contacts()                                │
│       │    ├─► load_contact_db()                                    │
│       │    ├─► search_by_name(query)                                │
│       │    ├─► frequency_sort() → most_contacted_first              │
│       │    ├─► resolve_wallet_address(contact)                      │
│       │    └─► return contact_list                                  │
│       │                                                              │
│       ├──► WellnessManager::monitor()                               │
│       │    ├─► track_screen_time()                                  │
│       │    ├─► detect_eye_strain()                                  │
│       │    │    ├─► blink_rate_analysis()                           │
│       │    │    ├─► viewing_distance_check()                        │
│       │    │    └─► suggest_break(every_20min)                      │
│       │    ├─► posture_tracking()                                   │
│       │    │    ├─► neck_angle_analysis()                           │
│       │    │    ├─► detect_slouching()                              │
│       │    │    └─► vibrate_reminder()                              │
│       │    └─► usage_analytics()                                    │
│       │                                                              │
│       └──► SettingsManager::configure()                             │
│            ├─► load_config_hierarchy()                              │
│            ├─► apply_user_preferences()                             │
│            ├─► validate_changes()                                   │
│            ├─► persist_to_storage()                                 │
│            └─► notify_affected_components()                         │
│                                                                      │
│  App Lifecycle: Created → Started → Running → Paused → Stopped     │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Layer 9: System Services - Internal Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 9: SYSTEM SERVICES                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │         SystemServices::monitor_and_maintain()              │    │
│  └────┬───────────────────────────────────────────────────────┘    │
│       │                                                              │
│       ├──► DiagnosticsManager::health_check()                       │
│       │    ├─► collect_system_metrics()                             │
│       │    │    ├─► cpu_usage → top_cmd()                           │
│       │    │    ├─► memory_usage → /proc/meminfo                    │
│       │    │    ├─► disk_usage → df_cmd()                           │
│       │    │    ├─► network_stats → netstat                         │
│       │    │    └─► battery_health → power_manager                  │
│       │    ├─► run_health_checks()                                  │
│       │    │    ├─► check_all_layers_responsive()                   │
│       │    │    ├─► verify_model_loaded()                           │
│       │    │    ├─► test_network_connectivity()                     │
│       │    │    └─► validate_blockchain_sync()                      │
│       │    ├─► generate_report()                                    │
│       │    ├─► trigger_alerts(if_unhealthy)                         │
│       │    └─► publish_event(Event::HealthReport)                   │
│       │                                                              │
│       ├──► RecoveryManager::handle_crash()                          │
│       │    ├─► detect_crash_signal(SIGSEGV/SIGABRT)                 │
│       │    ├─► capture_crash_dump()                                 │
│       │    │    ├─► stack_trace()                                   │
│       │    │    ├─► register_state()                                │
│       │    │    ├─► memory_dump()                                   │
│       │    │    └─► save_to_disk("/var/crash/")                     │
│       │    ├─► identify_failed_component()                          │
│       │    ├─► attempt_recovery_strategy()                          │
│       │    │    ├─► Strategy 1: Restart component                   │
│       │    │    ├─► Strategy 2: Reset to defaults                   │
│       │    │    ├─► Strategy 3: Fall back to minimal                │
│       │    │    └─► Strategy 4: Full system reboot                  │
│       │    ├─► restore_user_session()                               │
│       │    └─► log_incident()                                       │
│       │                                                              │
│       ├──► OTAManager::update()                                     │
│       │    ├─► check_for_updates()                                  │
│       │    │    ├─► query_update_server()                           │
│       │    │    ├─► compare_versions(current, available)            │
│       │    │    └─► verify_signature(manifest)                      │
│       │    ├─► download_update_package()                            │
│       │    │    ├─► download_chunks(resume_capable)                 │
│       │    │    ├─► verify_checksums()                              │
│       │    │    └─► validate_package_integrity()                    │
│       │    ├─► prepare_installation()                               │
│       │    │    ├─► backup_current_version()                        │
│       │    │    ├─► allocate_staging_partition()                    │
│       │    │    └─► create_rollback_point()                         │
│       │    ├─► install_atomically()                                 │
│       │    │    ├─► write_to_staging()                              │
│       │    │    ├─► verify_installation()                           │
│       │    │    ├─► update_boot_flags()                             │
│       │    │    └─► reboot_to_new_version()                         │
│       │    └─► verify_boot_success()                                │
│       │        ├─► if success: commit_update()                      │
│       │        └─► if fail: rollback_to_previous()                  │
│       │                                                              │
│       └──► SecurityManager::enforce()                               │
│            ├─► authenticate_user()                                  │
│            │    ├─► capture_biometric(iris/voice/face)              │
│            │    ├─► compare_with_stored_template()                  │
│            │    ├─► multi_factor_check()                            │
│            │    └─► return auth_token                               │
│            ├─► check_permissions(resource, action)                  │
│            │    ├─► load_acl(access_control_list)                   │
│            │    ├─► verify_user_role()                              │
│            │    ├─► check_resource_owner()                          │
│            │    └─► return allow/deny                               │
│            ├─► encrypt_data(data, key)                              │
│            │    ├─► aes_256_gcm_encrypt()                           │
│            │    ├─► generate_iv()                                   │
│            │    └─► return ciphertext                               │
│            └─► secure_storage_access()                              │
│                ├─► unlock_encrypted_storage()                       │
│                ├─► validate_access_patterns()                       │
│                └─► log_security_event()                             │
│                                                                      │
│  Watchdog: Monitors all layers, auto-restarts if hung              │
│  Profiler: Performance analysis, bottleneck detection              │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Core Architectural Patterns

### 1. Monad Pattern - Central Orchestrator

The **Monad** (`src/monad.rs`) is the single source of truth orchestrating all 9 layers:

```rust
pub struct Karana {
    // Layer 1: Hardware
    pub hardware: Arc<HardwareManager>,
    
    // Layer 2: Network
    pub p2p: Arc<P2PNetwork>,
    
    // Layer 3: Blockchain
    pub chain: Arc<Blockchain>,
    pub wallet: Arc<KaranaWallet>,
    pub ledger: Arc<Ledger>,
    pub celestia: Arc<CelestiaClient>,
    
    // Layer 4: Oracle
    pub oracle: Arc<OracleVeil>,
    pub intent_prover: Arc<IntentProver>,
    
    // Layer 5: Intelligence
    pub multimodal: Arc<MultimodalFusion>,
    pub scene_understanding: Arc<SceneUnderstanding>,
    
    // Layer 6: AI
    pub nlu: Arc<NLUEngine>,
    pub dialogue: Arc<DialogueManager>,
    pub reasoning: Arc<ReasoningEngine>,
    
    // Layer 7: Interface
    pub voice: Arc<VoiceCommandManager>,
    pub hud: Arc<HUDManager>,
    pub ar: Arc<ARRenderer>,
    pub gaze: Arc<GazeTracker>,
    pub gesture: Arc<GestureRecognizer>,
    
    // Layer 8: Applications
    pub timer: Arc<TimerManager>,
    pub notifications: Arc<NotificationManager>,
    pub navigation: Arc<NavigationEngine>,
    pub social: Arc<SocialManager>,
    pub settings: Arc<SettingsManager>,
    pub wellness: Arc<WellnessManager>,
    
    // Layer 9: System Services
    pub diagnostics: Arc<DiagnosticsManager>,
    pub recovery: Arc<RecoveryManager>,
    pub security: Arc<SecurityManager>,
    pub ota: Arc<OTAManager>,
    
    // Cross-Cutting Systems (Phases 46-52)
    pub resource_coordinator: Arc<ResourceCoordinator>,
    pub event_bus: Arc<EventBus>,
    pub capability_registry: Arc<CapabilityRegistry>,
    pub resilience_coordinator: Arc<ResilienceCoordinator>,
    pub ux_coordinator: Arc<UXCoordinator>,
    pub privacy_manager: Arc<PrivacyManager>,
    pub app_ecosystem: Arc<AppEcosystem>,
    pub distributed_coordinator: Arc<DistributedCoordinator>,
    
    pub fn tick(&mut self, delta_ms: u64) -> Result<()>,
    pub fn process_intent(&mut self, intent: &Intent) -> Result<CommandResult>,
}
```

**Tick Loop (30 second blocks):**
```
Every 30 seconds:
  1. Collect sensor data from Layer 1
  2. Update position & state (Layer 3)
  3. Process voice/input (Layer 7)
  4. Run AI inference (Layer 6)
  5. Update oracle state (Layer 4)
  6. Propose block (Layer 3)
  7. Broadcast to network (Layer 2)
  8. Render AR output (Layer 7)
```

---

## Layer Breakdown & Data Flow

### Layer 1: Hardware Abstraction

```
Physical Hardware
    ↓
┌─────────────────────────────────┐
│   Hardware Drivers              │
│ ┌──────────────────────────────┐│
│ │ CameraDriver    (v4l2/sim)   ││
│ │ IMUSensor       (accel/gyro) ││
│ │ AudioDriver     (mic/speaker)││
│ │ DisplayDriver   (OLED output)││
│ │ PowerDriver     (battery mgmt)││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   Sensor Fusion                 │
│ ┌──────────────────────────────┐│
│ │ IMU Fusion (Pose6DOF)        ││
│ │ GPS/SLAM Fusion (Position)   ││
│ │ Light Probe (Environment)    ││
│ │ Battery/Thermal Monitor      ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ CameraFrame, SensorData, BatteryStatus
```

**Key Types:**
```rust
pub trait CameraDriver: Send + Sync {
    fn capture_frame(&mut self) -> Result<CameraFrame>;
    fn set_resolution(&mut self, w: u32, h: u32) -> Result<()>;
}

pub struct SensorFusion {
    pub imu: IMUData,           // accel, gyro, mag
    pub pose: Pose6DOF,         // position + orientation
    pub velocity: Vector3,
    pub confidence: f32,
}

pub struct BatteryStatus {
    pub level: f32,             // 0.0-1.0
    pub temperature: f32,       // Celsius
    pub estimated_minutes: u32,
    pub thermal_state: ThermalState,
}
```

---

### Layer 2: P2P Network

```
┌─────────────────────────────────┐
│   libp2p Stack                  │
│ ┌──────────────────────────────┐│
│ │ mDNS Discovery               ││
│ │ Gossipsub Pubsub             ││
│ │ Kad DHT                      ││
│ │ Noise Protocol (Encryption)  ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ Network Messages
┌─────────────────────────────────┐
│   Message Types                 │
│ ┌──────────────────────────────┐│
│ │ BlockProposal { block }      ││
│ │ BlockVote { height, hash }   ││
│ │ Transaction { signed_tx }    ││
│ │ StateSync { blocks... }      ││
│ │ Peer { peer_id, addr }       ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ Gossip to all peers
```

**Key Types:**
```rust
pub struct P2PNetwork {
    pub swarm: Swarm<NetworkBehaviour>,
    pub peers: HashMap<PeerId, PeerInfo>,
    pub pending_blocks: VecDeque<SignedBlock>,
    
    pub broadcast(&self, msg: NetworkMessage) -> Result<()>,
    pub get_peer_info(&self, peer_id: PeerId) -> Option<PeerInfo>,
}

pub struct PeerInfo {
    pub peer_id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub last_seen: u64,
    pub blocks_shared: u32,
    pub reputation: f32,
}

pub enum NetworkMessage {
    BlockProposal(SignedBlock),
    BlockVote { height: u64, block_hash: [u8; 32] },
    Transaction(SignedTransaction),
    StateSync(Vec<Block>),
}
```

---

### Layer 3: Blockchain & Consensus

```
┌─────────────────────────────────┐
│   Block Production (30s)        │
│ ┌──────────────────────────────┐│
│ │ 1. Collect txs from pool    ││
│ │ 2. Create BlockBody         ││
│ │ 3. Compute state_root       ││
│ │ 4. Sign with Ed25519        ││
│ │ 5. Broadcast                ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   Block Verification           │
│ ┌──────────────────────────────┐│
│ │ Check signature              ││
│ │ Verify all txs               ││
│ │ Validate merkle_root         ││
│ │ Check chain continuity       ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   State Machine                 │
│ ┌──────────────────────────────┐│
│ │ Apply transactions           ││
│ │ Update account balances      ││
│ │ Record intents (for proofs)  ││
│ │ Update ledger state          ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
```

**Key Types:**
```rust
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<SignedTransaction>,
    pub state_root: [u8; 32],
}

pub struct BlockHeader {
    pub height: u64,
    pub timestamp: u64,
    pub prev_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub proposer: PublicKey,
    pub signature: Signature,  // Ed25519
}

pub struct SignedTransaction {
    pub tx: Transaction,
    pub signature: Signature,
    pub public_key: PublicKey,
}

pub enum Transaction {
    Transfer { to: String, amount: u64 },
    Stake { amount: u64 },
    Vote { governance_id: u64, choice: u32 },
    Intent { intent_proof: IntentProof },
    Custom(Vec<u8>),
}

pub struct Ledger {
    pub accounts: HashMap<String, AccountState>,
    pub nonce: HashMap<String, u64>,
    pub recorded_intents: Vec<RecordedIntent>,
}
```

**Wallet Integration:**
```rust
pub struct KaranaWallet {
    pub keypair: Ed25519Keypair,
    pub did: String,  // did:karana:base58(pubkey)
    
    pub sign(&self, data: &[u8]) -> Signature,
    pub verify(&self, sig: &Signature, data: &[u8]) -> bool,
}
```

---

### Layer 4: Oracle Bridge

```
Intent (Voice/Gesture Input)
    ↓
┌─────────────────────────────────┐
│   Veil - Intent Processor       │
│ ┌──────────────────────────────┐│
│ │ 1. Parse intent              ││
│ │ 2. Validate with ZK proof    ││
│ │ 3. Check permissions (RBAC)  ││
│ │ 4. Reserve gas               ││
│ │ 5. Create execution plan     ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│   Manifest - Output Renderer    │
│ ┌──────────────────────────────┐│
│ │ 1. AR overlays (Toast/Card)  ││
│ │ 2. Haptic feedback           ││
│ │ │ 3. Voice response (TTS)     ││
│ │ 4. Visual effects            ││
│ │ 5. Update blockchain state   ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ CommandResult
```

**Key Types:**
```rust
pub struct OracleVeil {
    pub process_command(
        &mut self,
        command: &str,
        context: &UserContext,
    ) -> Result<OracleResponse>,
}

pub struct OracleResponse {
    pub action: OracleAction,
    pub ui_manifest: UIManifest,
    pub haptic_pattern: Option<HapticPattern>,
    pub voice_response: Option<String>,
    pub blockchain_intent: Option<Intent>,
}

pub struct UIManifest {
    pub ar_overlays: Vec<AROverlay>,
    pub whisper: Option<WhisperNotification>,
    pub haptic: Option<HapticPattern>,
    pub duration_ms: Option<u64>,
}

pub enum AROverlay {
    Toast { text: String, duration_ms: u32 },
    Card { title: String, content: String },
    Confirmation { prompt: String, options: Vec<String> },
    Navigation { instruction: String, direction: Vector3 },
    Highlight { target: Vector3, color: Color },
}

pub struct IntentProof {
    pub intent: Intent,
    pub commitment: [u8; 32],
    pub range_proof: Option<RangeProof>,
    pub authorization_proof: AuthorizationProof,
}
```

---

### Layer 5: Intelligence

```
Scene Understanding
    ↓
┌─────────────────────────────────┐
│   Multimodal Fusion             │
│ ┌──────────────────────────────┐│
│ │ Voice Input (text)           ││
│ │ Gaze Input (eye direction)   ││
│ │ Gesture Input (hand pose)    ││
│ │ Context Input (location)     ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ Combined signal
┌─────────────────────────────────┐
│   Scene Understanding           │
│ ┌──────────────────────────────┐│
│ │ Object detection             ││
│ │ Semantic labeling            ││
│ │ Relationship graphs          ││
│ │ Attention prediction         ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ SceneDescription
┌─────────────────────────────────┐
│   Memory & Prediction           │
│ ┌──────────────────────────────┐││
│ │ Episodic memory (events)     │││
│ │ Semantic memory (facts)      │││
│ │ Procedural memory (skills)   │││
│ │ Predictive model (next step) │││
│ └──────────────────────────────┘││
└─────────────────────────────────┘│
    ↓ Context for Layer 6
```

**Key Types:**
```rust
pub struct MultimodalFusion {
    pub fuse_inputs(
        &self,
        voice: Option<&str>,
        gaze: Option<&GazePoint>,
        gesture: Option<&GestureType>,
        context: &VoiceContext,
    ) -> FusedCommand,
}

pub struct FusedCommand {
    pub primary_intent: Intent,
    pub confidence: f32,
    pub source: CommandSource,
    pub context_relevance: f32,
}

pub enum CommandSource {
    VoiceOnly,
    GazeOnly,
    GestureOnly,
    VoiceGaze,
    VoiceGesture,
    GazeGesture,
    All,
}

pub struct SceneDescription {
    pub foreground: Vec<DetectedObject>,
    pub background: Vec<DetectedObject>,
    pub relationships: Vec<ObjectRelationship>,
    pub activity: Option<ActivityLabel>,
    pub lighting: LightProbe,
}

pub struct DetectedObject {
    pub class_id: u32,
    pub class_name: String,
    pub position: Vector3,
    pub bbox: BoundingBox,
    pub confidence: f32,
}
```

---

### Layer 6: AI Engine

```
FusedCommand Input
    ↓
┌─────────────────────────────────┐
│   NLU Engine                    │
│ ┌──────────────────────────────┐│
│ │ Intent Classifier            ││
│ │  ├─ Pattern matching         ││
│ │  ├─ Confidence scoring       ││
│ │  └─ Alternative intents      ││
│ │ Entity Extractor             ││
│ │  ├─ Slot identification      ││
│ │  └─ Value normalization      ││
│ │ Semantic Parser              ││
│ │  └─ Slot filling             ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ SemanticFrame
┌─────────────────────────────────┐
│   Dialogue Manager              │
│ ┌──────────────────────────────┐│
│ │ Conversation state (FSM)     ││
│ │ Turn-taking management       ││
│ │ Context update               ││
│ │ Clarification detection      ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ DialogueState
┌─────────────────────────────────┐
│   Reasoning Engine              │
│ ┌──────────────────────────────┐│
│ │ Forward chaining (if-then)   ││
│ │ Constraint satisfaction      ││
│ │ Conflict resolution          ││
│ │ Explanation generation       ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ InferenceResult
┌─────────────────────────────────┐
│   Action Executor               │
│ ┌──────────────────────────────┐│
│ │ Safety validation            ││
│ │ Permission check (RBAC)      ││
│ │ Atomic execution             ││
│ │ Rollback on failure          ││
│ └──────────────────────────────┘│
└─────────────────────────────────┘
    ↓ ExecutionResult
```

**Key Types:**
```rust
pub struct NLUEngine {
    pub classify_intent(&self, text: &str) -> IntentClassification,
    pub extract_entities(&self, text: &str) -> Vec<Entity>,
    pub parse_semantics(&self, text: &str) -> SemanticFrame,
}

pub struct IntentClassification {
    pub primary: Intent,
    pub confidence: f32,
    pub alternatives: Vec<(Intent, f32)>,
}

pub enum Intent {
    Navigate { destination: String },
    OpenApp { app: String },
    Query { question: String },
    Control { device: String, action: String },
    Message { recipient: String, body: String },
    Capture { media_type: MediaType },
    Custom(String),
}

pub struct SemanticFrame {
    pub intent: Intent,
    pub slots: HashMap<String, SlotValue>,
    pub confidence: f32,
    pub complete: bool,
}

pub struct DialogueManager {
    pub state: DialogueState,
    pub history: VecDeque<DialogueTurn>,
    pub context: DialogueContext,
}

pub enum DialogueState {
    Idle,
    Listening,
    Processing,
    AwaitingConfirmation { prompt: String },
    AwaitingInput { input_type: InputType },
    Executing { action: String },
    Complete { result: String },
}

pub struct ReasoningEngine {
    pub facts: Vec<Fact>,
    pub rules: Vec<LogicalRule>,
    
    pub infer(&self, new_fact: &Fact) -> Vec<Fact>,
    pub explain(&self, conclusion: &Fact) -> ExplanationPath,
}

pub struct ActionExecutor {
    pub execute(
        &mut self,
        action: &IntentAction,
        context: &ExecutionContext,
    ) -> ExecutionResult,
}
```

---

### Layer 7: Interface

```
AR Renderer (GPU)
    ├─ Camera frame
    ├─ AR Tab positions
    ├─ HUD widgets
    ├─ Particle effects
    ├─ Lighting probe
    └─ Occlusion maps
         ↓
    ┌─────────────────────────────────┐
    │   Compositor                    │
    │ ┌──────────────────────────────┐│
    │ │ Layer composition            ││
    │ │ Depth sorting                ││
    │ │ Alpha blending               ││
    │ │ Post-processing              ││
    │ └──────────────────────────────┘│
    └─────────────────────────────────┘
         ↓ FrameBuffer → Display

Voice Input → Voice Pipeline
    ├─ VAD (Voice Activity Detection)
    ├─ Wake word detection
    ├─ Transcription (Whisper)
    └─ NLU → Dialogue → Oracle
         ↓ CommandResult

Gaze Input → Gaze Tracker
    ├─ Eye tracking
    ├─ Fixation detection
    ├─ Dwell selection (500ms)
    └─ Focus state
         ↓ GazeEvent

Gesture Input → Hand Detector → Gesture Recognizer
    ├─ Hand pose estimation
    ├─ Finger tracking
    ├─ Gesture classification (15+ types)
    └─ Confidence scoring
         ↓ GestureType
```

**Key Types:**
```rust
pub struct ARRenderer {
    pub scene: ARScene,
    pub camera: Camera,
    pub lighting: LightingEngine,
    pub effects: EffectsEngine,
    
    pub render(&mut self) -> FrameBuffer,
}

pub struct VoiceCommandManager {
    pub nlu_engine: Arc<NLUEngine>,
    pub synthesizer: Arc<VoiceSynthesizer>,
    pub context_manager: Arc<VoiceContextManager>,
    pub listener: Arc<ContinuousListener>,
    pub shortcuts: Arc<ShortcutManager>,
    
    pub process(&mut self, audio: &[f32]) -> CommandResult,
}

pub struct GazeTracker {
    pub current_gaze: GazePoint,
    pub fixation_duration: u32,
    pub dwell_threshold: u32,
    
    pub process_gaze(&mut self, raw: &EyeTrackingData) -> GazeEvent,
}

pub struct GestureRecognizer {
    pub models: HashMap<GestureType, GestureModel>,
    
    pub recognize(&self, hand: &HandPose) -> Option<(GestureType, f32)>,
}

pub enum GestureType {
    Pinch { strength: f32 },
    Grab { force: f32 },
    Point { target: Vector3 },
    Swipe { direction: Vector3, velocity: f32 },
    Rotate { axis: Vector3, angle: f32 },
    // ... 10+ more
}
```

---

### Layer 8: Applications

```
┌─────────────────────────────────┐
│ Timer       Navigation  Social   │
│ ├─ Countdown ├─ Routing  ├─ Chat│
│ ├─ Stopwatch ├─ POI      ├─ Pres│
│ └─ Lap       └─ Guidance └─ Cont│
├─────────────────────────────────┤
│ Settings    Wellness   Notifs   │
│ ├─ Profiles ├─ Eyes     ├─ Smart│
│ ├─ Cloud    ├─ Posture  ├─ Group│
│ └─ Sync     └─ Usage    └─ Primt│
└─────────────────────────────────┘
      ↓
    Layer 3 (Blockchain storage)
    Layer 4 (Oracle for execution)
    Layer 6 (AI for intelligence)
```

**Architecture Pattern:**
```rust
pub trait Application: Send + Sync {
    fn update(&mut self, delta_ms: u32, context: &AppContext) -> Result<()>,
    fn on_input(&mut self, event: &InputEvent) -> InputResponse,
    fn render(&self, target: &mut FrameBuffer) -> Result<()>,
}

pub struct AppContext {
    pub location: Location,
    pub time: SystemTime,
    pub user_preferences: HashMap<String, Value>,
    pub blockchain_state: Arc<Ledger>,
}

pub struct ApplicationManager {
    pub apps: HashMap<String, Box<dyn Application>>,
    pub active_app: Option<String>,
    
    pub launch(&mut self, app_id: &str) -> Result<()>,
    pub update_all(&mut self, delta_ms: u32) -> Result<()>,
}
```

---

### Layer 9: System Services

```
┌─────────────────────────────────┐
│   Diagnostics                   │
│ ├─ CPU/Memory metrics           │
│ ├─ Frame rate monitoring        │
│ ├─ Thermal management           │
│ ├─ Watchdog timer               │
│ └─ Performance profiling        │
├─────────────────────────────────┤
│   Security & Recovery           │
│ ├─ Multi-factor auth            │
│ ├─ Crash dumps                  │
│ ├─ Error logging                │
│ ├─ Auto-recovery                │
│ └─ Rollback points              │
├─────────────────────────────────┤
│   OTA Updates                   │
│ ├─ Secure download              │
│ ├─ Signature verification       │
│ ├─ Atomic install               │
│ └─ Rollback protection          │
└─────────────────────────────────┘
```

**Key Types:**
```rust
pub struct DiagnosticsManager {
    pub metrics: SystemMetrics,
    pub event_log: Vec<DiagnosticEvent>,
    
    pub report(&self) -> PerformanceReport,
    pub collect_metrics(&mut self),
}

pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub battery_level: f32,
    pub temperature: f32,
    pub frame_rate: f32,
}

pub struct OTAManager {
    pub check_updates(&self) -> Option<UpdateInfo>,
    pub download(&mut self, update: &UpdateInfo) -> Result<()>,
    pub install(&mut self) -> Result<()>,
    pub rollback(&mut self) -> Result<()>,
}
```

---

## Data Flow Examples

### Example 1: Voice Command → Action

```
1. Audio Input (Layer 1)
   └─ Microphone captures audio chunk

2. Voice Pipeline (Layer 7)
   ├─ VAD: "Is this speech?" → Yes
   ├─ Wake word: "Hey Karana" detected
   └─ Whisper: Transcribe → "navigate to home"

3. Multimodal Fusion (Layer 5)
   └─ Voice only (no gesture, gaze) → confidence 0.95

4. NLU Engine (Layer 6)
   ├─ Intent: Navigate
   ├─ Slot[destination]: "home"
   └─ Confidence: 0.97

5. Dialogue Manager (Layer 6)
   ├─ Current state: Idle
   ├─ New state: Executing
   └─ Next state: Complete

6. Oracle (Layer 4)
   ├─ Create Intent { Navigate { "home" } }
   ├─ Check ZK proof ✓
   ├─ Generate Manifest with AR overlay
   └─ Synthesize: "Navigating to home"

7. AR Renderer (Layer 7)
   ├─ Show navigation arrow
   ├─ Play haptic feedback
   └─ Output voice response

8. Blockchain (Layer 3)
   └─ Record intent in next block
```

### Example 2: Gesture Input → Tab Interaction

```
1. Camera Frame (Layer 1)
   └─ Hand in frame

2. Hand Detection (Layer 7)
   ├─ Hand pose estimation
   └─ Keypoint positions

3. Gesture Recognition (Layer 7)
   ├─ Pattern matching: "Pinch gesture"
   └─ Confidence: 0.94

4. AR Tab Focus (Layer 5)
   ├─ Gaze + gesture fusion
   ├─ Pinch on AR Tab → Click event
   └─ Tab gets focus

5. Multimodal Fusion (Layer 5)
   └─ Gesture + gaze combined → User intent clear

6. Application (Layer 8)
   └─ Tab processes click event

7. Oracle (Layer 4)
   └─ Create Intent { Control { "tab", "click" } }

8. Blockchain (Layer 3)
   └─ Record in ledger
```

### Example 3: Scene Understanding → Proactive Suggestion

```
1. Camera Frame (Layer 1)
   └─ Current scene: Kitchen

2. Scene Understanding (Layer 5)
   ├─ Detect: Stove, Pan, Ingredients
   ├─ Activity: Cooking
   └─ Attention: On stove

3. Memory (Layer 5)
   └─ Recall: User made soup yesterday

4. Prediction (Layer 5)
   └─ User likely cooking soup again

5. NLU (Layer 6)
   └─ Generate Intent { Query { "soup recipe?" } }

6. Oracle (Layer 4)
   ├─ Check ZK proof ✓
   └─ Generate Toast overlay: "Recipe: Tomato Soup"

7. AR Renderer (Layer 7)
   └─ Show AR overlay with recipe steps

8. Blockchain (Layer 3)
   └─ Record proactive suggestion
```

---

## Communication Patterns

### Synchronous (Blocking)

```rust
// Within same thread/async task
let result = voice_manager.process(audio)?;
let intent = nlu_engine.classify_intent(&text)?;
let scene = scene_understanding.analyze(frame)?;
```

### Asynchronous (Non-blocking)

```rust
// Across threads
pub struct EventBus {
    pub send_event(&self, event: SystemEvent) -> Result<()>,
    pub subscribe(&self, topic: &str) -> Receiver<SystemEvent>,
}

pub enum SystemEvent {
    IntentDetected(Intent),
    BlockProduced(Block),
    UserInput(InputEvent),
    AssetLoaded(AssetId),
}
```

### Message Passing (for Network)

```rust
// Over network via libp2p
pub enum NetworkMessage {
    BlockProposal(SignedBlock),
    StateSync(Vec<Block>),
    Transaction(SignedTransaction),
    PeerInfo(PeerInfo),
}
```

---

## State Management

### Global State (Monad)

The Monad holds mutable state for all layers. State is updated atomically each tick:

```
Tick 1: Read inputs → Process → Update state → Broadcast
Tick 2: Read inputs → Process → Update state → Broadcast
...
Tick 30: Produce block with all state changes
```

### Component-Level State (Immutable for reading)

```rust
// Thread-safe read access
pub fn get_current_gaze(&self) -> GazePoint {
    self.gaze.current_gaze.clone()
}

// Mutable update (requires &mut self)
pub fn update_voice_context(&mut self, ctx: VoiceContext) {
    self.voice.context = ctx;
}
```

### Distributed State (Blockchain)

```rust
// State stored in ledger (immutable history)
pub struct Ledger {
    pub blocks: Vec<Block>,
    pub state_root: [u8; 32],
    pub accounts: HashMap<String, AccountState>,
}

// Celestia DA layer (proof of availability)
pub struct CelestiaClient {
    pub submit_blob(&self, data: &[u8]) -> Result<BlobProof>,
    pub verify_availability(&self, proof: &BlobProof) -> bool,
}
```

---

## Performance Model

```
Resource Budget (30-second block):
├─ CPU: ~80% available for AI inference
├─ Memory: ~512 MB (heap) + 256 MB (stack)
├─ GPU: Real-time AR rendering (60 FPS)
├─ Network: Gossip messages (minimal bandwidth)
├─ Storage: RocksDB local state (append-only)
└─ Audio: Continuous listening + synthesis

Latency Targets:
├─ Voice → Intent: < 500ms
├─ Gaze + Gesture → Action: < 100ms
├─ AR Rendering → Display: < 16ms (60 FPS)
├─ Block proposal → Broadcast: < 1s
└─ Oracle → Response: < 100ms
```

---

## Extension Points

### Adding a New Layer

```rust
// 1. Define component
pub struct MyComponent {
    pub state: MyState,
}

impl MyComponent {
    pub fn update(&mut self, inputs: &InputData) -> Result<OutputData>,
    pub fn on_intent(&mut self, intent: &Intent) -> Result<()>,
}

// 2. Add to Monad
pub struct Karana {
    pub my_component: Arc<MyComponent>,
}

// 3. Call in tick loop
pub fn tick(&mut self, delta_ms: u64) -> Result<()> {
    let output = self.my_component.update(inputs)?;
    // ...
}

// 4. Expose as API
pub async fn my_api(&self, params: InputData) -> Result<OutputData> {
    self.my_component.expose_functionality(params)
}
```

### Adding a New Intent Type

```rust
// 1. Define in ai_layer/intent.rs
pub enum Intent {
    MyNewIntent { param1: String, param2: u32 },
}

// 2. Add NLU pattern in voice/nlu.rs
"my command" → Intent::MyNewIntent { param1, param2 }

// 3. Add executor in ai_layer/action_executor.rs
impl ActionExecutor {
    fn execute_my_new_intent(&mut self, params) -> ExecutionResult,
}

// 4. Add manifest generator in oracle/manifest.rs
Intent::MyNewIntent { .. } → UIManifest { ar_overlays, ... }
```

### Adding a New Application

```rust
// 1. Implement Application trait
pub struct MyApp {
    pub state: AppState,
}

impl Application for MyApp {
    fn update(&mut self, delta_ms: u32, ctx: &AppContext) -> Result<()>,
    fn on_input(&mut self, event: &InputEvent) -> InputResponse,
    fn render(&self, target: &mut FrameBuffer) -> Result<()>,
}

// 2. Register in application_manager
app_manager.register("my_app", Box::new(MyApp::new()))?;

// 3. Voice command to launch
"open my app" → Intent::OpenApp { app: "my_app" }
```

---

## Cross-Cutting Systems Flow Diagrams

### Event Bus - Message Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    EVENT BUS (Cross-Cutting)                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Publisher (Any Layer)                                               │
│       │                                                              │
│       ├─► Event::create()                                            │
│       │    ├─► id: Uuid::new()                                       │
│       │    ├─► timestamp: SystemTime::now()                          │
│       │    ├─► source: LayerId                                       │
│       │    ├─► category: EventCategory                               │
│       │    ├─► priority: EventPriority                               │
│       │    └─► payload: EventPayload                                 │
│       │                                                              │
│       ├─► EventBus::publish(event)                                   │
│       │    ├─► validate_event()                                      │
│       │    ├─► add_to_history(event)                                 │
│       │    ├─► EventRouter::route(event)                             │
│       │    │    ├─► match_rules(category, priority)                  │
│       │    │    ├─► select_subscribers(policy)                       │
│       │    │    │    ├─► All: Send to everyone                       │
│       │    │    │    ├─► First: Send to first available             │
│       │    │    │    ├─► RoundRobin: Rotate through list            │
│       │    │    │    ├─► CapabilityBased: Match capabilities        │
│       │    │    │    └─► return subscriber_list                      │
│       │    │    └─► return routing_decision                          │
│       │    └─► async_deliver_to_subscribers()                        │
│       │                                                              │
│       ▼                                                              │
│  Subscriber Channels (tokio::mpsc)                                   │
│       │                                                              │
│       ├─► Layer 1 Channel → Event { Hardware }                       │
│       ├─► Layer 2 Channel → Event { Network }                        │
│       ├─► Layer 3 Channel → Event { Blockchain }                     │
│       ├─► Layer 4 Channel → Event { Oracle }                         │
│       ├─► Layer 5 Channel → Event { Intelligence }                   │
│       ├─► Layer 6 Channel → Event { AI }                             │
│       ├─► Layer 7 Channel → Event { Interface }                      │
│       ├─► Layer 8 Channel → Event { Application }                    │
│       └─► Layer 9 Channel → Event { System }                         │
│                                                                      │
│  Subscriber (Any Layer)                                              │
│       │                                                              │
│       ├─► receive_event(event)                                       │
│       ├─► match event.category:                                      │
│       │    ├─► Hardware: process_hardware_event()                    │
│       │    ├─► Network: process_network_event()                      │
│       │    ├─► AI: process_ai_event()                                │
│       │    └─► ...                                                   │
│       └─► handle_event()                                             │
│                                                                      │
│  Event History (VecDeque<Event>)                                     │
│  ├─► Store last 1000 events                                          │
│  ├─► Enable replay for debugging                                     │
│  └─► Query by category/time                                          │
│                                                                      │
│  Priorities:                                                         │
│  Critical → Process immediately (< 1ms)                              │
│  High → Process within 100ms                                         │
│  Normal → Process within 1s                                          │
│  Low → Process when idle                                             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Resource Coordinator - Adaptive Management

```
┌─────────────────────────────────────────────────────────────────────┐
│              RESOURCE COORDINATOR (Cross-Cutting)                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Monitoring Loop (every 1 second)                                    │
│       │                                                              │
│       ├─► ResourceMonitor::take_snapshot()                           │
│       │    ├─► read_cpu_usage() → 0.0-1.0                            │
│       │    ├─► read_memory_usage() → bytes                           │
│       │    ├─► read_battery_level() → 0.0-1.0                        │
│       │    ├─► read_temperature() → Celsius                          │
│       │    ├─► calculate_resource_level()                            │
│       │    │    ├─► if battery > 60% && temp < 60° → Abundant       │
│       │    │    ├─► if battery 30-60% → Normal                       │
│       │    │    ├─► if battery 15-30% → Limited                      │
│       │    │    └─► if battery < 15% → Critical                      │
│       │    └─► store_in_history(snapshot)                            │
│       │                                                              │
│       ├─► ResourcePredictor::forecast()                              │
│       │    ├─► analyze_trends(history_300_samples)                   │
│       │    ├─► linear_regression(battery_drain)                      │
│       │    ├─► predict_5_minutes_ahead()                             │
│       │    │    ├─► expected_battery: 12%                            │
│       │    │    ├─► expected_temp: 78°C                              │
│       │    │    └─► confidence: 0.85                                 │
│       │    └─► recommend_actions()                                   │
│       │                                                              │
│       ├─► AdaptiveLedger::adjust_mode()                              │
│       │    ├─► classify_current_intent(intent_type)                  │
│       │    │    ├─► HighValue: Payment, critical ops                 │
│       │    │    └─► LowValue: Logging, analytics                     │
│       │    ├─► determine_mode(resource_level)                        │
│       │    │    ├─► Abundant/Normal → Full Mode                      │
│       │    │    ├─► Limited → Light Mode                             │
│       │    │    └─► Critical → Minimal Mode                          │
│       │    ├─► switch_mode_if_changed()                              │
│       │    │    ├─► Full: Store all blocks + validation              │
│       │    │    ├─► Light: Headers + summaries only                  │
│       │    │    └─► Minimal: Current state only                      │
│       │    ├─► prune_old_data(if light_mode)                         │
│       │    └─► checkpoint_state()                                    │
│       │                                                              │
│       ├─► AIProfileManager::adjust_profile()                         │
│       │    ├─► check_active_models()                                 │
│       │    ├─► determine_profile(resource_level)                     │
│       │    │    ├─► Abundant → Advanced                              │
│       │    │    ├─► Normal → Standard                                │
│       │    │    ├─► Limited → Basic                                  │
│       │    │    └─► Critical → UltraLow                              │
│       │    ├─► switch_profile()                                      │
│       │    │    ├─► Advanced: All models loaded                      │
│       │    │    ├─► Standard: Essential models                       │
│       │    │    ├─► Basic: Text + simple vision                      │
│       │    │    └─► UltraLow: Text only                              │
│       │    ├─► unload_unused_models()                                │
│       │    └─► schedule_high_priority_tasks()                        │
│       │                                                              │
│       └─► publish_event(Event::ResourceStateChanged)                 │
│            ├─► current_level: Limited                                │
│            ├─► ledger_mode: Light                                    │
│            ├─► ai_profile: Basic                                     │
│            └─► forecast: Critical in 3 min                           │
│                                                                      │
│  Feedback Loop:                                                      │
│  Layers subscribe → Adjust behavior → Report usage →                │
│  Monitor observes → Predicts → Adjusts → Layers react               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Privacy Manager - Data Control Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│              PRIVACY MANAGER (Cross-Cutting)                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Context Detection (continuous)                                      │
│       │                                                              │
│       ├─► detect_privacy_zone()                                      │
│       │    ├─► get_current_location() → GPS                          │
│       │    ├─► check_geofence(home, work, saved_locations)          │
│       │    ├─► classify_zone()                                       │
│       │    │    ├─► Home: Relaxed policies                           │
│       │    │    ├─► Work: Moderate policies                          │
│       │    │    ├─► Public: Strict policies                          │
│       │    │    ├─► Travel: Enhanced tracking protection            │
│       │    │    └─► Shopping: Minimal data collection               │
│       │    └─► auto_adjust_mode(zone)                                │
│       │                                                              │
│       ├─► DataRetentionManager::apply_policies()                     │
│       │    ├─► for each DataCategory:                                │
│       │    │    ├─► Messages: max_age=30d, max_count=1000           │
│       │    │    ├─► MediaFiles: max_age=90d                          │
│       │    │    ├─► Browsing: max_age=7d                             │
│       │    │    ├─► Location: max_age=24h                            │
│       │    │    ├─► Contacts: protected (no deletion)                │
│       │    │    ├─► Calendar: protected                              │
│       │    │    ├─► Health: max_age=30d                              │
│       │    │    └─► Transactions: protected                          │
│       │    ├─► load_category_items()                                 │
│       │    ├─► apply_age_filter()                                    │
│       │    ├─► apply_count_limit()                                   │
│       │    ├─► delete_expired_items()                                │
│       │    └─► log_cleanup_stats()                                   │
│       │                                                              │
│       ├─► EphemeralModeManager::handle_session()                     │
│       │    ├─► if privacy_zone == Public:                            │
│       │    │    ├─► auto_start_session()                             │
│       │    │    ├─► tag_all_new_data_ephemeral()                     │
│       │    │    └─► show_indicator("Ephemeral Mode Active")          │
│       │    ├─► track_session_data()                                  │
│       │    │    ├─► photos_taken → marked_for_deletion               │
│       │    │    ├─► messages_sent → marked_for_deletion              │
│       │    │    └─► browsing_history → marked_for_deletion           │
│       │    └─► on_zone_change(zone):                                 │
│       │        ├─► if leaving_public:                                │
│       │        │    ├─► delete_all_session_data()                    │
│       │        │    ├─► clear_cache()                                │
│       │        │    └─► zero_trace_cleanup()                         │
│       │        └─► show_confirmation("Data cleared")                 │
│       │                                                              │
│       ├─► PermissionTracker::monitor()                               │
│       │    ├─► on_permission_requested(app, permission):             │
│       │    │    ├─► check_privacy_zone()                             │
│       │    │    ├─► if Public: require_confirmation()                │
│       │    │    ├─► if Home: check_remembered_permissions()          │
│       │    │    ├─► log_usage(app, permission, timestamp)            │
│       │    │    └─► show_indicator(permission_type)                  │
│       │    ├─► track_active_permissions()                            │
│       │    │    ├─► Camera: green_dot_in_hud()                       │
│       │    │    ├─► Microphone: red_dot_in_hud()                     │
│       │    │    ├─► Location: blue_dot_in_hud()                      │
│       │    │    └─► Multiple: combined_indicator()                   │
│       │    └─► generate_daily_report()                               │
│       │        ├─► total_uses_per_permission                         │
│       │        ├─► apps_by_usage                                     │
│       │        ├─► hourly_patterns                                   │
│       │        └─► suspicious_activity_detection                     │
│       │                                                              │
│       └─► publish_event(Event::PrivacyStateChanged)                  │
│            ├─► current_zone: Public                                  │
│            ├─► ephemeral_mode: Active                                │
│            ├─► active_permissions: [Camera, Location]                │
│            └─► retention_policy: Strict                              │
│                                                                      │
│  User Control:                                                       │
│  - View all collected data                                           │
│  - Manually delete categories                                        │
│  - Export privacy reports                                            │
│  - Configure zone boundaries                                         │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Resilience Coordinator - Fault Management

```
┌─────────────────────────────────────────────────────────────────────┐
│            RESILIENCE COORDINATOR (Cross-Cutting)                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Health Monitoring Loop (every 5 seconds)                            │
│       │                                                              │
│       ├─► HealthMonitor::check_all_layers()                          │
│       │    ├─► for each LayerId:                                     │
│       │    │    ├─► ping_layer() → response_time                     │
│       │    │    ├─► check_responsiveness(timeout=1s)                 │
│       │    │    ├─► validate_functionality()                         │
│       │    │    ├─► classify_health()                                │
│       │    │    │    ├─► Healthy: All checks passed                  │
│       │    │    │    ├─► Degraded: Slow but working                  │
│       │    │    │    ├─► Unhealthy: Failures detected                │
│       │    │    │    └─► Unknown: No response                        │
│       │    │    └─► update_layer_status(layer, health)               │
│       │    └─► store_health_history()                                │
│       │                                                              │
│       ├─► CircuitBreaker::manage_per_layer()                         │
│       │    ├─► for each layer with circuit_breaker:                  │
│       │    │    ├─► check_failure_count()                            │
│       │    │    ├─► if failures >= threshold:                        │
│       │    │    │    ├─► open_circuit()                              │
│       │    │    │    ├─► block_requests_to_layer()                   │
│       │    │    │    ├─► start_timeout_timer(30s)                    │
│       │    │    │    └─► publish_event(CircuitOpened)                │
│       │    │    ├─► if circuit_open && timeout_elapsed:              │
│       │    │    │    ├─► enter_half_open_state()                     │
│       │    │    │    ├─► allow_test_request()                        │
│       │    │    │    └─► if success: close_circuit()                 │
│       │    │    └─► if success_count >= threshold:                   │
│       │    │         ├─► close_circuit()                             │
│       │    │         ├─► reset_failure_count()                       │
│       │    │         └─► publish_event(CircuitClosed)                │
│       │    └─► update_circuit_states()                               │
│       │                                                              │
│       ├─► MinimalModeManager::check_activation()                     │
│       │    ├─► check_triggers()                                      │
│       │    │    ├─► battery_level < 10%?                             │
│       │    │    ├─► temperature > 85°C?                              │
│       │    │    ├─► memory_available < 50MB?                         │
│       │    │    ├─► critical_error_detected?                         │
│       │    │    └─► network_total_failure?                           │
│       │    ├─► if should_activate():                                 │
│       │    │    ├─► stop_all_non_essential_services()                │
│       │    │    │    ├─► stop AI engine                              │
│       │    │    │    ├─► stop blockchain sync                        │
│       │    │    │    ├─► stop applications                           │
│       │    │    │    └─► stop distributed compute                    │
│       │    │    ├─► keep_essential_only()                            │
│       │    │    │    ├─► keep HUD (basic display)                    │
│       │    │    │    ├─► keep Voice (commands)                       │
│       │    │    │    └─► keep Wallet (emergency payments)            │
│       │    │    ├─► reduce_resource_usage()                          │
│       │    │    │    ├─► target: <10MB memory                        │
│       │    │    │    └─► target: <5% CPU                             │
│       │    │    ├─► show_minimal_mode_notice()                       │
│       │    │    └─► publish_event(MinimalModeActivated)              │
│       │    └─► monitor_for_recovery()                                │
│       │                                                              │
│       ├─► FeatureGateManager::enforce()                              │
│       │    ├─► for each Feature:                                     │
│       │    │    ├─► check_state(feature)                             │
│       │    │    ├─► if Emergency: block_completely()                 │
│       │    │    ├─► if Disabled: skip_execution()                    │
│       │    │    ├─► if Enabled: allow_execution()                    │
│       │    │    └─► check_dependencies()                             │
│       │    │         ├─► if ARTabs depends on ARRendering            │
│       │    │         └─► if ARRendering disabled → disable ARTabs    │
│       │    └─► apply_feature_state()                                 │
│       │                                                              │
│       └─► publish_event(Event::ResilienceStatus)                     │
│            ├─► layer_health: HashMap<LayerId, HealthStatus>          │
│            ├─► circuit_states: HashMap<LayerId, CircuitState>        │
│            ├─► minimal_mode: Active/Inactive                         │
│            └─► feature_gates: HashMap<Feature, FeatureState>         │
│                                                                      │
│  Recovery Strategies:                                                │
│  1. Restart failed component                                         │
│  2. Reset component to defaults                                      │
│  3. Fall back to minimal mode                                        │
│  4. Full system reboot (last resort)                                 │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Distributed Coordinator - Multi-Node Inference

```
┌─────────────────────────────────────────────────────────────────────┐
│          DISTRIBUTED COORDINATOR (Cross-Cutting)                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  Inference Request Flow                                              │
│       │                                                              │
│       ├─► receive_inference_request(model, input, params)            │
│       │    ├─► check_local_capability()                              │
│       │    ├─► if can_run_locally():                                 │
│       │    │    └─► run_local_inference() → fast_path                │
│       │    └─► else: continue_to_distribution                        │
│       │                                                              │
│       ├─► ComputeNodeProtocol::discover_nodes()                      │
│       │    ├─► broadcast_mDNS("Kāraṇa compute")                      │
│       │    ├─► receive_peer_responses()                              │
│       │    ├─► for each discovered_peer:                             │
│       │    │    ├─► query_capabilities(peer)                         │
│       │    │    │    ├─► cpu_cores, cpu_freq                         │
│       │    │    │    ├─► gpu_memory, ram                             │
│       │    │    │    ├─► acceleration: CUDA/Metal/ROCm/TPU           │
│       │    │    │    └─► current_load                                │
│       │    │    ├─► measure_latency(peer) → ping_time                │
│       │    │    ├─► validate_trust(peer) → signature_check           │
│       │    │    └─► add_to_registry(peer, capabilities)              │
│       │    └─► return available_nodes                                │
│       │                                                              │
│       ├─► ModelPartitioner::partition()                              │
│       │    ├─► load_model_info(model_name)                           │
│       │    │    ├─► total_layers: 64                                 │
│       │    │    ├─► total_params: 70B                                │
│       │    │    ├─► memory_per_layer: 1.2GB                          │
│       │    │    └─► compute_per_layer: 5 TFLOPs                      │
│       │    ├─► select_strategy(available_nodes, model)               │
│       │    │    ├─► if 4+ nodes: LayerWise (sequential)              │
│       │    │    ├─► if 2 powerful nodes: TensorParallel              │
│       │    │    ├─► if 3+ nodes: Pipeline                            │
│       │    │    └─► if mixed: Hybrid                                 │
│       │    ├─► partition_model(strategy, num_nodes)                  │
│       │    │    ├─► LayerWise:                                       │
│       │    │    │    ├─► Node 1: Layers 0-15                         │
│       │    │    │    ├─► Node 2: Layers 16-31                        │
│       │    │    │    ├─► Node 3: Layers 32-47                        │
│       │    │    │    └─► Node 4: Layers 48-63                        │
│       │    │    ├─► calculate_memory_requirements()                  │
│       │    │    ├─► estimate_coordination_overhead()                 │
│       │    │    └─► return PartitionedModel                          │
│       │    └─► validate_partitions()                                 │
│       │                                                              │
│       ├─► EdgeCloudPool::allocate_nodes()                            │
│       │    ├─► check_pool_capacity(requirements)                     │
│       │    ├─► select_nodes(strategy)                                │
│       │    │    ├─► LeastLoaded: Pick least busy                     │
│       │    │    ├─► LowestLatency: Pick fastest                      │
│       │    │    ├─► MostCapable: Pick most powerful                  │
│       │    │    ├─► RoundRobin: Rotate through pool                  │
│       │    │    └─► return selected_nodes                            │
│       │    ├─► reserve_resources(nodes, requirements)                │
│       │    ├─► if insufficient_capacity && auto_scale:               │
│       │    │    ├─► discover_additional_nodes()                      │
│       │    │    └─► add_to_pool()                                    │
│       │    └─► return allocated_nodes                                │
│       │                                                              │
│       ├─► DistributedInference::coordinate_execution()               │
│       │    ├─► assign_partitions_to_nodes()                          │
│       │    ├─► transfer_model_weights(if_not_cached)                 │
│       │    ├─► execute_strategy():                                   │
│       │    │    ├─► LayerWise (Sequential):                          │
│       │    │    │    ├─► Node1.run(input) → output1                  │
│       │    │    │    ├─► transfer(output1 → Node2)                   │
│       │    │    │    ├─► Node2.run(output1) → output2                │
│       │    │    │    ├─► transfer(output2 → Node3)                   │
│       │    │    │    ├─► Node3.run(output2) → output3                │
│       │    │    │    ├─► transfer(output3 → Node4)                   │
│       │    │    │    └─► Node4.run(output3) → final_output           │
│       │    │    ├─► TensorParallel (Parallel):                       │
│       │    │    │    ├─► broadcast(input → all_nodes)                │
│       │    │    │    ├─► Node1.run(left_half) & Node2.run(right)     │
│       │    │    │    ├─► gather_results()                            │
│       │    │    │    └─► combine_tensors() → final_output            │
│       │    │    └─► Pipeline:                                        │
│       │    │         ├─► Stage1(batch1) → Stage2(batch2)             │
│       │    │         └─► overlapped_execution()                      │
│       │    ├─► collect_outputs()                                     │
│       │    ├─► measure_metrics()                                     │
│       │    │    ├─► total_latency: 120ms                             │
│       │    │    ├─► tokens_per_second: 95                            │
│       │    │    ├─► coordination_overhead: 15ms                      │
│       │    │    └─► nodes_used: 4                                    │
│       │    └─► return InferenceResponse                              │
│       │                                                              │
│       └─► release_resources(allocated_nodes)                         │
│            ├─► mark_nodes_available()                                │
│            ├─► update_pool_statistics()                              │
│            └─► publish_event(InferenceCompleted)                     │
│                                                                      │
│  Security: Only model computations distributed, not user data        │
│  Privacy: Input/output stay on local device                          │
│  Efficiency: Automatic node selection based on capabilities          │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Concurrency Model

```
Monad Thread (Main loop)
├─ Tick every 16ms (60 FPS)
├─ Process all layer updates
├─ Produce blocks every 30s
└─ Broadcast to network

Audio Thread
├─ Capture microphone continuously
├─ VAD processing
└─ Send chunks to voice pipeline

Render Thread
├─ GPU operations
├─ AR composition
└─ Display output

Network Thread (libp2p)
├─ Gossip processing
├─ Block sync
└─ Peer discovery

Worker Pool (Tokio)
├─ Async I/O operations
├─ File operations
├─ Network requests
└─ Long-running tasks
```

**Synchronization:**
```rust
// Atomic state updates
pub struct AtomicState {
    pub inner: Arc<Mutex<State>>,
}

pub fn update(&self, func: impl FnOnce(&mut State)) {
    let mut state = self.inner.lock();
    func(&mut state);
}

// Lock-free if possible
pub struct RingBuffer<T: Clone> {
    pub push(&mut self, item: T),
    pub pop(&mut self) -> Option<T>,
}
```

---

## Security Architecture

```
┌─────────────────────────────────┐
│   Cryptographic Foundation      │
│ ├─ Ed25519 (signing)            │
│ ├─ AES-256-GCM (encryption)     │
│ ├─ PBKDF2-SHA256 (key derivation)
│ ├─ Groth16 (ZK proofs)          │
│ └─ Blake3 (hashing)             │
├─────────────────────────────────┤
│   Permission Model (RBAC)       │
│ ├─ Resource ownership           │
│ ├─ Action permissions           │
│ ├─ Time-based locks             │
│ └─ User consent verification    │
├─────────────────────────────────┤
│   Privacy (by Design)           │
│ ├─ Local-first processing       │
│ ├─ ZK intent proofs (no details)│
│ ├─ Encrypted storage            │
│ └─ No telemetry/tracking        │
└─────────────────────────────────┘
```

---

## Phase 46: Adaptive Resource Management

**Purpose**: Intelligent resource optimization for constrained AR hardware

### Resource Monitor (`resource/monitor.rs`)

```rust
pub struct ResourceMonitor {
    cpu_usage: f32,           // 0.0-1.0
    memory_used: u64,         // bytes
    thermal_state: ThermalState,
    battery_level: f32,       // 0.0-1.0
    history: VecDeque<ResourceSnapshot>,  // 300 samples
}

pub enum ResourceLevel {
    Abundant,    // >60% battery, <60°C, <50% CPU
    Normal,      // 30-60% battery, 60-70°C
    Limited,     // 15-30% battery, 70-80°C
    Critical,    // <15% battery, >80°C
}

pub struct ResourcePrediction {
    time_horizon: Duration,      // 5 minutes lookahead
    expected_battery: f32,
    expected_thermal: f32,
    recommended_actions: Vec<ResourceAction>,
}
```

**How It Works**:
1. **Continuous Monitoring**: Samples CPU/memory/thermal/battery every 100ms
2. **Historical Analysis**: Maintains 300-sample window (30 seconds at 100ms)
3. **Predictive Analytics**: 5-minute lookahead prevents throttling
4. **Capability Negotiation**: Layers request resources, monitor approves/denies

**Example Flow**:
```
Battery: 18% → ResourceLevel::Limited
Prediction: Will hit 15% in 3 minutes
Actions: 
  - Switch ledger to Light mode
  - Reduce AI to Basic profile
  - Pause background sync
```

### Adaptive Ledger (`resource/adaptive_ledger.rs`)

```rust
pub enum LedgerMode {
    Full,      // All blocks + full validation
    Light,     // Block headers + summaries
    Minimal,   // Current state only
}

pub enum IntentType {
    HighValue,   // Payments, critical operations
    LowValue,    // Logging, analytics
}

impl AdaptiveLedger {
    pub fn classify_intent(&self, intent: &Intent) -> IntentType;
    pub fn should_process(&self, intent: &Intent) -> bool;
    pub fn switch_mode(&mut self, mode: LedgerMode);
}
```

**Mode Transitions**:
- **Full → Light**: Battery <30% or temp >70°C
- **Light → Minimal**: Battery <15% or temp >80°C
- **Auto-pruning**: Removes old blocks in Light mode
- **Checkpointing**: Periodic state snapshots

### AI Profiles (`resource/ai_profiles.rs`)

```rust
pub enum AIProfile {
    UltraLow,   // Text-only, <50MB RAM
    Basic,      // Text + simple vision, <200MB
    Standard,   // Full multimodal, <500MB
    Advanced,   // All features + background tasks, <1GB
}

pub enum AICapability {
    TextGeneration,
    ImageRecognition,
    SpeechToText,
    Embedding,
    SceneUnderstanding,
}

impl AIProfileManager {
    pub fn current_capabilities(&self) -> Vec<AICapability>;
    pub fn can_run_model(&self, model: &str) -> bool;
    pub fn downgrade_if_needed(&mut self, resources: &ResourceSnapshot);
}
```

**Profile Decision Matrix**:
```
Battery > 40%, Temp < 65°C → Advanced
Battery 20-40%, Temp 65-75°C → Standard
Battery 10-20%, Temp 75-85°C → Basic
Battery < 10% or Temp > 85°C → UltraLow
```

### Resource Coordinator (`resource/mod.rs`)

```rust
pub struct ResourceCoordinator {
    monitor: ResourceMonitor,
    adaptive_ledger: AdaptiveLedger,
    ai_profiles: AIProfileManager,
}

impl ResourceCoordinator {
    pub async fn start(&mut self) {
        loop {
            let snapshot = self.monitor.take_snapshot();
            let prediction = self.monitor.predict_resources();
            
            // Auto-adjust ledger
            let ledger_mode = self.determine_ledger_mode(&snapshot);
            self.adaptive_ledger.switch_mode(ledger_mode);
            
            // Auto-adjust AI
            let ai_profile = self.determine_ai_profile(&snapshot);
            self.ai_profiles.switch_profile(ai_profile);
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
```

**Tests**: 22 tests covering monitoring, mode switching, predictions, coordination

---

## Phase 47: Capability-Based Architecture + Event Bus

**Purpose**: Decouple layers for extensibility and maintainability

### Layer Capabilities (`capability/traits.rs`)

```rust
pub enum LayerId {
    Hardware,      // Layer 1
    Network,       // Layer 2
    Blockchain,    // Layer 3
    Oracle,        // Layer 4
    Intelligence,  // Layer 5
    AI,            // Layer 6
    Interface,     // Layer 7
    Applications,  // Layer 8
    System,        // Layer 9
}

pub enum Capability {
    // Hardware capabilities
    CameraCapture, MicrophoneInput, DisplayOutput,
    IMUSensing, GPSPositioning, BatteryMonitoring,
    
    // Network capabilities
    PeerDiscovery, MessageBroadcast, BlockSync,
    
    // Blockchain capabilities
    TransactionProcessing, BlockValidation, StateQuery,
    
    // Intelligence capabilities
    SceneUnderstanding, ObjectRecognition, VoiceRecognition,
    
    // AI capabilities
    IntentClassification, EntityExtraction, ResponseGeneration,
    
    // Interface capabilities
    VoiceInput, GazeTracking, GestureRecognition,
    ARRendering, HapticFeedback,
    
    // Application capabilities
    TimerManagement, NotificationDisplay, NavigationRouting,
    
    // System capabilities
    Diagnostics, Recovery, OTAUpdate, Security,
}

pub struct CapabilityRequirements {
    required: Vec<Capability>,
    optional: Vec<Capability>,
    version: semver::Version,
}

pub trait Layer {
    fn id(&self) -> LayerId;
    fn capabilities(&self) -> Vec<Capability>;
    fn requirements(&self) -> CapabilityRequirements;
    fn state(&self) -> LayerState;
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
}
```

**How It Works**:
1. **Layer Registration**: Each layer advertises its capabilities
2. **Requirement Checking**: Layers declare required/optional capabilities
3. **Dynamic Discovery**: Registry enables runtime capability lookup
4. **Graceful Degradation**: Missing optional capabilities don't block startup

**Example**:
```rust
// AI Layer declares requirements
CapabilityRequirements {
    required: vec![
        Capability::CameraCapture,        // Must have camera
        Capability::MicrophoneInput,      // Must have mic
    ],
    optional: vec![
        Capability::GPSPositioning,       // Nice to have GPS
        Capability::IMUSensing,           // Nice to have IMU
    ],
}

// At startup, system checks if requirements are met
if !registry.has_capabilities(&ai_layer.requirements().required) {
    return Err("AI layer cannot start - missing camera or mic");
}
```

### Event Bus (`event_bus/core.rs`)

```rust
pub struct Event {
    id: Uuid,
    timestamp: SystemTime,
    source: LayerId,
    category: EventCategory,
    priority: EventPriority,
    payload: EventPayload,
    metadata: EventMetadata,
}

pub enum EventPriority {
    Critical,   // Process immediately
    High,       // Process within 100ms
    Normal,     // Process within 1s
    Low,        // Process when idle
}

pub enum EventCategory {
    System,     // System-level events
    Hardware,   // Hardware state changes
    Network,    // Network events
    Blockchain, // Blockchain events
    AI,         // AI processing events
    User,       // User interaction events
    Application,// App-level events
    Error,      // Error events
}

pub struct EventBus {
    subscribers: HashMap<EventCategory, Vec<Subscriber>>,
    history: VecDeque<Event>,  // Recent events
}

impl EventBus {
    pub async fn publish(&self, event: Event) -> Result<()>;
    pub async fn subscribe(&mut self, 
        category: EventCategory, 
        handler: Box<dyn EventHandler>
    ) -> SubscriptionId;
    pub async fn unsubscribe(&mut self, id: SubscriptionId);
}
```

**Event Flow**:
```
Layer 1 (Hardware) → Event { category: Hardware, payload: CameraFrameReady }
                  ↓
              Event Bus
                  ↓
    ┌─────────────┼─────────────┐
    ↓             ↓             ↓
Layer 5       Layer 6       Layer 7
(Scene)       (AI)          (AR)
```

### Event Router (`event_bus/router.rs`)

```rust
pub enum RoutingPolicy {
    All,              // Deliver to all subscribers
    First,            // Deliver to first available
    Random,           // Random selection
    RoundRobin,       // Rotate through subscribers
    CapabilityBased,  // Route based on capabilities
}

pub struct RoutingRule {
    category: EventCategory,
    priority: EventPriority,
    policy: RoutingPolicy,
    filters: Vec<EventFilter>,
}

impl EventRouter {
    pub fn add_rule(&mut self, rule: RoutingRule);
    pub fn route(&self, event: &Event) -> Vec<SubscriptionId>;
}
```

**Example Routing**:
```rust
// Critical events → All subscribers
RoutingRule {
    category: EventCategory::Error,
    priority: EventPriority::Critical,
    policy: RoutingPolicy::All,
}

// AI requests → Capability-based routing
RoutingRule {
    category: EventCategory::AI,
    policy: RoutingPolicy::CapabilityBased,
    // Routes to subscribers with AICapability
}
```

**Tests**: 18 tests (7 capability + 11 event bus)

---

## Phase 48: Fault Resilience & Graceful Degradation

**Purpose**: Ultra-reliable operation with intelligent failure recovery

### Minimal Mode (`resilience/minimal_mode.rs`)

```rust
pub struct MinimalModeManager {
    state: MinimalModeState,
    reason: Option<MinimalModeReason>,
    start_time: Option<SystemTime>,
}

pub enum MinimalModeState {
    Inactive,
    Activating,
    Active,
    Deactivating,
}

pub enum MinimalModeReason {
    LowBattery,        // <10% battery
    HighTemperature,   // >85°C
    LowMemory,         // <50MB free
    CriticalError,     // Unrecoverable error
    NetworkFailure,    // Total network loss
    UserRequested,     // Manual activation
    TestMode,          // Chaos testing
}

pub struct MinimalFeatures {
    hud: bool,         // ✓ Basic HUD only
    voice: bool,       // ✓ Voice commands
    wallet: bool,      // ✓ Emergency payments
    gesture: bool,     // ✗ No gestures
    gaze: bool,        // ✗ No gaze
    ai: bool,          // ✗ No AI
    blockchain: bool,  // ✗ No sync
}
```

**Minimal Mode Constraints**:
- **Memory**: <10MB total footprint
- **CPU**: <5% average usage
- **Features**: HUD + Voice + Wallet only
- **Battery**: Can run for hours on 5% battery

**Activation Logic**:
```rust
impl MinimalModeManager {
    pub fn should_activate(&self, resources: &ResourceSnapshot) -> bool {
        resources.battery_level < 0.10 ||
        resources.temperature > 85.0 ||
        resources.memory_available < 50_000_000
    }
    
    pub async fn activate(&mut self, reason: MinimalModeReason) {
        // 1. Stop all non-essential services
        self.stop_ai_engine().await;
        self.stop_blockchain_sync().await;
        self.stop_applications().await;
        
        // 2. Keep only essentials
        self.keep_hud().await;
        self.keep_voice().await;
        self.keep_wallet().await;
        
        // 3. Notify user
        self.show_minimal_mode_notice().await;
    }
}
```

### Health Monitor (`resilience/health_monitor.rs`)

```rust
pub struct HealthMonitor {
    layer_health: HashMap<LayerId, HealthStatus>,
    circuit_breakers: HashMap<LayerId, CircuitBreaker>,
    history: HashMap<LayerId, VecDeque<HealthCheckResult>>,
}

pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<SystemTime>,
    config: CircuitBreakerConfig,
}

pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Blocking all requests
    HalfOpen,   // Testing if recovered
}

pub struct CircuitBreakerConfig {
    failure_threshold: u32,      // Open after N failures
    timeout: Duration,            // Wait before HalfOpen
    success_threshold: u32,       // Close after N successes
}
```

**Circuit Breaker Logic**:
```
Closed (Normal)
    ↓ (3 failures)
Open (Blocked)
    ↓ (wait 30s)
HalfOpen (Testing)
    ↓ (1 success)
Closed (Recovered)
```

**Per-Layer Health Checks**:
```rust
impl HealthMonitor {
    pub async fn check_hardware(&self) -> HealthCheckResult {
        // Camera responsive? Sensors working?
    }
    
    pub async fn check_network(&self) -> HealthCheckResult {
        // Can discover peers? Can send messages?
    }
    
    pub async fn check_ai(&self) -> HealthCheckResult {
        // Models loaded? Inference working?
    }
    
    // ... for all 9 layers
}
```

### Feature Gates (`resilience/feature_gates.rs`)

```rust
pub enum Feature {
    // Core features
    VoiceCommands, GazeTracking, GestureRecognition,
    ARRendering, HapticFeedback,
    
    // AI features
    SceneUnderstanding, ObjectRecognition, IntentClassification,
    DialogueManagement, ResponseGeneration,
    
    // Blockchain features
    BlockProduction, TransactionValidation, PeerDiscovery,
    
    // Application features
    ARTabs, SpatialAnchors, Navigation, Social,
    TimerSystem, Notifications, Wellness,
    
    // System features
    Diagnostics, Recovery, OTAUpdates, Biometrics,
    
    // Advanced features (Phases 46-52)
    ResourceManagement, CapabilitySystem, EventBus,
    MinimalMode, PrivacyManagement, AppEcosystem,
    DistributedCompute,
}

pub struct FeatureGateManager {
    gates: HashMap<Feature, FeatureState>,
    dependencies: HashMap<Feature, Vec<Feature>>,
}

pub enum FeatureState {
    Enabled,
    Disabled,
    Emergency,  // Kill switch activated
}
```

**Feature Dependencies**:
```rust
Feature::ARTabs depends on:
  - Feature::ARRendering
  - Feature::SpatialAnchors
  - Feature::GazeTracking

If ARRendering is disabled → ARTabs auto-disables
```

**Emergency Kill Switches**:
```rust
impl FeatureGateManager {
    pub fn emergency_disable(&mut self, feature: Feature) {
        // Immediately disable feature + all dependents
        self.gates.insert(feature, FeatureState::Emergency);
        for dependent in self.find_dependents(feature) {
            self.gates.insert(dependent, FeatureState::Disabled);
        }
    }
}
```

### Chaos Testing (`resilience/chaos.rs`)

```rust
pub enum ChaosScenario {
    CameraFailure,      // Camera stops responding
    NetworkPartition,   // Lose all peers
    ByzantineNode,      // Malicious peer
    OTARollback,        // Update fails, rollback
    LowBattery,         // Sudden battery drop
    HighTemperature,    // Thermal throttling
    MemoryPressure,     // Memory exhaustion
    DiskFull,           // Storage full
}

pub struct ChaosTestFramework {
    active_scenarios: Vec<ChaosScenario>,
}

impl ChaosTestFramework {
    pub async fn inject(&mut self, scenario: ChaosScenario) {
        match scenario {
            ChaosScenario::CameraFailure => {
                // Simulate camera hardware failure
                hardware.camera.fail();
                // Verify system falls back to voice-only
                assert!(system.is_voice_only_mode());
            },
            ChaosScenario::NetworkPartition => {
                // Drop all network connections
                network.disconnect_all_peers();
                // Verify system continues offline
                assert!(system.can_operate_offline());
            },
            // ... more scenarios
        }
    }
}
```

**Tests**: 34 tests covering minimal mode, health monitoring, feature gates, chaos scenarios

---

## Phase 49: Progressive Disclosure UX

**Purpose**: 80% reduction in cognitive load for mainstream users

### Simple Intents (`ux/simple_intents.rs`)

```rust
pub struct IntentExpander {
    templates: Vec<IntentTemplate>,
    history: VecDeque<SimpleIntent>,
}

pub enum SimpleIntent {
    Message { recipient: String, content: Option<String> },
    Call { recipient: String },
    Navigate { destination: String },
    Search { query: String },
    Play { content: String },
    Photo,
    Video,
    Timer { duration: String },
    Reminder { content: String, when: Option<String> },
    Open { app: String },
    Share { content: String },
    Pay { recipient: String, amount: Option<String> },
}

pub struct IntentTemplate {
    pattern: String,              // "Hey, {action} {target}"
    action: SimpleAction,
    required_params: Vec<String>,
    optional_params: Vec<String>,
}
```

**Natural Language Patterns**:
```
"Hey, message Mom" → SimpleIntent::Message { recipient: "Mom", content: None }
"Hey, call Sarah" → SimpleIntent::Call { recipient: "Sarah" }
"Hey, navigate home" → SimpleIntent::Navigate { destination: "home" }
"Hey, play music" → SimpleIntent::Play { content: "music" }
"Hey, set timer 5 minutes" → SimpleIntent::Timer { duration: "5 minutes" }
```

**Intent Expansion**:
```rust
impl IntentExpander {
    pub fn expand(&self, simple: SimpleIntent) -> Intent {
        match simple {
            SimpleIntent::Message { recipient, content } => {
                // Expand to full intent
                Intent::SendMessage {
                    recipient: self.resolve_contact(recipient),
                    content: content.unwrap_or_else(|| self.prompt_for_content()),
                    encryption: EncryptionType::EndToEnd,
                    priority: MessagePriority::Normal,
                }
            },
            SimpleIntent::Navigate { destination } => {
                Intent::Navigate {
                    destination: self.resolve_location(destination),
                    mode: TransportMode::Auto,
                    ar_overlay: true,
                    traffic_aware: true,
                }
            },
            // ... more expansions
        }
    }
}
```

### Smart Defaults (`ux/smart_defaults.rs`)

```rust
pub struct SmartDefaults {
    context: DefaultContext,
    patterns: LearnedPatterns,
}

pub struct DefaultContext {
    current_time: SystemTime,
    current_location: Option<Location>,
    recent_contacts: Vec<Contact>,
    recent_locations: Vec<Location>,
}

pub struct LearnedPatterns {
    frequent_contacts: HashMap<String, f32>,  // name → frequency
    time_patterns: HashMap<Hour, Vec<Action>>, // hour → common actions
    location_patterns: HashMap<Location, Vec<Action>>,
}

impl SmartDefaults {
    pub fn suggest_time(&self, intent: &str) -> SystemTime {
        // "remind me tomorrow" at 8pm → suggests 9am next day
        if intent.contains("tomorrow") && self.context.current_time.hour() > 18 {
            return self.context.current_time + Duration::from_hours(13);
        }
        // "lunch meeting" → suggests 12:00pm
        if intent.contains("lunch") {
            return self.next_occurrence_of(12, 0);
        }
        // Default: 1 hour from now
        self.context.current_time + Duration::from_hours(1)
    }
    
    pub fn suggest_contact(&self, partial: &str) -> Vec<Contact> {
        // Combine: recent contacts + frequent contacts + name match
        let mut candidates = Vec::new();
        candidates.extend(self.match_recent(partial));
        candidates.extend(self.match_frequent(partial));
        candidates.extend(self.match_name(partial));
        candidates.sort_by_score();
        candidates.take(5)
    }
}
```

**Context-Aware Suggestions**:
```
5:00pm, near home:
  "Hey, navigate" → suggests "home"

9:00am, weekday:
  "Hey, call" → suggests work contacts

Saturday 2pm:
  "Hey, remind me" → suggests tomorrow 9am (not tonight)
```

### Interactive Tutorials (`ux/tutorials.rs`)

```rust
pub struct TutorialManager {
    tutorials: HashMap<TutorialId, Tutorial>,
    user_progress: HashMap<TutorialId, TutorialProgress>,
}

pub enum TutorialCategory {
    Basics,      // First-time setup, basic navigation
    Voice,       // Voice commands
    Gestures,    // Hand gestures
    Apps,        // Using apps
    Advanced,    // Power user features
}

pub struct Tutorial {
    id: TutorialId,
    title: String,
    category: TutorialCategory,
    steps: Vec<TutorialStep>,
    estimated_duration: Duration,
}

pub struct TutorialStep {
    instruction: String,
    demo: Option<DemoVideo>,
    validation: Box<dyn Fn(&SystemState) -> bool>,
    hint: Option<String>,
}
```

**Built-in Tutorials**:
```
1. "Welcome to Kāraṇa" (5 min)
   - Step 1: Say "Hey Kāraṇa" to wake up
   - Step 2: Try "Hey, what time is it?"
   - Step 3: Look at an object and say "What is this?"

2. "Voice Commands" (3 min)
   - Common patterns
   - Disambiguation
   - Error correction

3. "Hand Gestures" (7 min)
   - Pinch to select
   - Swipe to navigate
   - Grab to move

4. "Running Apps" (5 min)
   - Opening YouTube
   - Controlling with voice
   - Spatial positioning
```

### Persona Profiles (`ux/personas.rs`)

```rust
pub enum UserPersona {
    Casual,        // Minimal tech knowledge
    Professional,  // Business user
    Developer,     // Technical user
    PowerUser,     // Advanced features
}

pub struct PersonaManager {
    current_persona: UserPersona,
    preferences: PersonaPreferences,
}

pub struct PersonaPreferences {
    complexity_level: ComplexityLevel,
    feature_visibility: HashMap<Feature, bool>,
    default_actions: HashMap<Intent, Action>,
    privacy_mode: PrivacyMode,
}

pub enum ComplexityLevel {
    Simple,    // Hide advanced features
    Moderate,  // Show some advanced features
    Advanced,  // Show most features
    Expert,    // Show everything
}
```

**Persona Customization**:
```
Casual Persona:
  - Simple voice templates only
  - Auto-enable smart defaults
  - Hide technical details
  - Maximum privacy by default
  
Developer Persona:
  - Show all capabilities
  - Enable debugging features
  - Expose API details
  - Allow manual overrides
```

**Tests**: 25 tests covering intents, defaults, tutorials, personas

---

## Phase 50: Privacy-First Data Management

**Purpose**: 90% reduction in stored sensitive data, full user control

### Data Retention (`privacy/retention.rs`)

```rust
pub enum DataCategory {
    Messages,      // Text/voice messages
    MediaFiles,    // Photos, videos
    Browsing,      // Web history
    Location,      // GPS traces
    Contacts,      // Contact interactions
    Calendar,      // Events, meetings
    Health,        // Wellness data
    Transactions,  // Payment history
}

pub struct RetentionPolicy {
    category: DataCategory,
    max_age: Option<Duration>,
    max_count: Option<usize>,
    protected: bool,  // Prevent auto-deletion
}

pub struct DataRetentionManager {
    policies: HashMap<DataCategory, RetentionPolicy>,
    last_cleanup: SystemTime,
}

impl DataRetentionManager {
    pub async fn apply_policies(&mut self) {
        for (category, policy) in &self.policies {
            let items = self.load_category_items(category);
            
            // Age-based cleanup
            if let Some(max_age) = policy.max_age {
                let cutoff = SystemTime::now() - max_age;
                items.retain(|item| item.timestamp > cutoff);
            }
            
            // Count-based cleanup
            if let Some(max_count) = policy.max_count {
                if items.len() > max_count {
                    items.sort_by_timestamp();
                    items.truncate(max_count);
                }
            }
            
            self.save_category_items(category, items);
        }
    }
}
```

**Default Policies**:
```
Messages: Delete after 30 days or keep last 1000
MediaFiles: Delete after 90 days
Browsing: Delete after 7 days
Location: Delete after 24 hours
Contacts: Keep forever (protected)
Calendar: Keep forever (protected)
Health: Delete after 30 days
Transactions: Keep forever (protected)
```

### Ephemeral Mode (`privacy/ephemeral.rs`)

```rust
pub struct EphemeralModeManager {
    active: bool,
    session_start: Option<SystemTime>,
    session_data: Vec<DataItem>,
}

pub enum EphemeralMode {
    Off,
    Temporary,    // Manual session
    AutoPrivate,  // Auto-enable in public zones
}

impl EphemeralModeManager {
    pub async fn start_session(&mut self) {
        self.active = true;
        self.session_start = Some(SystemTime::now());
        self.session_data.clear();
        
        // Mark all new data as ephemeral
        self.tag_new_data_as_ephemeral();
    }
    
    pub async fn end_session(&mut self) {
        // Delete all session data
        for item in &self.session_data {
            self.delete_item(item).await;
        }
        
        self.active = false;
        self.session_start = None;
        self.session_data.clear();
    }
}
```

**Ephemeral Mode Behavior**:
```
Session active:
  - All photos → deleted on session end
  - All messages → deleted on session end
  - All browsing → deleted on session end
  - No persistent logs

Session ends (manual or auto):
  - Zero trace left on device
  - No recovery possible
```

### Permission Tracking (`privacy/permissions.rs`)

```rust
pub enum Permission {
    Camera,
    Microphone,
    Location,
    Contacts,
    Files,
    Network,
    Notifications,
    Bluetooth,
}

pub struct PermissionTracker {
    active_permissions: HashMap<Permission, Vec<UsageRecord>>,
    usage_stats: HashMap<Permission, PermissionStats>,
}

pub struct UsageRecord {
    permission: Permission,
    app: String,
    timestamp: SystemTime,
    duration: Duration,
    context: UsageContext,
}

pub struct PermissionStats {
    total_uses: u64,
    total_duration: Duration,
    apps: HashMap<String, u64>,  // app → use count
    hourly_pattern: [u64; 24],   // uses per hour
}

impl PermissionTracker {
    pub fn record_usage(&mut self, permission: Permission, app: &str) {
        let record = UsageRecord {
            permission,
            app: app.to_string(),
            timestamp: SystemTime::now(),
            duration: Duration::from_secs(0),  // Updated on stop
            context: self.current_context(),
        };
        
        self.active_permissions
            .entry(permission)
            .or_default()
            .push(record);
        
        // Visual indicator
        self.show_permission_indicator(permission);
    }
    
    pub fn generate_report(&self) -> PermissionReport {
        // Daily summary of all permission usage
    }
}
```

**Real-Time Indicators**:
```
Camera active: Green dot in HUD
Microphone active: Red dot in HUD
Location tracking: Blue dot in HUD
Multiple active: Combined indicator
```

### Privacy Manager (`privacy/mod.rs`)

```rust
pub enum PrivacyMode {
    Standard,   // Default policies
    Enhanced,   // Stricter policies
    Maximum,    // Paranoid mode
}

pub enum PrivacyZone {
    Home,       // Relaxed policies
    Work,       // Moderate policies
    Public,     // Strict policies
    Travel,     // Enhanced tracking protection
    Shopping,   // Minimal data collection
}

pub struct PrivacyManager {
    mode: PrivacyMode,
    zone: PrivacyZone,
    retention: DataRetentionManager,
    ephemeral: EphemeralModeManager,
    permissions: PermissionTracker,
}

impl PrivacyManager {
    pub fn determine_zone(&self, location: &Location) -> PrivacyZone {
        // Geo-fence based zone detection
        if self.is_home(location) { PrivacyZone::Home }
        else if self.is_work(location) { PrivacyZone::Work }
        else if self.is_airport_or_hotel(location) { PrivacyZone::Travel }
        else if self.is_shopping_area(location) { PrivacyZone::Shopping }
        else { PrivacyZone::Public }
    }
    
    pub fn auto_adjust(&mut self, zone: PrivacyZone) {
        match zone {
            PrivacyZone::Public => {
                // Strictest settings
                self.ephemeral.start_session();
                self.permissions.require_confirmation_for_all();
            },
            PrivacyZone::Home => {
                // Relaxed settings
                self.ephemeral.end_session();
                self.permissions.allow_remembered_permissions();
            },
            // ... more zone adjustments
        }
    }
}
```

**Privacy Zone Policies**:
```
Home Zone:
  - Remember permissions
  - Standard retention (30 days)
  - No ephemeral mode
  
Public Zone:
  - Confirm every permission
  - Enhanced retention (7 days)
  - Auto ephemeral mode
  
Travel Zone:
  - Location tracking minimized
  - Enhanced encryption
  - Frequent data cleanup
```

**Tests**: 32 tests (8 retention + 7 ephemeral + 7 permissions + 10 manager)

---

## Phase 51: App Ecosystem & Native Apps

**Purpose**: Enable mainstream app adoption on AR glasses

### Intent Protocol (`app_ecosystem/intent_protocol.rs`)

```rust
pub enum IntentType {
    Network { action: NetworkAction },
    Ledger { action: LedgerAction },
    Oracle { action: OracleAction },
    AI { action: AIAction },
    Share { content: String, target: Option<String> },
    Store { key: String, value: Vec<u8> },
    Query { key: String },
    Camera { mode: CameraMode },
    Microphone { mode: MicMode },
    Location,
    OpenApp { app_id: String },
    SendData { app_id: String, data: Vec<u8> },
}

pub enum NetworkAction {
    HttpRequest { url: String, method: String },
    WebSocket { url: String },
    P2PMessage { peer_id: String, data: Vec<u8> },
}

pub enum LedgerAction {
    SendTransaction { to: String, amount: u64 },
    QueryBalance { address: String },
    GetHistory { address: String, limit: usize },
}

pub enum OracleAction {
    GetPrice { symbol: String },
    GetWeather { location: String },
    Custom { query: String },
}

pub enum AIAction {
    TextGeneration { prompt: String, max_tokens: usize },
    ImageRecognition { image: Vec<u8> },
    VoiceToText { audio: Vec<u8> },
    GetEmbedding { text: String },
}

pub struct IntentRouter {
    apps: HashMap<String, AppHandle>,
    permissions: PermissionManager,
}

impl IntentRouter {
    pub async fn route_intent(&self, 
        app_id: &str, 
        intent: IntentType
    ) -> Result<IntentResponse> {
        // 1. Validate permission
        if !self.permissions.has_permission(app_id, &intent) {
            return Err("Permission denied");
        }
        
        // 2. Route to appropriate layer
        match intent {
            IntentType::Network { action } => {
                self.handle_network(app_id, action).await
            },
            IntentType::Ledger { action } => {
                self.handle_ledger(app_id, action).await
            },
            IntentType::AI { action } => {
                self.handle_ai(app_id, action).await
            },
            // ... more routing
        }
    }
}
```

**Example App Communication**:
```rust
// YouTube app wants to stream video
IntentType::Network {
    action: NetworkAction::HttpRequest {
        url: "https://youtube.com/watch?v=...",
        method: "GET",
    }
}

// WhatsApp wants to send message
IntentType::Ledger {
    action: LedgerAction::SendTransaction {
        to: recipient_address,
        amount: 0,  // Free message
    }
}

// Instagram wants AI image recognition
IntentType::AI {
    action: AIAction::ImageRecognition {
        image: photo_bytes,
    }
}
```

### Android Container (`app_ecosystem/android_container.rs`)

```rust
pub struct AndroidContainer {
    state: ContainerState,
    properties: AndroidProperties,
    apps: HashMap<String, AndroidApp>,
}

pub struct AndroidProperties {
    screen_width: u32,
    screen_height: u32,
    dpi: u32,
    api_level: u32,
    hardware_features: Vec<String>,
}

pub struct AndroidApp {
    package_name: String,
    activity: String,
    lifecycle_state: ActivityState,
    permissions: Vec<String>,
}

pub enum ActivityState {
    Created,
    Started,
    Resumed,
    Paused,
    Stopped,
    Destroyed,
}

impl AndroidContainer {
    pub async fn launch_app(&mut self, package: &str) -> Result<()> {
        // Waydroid-like approach
        // 1. Check if container running
        if !self.is_running() {
            self.start_container().await?;
        }
        
        // 2. Launch Android activity
        let app = self.apps.get_mut(package)
            .ok_or("App not installed")?;
        
        app.lifecycle_state = ActivityState::Created;
        self.send_intent_to_android(package).await?;
        app.lifecycle_state = ActivityState::Started;
        app.lifecycle_state = ActivityState::Resumed;
        
        Ok(())
    }
}
```

**Container Features**:
- Screen virtualization (1920x1080 → AR display)
- Touch event translation (gaze → touch)
- Audio routing (Android audio → spatial audio)
- Clipboard sync
- Notification forwarding

### Native Apps (`app_ecosystem/native_apps.rs`)

```rust
pub struct NativeApp {
    id: String,
    name: String,
    category: AppCategory,
    optimizations: AROptimizations,
    karana_integration: KaranaIntegration,
}

pub enum AppCategory {
    Video,
    Communication,
    Social,
    Productivity,
    Entertainment,
    Shopping,
    Navigation,
}

pub struct AROptimizations {
    spatial_controls: bool,     // Can position in 3D
    voice_commands: Vec<String>,
    gesture_support: bool,
    gaze_targeting: bool,
}

pub struct KaranaIntegration {
    wallet_connected: bool,     // Use Kāraṇa wallet
    privacy_config: PrivacyConfig,
    resource_profile: ResourceProfile,
}
```

**Pre-Configured Apps** (15 total):

1. **YouTube**
```rust
NativeApp {
    id: "com.google.android.youtube",
    optimizations: AROptimizations {
        spatial_controls: true,
        voice_commands: vec![
            "play video",
            "pause",
            "next video",
            "search for {query}",
        ],
        gesture_support: true,  // Swipe to skip
        gaze_targeting: true,    // Look at video to focus
    },
    karana_integration: KaranaIntegration {
        wallet_connected: true,  // Subscribe with Kāraṇa tokens
        privacy_config: PrivacyConfig {
            block_tracking: true,
            ephemeral_history: true,
        },
    },
}
```

2. **WhatsApp**
```rust
NativeApp {
    id: "com.whatsapp",
    optimizations: AROptimizations {
        voice_commands: vec![
            "call {contact}",
            "message {contact}",
            "read messages",
        ],
        gesture_support: false,  // Voice-first
        gaze_targeting: true,    // Look at contact to select
    },
    karana_integration: KaranaIntegration {
        wallet_connected: false,  // Uses own system
        privacy_config: PrivacyConfig {
            e2e_encryption: true,
            local_storage: true,
        },
    },
}
```

3-15: Gmail, Google Maps, Spotify, Instagram, Twitter, TikTok, Netflix, Amazon, Uber, Zoom, Discord, Telegram, Browser

### App Store (`app_ecosystem/app_store.rs`)

```rust
pub struct AppStore {
    listings: HashMap<String, AppListing>,
    security_scanner: SecurityScanner,
}

pub struct AppListing {
    app_id: String,
    name: String,
    developer: String,
    version: semver::Version,
    rating: f32,
    downloads: u64,
    screenshots: Vec<String>,
    security_status: SecurityStatus,
    sandbox_profile: SandboxProfile,
}

pub enum SecurityStatus {
    Verified,      // Passed all checks
    Unverified,    // Not yet scanned
    Suspicious,    // Warning signs detected
    Malicious,     // Known malware
}

pub enum SandboxProfile {
    Strict,     // Minimal permissions, isolated
    Moderate,   // Standard permissions
    Relaxed,    // More permissions (verified apps only)
}

pub struct SecurityScanner {
    malware_signatures: Vec<Signature>,
    permission_analyzer: PermissionAnalyzer,
}

impl SecurityScanner {
    pub async fn scan_app(&self, apk: &[u8]) -> SecurityReport {
        let mut report = SecurityReport::default();
        
        // 1. Malware signature scan
        report.malware_detected = self.check_signatures(apk);
        
        // 2. Permission analysis
        let perms = self.extract_permissions(apk);
        report.excessive_permissions = self.analyze_permissions(&perms);
        
        // 3. Network behavior analysis
        report.suspicious_urls = self.analyze_network_calls(apk);
        
        // 4. Code obfuscation detection
        report.obfuscated = self.detect_obfuscation(apk);
        
        report
    }
}
```

**Security Flow**:
```
App submission
    ↓
Automated scan
    ↓
Permission analysis
    ↓
Network behavior check
    ↓
Code review (for popular apps)
    ↓
Verification badge (if passed)
    ↓
Listed in store
```

**Tests**: Verified individually per module (intent protocol, container, native apps, store)

---

## Phase 52: Distributed Compute

**Purpose**: Run 70B+ models via edge cloud integration

### Compute Node Protocol (`distributed/compute_node.rs`)

```rust
pub struct ComputeNode {
    id: String,
    capabilities: NodeCapabilities,
    status: NodeStatus,
    resources: NodeResources,
    location: NodeLocation,
}

pub struct NodeCapabilities {
    cpu_cores: u32,
    cpu_freq_ghz: f32,
    gpu_memory_gb: f32,
    ram_gb: f32,
    storage_gb: f32,
    acceleration: Vec<HardwareAccel>,
}

pub enum HardwareAccel {
    CUDA { compute_capability: String },
    Metal,
    ROCm,
    OpenCL,
    Vulkan,
    NPU,
    TPU,
}

pub enum NodeStatus {
    Available,
    Busy,
    Offline,
    Unreachable,
    Maintenance,
}

pub struct NodeResources {
    cpu_usage: f32,      // 0.0-1.0
    gpu_usage: f32,
    memory_used_gb: f32,
    memory_total_gb: f32,
}

pub struct NodeLocation {
    lat: f64,
    lon: f64,
    latency_ms: f32,     // Measured latency to this node
}

pub struct ComputeNodeProtocol {
    discovered_nodes: HashMap<String, ComputeNode>,
    local_node: ComputeNode,
}

impl ComputeNodeProtocol {
    pub async fn discover_nodes(&mut self) -> Result<Vec<ComputeNode>> {
        // 1. Broadcast discovery message on local network
        let peers = self.network.discover_local_peers().await?;
        
        // 2. Query each peer for capabilities
        for peer in peers {
            let capabilities = self.query_capabilities(&peer).await?;
            let resources = self.query_resources(&peer).await?;
            
            self.discovered_nodes.insert(peer.id.clone(), ComputeNode {
                id: peer.id,
                capabilities,
                status: NodeStatus::Available,
                resources,
                location: self.measure_location(&peer).await?,
            });
        }
        
        Ok(self.discovered_nodes.values().cloned().collect())
    }
    
    pub fn select_nodes(&self, 
        requirements: &ComputeRequirements
    ) -> Vec<ComputeNode> {
        self.discovered_nodes.values()
            .filter(|node| node.meets_requirements(requirements))
            .sorted_by_latency()
            .take(requirements.max_nodes)
            .cloned()
            .collect()
    }
}
```

**Node Discovery Flow**:
```
1. Broadcast mDNS: "Kāraṇa compute node"
2. Receive responses with capabilities
3. Measure latency to each node
4. Store in registry
5. Periodically refresh (every 10s)
```

### Model Partitioning (`distributed/model_partitioning.rs`)

```rust
pub enum PartitionStrategy {
    LayerWise,      // Split by transformer layers
    TensorParallel, // Split tensors horizontally
    Pipeline,       // Pipeline parallelism
    Hybrid,         // Combination of above
}

pub struct ModelPartitioner {
    strategy: PartitionStrategy,
}

pub struct ModelPartition {
    id: usize,
    layer_range: Option<(usize, usize)>,  // For LayerWise
    tensor_slice: Option<TensorSlice>,     // For TensorParallel
    stage: Option<usize>,                  // For Pipeline
    memory_required_gb: f32,
    compute_required_tflops: f32,
}

pub struct PartitionedModel {
    model_name: String,
    total_layers: usize,
    total_params: u64,
    partitions: Vec<ModelPartition>,
    coordination_overhead_ms: f32,
}

impl ModelPartitioner {
    pub fn partition(&self, 
        model_info: &ModelInfo, 
        num_nodes: usize
    ) -> PartitionedModel {
        match self.strategy {
            PartitionStrategy::LayerWise => {
                self.partition_by_layers(model_info, num_nodes)
            },
            PartitionStrategy::TensorParallel => {
                self.partition_by_tensors(model_info, num_nodes)
            },
            PartitionStrategy::Pipeline => {
                self.partition_by_pipeline(model_info, num_nodes)
            },
            PartitionStrategy::Hybrid => {
                self.partition_hybrid(model_info, num_nodes)
            },
        }
    }
    
    fn partition_by_layers(&self, 
        model: &ModelInfo, 
        num_nodes: usize
    ) -> PartitionedModel {
        let layers_per_node = model.total_layers / num_nodes;
        let mut partitions = Vec::new();
        
        for i in 0..num_nodes {
            let start = i * layers_per_node;
            let end = if i == num_nodes - 1 {
                model.total_layers
            } else {
                (i + 1) * layers_per_node
            };
            
            partitions.push(ModelPartition {
                id: i,
                layer_range: Some((start, end)),
                tensor_slice: None,
                stage: None,
                memory_required_gb: model.memory_per_layer * (end - start) as f32,
                compute_required_tflops: model.compute_per_layer * (end - start) as f32,
            });
        }
        
        PartitionedModel {
            model_name: model.name.clone(),
            total_layers: model.total_layers,
            total_params: model.total_params,
            partitions,
            coordination_overhead_ms: 5.0 * num_nodes as f32,  // Sequential
        }
    }
}
```

**Partition Strategies**:

1. **LayerWise**: Sequential execution across nodes
```
Node 1: Layers 0-15  (16 layers)
Node 2: Layers 16-31 (16 layers)
Node 3: Layers 32-47 (16 layers)
Node 4: Layers 48-63 (16 layers)

Total: 64 layers (e.g., LLaMA-70B)
Coordination: Sequential, ~20ms overhead
```

2. **TensorParallel**: Parallel tensor operations
```
Node 1: Left half of all weight matrices
Node 2: Right half of all weight matrices

Each node processes same input, combines outputs
Coordination: Frequent, ~50ms overhead
```

3. **Pipeline**: Pipelined execution
```
Stage 1 (Node 1): Process batch 1
Stage 2 (Node 2): Process batch 2 while Node 1 does batch 3
Stage 3 (Node 3): Process batch 3 while Node 2 does batch 4

Throughput optimized, ~30ms overhead
```

4. **Hybrid**: Pipeline + Tensor Parallel
```
Stage 1: Nodes 1-2 (tensor parallel)
Stage 2: Nodes 3-4 (tensor parallel)

Best of both worlds, ~40ms overhead
```

### Distributed Inference (`distributed/distributed_inference.rs`)

```rust
pub struct DistributedInference {
    partitioner: ModelPartitioner,
    node_protocol: ComputeNodeProtocol,
    active_requests: HashMap<Uuid, InferenceRequest>,
}

pub struct InferenceRequest {
    id: Uuid,
    model: String,
    input: InferenceInput,
    parameters: InferenceParameters,
    assigned_nodes: Vec<String>,
}

pub enum InferenceInput {
    Text(String),
    Tokens(Vec<u32>),
    Image(Vec<u8>),
    Audio(Vec<f32>),
    Multimodal { text: String, image: Vec<u8> },
}

pub struct InferenceParameters {
    max_tokens: usize,
    temperature: f32,
    top_p: f32,
    top_k: usize,
}

pub struct InferenceResponse {
    request_id: Uuid,
    output: InferenceOutput,
    metrics: InferenceMetrics,
}

pub enum InferenceOutput {
    Text(String),
    Tokens(Vec<u32>),
    Image(Vec<u8>),
    Embedding(Vec<f32>),
    Error(String),
}

pub struct InferenceMetrics {
    total_latency_ms: f32,
    tokens_per_second: f32,
    nodes_used: usize,
    coordination_overhead_ms: f32,
}

impl DistributedInference {
    pub async fn infer(&mut self, 
        model: &str, 
        input: InferenceInput,
        params: InferenceParameters,
    ) -> Result<InferenceResponse> {
        let request_id = Uuid::new_v4();
        
        // 1. Discover available nodes
        let nodes = self.node_protocol.discover_nodes().await?;
        
        // 2. Partition model across nodes
        let model_info = self.get_model_info(model)?;
        let partitioned = self.partitioner.partition(&model_info, nodes.len());
        
        // 3. Assign partitions to nodes
        let assigned = self.assign_partitions(&partitioned, &nodes)?;
        
        // 4. Coordinate execution
        let start = Instant::now();
        let output = match partitioned.strategy {
            PartitionStrategy::LayerWise => {
                self.execute_sequential(&assigned, &input, &params).await?
            },
            PartitionStrategy::TensorParallel => {
                self.execute_parallel(&assigned, &input, &params).await?
            },
            PartitionStrategy::Pipeline => {
                self.execute_pipelined(&assigned, &input, &params).await?
            },
            PartitionStrategy::Hybrid => {
                self.execute_hybrid(&assigned, &input, &params).await?
            },
        };
        let latency = start.elapsed().as_secs_f32() * 1000.0;
        
        // 5. Return response with metrics
        Ok(InferenceResponse {
            request_id,
            output,
            metrics: InferenceMetrics {
                total_latency_ms: latency,
                tokens_per_second: self.calculate_tps(&output, latency),
                nodes_used: assigned.len(),
                coordination_overhead_ms: partitioned.coordination_overhead_ms,
            },
        })
    }
}
```

**Execution Flow**:
```
User request: "Explain quantum computing"
    ↓
1. Discover 4 nodes (laptop, phone, desktop, friend's device)
2. Partition LLaMA-70B across nodes (LayerWise)
3. Node 1: Layers 0-15 → intermediate output
4. Node 2: Layers 16-31 → intermediate output
5. Node 3: Layers 32-47 → intermediate output
6. Node 4: Layers 48-63 → final output
7. Stream tokens back to user
    ↓
Response: "Quantum computing is..." (120ms latency, 85 tokens/sec)
```

### Edge Cloud Pooling (`distributed/edge_cloud.rs`)

```rust
pub struct EdgeCloudPool {
    pools: HashMap<String, ResourcePool>,
}

pub struct ResourcePool {
    name: String,
    nodes: Vec<ComputeNode>,
    policy: PoolPolicy,
    capacity: PoolCapacity,
}

pub struct PoolPolicy {
    max_nodes: usize,
    min_nodes: usize,
    auto_scale: bool,
    priority: PoolPriority,
    workload_type: WorkloadType,
}

pub enum PoolPriority {
    Low,
    Medium,
    High,
    Critical,
}

pub enum WorkloadType {
    Inference,          // AI inference
    Training,           // Model training
    DataProcessing,     // ETL, analytics
    Rendering,          // 3D rendering
    Gaming,             // Real-time gaming
    General,            // Mixed workloads
}

pub struct PoolCapacity {
    total_cpu_cores: u32,
    total_gpu_memory_gb: f32,
    total_ram_gb: f32,
    available_cpu_cores: u32,
    available_gpu_memory_gb: f32,
    available_ram_gb: f32,
}

pub enum NodeSelectionStrategy {
    RoundRobin,     // Rotate through nodes
    LeastLoaded,    // Pick least busy
    LowestLatency,  // Pick fastest
    MostCapable,    // Pick most powerful
    Random,         // Random selection
}

impl EdgeCloudPool {
    pub async fn allocate(&mut self, 
        pool_name: &str,
        requirements: &ComputeRequirements,
        strategy: NodeSelectionStrategy,
    ) -> Result<Vec<ComputeNode>> {
        let pool = self.pools.get_mut(pool_name)
            .ok_or("Pool not found")?;
        
        // 1. Check if enough capacity
        if !pool.has_capacity(requirements) {
            if pool.policy.auto_scale {
                self.scale_up(pool_name).await?;
            } else {
                return Err("Insufficient capacity");
            }
        }
        
        // 2. Select nodes based on strategy
        let selected = match strategy {
            NodeSelectionStrategy::LeastLoaded => {
                pool.nodes.iter()
                    .filter(|n| n.meets_requirements(requirements))
                    .min_by_key(|n| n.resources.cpu_usage as u32)
            },
            NodeSelectionStrategy::LowestLatency => {
                pool.nodes.iter()
                    .filter(|n| n.meets_requirements(requirements))
                    .min_by_key(|n| n.location.latency_ms as u32)
            },
            // ... more strategies
        };
        
        // 3. Reserve resources
        if let Some(node) = selected {
            self.reserve_resources(node, requirements)?;
            Ok(vec![node.clone()])
        } else {
            Err("No suitable nodes found")
        }
    }
    
    pub async fn scale_up(&mut self, pool_name: &str) -> Result<()> {
        // Discover new nodes and add to pool
    }
    
    pub async fn scale_down(&mut self, pool_name: &str) -> Result<()> {
        // Remove underutilized nodes from pool
    }
}
```

**Auto-Scaling Logic**:
```
Pool utilization > 80% for 5 minutes → Scale up (add nodes)
Pool utilization < 20% for 10 minutes → Scale down (remove nodes)

Scale up:
  1. Discover new capable nodes
  2. Add to pool
  3. Redistribute workload

Scale down:
  1. Identify least-used nodes
  2. Drain workload to other nodes
  3. Remove from pool
```

**Example Pool Configuration**:
```rust
ResourcePool {
    name: "inference_pool",
    nodes: vec![laptop, phone, desktop],
    policy: PoolPolicy {
        max_nodes: 10,
        min_nodes: 2,
        auto_scale: true,
        priority: PoolPriority::High,
        workload_type: WorkloadType::Inference,
    },
}
```

**Tests**: 28 tests covering node protocol, partitioning, inference coordination, pooling

---

## Summary

Kāraṇa OS uses a **9-layer modular architecture with 8 cross-cutting systems** where:

1. **Hardware** provides raw sensor inputs
2. **Network** enables peer discovery & state sync
3. **Blockchain** records immutable transaction history
4. **Oracle** bridges AI decisions to blockchain operations
5. **Intelligence** fuses multimodal inputs & predicts user intent
6. **AI Engine** classifies intents & executes safely
7. **Interface** renders AR/voice/haptic outputs
8. **Applications** provide domain-specific functionality
9. **System Services** ensure reliability & security

**Cross-Cutting Systems** (Phases 46-52):
- **Resource Management**: Adaptive optimization for constrained hardware
- **Capability Architecture**: Decoupled layer interfaces with discovery
- **Event Bus**: Async pub/sub communication between layers
- **Resilience**: Graceful degradation and fault tolerance
- **Progressive UX**: Mainstream accessibility with hidden complexity
- **Privacy Management**: User-controlled data retention and protection
- **App Ecosystem**: Native app support with AR optimizations
- **Distributed Compute**: Edge cloud integration for large models

The **Monad** orchestrator runs a 30-second tick loop, updating all layers synchronously while maintaining thread-safe state through atomic operations and message passing. Cross-cutting systems span all layers, providing horizontal services that enhance every layer's capabilities.
