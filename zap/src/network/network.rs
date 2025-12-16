use crate::network::transport::Transport;
use anyhow::Result;
use std::net::SocketAddr;
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
        
        Ok(())
    }

    /// Connect to a specific peer using peer ID
    async fn connect_to_peer_with_id(&self, addr: SocketAddr, peer_id: [u8; 32]) -> Result<()> {
        let peer_pubkey = PublicKey::from(peer_id);
        
        println!("[STREAM]   Establishing TCP connection to {}...", addr);
        let stream = self.transport.connect(addr, peer_pubkey).await?;
        
        println!("[STREAM]   ✓ Noise handshake complete");
        
        // TODO: Begin state sync protocol
        // For now, just verify we can connect
        drop(stream);
        
        Ok(())
    }

    /// Connect to mainnet seed peers using Noise IK handshake
    pub async fn connect_to_mainnet_seeds(&self) -> Result<()> {
        use crate::config::seeds::{mainnet_seeds, resolve_seed};
        
        let seeds = mainnet_seeds();
        println!("[STREAM] Attempting to connect to {} mainnet seed(s)", seeds.len());
        
        for seed in &seeds {
            println!("[STREAM] Connecting to {} (port {})", seed.dns_name, seed.port);
            println!("[STREAM]   Peer ID: {}", hex::encode(&seed.peer_id));
            
            // Resolve DNS to IP addresses
            match resolve_seed(seed).await {
                Ok(addrs) => {
                    println!("[STREAM]   Resolved to {} address(es)", addrs.len());
                    for addr in &addrs {
                        println!("[STREAM]     - {}", addr);
                    }
                    
                    // Try to connect to the first resolved address
                    if let Some(socket_addr) = addrs.first() {
                        match self.connect_to_peer_with_id(*socket_addr, seed.peer_id).await {
                            Ok(()) => {
                                println!("[STREAM] ✓ Successfully connected to {}", seed.dns_name);
                                // For now, just connect to one seed
                                return Ok(());
                            }
                            Err(e) => {
                                println!("[STREAM] ✗ Failed to connect: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("[STREAM] ✗ Failed to resolve {}: {}", seed.dns_name, e);
                }
            }
        }
        
        anyhow::bail!("Failed to connect to any mainnet seeds")
    }

    pub fn start(&self) -> Result<()> {
        println!("Zap network starting...");
        Ok(())
    }

    pub fn broadcast_transaction(&self, _txn: &[u8]) {
        // Mock broadcast
    }
}
