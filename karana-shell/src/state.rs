use druid::{Data, Lens};
use im::Vector;
use crate::client::KaranaClient;
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub intent_input: String,
    pub active_panels: Vector<PanelData>,
    pub system_status: String,
    pub is_processing: bool,
    #[data(ignore)]
    pub client: Arc<KaranaClient>,
    // DAO/Governance State
    pub dao_proposal_active: bool,
    pub dao_proposal_text: String,
}

#[derive(Clone, Data, Lens)]
pub struct PanelData {
    pub id: String,
    pub title: String,
    pub content: String,
    pub panel_type: String, // "graph", "list", "code"
    pub is_verified: bool,
    pub proof_hash: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            intent_input: String::new(),
            active_panels: Vector::new(),
            system_status: "Symbiotic Link: Active".to_string(),
            is_processing: false,
            client: Arc::new(KaranaClient::new()),
            dao_proposal_active: false,
            dao_proposal_text: String::new(),
        }
    }
}
