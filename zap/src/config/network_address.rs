use serde::{Deserialize, Serialize, Serializer, Deserializer, de};
use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};
use x25519_dalek::PublicKey;

// See zapatos/types/src/network_address/mod.rs

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Protocol {
    Ip4(Ipv4Addr),   // 0
    Ip6(Ipv6Addr),   // 1
    Dns(String),     // 2 (Simplified DnsName)
    Dns4(String),    // 3
    Dns6(String),    // 4
    Tcp(u16),        // 5
    Memory(u16),     // 6
    NoiseIK(#[serde(with = "x25519_serde")] PublicKey), // 7
    Handshake(u8),   // 8
}

mod x25519_serde {
    use super::*;
    use serde::{de, Serializer, Deserializer};

    pub fn serialize<S>(key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
             serializer.serialize_str(&hex::encode(key.as_bytes()))
        } else {
             // For BCS, we want to serialize as bytes.
             // But careful: Zapatos might serialize PublicKey as `[u8; 32]`.
             // `serde_bytes` is often used.
             // If we just serialize bytes, it becomes `len + bytes` (Vec) or just bytes (tuple)?
             // PublicKey is 32 bytes.
             // We should probably serialize as `[u8; 32]`.
             // Let's coerce to [u8; 32].
             let bytes = *key.as_bytes();
             bytes.serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
             let s = String::deserialize(deserializer)?;
             let bytes = hex::decode(s).map_err(de::Error::custom)?;
             let array: [u8; 32] = bytes.try_into().map_err(|_| de::Error::custom("invalid key length"))?;
             Ok(PublicKey::from(array))
        } else {
             // Zapatos serializes PublicKey as a length-prefixed byte vector (32 bytes).
             // [u8; 32] deserialization expects fixed bytes without length.
             // Vec<u8> deserialization expects length prefix.
             let bytes = <Vec<u8>>::deserialize(deserializer)?;
             if bytes.len() != 32 {
                 return Err(de::Error::custom(format!("invalid key length: {}", bytes.len())));
             }
             let array: [u8; 32] = bytes.try_into().map_err(|_| de::Error::custom("should be 32 bytes"))?;
             Ok(PublicKey::from(array))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkAddress(pub Vec<Protocol>);

impl NetworkAddress {
    pub fn find_ip_addr(&self) -> Option<IpAddr> {
        self.0.iter().find_map(|proto| match proto {
            Protocol::Ip4(addr) => Some(IpAddr::V4(*addr)),
            Protocol::Ip6(addr) => Some(IpAddr::V6(*addr)),
            _ => None,
        })
    }
    
    pub fn find_dns_name(&self) -> Option<String> {
        self.0.iter().find_map(|proto| match proto {
            Protocol::Dns(s) | Protocol::Dns4(s) | Protocol::Dns6(s) => Some(s.clone()),
            _ => None,
        })
    }

    pub fn find_port(&self) -> Option<u16> {
        self.0.iter().find_map(|proto| match proto {
            Protocol::Tcp(port) => Some(*port),
            _ => None,
        })
    }

    pub fn find_noise_proto(&self) -> Option<PublicKey> {
        self.0.iter().find_map(|proto| match proto {
            Protocol::NoiseIK(pubkey) => Some(*pubkey),
            _ => None,
        })
    }
}

// Custom Serde to match Zapatos NetworkAddress serialization (BCS wrapping)
// In Zapatos:
// Serialize: Wrapper(bcs::to_bytes(&protocols)) -> Serializer
// Deserialize: Wrapper -> bcs::from_bytes -> Vec<Protocol>

impl Serialize for NetworkAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            // Simplify for now: implement Display/FromStr if needed, or just fail
             serializer.serialize_str("NetworkAddress(..)") 
        } else {
             #[derive(Serialize)]
             #[serde(rename = "NetworkAddress")]
             struct Wrapper<'a>(#[serde(with = "serde_bytes")] &'a [u8]);

             bcs::to_bytes(&self.0)
                 .map_err(serde::ser::Error::custom)
                 .and_then(|v| Wrapper(&v).serialize(serializer))
        }
    }
}

impl<'de> Deserialize<'de> for NetworkAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
             // Not implemented for now in this simplified version
             let _s = <String>::deserialize(deserializer)?;
             Ok(NetworkAddress(vec![])) 
        } else {
             #[derive(Deserialize)]
             #[serde(rename = "NetworkAddress")]
             struct Wrapper(#[serde(with = "serde_bytes")] Vec<u8>);

             Wrapper::deserialize(deserializer)
                 .and_then(|v| bcs::from_bytes(&v.0).map_err(de::Error::custom))
                 .map(NetworkAddress)
        }
    }
}
