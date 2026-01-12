// Kāraṇa OS - Privacy Module
pub mod audit;
pub mod data_retention;
pub mod encryption;
pub mod ephemeral;
pub mod face_blur;
pub mod permissions;
pub mod retention;
pub mod dashboard; // Phase 60: Privacy dashboard

pub use audit::*;
pub use data_retention::*;
pub use encryption::*;
pub use ephemeral::*;
pub use face_blur::*;
pub use permissions::*;
pub use retention::*;
pub use dashboard::PrivacyDashboard;
