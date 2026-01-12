# Kāraṇa OS - Mermaid Architecture Diagrams

This document contains interactive Mermaid diagrams for the Kāraṇa OS architecture. These diagrams render natively in GitHub and VS Code with Mermaid support.

## Table of Contents
1. [System Architecture Diagram](#system-architecture-diagram)
2. [Data Flow Diagram](#data-flow-diagram)
3. [Sequence Diagram](#sequence-diagram)
4. [Component Diagram](#component-diagram)
5. [State Diagram](#state-diagram)
6. [Event Bus Architecture](#event-bus-architecture)
7. [Cross-Cutting Systems Integration](#cross-cutting-systems-integration)
8. [Layer Communication Flow](#layer-communication-flow)

---

## System Architecture Diagram

Complete 9-layer architecture showing all system components and their relationships.

```mermaid
graph TB
    subgraph UX["User Experience Layer"]
        Voice[Voice UI<br/>Whisper STT]
        Gesture[Gesture UI<br/>MediaPipe]
        Gaze[Gaze UI<br/>Eye Tracking]
        Haptic[Haptic Feedback]
        HUD[AR HUD<br/>Three.js/WebGL]
        Fusion[Multimodal Fusion<br/>500ms window]
    end

    subgraph L8["Layer 8: Applications"]
        Timer[Timer App]
        Nav[Navigation App]
        Social[Social App]
        Settings[Settings App]
        TabMgr[AR Tab Manager]
        AppRuntime[App Runtime Engine]
    end

    subgraph L7["Layer 7: Interface"]
        UICore[UI Core]
        Renderer[3D Renderer]
        InputMgr[Input Manager]
        AREngine[AR Engine]
    end

    subgraph L6["Layer 6: AI Engine"]
        NLU[NLU Engine<br/>Intent Classification]
        Dialogue[Dialogue Manager<br/>Context Tracking]
        Reasoning[Reasoning Engine<br/>Chain-of-Thought]
        ActionExec[Action Executor<br/>Tool Calling]
        Response[Response Generator<br/>Phi-3 Mini 3.8B]
    end

    subgraph L5["Layer 5: Intelligence"]
        CV[Computer Vision<br/>YOLOv8, CLIP, BLIP-2]
        Scene[Scene Understanding<br/>Segmentation]
        Spatial[Spatial Computing<br/>ORB-SLAM3]
        MultiModal[Multimodal Fusion<br/>Vision+Audio+IMU]
    end

    subgraph L4["Layer 4: Oracle Bridge"]
        IntentProc[Intent Processor<br/>AI→Blockchain]
        ToolReg[Tool Registry<br/>5 tools]
        ZKProof[ZK Proof Engine<br/>Groth16/PLONK]
        OracleMgr[Oracle Request Manager]
    end

    subgraph L3["Layer 3: Blockchain"]
        BlockCore[Blockchain Core<br/>42,891 blocks]
        TxPool[Transaction Pool<br/>100 TPS]
        Wallet[Wallet System<br/>Ed25519]
        State[State Management<br/>Merkle Patricia Trie]
        DAO[DAO Governance]
    end

    subgraph L2["Layer 2: P2P Network"]
        LibP2P[libp2p Node<br/>Ed25519 PeerId]
        Discovery[Peer Discovery<br/>mDNS+Kademlia DHT]
        GossipSub[GossipSub Messaging]
        BlockSync[Block Sync Protocol]
        ConnMgr[Connection Manager]
    end

    subgraph L1["Layer 1: Hardware Abstraction"]
        Camera[Camera Manager<br/>1280x720@30fps]
        Sensors[Sensor Fusion<br/>IMU+GPS+Mag]
        Audio[Audio Manager<br/>48kHz Stereo]
        Display[Display Manager<br/>1080p OLED]
        Power[Power Manager]
        HapticHW[Haptic Manager]
    end

    subgraph L9["Layer 9: System Services"]
        OTA[OTA Updates]
        Security[Security<br/>Auth+Encryption]
        Diagnostics[Diagnostics]
        Recovery[Recovery]
        PowerSvc[Power Service]
        Logger[System Logger]
    end

    subgraph CrossCut["Cross-Cutting Systems"]
        EventBus[Event Bus<br/>tokio::broadcast]
        AsyncOrch[Async Orchestrator<br/>Priority Scheduler]
        CapReg[Capability Registry<br/>Service Discovery]
        MonadOrch[Monad Orchestrator<br/>30s tick loop]
        ResourceCoord[Resource Coordinator<br/>Adaptive Management]
        Resilience[Resilience Coordinator<br/>Fault Tolerance]
    end

    %% User Experience connections
    Voice --> Fusion
    Gesture --> Fusion
    Gaze --> Fusion
    Haptic --> Fusion
    HUD --> Fusion
    Fusion --> AppRuntime

    %% Layer 8 connections
    Timer --> AppRuntime
    Nav --> AppRuntime
    Social --> AppRuntime
    Settings --> AppRuntime
    TabMgr --> AppRuntime
    AppRuntime --> UICore
    AppRuntime --> NLU

    %% Layer 7 connections
    UICore --> Renderer
    InputMgr --> UICore
    AREngine --> Renderer

    %% Layer 6 connections
    NLU --> Dialogue
    Dialogue --> Reasoning
    Reasoning --> ActionExec
    ActionExec --> Response
    ActionExec --> IntentProc

    %% Layer 5 connections
    CV --> Scene
    Scene --> Spatial
    Spatial --> MultiModal
    MultiModal --> NLU

    %% Layer 4 connections
    IntentProc --> ToolReg
    ToolReg --> ZKProof
    ZKProof --> OracleMgr
    OracleMgr --> BlockCore

    %% Layer 3 connections
    BlockCore --> TxPool
    TxPool --> Wallet
    Wallet --> State
    State --> DAO
    BlockCore --> GossipSub

    %% Layer 2 connections
    LibP2P --> Discovery
    Discovery --> GossipSub
    GossipSub --> BlockSync
    BlockSync --> ConnMgr

    %% Layer 1 connections
    Camera --> Sensors
    Sensors --> Audio
    Audio --> Display
    Display --> Power
    Power --> HapticHW
    Camera --> CV
    Sensors --> Spatial
    Audio --> NLU
    Display --> Renderer
    HapticHW --> Haptic

    %% Layer 9 connections
    OTA -.-> Logger
    Security -.-> Logger
    Diagnostics -.-> Logger
    Recovery -.-> Logger
    PowerSvc -.-> Logger

    %% Cross-cutting connections to all layers
    EventBus -.-> AppRuntime
    EventBus -.-> NLU
    EventBus -.-> CV
    EventBus -.-> IntentProc
    EventBus -.-> BlockCore
    EventBus -.-> LibP2P
    EventBus -.-> Camera

    AsyncOrch -.-> AppRuntime
    AsyncOrch -.-> NLU
    AsyncOrch -.-> CV

    MonadOrch -.-> AppRuntime
    MonadOrch -.-> NLU
    MonadOrch -.-> IntentProc
    MonadOrch -.-> BlockCore
    MonadOrch -.-> LibP2P
    MonadOrch -.-> Camera

    ResourceCoord -.-> NLU
    ResourceCoord -.-> CV
    ResourceCoord -.-> BlockCore

    Resilience -.-> AppRuntime
    Resilience -.-> NLU
    Resilience -.-> BlockCore

    style UX fill:#e1f5ff
    style L8 fill:#fff4e1
    style L7 fill:#ffe1f5
    style L6 fill:#e1ffe1
    style L5 fill:#f5e1ff
    style L4 fill:#ffe1e1
    style L3 fill:#e1ffe1
    style L2 fill:#ffe1f5
    style L1 fill:#f5ffe1
    style L9 fill:#e1e1ff
    style CrossCut fill:#ffffe1
```

---

## Data Flow Diagram

Detailed flow from voice input to action execution.

```mermaid
graph LR
    A[User Voice Input] -->|Audio Stream| B[Whisper STT<br/>48kHz Stereo]
    B -->|Transcribed Text| C[NLU Engine<br/>Intent Classification]
    C -->|Intent + Entities| D[Dialogue Manager<br/>Context Tracking]
    D -->|Contextualized Intent| E[Reasoning Engine<br/>Chain-of-Thought]
    E -->|Action Plan| F[Action Executor<br/>Tool Selection]
    
    F -->|launch_app| G1[App Runtime<br/>Launch Application]
    F -->|navigate| G2[Navigation System<br/>Route Planning]
    F -->|wallet| G3[Wallet Manager<br/>Transaction]
    F -->|create_task| G4[Task Manager<br/>Create Task]
    F -->|search| G5[Search Engine<br/>Web Query]
    
    F -->|Intent + Params| H[Intent Processor<br/>Oracle Bridge]
    H -->|Validate| I[ZK Proof Engine<br/>Generate Proof]
    I -->|Proof| J[Oracle Request Manager<br/>Queue Request]
    J -->|Oracle Request| K[Blockchain Core<br/>Execute Tool]
    K -->|Result| L[Transaction Pool<br/>Settle TX]
    L -->|Confirmation| M[Oracle Response]
    
    M -->|Tool Result| N[Response Generator<br/>Phi-3 Mini]
    G1 -->|Success| N
    G2 -->|Success| N
    G3 -->|Success| N
    G4 -->|Success| N
    G5 -->|Success| N
    
    N -->|Natural Language| O[TTS / UI Display]
    O -->|Speech + Visual| P[User Feedback]
    
    subgraph Timing["⏱️ Timing"]
        T1[STT: 200-300ms]
        T2[NLU: 50-100ms]
        T3[Dialogue: 20-50ms]
        T4[Reasoning: 100-200ms]
        T5[Tool Exec: 20-200ms]
        T6[Oracle: 200-500ms]
        T7[Response: 200-400ms]
        T8[Total: 1.1-1.5s]
    end
    
    style A fill:#e1f5ff
    style P fill:#e1ffe1
    style H fill:#ffe1e1
    style K fill:#e1ffe1
    style Timing fill:#ffffe1
```

---

## Sequence Diagram

Complete request/response flow for a voice command.

```mermaid
sequenceDiagram
    participant User
    participant Voice as Voice UI
    participant Whisper as Whisper STT
    participant NLU as NLU Engine
    participant Dialogue as Dialogue Manager
    participant Reasoning as Reasoning Engine
    participant ActionExec as Action Executor
    participant Oracle as Oracle Bridge
    participant ZKProof as ZK Proof Engine
    participant Blockchain
    participant ToolRegistry as Tool Registry
    participant App as Application
    participant Response as Response Generator
    participant TTS as Text-to-Speech

    User->>Voice: "Launch the timer app"
    Voice->>Whisper: Audio Stream (48kHz)
    Whisper->>Whisper: Transcribe (200-300ms)
    Whisper->>NLU: "Launch the timer app"
    
    NLU->>NLU: Extract Intent & Entities (50-100ms)
    Note over NLU: Intent: "launch_app"<br/>Entity: "timer"
    NLU->>Dialogue: Intent + Entities
    
    Dialogue->>Dialogue: Load Context (20-50ms)
    Note over Dialogue: Previous: None<br/>Session: Active
    Dialogue->>Reasoning: Contextualized Intent
    
    Reasoning->>Reasoning: Plan Actions (100-200ms)
    Note over Reasoning: Chain-of-Thought:<br/>1. Identify app<br/>2. Check permissions<br/>3. Execute launch
    Reasoning->>ActionExec: Action Plan
    
    ActionExec->>ToolRegistry: Query "launch_app" tool
    ToolRegistry-->>ActionExec: Tool Definition
    
    alt Direct Tool Execution
        ActionExec->>App: Launch Timer App
        App-->>ActionExec: Success
    else Oracle-Required Tool
        ActionExec->>Oracle: Intent + Params
        Oracle->>ZKProof: Generate Authorization Proof
        ZKProof-->>Oracle: Proof (Groth16)
        Oracle->>Blockchain: Submit Oracle Request
        Blockchain->>Blockchain: Execute Tool (200-500ms)
        Blockchain-->>Oracle: Tool Result
        Oracle-->>ActionExec: Result + Confirmation
    end
    
    ActionExec->>Response: Tool Result
    Response->>Response: Generate Response (200-400ms)
    Note over Response: Phi-3 Mini (3.8B)<br/>Natural Language
    Response->>TTS: "Timer app launched successfully"
    TTS->>User: Speech Output
    
    Note over User,TTS: Total Time: 1.1-1.5 seconds
```

---

## Component Diagram

Detailed breakdown of major subsystems and their interfaces.

```mermaid
graph TB
    subgraph AISubsystem["AI/ML Subsystem"]
        direction TB
        NLUComp[NLU Component<br/>───────────<br/>+ classify_intent()<br/>+ extract_entities()<br/>+ compute_confidence()]
        DialogueComp[Dialogue Component<br/>───────────<br/>+ track_context()<br/>+ resolve_references()<br/>+ manage_session()]
        ReasoningComp[Reasoning Component<br/>───────────<br/>+ chain_of_thought()<br/>+ plan_actions()<br/>+ visual_reasoning()]
        VisionComp[Vision Component<br/>───────────<br/>+ detect_objects()<br/>+ segment_scene()<br/>+ track_objects()]
        
        NLUComp --> DialogueComp
        DialogueComp --> ReasoningComp
        VisionComp --> ReasoningComp
    end

    subgraph BlockchainSubsystem["Blockchain Subsystem"]
        direction TB
        ConsensusComp[Consensus Component<br/>───────────<br/>+ propose_block()<br/>+ validate_block()<br/>+ finalize_block()]
        TxComp[Transaction Component<br/>───────────<br/>+ create_tx()<br/>+ validate_tx()<br/>+ execute_tx()]
        StateComp[State Component<br/>───────────<br/>+ read_state()<br/>+ write_state()<br/>+ compute_root()]
        WalletComp[Wallet Component<br/>───────────<br/>+ sign_tx()<br/>+ verify_signature()<br/>+ manage_keys()]
        
        TxComp --> ConsensusComp
        ConsensusComp --> StateComp
        WalletComp --> TxComp
    end

    subgraph NetworkSubsystem["P2P Network Subsystem"]
        direction TB
        P2PComp[P2P Component<br/>───────────<br/>+ connect_peer()<br/>+ disconnect_peer()<br/>+ send_message()]
        DiscoveryComp[Discovery Component<br/>───────────<br/>+ discover_peers()<br/>+ announce_self()<br/>+ query_dht()]
        GossipComp[Gossip Component<br/>───────────<br/>+ publish()<br/>+ subscribe()<br/>+ validate_msg()]
        SyncComp[Sync Component<br/>───────────<br/>+ sync_blocks()<br/>+ request_block()<br/>+ verify_chain()]
        
        DiscoveryComp --> P2PComp
        P2PComp --> GossipComp
        GossipComp --> SyncComp
    end

    subgraph OracleSubsystem["Oracle Bridge Subsystem"]
        direction TB
        IntentComp[Intent Component<br/>───────────<br/>+ parse_intent()<br/>+ validate_params()<br/>+ map_to_tool()]
        ZKComp[ZK Proof Component<br/>───────────<br/>+ generate_proof()<br/>+ verify_proof()<br/>+ setup_circuit()]
        ToolComp[Tool Component<br/>───────────<br/>+ register_tool()<br/>+ execute_tool()<br/>+ get_definition()]
        OracleComp[Oracle Manager<br/>───────────<br/>+ queue_request()<br/>+ settle_request()<br/>+ timeout_handler()]
        
        IntentComp --> ToolComp
        ToolComp --> ZKComp
        ZKComp --> OracleComp
    end

    subgraph HardwareSubsystem["Hardware Abstraction Subsystem"]
        direction TB
        CameraComp[Camera Component<br/>───────────<br/>+ capture_frame()<br/>+ set_exposure()<br/>+ set_white_balance()]
        SensorComp[Sensor Component<br/>───────────<br/>+ read_imu()<br/>+ read_gps()<br/>+ read_magnetometer()]
        AudioComp[Audio Component<br/>───────────<br/>+ capture_audio()<br/>+ play_audio()<br/>+ spatial_audio()]
        DisplayComp[Display Component<br/>───────────<br/>+ render_frame()<br/>+ set_brightness()<br/>+ get_resolution()]
        
        CameraComp -.->|Frame Data| VisionComp
        SensorComp -.->|IMU Data| VisionComp
        AudioComp -.->|Audio Data| NLUComp
    end

    subgraph CoreInfrastructure["Core Infrastructure"]
        direction TB
        EventBusComp[Event Bus<br/>───────────<br/>+ publish()<br/>+ subscribe()<br/>+ unsubscribe()]
        OrchestratorComp[Orchestrator<br/>───────────<br/>+ schedule()<br/>+ execute()<br/>+ cancel()]
        ResourceComp[Resource Manager<br/>───────────<br/>+ allocate()<br/>+ deallocate()<br/>+ monitor()]
        ResilienceComp[Resilience Manager<br/>───────────<br/>+ health_check()<br/>+ circuit_break()<br/>+ recover()]
    end

    %% Cross-component connections
    ReasoningComp -->|Intent| IntentComp
    OracleComp -->|Oracle Request| TxComp
    TxComp -->|Transaction| GossipComp
    SyncComp -->|Blocks| ConsensusComp

    %% Infrastructure connections
    EventBusComp -.->|Events| NLUComp
    EventBusComp -.->|Events| ConsensusComp
    EventBusComp -.->|Events| P2PComp
    EventBusComp -.->|Events| IntentComp
    
    OrchestratorComp -.->|Schedule| NLUComp
    OrchestratorComp -.->|Schedule| ConsensusComp
    OrchestratorComp -.->|Schedule| CameraComp
    
    ResourceComp -.->|Monitor| NLUComp
    ResourceComp -.->|Monitor| VisionComp
    ResourceComp -.->|Monitor| ConsensusComp
    
    ResilienceComp -.->|Health| NLUComp
    ResilienceComp -.->|Health| ConsensusComp
    ResilienceComp -.->|Health| P2PComp

    style AISubsystem fill:#e1ffe1
    style BlockchainSubsystem fill:#ffe1f5
    style NetworkSubsystem fill:#e1f5ff
    style OracleSubsystem fill:#ffe1e1
    style HardwareSubsystem fill:#f5ffe1
    style CoreInfrastructure fill:#ffffe1
```

---

## State Diagram

Layer lifecycle and health states with transitions.

```mermaid
stateDiagram-v2
    [*] --> Uninitialized
    
    Uninitialized --> Initializing: start()
    Initializing --> Healthy: init_success()
    Initializing --> Failed: init_failure()
    
    Healthy --> Degraded: performance_drop()
    Healthy --> Recovering: error_detected()
    Healthy --> Stopped: shutdown()
    
    Degraded --> Healthy: performance_restored()
    Degraded --> Recovering: critical_error()
    Degraded --> MinimalMode: resource_critical()
    
    Recovering --> Healthy: recovery_success()
    Recovering --> Failed: recovery_failure()
    Recovering --> MinimalMode: repeated_failures()
    
    MinimalMode --> Recovering: resources_available()
    MinimalMode --> Failed: minimal_mode_failure()
    MinimalMode --> Healthy: full_recovery()
    
    Failed --> Recovering: retry()
    Failed --> Stopped: permanent_failure()
    
    Stopped --> [*]
    
    state Healthy {
        [*] --> Active
        Active --> Idle: low_activity()
        Idle --> Active: high_activity()
        Active --> Processing: task_received()
        Processing --> Active: task_completed()
    }
    
    state Degraded {
        [*] --> ReducedPerformance
        ReducedPerformance --> ThrottledMode: high_load()
        ThrottledMode --> ReducedPerformance: load_decreased()
    }
    
    state Recovering {
        [*] --> Diagnosing
        Diagnosing --> Repairing: issue_identified()
        Repairing --> Testing: repair_complete()
        Testing --> Diagnosing: test_failed()
        Testing --> [*]: test_passed()
    }
    
    state MinimalMode {
        [*] --> CoreOnly
        CoreOnly --> EssentialServices: stabilized()
        EssentialServices --> CoreOnly: instability_detected()
        note right of CoreOnly
            <10MB RAM
            Critical functions only
            No AI/ML
            Light blockchain sync
        end note
    }
    
    note right of Healthy
        Normal Operation
        All features active
        Performance targets met
        Resources within limits
    end note
    
    note right of Degraded
        Partial functionality
        Performance below target
        Resources constrained
        Non-critical features disabled
    end note
    
    note right of Recovering
        Self-healing active
        Circuit breakers engaged
        Automatic diagnostics
        Attempting restoration
    end note
    
    note right of Failed
        Layer unresponsive
        Cannot perform functions
        Requires manual intervention
        System may continue without
    end note
```

---

## Event Bus Architecture

Detailed view of the Event Bus message routing and prioritization.

```mermaid
graph TB
    subgraph Publishers["Event Publishers"]
        direction LR
        P1[Hardware Layer<br/>Camera, Sensors]
        P2[Network Layer<br/>P2P Messages]
        P3[Blockchain Layer<br/>Blocks, Transactions]
        P4[AI Layer<br/>Intents, Results]
        P5[Applications<br/>User Actions]
    end

    subgraph EventBus["Event Bus Core"]
        direction TB
        Router[Event Router<br/>Category-based]
        PriorityQ[Priority Queue<br/>Critical→Normal→Low]
        Dispatcher[Event Dispatcher<br/>tokio::broadcast]
        
        Router --> PriorityQ
        PriorityQ --> Dispatcher
    end

    subgraph EventTypes["Event Categories"]
        direction LR
        E1[HARDWARE_EVENT<br/>Camera frame, Sensor data]
        E2[NETWORK_EVENT<br/>Peer connected, Block received]
        E3[BLOCKCHAIN_EVENT<br/>Block finalized, TX confirmed]
        E4[AI_EVENT<br/>Intent classified, Action completed]
        E5[APP_EVENT<br/>App launched, Tab switched]
        E6[SYSTEM_EVENT<br/>Resource alert, Error occurred]
        E7[ORACLE_EVENT<br/>Request queued, Proof generated]
    end

    subgraph Subscribers["Event Subscribers"]
        direction LR
        S1[AI Engine<br/>Vision, NLU]
        S2[Blockchain Core<br/>Consensus, State]
        S3[Oracle Bridge<br/>Intent Processor]
        S4[Applications<br/>UI Updates]
        S5[System Services<br/>Diagnostics, Logger]
    end

    subgraph Priority["Priority Handling"]
        Critical[Critical Priority<br/>────────<br/>System errors<br/>Security events<br/>Hardware failures<br/>Delivery: <1ms]
        Normal[Normal Priority<br/>────────<br/>User intents<br/>Block updates<br/>App events<br/>Delivery: <5ms]
        Low[Low Priority<br/>────────<br/>Diagnostics<br/>Analytics<br/>Metrics<br/>Delivery: <50ms]
    end

    %% Publisher connections
    P1 -->|Publish| Router
    P2 -->|Publish| Router
    P3 -->|Publish| Router
    P4 -->|Publish| Router
    P5 -->|Publish| Router

    %% Router to Event Types
    Router -.->|Route| E1
    Router -.->|Route| E2
    Router -.->|Route| E3
    Router -.->|Route| E4
    Router -.->|Route| E5
    Router -.->|Route| E6
    Router -.->|Route| E7

    %% Priority assignment
    E6 --> Critical
    E7 --> Critical
    E3 --> Normal
    E4 --> Normal
    E5 --> Normal
    E1 --> Low
    E2 --> Low

    %% Dispatcher to Subscribers
    Dispatcher -->|Subscribe| S1
    Dispatcher -->|Subscribe| S2
    Dispatcher -->|Subscribe| S3
    Dispatcher -->|Subscribe| S4
    Dispatcher -->|Subscribe| S5

    %% Metrics
    Metrics[📊 Metrics<br/>────────<br/>100+ Event Types<br/>1000+ msgs/sec<br/><5ms avg latency<br/>99.9% delivery rate<br/>tokio::sync::broadcast]

    style Publishers fill:#e1f5ff
    style EventBus fill:#ffe1e1
    style EventTypes fill:#e1ffe1
    style Subscribers fill:#f5e1ff
    style Priority fill:#ffffe1
    style Metrics fill:#ffe1f5
```

---

## Cross-Cutting Systems Integration

How core infrastructure systems interact with all layers.

```mermaid
graph TB
    subgraph Layers["9-Layer Stack"]
        direction TB
        L1[Layer 1: Hardware]
        L2[Layer 2: Network]
        L3[Layer 3: Blockchain]
        L4[Layer 4: Oracle]
        L5[Layer 5: Intelligence]
        L6[Layer 6: AI Engine]
        L7[Layer 7: Interface]
        L8[Layer 8: Applications]
        L9[Layer 9: System Services]
        
        L1 -.-> L2
        L2 -.-> L3
        L3 -.-> L4
        L4 -.-> L5
        L5 -.-> L6
        L6 -.-> L7
        L7 -.-> L8
    end

    subgraph EventBusSys["Event Bus System"]
        EB[Event Bus]
        EB_Pub[Publishers]
        EB_Sub[Subscribers]
        EB_Route[Router]
        
        EB_Pub --> EB
        EB --> EB_Route
        EB_Route --> EB_Sub
    end

    subgraph AsyncOrchSys["Async Orchestrator"]
        AO[Orchestrator Core]
        AO_Sched[Priority Scheduler<br/>BinaryHeap]
        AO_Exec[Task Executor<br/>tokio::spawn]
        AO_Timeout[Deadline Enforcer]
        
        AO_Sched --> AO
        AO --> AO_Exec
        AO --> AO_Timeout
    end

    subgraph CapRegSys["Capability Registry"]
        CR[Registry Core]
        CR_Disc[Service Discovery]
        CR_Map[Capability Map<br/>40+ capabilities]
        CR_Swap[Dynamic Swapping]
        
        CR_Disc --> CR
        CR --> CR_Map
        CR --> CR_Swap
    end

    subgraph MonadSys["Monad Orchestrator"]
        MO[Monad Core]
        MO_Tick[tick() Loop<br/>30s blocks]
        MO_Coord[State Coordination<br/>Arc<Mutex<T>>]
        MO_Seq[Sequential Updates]
        
        MO_Tick --> MO
        MO --> MO_Coord
        MO_Coord --> MO_Seq
    end

    subgraph ResourceSys["Resource Coordinator"]
        RC[Coordinator Core]
        RC_Mon[Monitor<br/>CPU/GPU/RAM/Battery]
        RC_Prof[AI Profiles<br/>5 levels]
        RC_Mode[Ledger Modes<br/>Full/Light/Minimal]
        RC_Pred[Predictive<br/>5min forecast]
        
        RC_Mon --> RC
        RC --> RC_Prof
        RC --> RC_Mode
        RC --> RC_Pred
    end

    subgraph ResilienceSys["Resilience Coordinator"]
        RS[Resilience Core]
        RS_Health[Health Monitor]
        RS_CB[Circuit Breaker<br/>9 breakers]
        RS_Min[Minimal Mode<br/><10MB fallback]
        RS_Rec[Auto-Recovery]
        
        RS_Health --> RS
        RS --> RS_CB
        RS --> RS_Min
        RS --> RS_Rec
    end

    %% Event Bus connections
    EB -.->|Events| L1
    EB -.->|Events| L2
    EB -.->|Events| L3
    EB -.->|Events| L4
    EB -.->|Events| L5
    EB -.->|Events| L6
    EB -.->|Events| L7
    EB -.->|Events| L8
    EB -.->|Events| L9

    %% Async Orchestrator connections
    AO -.->|Schedule| L1
    AO -.->|Schedule| L5
    AO -.->|Schedule| L6
    AO -.->|Schedule| L8

    %% Capability Registry connections
    CR -.->|Discover| L1
    CR -.->|Discover| L2
    CR -.->|Discover| L5
    CR -.->|Discover| L6
    CR -.->|Discover| L8

    %% Monad Orchestrator connections
    MO -.->|Coordinate| L1
    MO -.->|Coordinate| L2
    MO -.->|Coordinate| L3
    MO -.->|Coordinate| L4
    MO -.->|Coordinate| L5
    MO -.->|Coordinate| L6
    MO -.->|Coordinate| L7
    MO -.->|Coordinate| L8
    MO -.->|Coordinate| L9

    %% Resource Coordinator connections
    RC -.->|Adapt| L1
    RC -.->|Adapt| L3
    RC -.->|Adapt| L5
    RC -.->|Adapt| L6

    %% Resilience Coordinator connections
    RS -.->|Monitor| L1
    RS -.->|Monitor| L2
    RS -.->|Monitor| L3
    RS -.->|Monitor| L6
    RS -.->|Monitor| L8

    style Layers fill:#e1f5ff
    style EventBusSys fill:#ffe1e1
    style AsyncOrchSys fill:#e1ffe1
    style CapRegSys fill:#f5e1ff
    style MonadSys fill:#ffffe1
    style ResourceSys fill:#ffe1f5
    style ResilienceSys fill:#f5ffe1
```

---

## Layer Communication Flow

Message passing patterns between layers.

```mermaid
sequenceDiagram
    participant Hardware as L1: Hardware
    participant Network as L2: Network
    participant Blockchain as L3: Blockchain
    participant Oracle as L4: Oracle
    participant Intelligence as L5: Intelligence
    participant AI as L6: AI Engine
    participant Interface as L7: Interface
    participant Apps as L8: Applications
    participant EventBus as Event Bus
    participant Orchestrator as Monad Orchestrator

    Note over Orchestrator: 30-second tick() begins

    %% Hardware Layer Updates
    Orchestrator->>Hardware: tick()
    Hardware->>Hardware: Read sensors, capture frames
    Hardware->>EventBus: HARDWARE_EVENT (camera frame)
    EventBus->>Intelligence: Forward event
    Hardware-->>Orchestrator: Status: Healthy

    %% Network Layer Sync
    Orchestrator->>Network: tick()
    Network->>Network: Sync peers, process messages
    Network->>EventBus: NETWORK_EVENT (block received)
    EventBus->>Blockchain: Forward event
    Network-->>Orchestrator: Status: Healthy

    %% Blockchain Layer Consensus
    Orchestrator->>Blockchain: tick()
    Blockchain->>Blockchain: Process transactions, finalize block
    Blockchain->>EventBus: BLOCKCHAIN_EVENT (block finalized)
    EventBus->>Oracle: Forward event
    EventBus->>Apps: Forward event
    Blockchain-->>Orchestrator: Status: Healthy

    %% Oracle Layer Processing
    Orchestrator->>Oracle: tick()
    Oracle->>Oracle: Process queued requests
    Oracle->>Blockchain: Submit oracle transactions
    Oracle->>EventBus: ORACLE_EVENT (request settled)
    EventBus->>AI: Forward event
    Oracle-->>Orchestrator: Status: Healthy

    %% Intelligence Layer Processing
    Orchestrator->>Intelligence: tick()
    Intelligence->>Intelligence: Process vision, update spatial map
    Intelligence->>EventBus: AI_EVENT (objects detected)
    EventBus->>AI: Forward event
    EventBus->>Interface: Forward event
    Intelligence-->>Orchestrator: Status: Healthy

    %% AI Layer Inference
    Orchestrator->>AI: tick()
    AI->>AI: Process pending intents, execute tools
    AI->>EventBus: AI_EVENT (intent processed)
    AI->>Oracle: Intent requires blockchain tool
    EventBus->>Apps: Forward event
    EventBus->>Interface: Forward event
    AI-->>Orchestrator: Status: Healthy

    %% Interface Layer Rendering
    Orchestrator->>Interface: tick()
    Interface->>Interface: Render AR scene, update HUD
    Interface->>EventBus: UI_EVENT (frame rendered)
    Interface-->>Orchestrator: Status: Healthy

    %% Application Layer Updates
    Orchestrator->>Apps: tick()
    Apps->>Apps: Update app states, process user input
    Apps->>AI: User intent from app
    Apps->>EventBus: APP_EVENT (tab switched)
    EventBus->>Interface: Forward event
    Apps-->>Orchestrator: Status: Healthy

    Note over Orchestrator: tick() complete - 30s elapsed
    Note over Orchestrator: All layers healthy, cycle repeats

    %% Alternative path: Layer failure
    alt Layer Failure Detected
        Hardware->>EventBus: SYSTEM_EVENT (error)
        EventBus->>Orchestrator: Forward critical event
        Orchestrator->>Orchestrator: Trigger resilience recovery
        Note over Orchestrator: Circuit breaker activated<br/>Minimal mode fallback<br/>Auto-recovery initiated
    end
```

---

## Performance Metrics Dashboard

```mermaid
graph TB
    subgraph Latency["⏱️ Latency Metrics"]
        L1[Voice → Action<br/>1.1-1.5 seconds]
        L2[Event Bus Delivery<br/><5ms average]
        L3[Object Detection<br/>25ms per frame]
        L4[Block Finality<br/>12 seconds]
        L5[Tool Execution<br/>20-200ms]
        L6[SLAM Tracking<br/>8ms per frame]
    end

    subgraph Throughput["📊 Throughput"]
        T1[Blockchain TPS<br/>100 transactions/sec]
        T2[Event Bus<br/>1000+ msgs/sec]
        T3[AR Rendering<br/>60-90 FPS]
        T4[Camera Feed<br/>30 FPS @ 720p]
        T5[Audio Processing<br/>48kHz Stereo]
    end

    subgraph Resources["💾 Resource Usage"]
        R1[RAM Idle<br/>1.5GB]
        R2[RAM Active<br/>2.2GB]
        R3[RAM Peak<br/>3.2GB]
        R4[Blockchain State<br/>~500MB]
        R5[AI Models<br/>~800MB]
        R6[Minimal Mode<br/><10MB]
    end

    subgraph Reliability["✅ Reliability"]
        REL1[Oracle Success<br/>98%+]
        REL2[System Uptime<br/>99.9%]
        REL3[P2P Connection<br/>95%+]
        REL4[Block Production<br/>99.8%]
        REL5[Test Coverage<br/>98%+]
    end

    subgraph Scale["📈 Scalability"]
        S1[Connected Peers<br/>50-200]
        S2[AR Anchors<br/>1000+ persistent]
        S3[Blockchain Height<br/>42,891 blocks]
        S4[Concurrent Tabs<br/>10 apps]
        S5[Conversation Memory<br/>2048 tokens]
    end

    style Latency fill:#e1ffe1
    style Throughput fill:#e1f5ff
    style Resources fill:#ffe1f5
    style Reliability fill:#ffffe1
    style Scale fill:#f5e1ff
```

---

## Summary

These Mermaid diagrams provide interactive, professional visualizations of the Kāraṇa OS architecture:

1. **System Architecture**: Complete 9-layer overview with all connections
2. **Data Flow**: Detailed flow from voice input to action execution
3. **Sequence Diagram**: Step-by-step request/response with timing
4. **Component Diagram**: Subsystem interfaces and dependencies
5. **State Diagram**: Layer lifecycle and health state transitions
6. **Event Bus**: Message routing and prioritization system
7. **Cross-Cutting Integration**: Infrastructure systems across all layers
8. **Layer Communication**: Message passing patterns in tick() cycle
9. **Performance Metrics**: Key performance indicators dashboard

All diagrams render natively in GitHub, VS Code, and other Mermaid-compatible viewers. They can be clicked, zoomed, and exported as PNG/SVG.
