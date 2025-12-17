use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

// BLS12-381 Public Key (compressed) is 48 bytes
// BLS12-381 Signature (compressed) is 96 bytes

#[derive(Clone, Eq, PartialEq)]
pub struct PublicKey {
    bytes: Vec<u8>, // Should be 48 bytes
}

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 48 {
            return Err(format!("Invalid public key length: {}", bytes.len()));
        }
        Ok(Self { bytes: bytes.to_vec() })
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", hex::encode(&self.bytes))
    }
}

use serde::de::Visitor;

// Custom serialization to match `aptos-crypto` behavior
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        if serializer.is_human_readable() {
             serializer.serialize_str(&hex::encode(&self.bytes))
        } else {
             // We serialize as a byte slice, but since we want it to be compatible 
             // with [u8; 48], we should be careful. 
             // BCS treats [u8; N] as just bytes.
             // BCS treats &[u8] as variable length bytes (length prefix).
             // However, `aptos-crypto` uses `[u8; 48]` which serializes as 48 bytes.
             // If we use `serializer.serialize_bytes`, serde/bcs adds a length prefix!
             // We MUST serialize as a tuple/seq if we want compatibility with `[u8; 48]` in some formats,
             // BUT `bcs` crate optimization for `[u8; N]` is specific to arrays.
             // To emulate [u8; 48] in BCS without actually having [u8; 48] type support in Serde generic:
             // We have to use a Tuple Struct or Tuple of 48 elements? No.
             // 
             // Actually, `serde_bytes::Bytes` typically serializes as `bytes` type.
             // In BCS, `bytes` type is length-prefixed.
             // We NEED to serialize as a fixed-length sequence to match `[u8; 48]`.
             // 
             // WORKAROUND: Cast to [u8; 48] and serialize it. 
             // But we can't cast Vec.
             // We can create a temporary array.
             let mut array = [0u8; 48];
             array.copy_from_slice(&self.bytes);
             // Serde for [u8; 48] is... ?
             // If I use `serde_big_array` or similar it works. 
             // But by default `[u8; 48]` doesn't impl Serialize.
             // 
             // WAIT. `aptos-crypto` implements `Serialize` for `PublicKey` by delegating to `self.to_bytes().serialize(serializer)`.
             // `bls12381::PublicKey::to_bytes()` returns `[u8; 48]`.
             // So `aptos-crypto` depends on `[u8; 48]` implementing Serialize.
             // It seems standard Serde started supporting larger arrays via const generics recently or `aptos` enables a feature.
             // 
             // If `zap` compiles with the same toolchain and deps, maybe it supports it?
             // But `zap` failed with "no function ... deserialize for [u8; 48]".
             // This suggests Serde wasn't supporting it or it wasn't imported.
             
             // I will implement a serializer that serializes each byte individually as a tuple/seq 
             // to simulate a fixed array? 
             // No, BCS [u8; N] is just N bytes.
             // Tuple of 48 u8s?
             
             // Let's try `serializer.serialize_tuple(48)`.
             use serde::ser::SerializeTuple;
             let mut tup = serializer.serialize_tuple(48)?;
             for byte in &self.bytes {
                 tup.serialize_element(byte)?;
             }
             tup.end()
        }
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        if deserializer.is_human_readable() {
             let s = String::deserialize(deserializer)?;
             let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
             PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
        } else {
             struct ArrayVisitor;
             impl<'de> Visitor<'de> for ArrayVisitor {
                 type Value = PublicKey;
                 fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                     formatter.write_str("a byte array of length 48")
                 }
                 
                 fn visit_seq<A>(self, mut seq: A) -> Result<PublicKey, A::Error>
                 where A: serde::de::SeqAccess<'de> {
                     let mut bytes = Vec::with_capacity(48);
                     for _ in 0..48 {
                         if let Some(byte) = seq.next_element()? {
                             bytes.push(byte);
                         } else {
                             return Err(serde::de::Error::invalid_length(bytes.len(), &self));
                         }
                     }
                     // If there are more? BCS for fixed array [u8; 48] just reads 48 items.
                     PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
                 }
             }
             
             // Tell deserializer we expect a tuple of size 48
             deserializer.deserialize_tuple(48, ArrayVisitor)
        }
    }
}


#[derive(Clone, Eq, PartialEq)]
pub struct Signature {
    bytes: Vec<u8>, // Should be 96 bytes
}

impl Signature {
    pub fn dummy_signature() -> Self {
        Self { bytes: vec![0u8; 96] }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
         if bytes.len() != 96 {
            return Err(format!("Invalid signature length: {}", bytes.len()));
        }
        Ok(Self { bytes: bytes.to_vec() })
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({})", hex::encode(&self.bytes))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        if serializer.is_human_readable() {
             serializer.serialize_str(&hex::encode(&self.bytes))
        } else {
             use serde::ser::SerializeTuple;
             let mut tup = serializer.serialize_tuple(96)?;
             for byte in &self.bytes {
                 tup.serialize_element(byte)?;
             }
             tup.end()
        }
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        if deserializer.is_human_readable() {
             let s = String::deserialize(deserializer)?;
             let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
             Signature::from_bytes(&bytes).map_err(serde::de::Error::custom)
        } else {
             struct ArrayVisitor;
             impl<'de> Visitor<'de> for ArrayVisitor {
                 type Value = Signature;
                 fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                     formatter.write_str("a byte array of length 96")
                 }
                 
                 fn visit_seq<A>(self, mut seq: A) -> Result<Signature, A::Error>
                 where A: serde::de::SeqAccess<'de> {
                     let mut bytes = Vec::with_capacity(96);
                     for _ in 0..96 {
                         if let Some(byte) = seq.next_element()? {
                             bytes.push(byte);
                         } else {
                             return Err(serde::de::Error::invalid_length(bytes.len(), &self));
                         }
                     }
                     Signature::from_bytes(&bytes).map_err(serde::de::Error::custom)
                 }
             }
             deserializer.deserialize_tuple(96, ArrayVisitor)
        }
    }
}
