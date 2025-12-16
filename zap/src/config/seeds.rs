/// Mainnet seed peer configuration
/// 
/// Discovered from mainnet ValidatorSet via:
/// curl -s "https://fullnode.mainnet.aptoslabs.com/v1/accounts/0x1/resource/0x1::stake::ValidatorSet"
use std::net::SocketAddr;

/// Seed peer information with x25519 public key for Noise handshake
#[derive(Clone, Debug)]
pub struct SeedPeer {
    pub dns_name: String,
    pub port: u16,
    pub peer_id: [u8; 32],  // x25519 public key from NoiseIK protocol
}

/// Get mainnet seed peers with their peer IDs
///
/// These are actual validator addresses from the mainnet ValidatorSet.
/// The peer IDs were extracted from the BCS-encoded network_addresses field.
pub fn mainnet_seeds() -> Vec<SeedPeer> {
    vec![
        // Public Fullnode from Bison Trails (port 6182 - accepts public connections)
        SeedPeer {
            dns_name: "fullnode.bbb76d2d-02b5-4e3e-bfc3-9f10a2e69849.aptos.bison.run".to_string(),
            port: 6182,
            peer_id: hex_literal::hex!("202494f31865a994a7ef8c2723a5f3fcfa05a8dad872e7420de8c542dac59fb1"),
        },
        // Validator from Bison Trails (port 6180 - validator-only, for reference)
        // SeedPeer {
        //     dns_name: "validator.bbb76d2d-02b5-4e3e-bfc3-9f10a2e69849.aptos.bison.run".to_string(),
        //     port: 6180,
        //     peer_id: hex_literal::hex!("203601215a079b0114a32104bd02149cf2258a206c8f8c79790e0684f4adfeae"),
        // },
        // TODO: Add more PFN addresses from other validators' fullnode_addresses field
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
