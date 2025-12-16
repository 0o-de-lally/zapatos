/// Test to deserialize validator network addresses from mainnet
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_validator_network_address() {
        // This is the hex-encoded network_addresses from validator index 0
        // From: curl -s "https://fullnode.mainnet.aptoslabs.com/v1/accounts/0x1/resource/0x1::stake::ValidatorSet"
        let hex_str = "016804023e76616c696461746f722e62626237366432642d303262352d346533652d626663332d3966313061326536393834392e6170746f732e6269736f6e2e72756e05241807203601215a079b0114a32104bd02149cf2258a206c8f8c79790e0684f4adfeae400800";
        
        let bytes = hex::decode(hex_str).unwrap();
        
        // The bytes are BCS-encoded Vec<NetworkAddress>
        // Let's try to deserialize
        let result: Result<Vec<aptos_types::network_address::NetworkAddress>, _> = bcs::from_bytes(&bytes);
        
        match result {
            Ok(addrs) => {
                println!("Successfully deserialized {} network address(es)", addrs.len());
                for addr in &addrs {
                    println!("Address: {}", addr);
                    if let Some(pubkey) = addr.find_noise_proto() {
                        println!("  Peer ID (x25519): {:?}", pubkey);
                    }
                    if let Some(port) = addr.find_port() {
                        println!("  Port: {}", port);
                    }
                }
            }
            Err(e) => {
                println!("Failed to deserialize: {}", e);
                // Try as single NetworkAddress instead of Vec
                let single_result: Result<aptos_types::network_address::NetworkAddress, _> = bcs::from_bytes(&bytes);
                if let Ok(addr) = single_result {
                    println!("Deserialized as single address: {}", addr);
                }
            }
        }
    }
}
