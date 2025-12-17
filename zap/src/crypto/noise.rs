use anyhow::Result;
use ring::aead::{self, Aad, LessSafeKey, UnboundKey};
use hkdf::Hkdf;
use sha2::Digest;
use x25519_dalek::{PublicKey, StaticSecret};
use std::io::{Cursor, Read, Write};
use std::convert::TryInto;

pub const NOISE_ID_SIZE: usize = 32;
pub const MAX_SIZE_NOISE_MSG: usize = 65535;
pub const AES_GCM_TAGLEN: usize = 16;
const PROTOCOL_NAME: &[u8] = b"Noise_IK_25519_AESGCM_SHA256\0\0\0\0";
const AES_NONCE_SIZE: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum NoiseError {
    #[error("noise: the received message is too short")]
    MsgTooShort,
    #[error("noise: HKDF has failed")]
    Hkdf,
    #[error("noise: encryption has failed")]
    Encrypt,
    #[error("noise: could not decrypt the received data")]
    Decrypt,
    #[error("noise: the public key received is of the wrong format")]
    WrongPublicKeyReceived,
    #[error("noise: session was closed due to decrypt error")]
    SessionClosed,
    #[error("noise: the payload that we are trying to send is too large")]
    PayloadTooLarge,
    #[error("noise: the message we received is too large")]
    ReceivedMsgTooLarge,
    #[error("noise: the response buffer passed as argument is too small")]
    ResponseBufferTooSmall,
    #[error("noise: the nonce exceeds the maximum u64 value")]
    NonceOverflow,
}

pub struct NoiseConfig {
    private_key: StaticSecret,
    public_key: PublicKey,
}

#[derive(Clone)]
pub struct InitiatorHandshakeState {
    h: Vec<u8>,
    ck: Vec<u8>,
    e: StaticSecret,
    rs: PublicKey,
}

#[derive(Clone)]
pub struct ResponderHandshakeState {
    h: Vec<u8>,
    ck: Vec<u8>,
    rs: PublicKey,
    re: PublicKey,
}

pub struct NoiseSession {
    valid: bool,
    remote_public_key: PublicKey,
    write_key: Vec<u8>,
    write_nonce: u64,
    read_key: Vec<u8>,
    read_nonce: u64,
}

impl NoiseConfig {
    pub fn new(private_key: StaticSecret) -> Self {
        let public_key = PublicKey::from(&private_key);
        Self {
            private_key,
            public_key,
        }
    }
    
