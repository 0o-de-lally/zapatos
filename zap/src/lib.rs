use clap::Parser;
use std::path::PathBuf;
use crate::mode::NodeMode;

pub mod config;
pub mod crypto;
pub mod network;
pub mod state_sync;
pub mod storage;
pub mod types;
pub mod mode;

#[cfg(test)]
pub mod tests;

#[derive(Clone, Debug, Parser)]
#[clap(name = "Aptos Node", author, version)]
pub struct NodeArgs {
    /// Node operational mode
    #[clap(short = 'm', long, default_value = "stream")]
    pub mode: NodeMode,
    
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

impl NodeArgs {
    pub async fn run(self) -> anyhow::Result<()> {
        println!("Starting Aptos Node in {} mode...", self.mode);
        
        match self.mode {
            NodeMode::Stream => self.run_streaming().await,
            NodeMode::FullNode => {
                anyhow::bail!("FullNode mode not yet implemented")
            }
            NodeMode::Validator => {
                anyhow::bail!("Validator mode not yet implemented")
            }
        }
    }

    async fn run_streaming(self) -> anyhow::Result<()> {
        println!("[INFO] Starting node in Streaming mode");
        println!("[INFO] Config path: {:?}", self.config);
        
        let _node_config = if let Some(config_path) = self.config {
             crate::config::NodeConfig::load_from_path(config_path).expect("Failed to load config")
        } else {
             crate::config::NodeConfig::default()
        };
        
        // Load Waypoint
        if let Some(waypoint_path) = &self.waypoint_file {
            let waypoint_str = std::fs::read_to_string(waypoint_path)
                .map_err(|e| anyhow::anyhow!("Failed to read waypoint file: {}", e))?;
            let waypoint = waypoint_str.trim().parse::<crate::types::waypoints::Waypoint>()?;
            println!("[INFO] Loaded waypoint: {}", waypoint);
        }

        // Load Genesis
        if let Some(genesis_path) = &self.genesis_file {
             let genesis_bytes = std::fs::read(genesis_path)
                .map_err(|e| anyhow::anyhow!("Failed to read genesis file: {}", e))?;
            let _genesis_txn: crate::types::transaction::Transaction = bcs::from_bytes(&genesis_bytes)?;
            println!("[INFO] Loaded genesis ({} bytes)", genesis_bytes.len());
        }

        // Initialize network (no storage in streaming mode)
        let network = std::sync::Arc::new(crate::network::Network::new(Some(std::path::PathBuf::from(".")))?);
        
        println!("[STREAM] Node initialized in streaming mode");
        
        // Connect to mainnet seed peers
        println!("[STREAM] Initiating connection to Aptos mainnet...");
        network.connect_to_mainnet_seeds().await?;
        
        println!("[STREAM] Waiting for state sync updates...");
        
        // In streaming mode, we just keep the node running
        // State sync will log updates as they come in
        if let (Some(addr_str), Some(peer_id_hex)) = (self.peer_address, self.peer_id) {
            let net = crate::network::Network::new(Some(std::path::PathBuf::from(".")))?;
            net.connect_to_peer(&addr_str, &peer_id_hex).await?;
        } else {
             println!("[INFO] No peer specified. Use --peer-address and --peer-id to connect.");
        }
        
        println!("[STREAM] Node running in streaming mode (Press Ctrl+C to exit)");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
