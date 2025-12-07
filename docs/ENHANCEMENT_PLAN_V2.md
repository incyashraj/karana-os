# Kāraṇa OS Enhancement Plan V2 - Implementation Guide

## Overview

This document provides a comprehensive guide to the architectural improvements implemented in Enhancement Plan V2 (Phases 54-63).

## Phase 54: Monad Decoupling & Event-Driven Architecture

### Components
- **Capability System** (`capabilities/mod.rs`): Generic capability traits for layer composition
- **Event-Driven Orchestrator** (`orchestrator/mod.rs`): Async event-driven coordination

### Key Features
- Generic capability system replacing monad-specific traits
- Async event-driven architecture with publish/subscribe
- Event routing and handler management
- Layer independence through capability composition

### Usage Example
```rust
use karana_core::capabilities::{Capability, CapabilityContext};
use karana_core::orchestrator::{Orchestrator, Event};

// Define custom capability
struct MyCapability;

impl Capability for MyCapability {
    fn name(&self) -> &str { "my_capability" }
    async fn execute(&self, ctx: &CapabilityContext) -> Result<()> {
        // Implementation
        Ok(())
    }
}

// Use orchestrator
let mut orch = Orchestrator::new();
orch.subscribe("my_event", |event| async move {
    println!("Received: {:?}", event);
}).await;

orch.publish(Event::new("my_event", vec![])).await?;
```

## Phase 55: Model Optimization & Distributed Computing

### Components
- **Model Distillation** (`ai/distillation/mod.rs`): Model quantization and optimization
- **Workload Splitter** (`distributed/workload_splitter.rs`): Intelligent compute placement
- **Thermal Governor** (`resource/thermal.rs`): Predictive thermal management
- **Intent Scheduler** (`ai/intent_scheduler.rs`): Context-aware AI task scheduling

### Key Features
- Quantization levels: FP32, FP16, INT8, INT4, Binary
- Device placement: OnHead, BeltWorn, Phone, Cloud
- Thermal prediction with 30s horizon
- Interaction states: Active, Idle, Passive, Background, Sleep

### Usage Example
```rust
use karana_core::ai::distillation::{ModelOptimizer, QuantizationLevel};
use karana_core::distributed::workload_splitter::{WorkloadSplitter, ComputeNode};

// Quantize model
let optimizer = ModelOptimizer::new();
let quantized = optimizer.quantize(model_data, QuantizationLevel::INT8)?;

// Place workload
let splitter = WorkloadSplitter::new();
splitter.add_node(ComputeNode::BeltWorn {
    compute_tflops: 2.0,
    memory_gb: 8.0,
    // ...
});

let placement = splitter.place_workload(workload).await?;
```

## Phase 56: Chaos Engineering & Fault Injection

### Components
- **Fault Injector** (`testing/fault_injection/mod.rs`): Controlled failure simulation
- **Chaos Tester** (`testing/chaos/mod.rs`): Automated chaos scenarios
- **Recovery Validator** (`testing/recovery/mod.rs`): Post-failure validation
- **Reliability Tester** (`testing/mod.rs`): Integrated testing coordinator

### Key Features
- 12 fault types (Camera, Network, Ledger, Memory, etc.)
- 5 predefined scenarios (Camera Failure, Network Partition, etc.)
- Recovery time tracking
- Comprehensive reliability scoring

### Usage Example
```rust
use karana_core::testing::{ReliabilityTester, TestConfig};

let config = TestConfig {
    chaos_scenarios: vec!["camera_failure", "network_partition"],
    validation_checks: vec!["responsiveness", "data_integrity"],
    max_recovery_time_ms: 5000,
};

let tester = ReliabilityTester::new(config);
let report = tester.run_tests().await?;
println!("Reliability score: {:.2}", report.score);
```

## Phase 57: Feature Flag System

### Components
- **Feature Flags** (`feature_flags/mod.rs`): Runtime feature toggling and build profiles

### Key Features
- 4 build profiles: Minimal (256MB), Standard (512MB), Full (1024MB), Development (2048MB)
- 13 default features with dependency management
- Runtime feature overrides
- Memory budget enforcement

### Usage Example
```rust
use karana_core::feature_flags::{FeatureFlagManager, BuildProfile};

let mut manager = FeatureFlagManager::with_profile(BuildProfile::Standard);

if manager.is_enabled("voice_commands") {
    // Enable voice commands
}

manager.set_override("advanced_ai", true)?;
```

## Phase 58: Progressive Disclosure UX

### Components
- **Progressive UX** (`ux/progressive.rs`): Simplified UX layer

### Key Features
- 4 UX levels: Beginner, Intermediate, Advanced, Expert
- Smart defaults system
- Onboarding tutorial with 4 steps
- Feature visibility control

### Usage Example
```rust
use karana_core::ux::{ProgressiveUX, UXLevel};

let mut ux = ProgressiveUX::new();
ux.set_level(UXLevel::Intermediate);

if ux.is_feature_visible("blockchain_wallet") {
    // Show blockchain features
}

let next_step = ux.next_onboarding_step();
```

## Phase 59: Security Defaults

### Components
- **Security Manager** (`security/defaults.rs`): Permission presets and spending guards

