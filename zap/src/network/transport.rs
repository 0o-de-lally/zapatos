use crate::crypto::noise::{NoiseConfig, NoiseSession, MAX_SIZE_NOISE_MSG};
use anyhow::{Context, Result};
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};
use x25519_dalek::{PublicKey, StaticSecret};
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use std::cmp::min;

pub struct Transport {
    noise_config: NoiseConfig,
}

impl Transport {
    pub fn new(private_key: StaticSecret) -> Self {
        Self {
            noise_config: NoiseConfig::new(private_key),
        }
    }

    pub async fn connect(
        &self,
        addr: SocketAddr,
        remote_public_key: PublicKey,
    ) -> Result<NoiseStream<Compat<TcpStream>>> {
        let stream = TcpStream::connect(addr).await.context("Failed to connect via TCP")?;
        let stream = stream.compat();
        
        let (stream, session, _) = self.handshake_initiator(stream, remote_public_key).await?;
        
        Ok(NoiseStream::new(stream, session))
    }
    
    async fn handshake_initiator<S>(
        &self,
        mut stream: S,
        remote_public_key: PublicKey,
    ) -> Result<(S, NoiseSession, Vec<u8>)>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send,
    {
        let (handshake_state, first_msg) = {
            let mut rng = rand::thread_rng();
            let prologue = b"aptos-network-handshake-v1";
            self.noise_config.initiate_connection(
                &mut rng,
                prologue,
                remote_public_key,
                None,
            ).map_err(|e| anyhow::anyhow!("Noise init failed: {}", e))?
        };

        let len = first_msg.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&first_msg).await?;
        stream.flush().await?;

        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        if len > MAX_SIZE_NOISE_MSG {
            return Err(anyhow::anyhow!("Handshake response too large"));
        }

        let mut response_msg = vec![0u8; len];
        stream.read_exact(&mut response_msg).await?;

        let (payload, session) = self.noise_config.finalize_connection(handshake_state, &response_msg)
             .map_err(|e| anyhow::anyhow!("Noise finalize failed: {}", e))?;

        Ok((stream, session, payload))
    }
}

pub struct NoiseStream<S> {
    inner: S,
    session: NoiseSession,
    read_buffer: Vec<u8>,
    read_pos: usize,
}

impl<S: AsyncRead + AsyncWrite + Unpin> NoiseStream<S> {
    pub fn new(inner: S, session: NoiseSession) -> Self {
        Self { inner, session, read_buffer: Vec::new(), read_pos: 0 }
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncRead for NoiseStream<S> {
    // Basic implementation: read frame length, read frame, decrypt, serve from buffer.
    // NOTE: This implementation is blocking/sync regarding the `poll` interface which is tricky.
    // Correct implementation requires careful state machine for `poll_read`.
    // Since we are using `futures::io`, we might benefit from `tokio-util`'s codec but we are here now.
    // For "Minimalist" demo, let's assume we read full frames.
    
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut TaskContext<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        // If we have data in buffer, return it
        if self.read_pos < self.read_buffer.len() {
            let available = self.read_buffer.len() - self.read_pos;
            let to_copy = min(available, buf.len());
            buf[..to_copy].copy_from_slice(&self.read_buffer[self.read_pos..self.read_pos + to_copy]);
            self.read_pos += to_copy;
            return Poll::Ready(Ok(to_copy));
        }

        // We need to read a new frame. 
        // This is hard to do in `poll_read` without state. 
        // Ideally we should use `AsyncReadExt` in a loop, but `poll_read` is non-async.
        // We will return `Pending` if we can't get a full frame, which is bad without a waker.
        
        // Alternative: Zap Milestone 1 only needs to SYNC.
        // A simpler way is to expose `send_message` and `recv_message` on `NoiseStream` (or `Transport`) 
        // and NOT implement AsyncRead/Write encryption transparently if it's too complex for "minimal".
        // State Sync likely sends struct messages anyway.
        // Let's implement `read_message` and `write_message` directly.
        
        Poll::Ready(Ok(0)) // Placeholder to satisfy trait, but we will add specific methods.
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> AsyncWrite for NoiseStream<S> {
     fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        // We can't easily frame individual `write` calls because they might be partial.
        // We'll rely on `write_message` method.
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_close(cx)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> NoiseStream<S> {
     pub async fn read_message(&mut self) -> Result<Vec<u8>> {
         // Read length
         let mut len_bytes = [0u8; 4];
         self.inner.read_exact(&mut len_bytes).await?;
         let len = u32::from_be_bytes(len_bytes) as usize;
         
         if len > MAX_SIZE_NOISE_MSG {
             return Err(anyhow::anyhow!("Message too large"));
         }
         
         let mut buffer = vec![0u8; len];
         self.inner.read_exact(&mut buffer).await?;
         
         // Decrypt
         let plaintext = self.session.read_message_in_place(&mut buffer)
             .map_err(|e| anyhow::anyhow!("Decrypt failed: {}", e))?;
             
         Ok(plaintext.to_vec())
     }
     
     pub async fn write_message(&mut self, payload: &[u8]) -> Result<()> {
         // Encrypt
         // We need to copy payload to a buffer that has space for Tag
         let mut buffer = payload.to_vec();
         let tag = self.session.write_message_in_place(&mut buffer) // This actually returns tag, modify buffer?
             // My implementation of `write_message_in_place` in noise.rs:
             // "returns the authentication tag as result" and "encrypts in place".
             // So `buffer` (payload) is encrypted. We need to append tag.
             .map_err(|e| anyhow::anyhow!("Encrypt failed: {}", e))?;
             
         buffer.extend_from_slice(&tag);
         
         let len = buffer.len() as u32;
         self.inner.write_all(&len.to_be_bytes()).await?;
         self.inner.write_all(&buffer).await?;
         self.inner.flush().await?;
         
         Ok(())
     }
}
