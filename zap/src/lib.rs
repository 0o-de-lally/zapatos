use clap::Parser;
use std::path::PathBuf;
use crate as zap;

pub mod config;
pub mod crypto;
pub mod network;
pub mod state_sync;
pub mod storage;
pub mod types;

#[cfg(test)]
pub mod tests;

#[derive(Clone, Debug, Parser)]
#[clap(name = "Zap", author, version)]
pub struct ZapArgs {
    /// Path to node configuration file
    #[clap(short = 'f', long)]
    pub config: Option<PathBuf>,
    
    #[clap(long)]
    pub peer_address: Option<String>, // e.g. "127.0.0.1:6180"
    
    #[clap(long)]
    pub peer_id: Option<String>, // e.g. "hex_pubkey"

    /// Path to genesis blob
    #[clap(long, value_parser)]
    pub genesis_file: Option<PathBuf>,

    /// Path to waypoint file
    #[clap(long, value_parser)]
    pub waypoint_file: Option<PathBuf>,
}

impl ZapArgs {
    pub async fn run(self) -> anyhow::Result<()> {
        println!("Zap starting...");
        println!("Config path: {:?}", self.config);
        
        let node_config = if let Some(config_path) = self.config {
             zap::config::NodeConfig::load_from_path(config_path).expect("Failed to load config")
        } else {
             zap::config::NodeConfig::default()
        };
        let height = 0;
        println!("Loaded config: {:?}", node_config);
        
        // Initialize components
        let storage = std::sync::Arc::new(zap::storage::AptosDB::open(&node_config.base.data_dir).expect("Failed to open DB"));
        let network = std::sync::Arc::new(zap::network::Network::new());
        let _state_sync = zap::state_sync::StateSync::new(network.clone(), storage.clone());

        println!("Zap node initialized.");
        
        // Load Waypoint
        if let Some(waypoint_path) = &self.waypoint_file {
            let waypoint_str = std::fs::read_to_string(waypoint_path)
                .map_err(|e| anyhow::anyhow!("Failed to read waypoint file: {}", e))?;
            let waypoint = waypoint_str.trim().parse::<crate::types::waypoints::Waypoint>()?;
            println!("Loaded Waypoint: {}", waypoint);
        }

        // Load Genesis
        if let Some(genesis_path) = &self.genesis_file {
             let genesis_bytes = std::fs::read(genesis_path)
                .map_err(|e| anyhow::anyhow!("Failed to read genesis file: {}", e))?;
            // Attempt to deserialize to verify it's a valid Transaction of sorts
            let genesis_txn: crate::types::transaction::Transaction = bcs::from_bytes(&genesis_bytes)?;
            println!("Loaded Genesis Transaction ({} bytes)", genesis_bytes.len());
            // TODO: In real node, we would use this to bootstrap DB
        }

        // In a real implementation, we would start the runtimes here.
        if let (Some(addr_str), Some(peer_id_hex)) = (self.peer_address, self.peer_id) {
            // The original code used `network.clone()` from the initialized network.
            // The instruction snippet creates a new `Network::new()`.
            // Sticking to the instruction's snippet for this part.
            let net = zap::network::Network::new(); 
            net.connect_to_peer(&addr_str, &peer_id_hex).await?;
        } else {
             println!("No peer specified. Use --peer-address and --peer-id to connect.");
        }

        println!("Zap node running... (Press Ctrl+C to exit)");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