### Key Features
- 4 security presets: Paranoid, High, Balanced, Relaxed
- 8 permission types
- Spending limits (daily + per-transaction)
- Transaction cooldown
- Recovery configuration

### Usage Example
```rust
use karana_core::security::{SecurityDefaults, SecurityPreset, Permission};

let mut manager = SecurityDefaults::new(SecurityPreset::Balanced);

if manager.has_permission(Permission::Camera) {
    // Access camera
}

manager.record_transaction(5.0)?; // Check spending limits
```

## Phase 60: Privacy Dashboard (Enhanced)

### Components
- **Privacy System** (`privacy/mod.rs`): Enhanced with dashboard capabilities

### Key Features
- Data retention policies per category
- Ephemeral mode with auto-triggers
- Privacy metrics tracking
- Category-based data management

### Usage Example
```rust
use karana_core::privacy::{PrivacyDashboard, DataCategory};

let mut dashboard = PrivacyDashboard::new();

dashboard.add_data(DataCategory::Camera, "img1".to_string(), 1_000_000);
dashboard.enable_ephemeral_mode();

let deleted = dashboard.clean_expired_data().await?;
```

## Phase 61: Intent API

### Components
- **Intent API** (`api/intent.rs`): External app integration

### Key Features
- App registration with API keys
- 7 intent types (Capture, DisplayAR, Voice, Navigate, etc.)
- Intent status tracking
- Permission-based access control

### Usage Example
```rust
use karana_core::api::intent::{IntentAPI, Intent, CaptureMode};

let api = IntentAPI::new();
let registration = api.register_app("MyApp".to_string(), vec!["camera".to_string()]).await?;

let intent_id = api.submit_intent(
    &registration.api_key,
    Intent::Capture { mode: CaptureMode::Photo },
).await?;

let status = api.get_status(&intent_id).await?;
```

## Phase 62: Interoperability

### Components
- **Companion Protocol** (`interop/mod.rs`): Cross-device synchronization
- **Desktop Bridge** (`interop/mod.rs`): Desktop integration

### Key Features
- Device pairing with 6-digit codes
- 5 sync message types
- Multi-device support (Glasses, Phone, Tablet, Desktop, Watch)
- File sync and notification sync

### Usage Example
```rust
use karana_core::interop::{CompanionProtocol, DesktopBridge, DeviceInfo, DeviceType};

let protocol = Arc::new(CompanionProtocol::new());

let device = DeviceInfo {
    device_id: "phone_1".to_string(),
    device_type: DeviceType::Phone,
    name: "My Phone".to_string(),
    os_version: "1.0".to_string(),
    capabilities: vec!["sync".to_string()],
};

let code = protocol.start_pairing(device).await?;
protocol.complete_pairing("phone_1", &code).await?;

// Use desktop bridge
let mut bridge = DesktopBridge::new(protocol);
bridge.connect("desktop_1".to_string()).await?;
bridge.send_file("test.txt".to_string(), data).await?;
```

## Migration Guide

### From Monad System to Capability System

**Before (Monad-based):**
```rust
impl LayerMonad for MyLayer {
    fn apply(&self, context: &Context) -> Result<()> {
        // ...
    }
}
```

**After (Capability-based):**
```rust
impl Capability for MyLayer {
    fn name(&self) -> &str { "my_layer" }
    async fn execute(&self, ctx: &CapabilityContext) -> Result<()> {
        // ...
    }
}
```

### Adding Feature Flags

To add feature flag control to your code:

```rust
use karana_core::feature_flags::FeatureFlagManager;

let manager = FeatureFlagManager::with_profile(BuildProfile::Standard);

if manager.is_enabled("your_feature") {
    // Feature-gated code
}
```

### Implementing Chaos Testing

To add chaos testing to your module:

```rust
use karana_core::testing::fault_injection::{FaultInjector, FaultType};

let injector = FaultInjector::new();
injector.inject(FaultType::Camera, 5)?; // Severity 5

// Your code that should handle camera failures
```

## Performance Considerations

### Model Optimization
- INT8 quantization: ~75% size reduction, 4x speedup, <1% accuracy loss
- INT4 quantization: ~87.5% size reduction, 8x speedup, ~2% accuracy loss

### Thermal Management
- Prediction horizon: 30 seconds
- Throttling temperature: 38°C
- Emergency offload: 43°C

### Workload Distribution
- OnHead: 0.5 TFLOPS, 4GB RAM (lightweight tasks only)
- BeltWorn: 2.0 TFLOPS, 8GB RAM (main compute)
- Cloud: Unlimited (heavy workloads)

## Testing

All phases include comprehensive unit tests. Run tests with:

```bash
cargo test --lib
```

To run chaos engineering tests (disabled by default):

```bash
cargo test --lib --features chaos_testing
```

## Contributing

When extending these systems:

1. **Capabilities**: Implement the `Capability` trait
2. **Feature Flags**: Register new features in `DEFAULT_FEATURES`
3. **Fault Injection**: Add new fault types to `FaultType` enum
4. **Intent API**: Extend `Intent` enum for new operations

## Documentation

- API Reference: `cargo doc --open`
- Examples: See `examples/` directory
- Architecture: See `docs/architecture.md`

## Support

For issues or questions:
- GitHub Issues: https://github.com/incyashraj/karana-os/issues
- Documentation: https://karana-os.dev/docs
