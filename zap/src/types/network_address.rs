/// Simplified NetworkAddress parser for peer discovery
/// 
/// This is a minimal implementation to extract peer information from
/// BCS-encoded network addresses without importing aptos-types.

use serde::{Deserialize, Deserializer, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};

/// DNS name wrapper - must match aptos-types encoding
#[derive(Clone, Debug, Serialize)]
pub struct DnsName(pub String);

impl<'de> Deserialize<'de> for DnsName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapper(String);
        
        let wrapper = Wrapper::deserialize(deserializer)?;
        Ok(DnsName(wrapper.0))
    }
}

/// A simplified network address that can be deserialized from BCS
/// This is a newtype wrapper around Vec<Protocol>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct NetworkAddress {
    protocols: Vec<Protocol>,
}

/// Protocol stack components
/// IMPORTANT: Variant order must match aptos-types for BCS compatibility
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Protocol {
    Ip4(Ipv4Addr),        // variant 0
    Ip6(Ipv6Addr),        // variant 1  
    Dns(DnsName),         // variant 2
    Dns4(DnsName),        // variant 3
    Dns6(DnsName),        // variant 4
    Tcp(u16),             // variant 5
    Memory(u16),          // variant 6
    NoiseIK([u8; 32]),    // variant 7 - x25519 public key
    Handshake(u8),        // variant 8
}

impl NetworkAddress {
    /// Extract the x25519 peer ID from NoiseIK protocol
    pub fn find_noise_proto(&self) -> Option<[u8; 32]> {
        self.protocols.iter().find_map(|proto| match proto {
            Protocol::NoiseIK(pubkey) => Some(*pubkey),
            _ => None,
        })
    }

    /// Extract the TCP port
    pub fn find_port(&self) -> Option<u16> {
        self.protocols.iter().find_map(|proto| match proto {
            Protocol::Tcp(port) => Some(*port),
            _ => None,
        })
    }

    /// Extract DNS name
    pub fn find_dns_name(&self) -> Option<String> {
        self.protocols.iter().find_map(|proto| match proto {
            Protocol::Dns(name) | Protocol::Dns4(name) | Protocol::Dns6(name) => {
                Some(name.0.clone())
            }
            _ => None,
        })
    }
}
