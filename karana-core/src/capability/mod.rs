// Capability Module - Layer capability system
// Phase 47: Enable decoupled layer architecture

pub mod traits;

pub use traits::{
    Layer, LayerId, LayerState, Capability, CapabilityRequirements,
    CapabilityAdvertisement, CapabilityRegistry,
};
