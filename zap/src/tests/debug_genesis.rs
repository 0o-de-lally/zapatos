#[cfg(test)]
mod debug_genesis {
    use std::fs;

    #[test]
    fn debug_genesis_structure() {
        let genesis_bytes = fs::read("fixtures/mainnet/genesis.blob")
            .expect("Failed to read genesis.blob");
        
        println!("Genesis blob size: {} bytes", genesis_bytes.len());
        println!("First 100 bytes (hex): {}", hex::encode(&genesis_bytes[..100.min(genesis_bytes.len())]));
        
        // Try to deserialize just to see the error
        let result: Result<crate::types::transaction::Transaction, _> = bcs::from_bytes(&genesis_bytes);
        match result {
            Ok(_) => println!("SUCCESS!"),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
