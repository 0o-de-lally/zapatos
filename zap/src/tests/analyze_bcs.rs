/// Test to analyze the raw BCS structure
#[cfg(test)]
mod tests {
    #[test]
    fn test_analyze_bcs_structure() {
        let hex_str = "016804023e76616c696461746f722e62626237366432642d303262352d346533652d626663332d3966313061326536393834392e6170746f732e6269736f6e2e72756e05241807203601215a079b0114a32104bd02149cf2258a206c8f8c79790e0684f4adfeae400800";
        
        let bytes = hex::decode(hex_str).unwrap();
        
        println!("Total bytes: {}", bytes.len());
        println!("First 20 bytes: {:02x?}", &bytes[0..20]);
        
        // BCS format for Vec<T>:
        // - ULEB128 length
        // - Then each element
        
        // First byte should be vec length
        println!("Vec length: {}", bytes[0]);
        
        // Next should be the NetworkAddress
        // BCS for NetworkAddress wraps it with length
        println!("NetworkAddress length: {}", bytes[1]);
        
        // Now the actual protocols Vec
        println!("Protocols vec length: {}", bytes[2]);
        
        // First protocol variant
        println!("First protocol variant index: {}", bytes[3]);
        
        // Let's manually parse to understand the structure
        let mut offset = 0;
        println!("\n=== Manual parsing ===");
        println!("Offset {}: Vec<NetworkAddress> length = {}", offset, bytes[offset]);
        offset += 1;
        
        println!("Offset {}: NetworkAddress BCS wrapper length = {}", offset, bytes[offset]);
        offset += 1;
        
        println!("Offset {}: Vec<Protocol> length = {}", offset, bytes[offset]);
        offset += 1;
        
        // Protocol 0
        println!("Offset {}: Protocol variant = {}", offset, bytes[offset]);
        offset += 1;
        
        // If it's Dns (variant 2), next is string length
        if bytes[3] == 2 {
            println!("Offset {}: DNS name length = {}", offset, bytes[offset]);
            let dns_len = bytes[offset] as usize;
            offset += 1;
            let dns_name = String::from_utf8(bytes[offset..offset+dns_len].to_vec()).unwrap();
            println!("DNS name: {}", dns_name);
            offset += dns_len;
            
            println!("Offset {}: Next protocol variant = {}", offset, bytes[offset]);
        }
    }
}
