use anyhow::Result;
use std::sync::Arc;
use crate::net::KaranaSwarm;

pub struct KaranaActor {
    #[allow(dead_code)]
    swarm: Arc<KaranaSwarm>,
}

impl KaranaActor {
    pub fn new(swarm: &Arc<KaranaSwarm>) -> Result<Self> {
        Ok(Self {
            swarm: swarm.clone(),
        })
    }

    pub async fn ignite(&self) -> Result<()> {
        log::info!("Runtime Ignited");
        Ok(())
    }
}
