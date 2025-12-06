// Event Bus Module - Decoupled layer communication
// Phase 47: Message-passing event architecture

pub mod core;
pub mod router;

pub use core::{
    Event, EventId, EventPriority, EventCategory, EventMetadata, EventPayload,
    EventHandler, EventBus, EventBusStatistics,
};

pub use router::{
    EventRouter, RoutingPolicy, RoutingRule, RouterConfig, RouterStatistics,
};
