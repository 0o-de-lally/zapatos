use zap::network::transport::Transport;
use zap::network::handshake::{HandshakeMsg, ChainId, NetworkId};
use zap::crypto::noise::{NoiseConfig, handshake_init_msg_len, handshake_resp_msg_len, NoiseSession};
use x25519_dalek::{StaticSecret, PublicKey};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::compat::TokioAsyncReadCompatExt;
use futures::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_internal_handshake() {
    // 1. Setup Server Config
    let server_key = StaticSecret::new(rand::thread_rng());
    let server_pub = PublicKey::from(&server_key);
    let server_noise_config = NoiseConfig::new(server_key);
    
    // 2. Start Listener
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut stream = stream.compat();
        
        // --- Server Noise Logic ---
        // Read 168 bytes
        // Prologue (64) + InitMsg (104)
        let mut buffer = vec![0u8; 64 + 104]; 
        stream.read_exact(&mut buffer).await.unwrap();
        
        let (prologue, noise_msg) = buffer.split_at(64);
        
        // Verify prologue
        // Prologue = ClientPeerId (32) | ServerPubKey (32)
        // We can check if ServerPubKey matches our pub key
        let client_peer_id_bytes = &prologue[..32];
        let target_pub_key_bytes = &prologue[32..];
        assert_eq!(target_pub_key_bytes, server_pub.as_bytes(), "Client sent wrong target pub key in prologue");
        
        // Perform Noise Handshake Responder
        let (remote_static, handshake_state, payload) = server_noise_config.parse_client_init_message(
            prologue,
            noise_msg
        ).unwrap();
        
        // Check payload (timestamp) - 8 bytes
        assert_eq!(payload.len(), 8);
        println!("Server received timestamp payload: {:?}", payload);
        
        // Verify remote static matches client_peer_id in prologue
        // In our simple model, PeerId == PubKey bytes.
        assert_eq!(remote_static.as_bytes(), client_peer_id_bytes, "Remote static key mismatch with prologue");
        
        // Send Response
        // Payload: None (or empty?)
        // We'll send empty payload for now.
        let mut resp_buffer = vec![0u8; 48]; // handshake_resp_msg_len(0) = 48
        let session = {
            let mut rng = rand::thread_rng();
            server_noise_config.respond_to_client(
                &mut rng,
                handshake_state,
                None,
                &mut resp_buffer
            ).unwrap()
        };
        
        stream.write_all(&resp_buffer).await.unwrap();
        
        // Noise Handshake Complete.
        let mut session: NoiseSession = session;
        
        // --- HandshakeMsg Exchange ---
        // 1. Client sends HandshakeMsg
        // Length prefix (4 bytes BE) + Encrypted Msg
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.unwrap();
        let len = u32::from_be_bytes(len_buf) as usize;
        
        let mut encrypted_msg = vec![0u8; len];
        stream.read_exact(&mut encrypted_msg).await.unwrap();
        
        let msg_bytes = session.read_message_in_place(&mut encrypted_msg).unwrap();
        let client_handshake: HandshakeMsg = bcs::from_bytes(msg_bytes).unwrap();
        println!("Server received handshake: {:?}", client_handshake);
        
        // 2. Server sends HandshakeMsg
        let server_handshake = HandshakeMsg::new(ChainId::MAINNET, NetworkId::Public);
        let mut msg_bytes = bcs::to_bytes(&server_handshake).unwrap();
        
        // Encrypt
        let auth_tag = session.write_message_in_place(&mut msg_bytes).unwrap();
        msg_bytes.extend_from_slice(&auth_tag);
        
        // Send Length + Encrypted
        let len = msg_bytes.len() as u32;
        stream.write_all(&len.to_be_bytes()).await.unwrap();
        stream.write_all(&msg_bytes).await.unwrap();
        
    });
    
    // 3. Client Logic
    let client_key = StaticSecret::new(rand::thread_rng());
    let client_transport = Transport::new(client_key);
    
    let mut stream = client_transport.connect(addr, server_pub).await.expect("Client connect failed");
    
    // Now simulate `network.rs` exchange (which is MANUAL in network.rs, not Transport)
    // Wait, `network.rs` `connect_to_peer` implementation sends `HandshakeMsg`.
    // But we are in a test where we called `client_transport.connect`.
    // We need to verify that `client_transport.connect` returns a working `NoiseStream`.
    // Then use that stream to send HandshakeMsg.
    
    // 1. Send HandshakeMsg
    let our_handshake = HandshakeMsg::new(ChainId::MAINNET, NetworkId::Public);
    let msg_bytes = bcs::to_bytes(&our_handshake).expect("bcs ser");
    stream.write_message(&msg_bytes).await.expect("write msg");
    
    // 2. Receive HandshakeMsg
    let resp_bytes = stream.read_message().await.expect("read msg");
    let their_handshake: HandshakeMsg = bcs::from_bytes(&resp_bytes).expect("bcs de");
    
    assert_eq!(their_handshake.chain_id, ChainId::MAINNET);
    
    server_handle.await.expect("Server task failed");
}
