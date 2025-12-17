use crate::network::transport::Transport;
use anyhow::Result;
use std::net::SocketAddr;
use std::path::PathBuf;
use x25519_dalek::{PublicKey, StaticSecret};
use hex::FromHex;

const IDENTITY_KEY_FILE: &str = "ephemeral_identity_key";

pub struct Network {
    transport: Transport,
}

impl Network {
    pub fn new(data_dir: Option<PathBuf>) -> Result<Self> {
        // Load or generate ephemeral identity (matching Aptos fullnode behavior)
        let private_key = Self::load_or_generate_identity(data_dir)?;
        
        Ok(Self {
            transport: Transport::new(private_key),
        })
    }
    
    /// Load ephemeral identity from disk, or generate and save a new one
    fn load_or_generate_identity(data_dir: Option<PathBuf>) -> Result<StaticSecret> {
        let identity_path = data_dir
            .unwrap_or_else(|| PathBuf::from("."))
            .join(IDENTITY_KEY_FILE);
        
        // Try to load existing identity
        if identity_path.exists() {
            let bytes = std::fs::read(&identity_path)?;
            if bytes.len() == 32 {
                let mut key_bytes = [0u8; 32];
                key_bytes.copy_from_slice(&bytes);
                println!("[NETWORK] Loaded ephemeral identity from {:?}", identity_path);
                return Ok(StaticSecret::from(key_bytes));
            }
        }
        
        // Generate new identity
        let mut rng = rand::thread_rng();
        let private_key = StaticSecret::new(&mut rng);
        let public_key = PublicKey::from(&private_key);
        
        // Save to disk
        std::fs::write(&identity_path, private_key.to_bytes())?;
        println!("[NETWORK] Generated new ephemeral identity: {}", hex::encode(public_key.as_bytes()));
        println!("[NETWORK] Saved to {:?}", identity_path);
        
        Ok(private_key)
    }

    pub async fn connect_to_peer(&self, addr_str: &str, peer_id_hex: &str) -> Result<()> {
        let addr: SocketAddr = addr_str.parse()?;
        let peer_id_bytes = <[u8; 32]>::from_hex(peer_id_hex)?;
        let peer_id = PublicKey::from(peer_id_bytes);

        println!("Connecting to {} ({})", addr, peer_id_hex);
        let mut stream = self.transport.connect(addr, peer_id).await?;
        println!("Connected and Handshake/Noise established!");
        
        // --- Aptos Handshake Protocol v1 ---
        use crate::network::handshake::{HandshakeMsg, ChainId, NetworkId};
        
        // 1. Send our HandshakeMsg
        // Minimal defaults: Mainnet, Public Network
        let our_handshake = HandshakeMsg::new(ChainId::MAINNET, NetworkId::Public);
        let msg_bytes = bcs::to_bytes(&our_handshake)?;
        stream.write_message(&msg_bytes).await?;
        println!("[HANDSHAKE] Sent HandshakeMsg: {:?}", our_handshake);
        
        // 2. Receive their HandshakeMsg
        let resp_bytes = stream.read_message().await?;
        let their_handshake: HandshakeMsg = bcs::from_bytes(&resp_bytes)?;
        println!("[HANDSHAKE] Received HandshakeMsg: {:?}", their_handshake);
        
        // 3. Negotiate
        let (version, protocols) = our_handshake.perform_handshake(&their_handshake)?;
        println!("[HANDSHAKE] Negotiated Version: {:?}, Protocols: {:?}", version, protocols);
        
        // --- Application Layer ---
        // Verify StorageServiceRpc is supported
        // In a real implementation we would dynamically dispatch based on protocols.
        // For now, assume if handshake passed, we can try GetServerProtocolVersion.
        
        use crate::state_sync::message::{
    DataRequest, StorageServiceRequest, StorageServiceResponseWrapper, DataResponse,
};
        
        // Send GetStorageServerSummary to find latest ledger info
        let request = StorageServiceRequest::new(
            DataRequest::GetStorageServerSummary,
            false, // no compression
        );
        
        let msg_bytes = bcs::to_bytes(&request)?;
        stream.write_message(&msg_bytes).await?;
        println!("[SYNC] Sent GetStorageServerSummary request");
        
        // Read response
        let resp_bytes = stream.read_message().await?;
        println!("[SYNC] Received {} bytes response", resp_bytes.len());
        
        // Try deserializing as the Wrapper (Enum)
        let response: StorageServiceResponseWrapper = bcs::from_bytes(&resp_bytes)?;
        
        match response {
            StorageServiceResponseWrapper::RawResponse(DataResponse::StorageServerSummary(summary)) => {
                if let Some(ledger_info) = summary.data_summary.synced_ledger_info {
                    println!("[SYNC] ✓ Latest Synced Ledger Info:");
                    // The Display impl for LedgerInfoWithSignatures prints version/epoch/ts
                    println!("[SYNC]   {}", ledger_info); 
                    println!("[SYNC]   Block ID: {}", ledger_info.ledger_info().consensus_block_id());
                } else {
                    println!("[SYNC] Peer has no synced ledger info available.");
                }
            }
            StorageServiceResponseWrapper::RawResponse(other) => println!("[SYNC] Received unexpected data response: {:?}", other),
            StorageServiceResponseWrapper::CompressedResponse(_) => println!("[SYNC] Received compressed response (unexpected)"),
        }
        
        Ok(())
    }

    /// Connect to a specific peer using peer ID
    async fn connect_to_peer_with_id(
        &self,
        addr: SocketAddr,
        peer_id: PublicKey,
        dns_name: &str,
    ) -> Result<()> {
        println!("[STREAM]   Establishing TCP connection to {}...", addr);
        let stream = tokio::net::TcpStream::connect(addr).await?;
        
        // Get our own peer ID (derived from our public key)
        let our_peer_id = self.transport.get_peer_id();
        
        // Perform Noise handshake with peer ID
        use tokio_util::compat::TokioAsyncReadCompatExt;
        let stream = stream.compat();
        self.transport.handshake(stream, our_peer_id, peer_id).await?;
        
        println!("[STREAM] ✓ Successfully connected to {} ({})", dns_name, addr);
        Ok(())
    }

    /// Connect to mainnet seed peers using Noise IK handshake
    pub async fn connect_to_mainnet_seeds(&self) -> Result<()> {
        use crate::config::seeds::{get_seeds, resolve_seed};
        
        let seeds = get_seeds().await;
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
                         let peer_id_pub = PublicKey::from(seed.peer_id);
                         match self.connect_to_peer_with_id(*socket_addr, peer_id_pub, &seed.dns_name).await {
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
