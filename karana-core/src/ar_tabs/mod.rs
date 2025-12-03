//! # AR Tabs Module
//!
//! Persistent AR tabs that float in space and survive user movement.
//! Part of the Spatial Symbiosis vision - replacing phones/PCs with
//! AR tabs pinned to real-world locations.
//!
//! ## Core Concepts
//!
//! - **ARTab**: A persistent tab pinned in space via a SpatialAnchor
//! - **TabContent**: What the tab displays (browser, video, game, etc.)
//! - **TabManager**: Multi-tab management with focus, minimize, close
//! - **TabInteraction**: Gaze/voice/gesture control for tabs
//!
//! ## Example
//!
//! ```rust,ignore
//! use karana_core::ar_tabs::{TabManager, TabContent, TabSize};
//! use karana_core::spatial::SpatialAnchor;
//!
//! let mut manager = TabManager::new();
//!
//! // Pin browser to kitchen counter
//! let browser = TabContent::browser("https://nytimes.com");
//! let anchor = spatial_system.create_anchor(current_pos, "counter").await?;
//! let tab_id = manager.pin_tab(browser, TabSize::default(), anchor).await?;
//!
//! // Later, when returning to kitchen:
//! // Tab automatically relocates and becomes visible
//! ```

pub mod tab;
pub mod manager;
pub mod browser;
pub mod interaction;
pub mod render;

pub use tab::{
    ARTab, TabId, TabContent, TabSize, TabState, TabStyle,
    TabMetadata, TabPermissions, InteractionZone,
};
pub use manager::{TabManager, TabLayout, LayoutMode, TabSnapshot};
pub use browser::{BrowserInstance, BrowserConfig, ScrollDirection, ScrollAmount};
pub use interaction::{
    TabInteraction, InteractionEvent, GazeState, DwellConfig,
    VoiceTabCommand, GestureType, TabGesture,
};
pub use render::{TabRenderer, RenderConfig, CompositeFrame, TabOverlay};
