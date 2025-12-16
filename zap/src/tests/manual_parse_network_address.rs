/// Manual BCS parser for NetworkAddress - bypassing serde issues
#[cfg(test)]
mod tests {
    #[test]
    fn test_manual_parse_network_address() {
        let hex_str = "016804023e76616c696461746f722e62626237366432642d303262352d346533652d626663332d3966313061326536393834392e6170746f732e6269736f6e2e72756e05241807203601215a079b0114a32104bd02149cf2258a206c8f8c79790e0684f4adfeae400800";
        
        let bytes = hex::decode(hex_str).unwrap();
        
        // Parse manually
        let mut offset = 0;
        
        // Vec<Wrapper> length
        let vec_len = bytes[offset];
        offset += 1;
        println!("Vec length: {}", vec_len);
        
        // Wrapper bytes length  
        let wrapper_len = bytes[offset];
        offset += 1;
        println!("Wrapper length: {}", wrapper_len);
        
        // Now parse the protocols
        let protocols_len = bytes[offset];
        offset += 1;
        println!("Protocols count: {}", protocols_len);
        
        let mut dns_name = None;
        let mut port = None;
        let mut peer_id = None;
        
        for _ in 0..protocols_len {
            let variant = bytes[offset];
            offset += 1;
            
            match variant {
                2 | 3 | 4 => { // Dns, Dns4, Dns6
                    let name_len = bytes[offset] as usize;
                    offset += 1;
                    let name_bytes = &bytes[offset..offset + name_len];
                    dns_name = Some(String::from_utf8(name_bytes.to_vec()).unwrap());
                    offset += name_len;
                    println!("DNS: {}", dns_name.as_ref().unwrap());
                }
                5 => { // Tcp
                    let p = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                    port = Some(p);
                    offset += 2;
                    println!("Port: {}", p);
                }
                7 => { // NoiseIK
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&bytes[offset..offset + 32]);
                    peer_id = Some(key);
                    offset += 32;
                    println!("Peer ID: {}", hex::encode(key));
                }
                8 => { // Handshake
                    let version = bytes[offset];
                    offset += 1;
                    println!("Handshake version: {}", version);
                }
                _ => {
                    println!("Unknown variant: {}", variant);
                    break;
                }
            }
        }
        
        println!("\n✓ Successfully parsed!");
        println!("  DNS: {}", dns_name.unwrap());
        println!("  Port: {}", port.unwrap());
        println!("  Peer ID: {}", hex::encode(peer_id.unwrap()));
    }
}