    /// Get our public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn initiate_connection(
        &self,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        prologue: &[u8],
        remote_public: PublicKey,
        payload: Option<&[u8]>,
    ) -> Result<(InitiatorHandshakeState, Vec<u8>), NoiseError> {
        let payload_len = payload.map(<[u8]>::len).unwrap_or(0);
        let mut response_buffer = vec![0u8; handshake_init_msg_len(payload_len)];
        
        // ... (check errors skipped since we alloc correct size)

        let mut h = PROTOCOL_NAME.to_vec();
        let mut ck = PROTOCOL_NAME.to_vec();
        let rs = remote_public;
        mix_hash(&mut h, prologue);
        mix_hash(&mut h, rs.as_bytes());

    // -> e
        let e = StaticSecret::new(rng);

        let e_pub = PublicKey::from(&e);

        mix_hash(&mut h, e_pub.as_bytes());
        let mut writer = Cursor::new(&mut response_buffer);
        writer.write(e_pub.as_bytes()).map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // -> es
        let dh_output = e.diffie_hellman(&rs);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // -> s
        let aead = aes_key(&k[..]);
        let mut in_out = self.public_key.as_bytes().to_vec();
        let nonce = aead::Nonce::assume_unique_for_key([0u8; AES_NONCE_SIZE]);

        aead.seal_in_place_append_tag(nonce, Aad::from(&h), &mut in_out)
            .map_err(|_| NoiseError::Encrypt)?;
        
        mix_hash(&mut h, &in_out[..]);
        writer.write(&in_out[..]).map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // -> ss
        let dh_output = self.private_key.diffie_hellman(&rs);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // -> payload
        let aead = aes_key(&k[..]);
        let mut in_out = payload.unwrap_or(&[]).to_vec();
        let nonce = aead::Nonce::assume_unique_for_key([0u8; AES_NONCE_SIZE]);
        aead.seal_in_place_append_tag(nonce, Aad::from(&h), &mut in_out)
            .map_err(|_| NoiseError::Encrypt)?;

        mix_hash(&mut h, &in_out[..]);
        writer.write(&in_out[..]).map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        Ok((InitiatorHandshakeState { h, ck, e, rs }, response_buffer))
    }

    pub fn finalize_connection(
        &self,
        handshake_state: InitiatorHandshakeState,
        received_message: &[u8],
    ) -> Result<(Vec<u8>, NoiseSession), NoiseError> {
        if received_message.len() > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::ReceivedMsgTooLarge);
        }

         let InitiatorHandshakeState {
            mut h,
            mut ck,
            e,
            rs,
        } = handshake_state;

        // <- e
        let mut re_bytes = [0u8; 32];
        let mut cursor = Cursor::new(received_message);
        cursor.read_exact(&mut re_bytes).map_err(|_| NoiseError::MsgTooShort)?;
        mix_hash(&mut h, &re_bytes);
        let re = PublicKey::from(re_bytes);

         // <- ee
         let dh_output = e.diffie_hellman(&re);
         mix_key(&mut ck, dh_output.as_bytes())?;

         // <- se
         let dh_output = self.private_key.diffie_hellman(&re);
         let k = mix_key(&mut ck, dh_output.as_bytes())?;

         // <- payload
         let offset = cursor.position() as usize;
         let aead = aes_key(&k[..]);
         let mut in_out = received_message[offset..].to_vec();
         let nonce = aead::Nonce::assume_unique_for_key([0u8; AES_NONCE_SIZE]);
         let plaintext = aead.open_in_place(nonce, Aad::from(&h), &mut in_out)
             .map_err(|_| NoiseError::Decrypt)?;

         let (k1, k2) = hkdf_split(&ck, None)?;
         let session = NoiseSession::new(k1, k2, rs);

         Ok((plaintext.to_vec(), session))
    }

    // --- Responder Logic ---

    pub fn parse_client_init_message(
        &self,
        prologue: &[u8],
        received_message: &[u8],
    ) -> Result<(PublicKey, ResponderHandshakeState, Vec<u8>), NoiseError> {
        if received_message.len() > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::ReceivedMsgTooLarge);
        }
        
        let mut h = PROTOCOL_NAME.to_vec();
        let mut ck = PROTOCOL_NAME.to_vec();
        mix_hash(&mut h, prologue);
        mix_hash(&mut h, self.public_key.as_bytes());

        let mut cursor = Cursor::new(received_message);

        // <- e
        let mut re_bytes = [0u8; 32];
        cursor.read_exact(&mut re_bytes).map_err(|_| NoiseError::MsgTooShort)?;
        mix_hash(&mut h, &re_bytes);
        let re = PublicKey::from(re_bytes);

        // <- es
        let dh_output = self.private_key.diffie_hellman(&re);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // <- s
        let mut encrypted_rs = [0u8; 32 + AES_GCM_TAGLEN];
        cursor.read_exact(&mut encrypted_rs).map_err(|_| NoiseError::MsgTooShort)?;

        let aead = aes_key(&k[..]);
        let mut in_out = encrypted_rs.to_vec();
        let nonce = aead::Nonce::assume_unique_for_key([0u8; AES_NONCE_SIZE]);
        let rs_bytes = aead.open_in_place(nonce, Aad::from(&h), &mut in_out)
            .map_err(|_| NoiseError::Decrypt)?;
            
        let mut rs_arr = [0u8; 32];
        if rs_bytes.len() != 32 { return Err(NoiseError::Decrypt); }
        rs_arr.copy_from_slice(rs_bytes);
        let rs = PublicKey::from(rs_arr);
        
        mix_hash(&mut h, &encrypted_rs);

        // <- ss
        let dh_output = self.private_key.diffie_hellman(&rs);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // <- payload
        let offset = cursor.position() as usize;
        let received_encrypted_payload = &received_message[offset..];
        
        let aead = aes_key(&k[..]);
        let mut in_out = received_encrypted_payload.to_vec();
        let nonce = aead::Nonce::assume_unique_for_key([0u8; AES_NONCE_SIZE]);
        let received_payload = aead.open_in_place(nonce, Aad::from(&h), &mut in_out)
            .map_err(|_| NoiseError::Decrypt)?;
            
        mix_hash(&mut h, received_encrypted_payload);

        let state = ResponderHandshakeState { h, ck, rs, re };
        Ok((rs, state, received_payload.to_vec()))
    }

    pub fn respond_to_client(
        &self,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        handshake_state: ResponderHandshakeState,
        payload: Option<&[u8]>,
        response_buffer: &mut [u8],
    ) -> Result<NoiseSession, NoiseError> {
        let payload_len = payload.map(<[u8]>::len).unwrap_or(0);
        let required = handshake_resp_msg_len(payload_len);
        if response_buffer.len() < required {
            return Err(NoiseError::ResponseBufferTooSmall);
        }

        let ResponderHandshakeState { mut h, mut ck, rs, re } = handshake_state;

        // -> e
        let e = StaticSecret::new(rng);
        let e_pub = PublicKey::from(&e);

        mix_hash(&mut h, e_pub.as_bytes());
        let mut writer = Cursor::new(response_buffer);
        writer.write(e_pub.as_bytes()).map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // -> ee
        let dh_output = e.diffie_hellman(&re);
        mix_key(&mut ck, dh_output.as_bytes())?;

        // -> se
        let dh_output = e.diffie_hellman(&rs);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // -> payload
        let aead = aes_key(&k[..]);
        let mut in_out = payload.unwrap_or(&[]).to_vec();
        let nonce = aead::Nonce::assume_unique_for_key([0u8; AES_NONCE_SIZE]);
        aead.seal_in_place_append_tag(nonce, Aad::from(&h), &mut in_out)
            .map_err(|_| NoiseError::Encrypt)?;

        mix_hash(&mut h, &in_out[..]);
        writer.write(&in_out[..]).map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        let (k1, k2) = hkdf_split(&ck, None)?;
        // Responder: Write=k2, Read=k1 (Spec: split returns (temp_k1, temp_k2). Alice uses k1 to write, Bob uses k1 to read.)
        // Initiator (Alice): new(k1, k2, rs).
        // Responder (Bob): new(k2, k1, rs).
        Ok(NoiseSession::new(k2, k1, rs))
    }
}

