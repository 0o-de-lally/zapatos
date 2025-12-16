use crate::network::transport::Transport;
use crate::crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use anyhow::Result;
use std::net::SocketAddr;
use std::str::FromStr;
use x25519_dalek::{PublicKey, StaticSecret};
use hex::FromHex;

pub struct Network {
    transport: Transport,
}

impl Network {
    pub fn new() -> Self {
        // Generate a random static key for ourselves
        let mut rng = rand::thread_rng();
        let private_key = StaticSecret::new(&mut rng);
        
        Self {
            transport: Transport::new(private_key),
        }
    }

    pub async fn connect_to_peer(&self, addr_str: &str, peer_id_hex: &str) -> Result<()> {
        let addr: SocketAddr = addr_str.parse()?;
        let peer_id_bytes = <[u8; 32]>::from_hex(peer_id_hex)?;
        let peer_id = PublicKey::from(peer_id_bytes);

        println!("Connecting to {} ({})", addr, peer_id_hex);
        let mut stream = self.transport.connect(addr, peer_id).await?;
        println!("Connected and Handshake/Noise established!");
        
        // Send a simple request: GetServerProtocolVersion
        use crate::state_sync::message::{StorageServiceRequest, DataRequest, StorageServiceResponse, DataResponse};
        
        let request = StorageServiceRequest {
            data_request: DataRequest::GetServerProtocolVersion,
            use_compression: false, 
        };
        
        let msg_bytes = bcs::to_bytes(&request)?;
        stream.write_message(&msg_bytes).await?;
        println!("Sent GetServerProtocolVersion request");
        
        // Read response
        let resp_bytes = stream.read_message().await?;
        println!("Received {} bytes", resp_bytes.len());
        
        let response: StorageServiceResponse = bcs::from_bytes(&resp_bytes)?;
        println!("Received Response: {:?}", response);
        
        match response {
             StorageServiceResponse::RawResponse(DataResponse::ServerProtocolVersion(v)) => {
                 println!("Server Protocol Version: {}", v.protocol_version);
             }
             _ => println!("Received other response"),
        }
        
        // Keep connection open slightly
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        println!("Zap network starting...");
        Ok(())
    }

    pub fn broadcast_transaction(&self, _txn: &[u8]) {
        // Mock broadcast
    }
}
