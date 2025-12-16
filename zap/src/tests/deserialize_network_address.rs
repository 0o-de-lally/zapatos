/// Test to deserialize validator network addresses from mainnet
#[cfg(test)]
mod tests {
    use serde::Deserialize;
    
    #[test]
    fn test_deserialize_validator_network_address() {
        // This is the hex-encoded network_addresses from validator index 0
        let hex_str = "016804023e76616c696461746f722e62626237366432642d303262352d346533652d626663332d3966313061326536393834392e6170746f732e6269736f6e2e72756e05241807203601215a079b0114a32104bd02149cf2258a206c8f8c79790e0684f4adfeae400800";
        
        let bytes = hex::decode(hex_str).unwrap();
        
        // The format is: Vec<BCS-wrapped-bytes>
        // Each element is length-prefixed bytes containing the actual NetworkAddress
        #[derive(Deserialize)]
        struct Wrapper(#[serde(with = "serde_bytes")] Vec<u8>);
        
        let result: Result<Vec<Wrapper>, _> = bcs::from_bytes(&bytes);
        
        match result {
            Ok(wrappers) => {
                println!("Successfully deserialized {} wrapped address(es)", wrappers.len());
                for wrapper in &wrappers {
                    println!("Inner bytes length: {}", wrapper.0.len());
                    
                    // Try deserializing as Vec<Protocol> directly
                    use crate::types::network_address::Protocol;
                    let protocols_result: Result<Vec<Protocol>, _> = bcs::from_bytes(&wrapper.0);
                    
                    match protocols_result {
                        Ok(protocols) => {
                            println!("\n✓ Successfully parsed {} protocols!", protocols.len());
                            
                            // Extract info from protocols
                            let dns = protocols.iter().find_map(|p| match p {
                                Protocol::Dns(name) | Protocol::Dns4(name) | Protocol::Dns6(name) => {
                                    Some(name.0.clone())
                                }
                                _ => None,
                            });
                            let port = protocols.iter().find_map(|p| match p {
                                Protocol::Tcp(port) => Some(*port),
                                _ => None,
                            });
                            let peer_id = protocols.iter().find_map(|p| match p {
                                Protocol::NoiseIK(key) => Some(*key),
                                _ => None,
                            });
                            
                            if let Some(dns) = dns {
                                println!("  DNS: {}", dns);
                            }
                            if let Some(port) = port {
                                println!("  Port: {}", port);
                            }
                            if let Some(peer_id) = peer_id {
                                println!("  Peer ID (x25519): {}", hex::encode(peer_id));
                            }
                        }
                        Err(e) => println!("Failed to deserialize protocols: {}", e),
                    }
                }
            }
            Err(e) => {
                println!("Failed to deserialize wrappers: {}", e);
            }
        }
    }
}
