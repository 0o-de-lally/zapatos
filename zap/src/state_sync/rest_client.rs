/// REST API client for Aptos mainnet - ONLY for peer discovery
use anyhow::Result;
use serde::{Deserialize, Serialize};

const MAINNET_API_URL: &str = "https://fullnode.mainnet.aptoslabs.com/v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerInfo {
    pub chain_id: u8,
    pub epoch: String,
    pub ledger_version: String,
    pub oldest_ledger_version: String,
    pub ledger_timestamp: String,
    pub node_role: String,
    pub oldest_block_height: String,
    pub block_height: String,
    pub git_hash: Option<String>,
}

/// Peer information from the network
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub addresses: Vec<String>,
    pub role: String,
}

pub struct RestClient {
    client: reqwest::Client,
    base_url: String,
}

impl RestClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: MAINNET_API_URL.to_string(),
        }
    }

    /// Get the current ledger info from mainnet
    pub async fn get_ledger_info(&self) -> Result<LedgerInfo> {
        let url = format!("{}/", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to get ledger info: {}", response.status());
        }
        
        let ledger_info: LedgerInfo = response.json().await?;
        Ok(ledger_info)
    }

    /// Discover active peer addresses from the network
    /// This is used ONLY for bootstrapping P2P connections
    pub async fn discover_peers(&self) -> Result<Vec<String>> {
        // Try to get peer information from the API
        // Note: The actual endpoint may vary, this is a placeholder
        // We may need to parse from validator set or other sources
        
        println!("[STREAM] Attempting to discover peers via REST API...");
        
        // For now, return empty - we'll need to find the right endpoint
        // or extract peer info from validator set in genesis
        Ok(vec![])
    }
}

impl Default for RestClient {
    fn default() -> Self {
        Self::new()
    }
}
