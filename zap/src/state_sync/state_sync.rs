use crate::network::Network;
use crate::storage::AptosDB;
use anyhow::Result;
use std::sync::Arc;

pub struct StateSync {
    network: Arc<Network>,
    storage: Arc<AptosDB>,
}

impl StateSync {
    pub fn new(network: Arc<Network>, storage: Arc<AptosDB>) -> Self {
        Self { network, storage }
    }

    pub fn start(&self) -> Result<()> {
        println!("Zap state sync starting...");
        // Logic to sync calls network.broadcast_transaction(...) and storage.save_transaction(...)
        Ok(())
    }
}
