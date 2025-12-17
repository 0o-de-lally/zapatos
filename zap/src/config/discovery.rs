use crate::config::seeds::SeedPeer;
use crate::config::network_address::NetworkAddress;
use anyhow::Result;
use serde::Deserialize;
use hex::FromHex;

const MAINNET_VALIDATOR_SET_URL: &str = "https://api.mainnet.aptoslabs.com/v1/accounts/0x1/resource/0x1::stake::ValidatorSet";

#[derive(Deserialize, Debug)]
struct ValidatorSetResponse {
    data: ValidatorSetData,
}

#[derive(Deserialize, Debug)]
struct ValidatorSetData {
    active_validators: Vec<ValidatorInfo>,
}

#[derive(Deserialize, Debug)]
struct ValidatorInfo {
    config: ValidatorConfig,
}

#[derive(Deserialize, Debug)]
struct ValidatorConfig {
    // This is a hex string in the JSON response
    fullnode_addresses: String, 
}

pub async fn fetch_mainnet_seeds() -> Result<Vec<SeedPeer>> {
    println!("[DISCOVERY] Fetching Mainnet ValidatorSet from API...");
    let response = reqwest::get(MAINNET_VALIDATOR_SET_URL).await?;
    let json: ValidatorSetResponse = response.json().await?;
    
    let mut seeds = Vec::new();
    
    println!("[DISCOVERY] Found {} active validators", json.data.active_validators.len());
    
    for (i, validator) in json.data.active_validators.iter().enumerate() {
        // Decode hex string to bytes
        let hex_str = validator.config.fullnode_addresses.trim_start_matches("0x");
        let bytes = Vec::from_hex(hex_str)?;
        
        // The bytes are a BCS serialized Vec<NetworkAddress>. 
        // Note: The ValidatorConfig struct in Zapatos defines it as `Vec<u8>`, which is BCS serialized `Vec<NetworkAddress>`.
        // However, BCS serialization of `Vec<T>` is `[len (uleb)][item1][item2]...`
        // `NetworkAddress` serialization itself is `[len][bytes]`.
        
        // Wait, `ValidatorConfig` in Zapatos:
        // `pub fullnode_network_addresses: Vec<u8>`
        // Which says "This is an bcs serialized Vec<NetworkAddress>"
        
        // Let's try to deserialize `Vec<NetworkAddress>` from `bytes`.
        match bcs::from_bytes::<Vec<NetworkAddress>>(&bytes) {
            Ok(addrs) => {
                // println!("[DISCOVERY] Validator {} parsed {} addrs", i, addrs.len());
                for addr in addrs {
                    // We need a peer with IP/DNS, Port, and NoiseIK
                    if let (Some(dns_name), Some(port), Some(peer_id_key)) = (
                        addr.find_dns_name().or_else(|| addr.find_ip_addr().map(|ip| ip.to_string())), 
                        addr.find_port(), 
                        addr.find_noise_proto()
                    ) {
                         seeds.push(SeedPeer {
                             dns_name,
                             port,
                             peer_id: *peer_id_key.as_bytes(),
                         });
                    }
                }
            }
            Err(e) => {
                if i < 3 {
                    println!("[DISCOVERY] Failed to parse addresses for validator {}: {}", i, e);
                    println!("[DISCOVERY]   Hex: {}", hex_str);
                }
            }
        }
    }
    
    println!("[DISCOVERY] Extracted {} valid seed peers", seeds.len());
    Ok(seeds)
}