impl NoiseSession {
     fn new(write_key: Vec<u8>, read_key: Vec<u8>, remote_public_key: PublicKey) -> Self {
        Self {
            valid: true,
            remote_public_key,
            write_key,
            write_nonce: 0,
            read_key,
            read_nonce: 0,
        }
    }

    pub fn write_message_in_place(&mut self, message: &mut [u8]) -> Result<Vec<u8>, NoiseError> {
        if !self.valid { return Err(NoiseError::SessionClosed); }
        if message.len() > MAX_SIZE_NOISE_MSG - AES_GCM_TAGLEN { return Err(NoiseError::PayloadTooLarge); }

        let write_key = aes_key(&self.write_key);
        let mut nonce = [0u8; 4].to_vec();
        nonce.extend_from_slice(&self.write_nonce.to_be_bytes());
        let nonce = aead::Nonce::assume_unique_for_key(nonce.try_into().unwrap());
        
        let tag = write_key.seal_in_place_separate_tag(nonce, Aad::empty(), message)
            .map_err(|_| NoiseError::Encrypt)?;

        self.write_nonce = self.write_nonce.checked_add(1).ok_or(NoiseError::NonceOverflow)?;
        Ok(tag.as_ref().to_vec())
    }

    pub fn read_message_in_place<'a>(&mut self, message: &'a mut [u8]) -> Result<&'a [u8], NoiseError> {
        if !self.valid { return Err(NoiseError::SessionClosed); }
        let len = message.len();
        if len > MAX_SIZE_NOISE_MSG {
             self.valid = false;
             return Err(NoiseError::ReceivedMsgTooLarge);
        }
        if len < AES_GCM_TAGLEN {
             self.valid = false;
             return Err(NoiseError::ResponseBufferTooSmall);
        }

        let read_key = aes_key(&self.read_key);
        let mut nonce = [0u8; 4].to_vec();
        nonce.extend_from_slice(&self.read_nonce.to_be_bytes());
        let nonce = aead::Nonce::assume_unique_for_key(nonce.try_into().unwrap());

        read_key.open_in_place(nonce, Aad::empty(), message).map_err(|_| {
             self.valid = false;
             NoiseError::Decrypt
        })?;

        let (buffer, _) = message.split_at_mut(len - AES_GCM_TAGLEN);
        self.read_nonce = self.read_nonce.checked_add(1).ok_or(NoiseError::NonceOverflow)?;
        Ok(buffer)
    }
    
    pub fn get_remote_static(&self) -> PublicKey {
        self.remote_public_key
    }
}

// Helpers

fn aes_key(key: &[u8]) -> LessSafeKey {
    LessSafeKey::new(UnboundKey::new(&aead::AES_256_GCM, key).expect("Unexpected AES key length"))
}

fn hash(data: &[u8]) -> Vec<u8> {
    sha2::Sha256::digest(data).to_vec()
}

fn mix_hash(h: &mut Vec<u8>, data: &[u8]) {
    h.extend_from_slice(data);
    *h = hash(h);
}

fn hkdf_split(ck: &[u8], dh_output: Option<&[u8]>) -> Result<(Vec<u8>, Vec<u8>), NoiseError> {
    let dh_output = dh_output.unwrap_or(&[]);
    
    // Hkdf::extract(salt, ikm)
    // Noise spec: "HKDF(ck, dh_output, 2)" where ck is salt, dh_output is ikm.
    // If dh_output is empty, we pass empty slice.
    
    // Hkdf::new(salt, ikm) handles extract internally and returns Hkdf object ready for expand
    let hk = Hkdf::<sha2::Sha256>::new(Some(ck), dh_output);
    
    let mut okm = [0u8; 64];
    hk.expand(&[], &mut okm).map_err(|_| NoiseError::Hkdf)?;
    
    let (k1, k2) = okm.split_at(32);
    Ok((k1.to_vec(), k2.to_vec()))
}

fn mix_key(ck: &mut Vec<u8>, dh_output: &[u8]) -> Result<Vec<u8>, NoiseError> {
    let (new_ck, k) = hkdf_split(ck, Some(dh_output))?;
    *ck = new_ck;
    Ok(k)
}

pub const fn encrypted_len(plaintext_len: usize) -> usize {
    plaintext_len + AES_GCM_TAGLEN
}

pub const fn handshake_init_msg_len(payload_len: usize) -> usize {
    32 + encrypted_len(32) + encrypted_len(payload_len)
}

pub const fn handshake_resp_msg_len(payload_len: usize) -> usize {
    32 + encrypted_len(payload_len)
}
