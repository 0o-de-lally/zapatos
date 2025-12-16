use crate::network::Network;
use crate::storage::AptosDB;
use anyhow::Result;
use std::sync::Arc;

pub struct StateSync {
    _network: Arc<Network>,
    _storage: Arc<AptosDB>,
}

impl StateSync {
    pub fn new(network: Arc<Network>, storage: Arc<AptosDB>) -> Self {
        Self { 
            _network: network, 
            _storage: storage 
        }
    }

    pub fn start(&self) -> Result<()> {
        println!("Zap state sync starting...");
        // Logic to sync calls network.broadcast_transaction(...) and storage.save_transaction(...)
        Ok(())
    }
}
