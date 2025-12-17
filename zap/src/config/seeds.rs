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

/// Get mainnet seed peers (Dynamic Discovery + Fallback)
pub async fn get_seeds() -> Vec<SeedPeer> {
    use crate::config::discovery::fetch_mainnet_seeds;
    
    match fetch_mainnet_seeds().await {
        Ok(seeds) if !seeds.is_empty() => {
             return seeds;
        }
        Ok(_) => println!("[SEEDS] Discovery returned no seeds, using hardcoded fallback."),
        Err(e) => println!("[SEEDS] Discovery failed ({}), using hardcoded fallback.", e),
    }

    mainnet_seeds()
}

/// Hardcoded fallback seeds
pub fn mainnet_seeds() -> Vec<SeedPeer> {
    vec![
        // Bison Trails Public Fullnode (extracted from ValidatorSet on-chain)
        SeedPeer {
            dns_name: "fullnode.bbb76d2d-02b5-4e3e-bfc3-9f10a2e69849.aptos.bison.run".to_string(),
            port: 6182,
            peer_id: hex_literal::hex!("202494f31865a994a7ef8c2723a5f3fcfa05a8dad872e7420de8c542dac59fb1"),
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
