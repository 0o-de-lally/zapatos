/// Mainnet seed peer configuration
/// 
/// Discovered from mainnet ValidatorSet via:
/// curl -s "https://fullnode.mainnet.aptoslabs.com/v1/accounts/0x1/resource/0x1::stake::ValidatorSet"
use std::net::SocketAddr;

/// Seed peer information
#[derive(Clone, Debug)]
pub struct SeedPeer {
    pub dns_name: String,
    pub port: u16,
}

/// Get mainnet seed peers
/// 
/// These are actual validator addresses from the mainnet ValidatorSet.
/// The network_addresses field contains hex-encoded multiaddr format.
pub fn mainnet_seeds() -> Vec<SeedPeer> {
    vec![
        // Validator from Bison Trails (index 0)
        SeedPeer {
            dns_name: "validator.bbb76d2d-02b5-4e3e-bfc3-9f10a2e69849.aptos.bison.run".to_string(),
            port: 6180,
        },
        // Validator from Stakely (index 2)  
        SeedPeer {
            dns_name: "aptos-validator.stakely.io".to_string(),
            port: 6180,
        },
        // Validator from RhinoStake (index 3)
        SeedPeer {
            dns_name: "node.aptos.rhinostake.com".to_string(),
            port: 6180,
        },
    ]
}

/// Resolve DNS name to socket addresses
pub async fn resolve_seed(seed: &SeedPeer) -> anyhow::Result<Vec<SocketAddr>> {
    let addr_str = format!("{}:{}", seed.dns_name, seed.port);
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host(&addr_str)
        .await?
        .collect();
    
    if addrs.is_empty() {
        anyhow::bail!("Failed to resolve DNS for {}", seed.dns_name);
    }
    
    Ok(addrs)
}
