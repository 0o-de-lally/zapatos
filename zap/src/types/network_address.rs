/// Simplified NetworkAddress parser for peer discovery
/// 
/// This is a minimal implementation to extract peer information from
/// BCS-encoded network addresses without importing aptos-types.

use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, Ipv6Addr};

/// A simplified network address that can be deserialized from BCS
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkAddress {
    protocols: Vec<Protocol>,
}

/// Protocol stack components
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Protocol {
    Ip4(Ipv4Addr),
    Ip6(Ipv6Addr),
    Dns(String),
    Dns4(String),
    Dns6(String),
    Tcp(u16),
    Memory(u16),
    NoiseIK([u8; 32]),  // x25519 public key
    Handshake(u8),
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
            Protocol::Dns(name) | Protocol::Dns4(name) | Protocol::Dns6(name) => Some(name.clone()),
            _ => None,
        })
    }
}
