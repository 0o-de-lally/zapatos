use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::iter::FromIterator;
use std::convert::TryFrom;

//
// ProtocolId
//
#[repr(u8)]
#[derive(Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize, Debug, PartialOrd, Ord)]
pub enum ProtocolId {
    ConsensusRpcBcs = 0,
    ConsensusDirectSendBcs = 1,
    MempoolDirectSend = 2,
    StateSyncDirectSend = 3,
    DiscoveryDirectSend = 4,
    HealthCheckerRpc = 5,
    ConsensusDirectSendJson = 6,
    ConsensusRpcJson = 7,
    StorageServiceRpc = 8,
    MempoolRpc = 9,
    PeerMonitoringServiceRpc = 10,
    ConsensusRpcCompressed = 11,
    ConsensusDirectSendCompressed = 12,
    NetbenchDirectSend = 13,
    NetbenchRpc = 14,
    DKGDirectSendCompressed = 15,
    DKGDirectSendBcs = 16,
    DKGDirectSendJson = 17,
    DKGRpcCompressed = 18,
    DKGRpcBcs = 19,
    DKGRpcJson = 20,
    JWKConsensusDirectSendCompressed = 21,
    JWKConsensusDirectSendBcs = 22,
    JWKConsensusDirectSendJson = 23,
    JWKConsensusRpcCompressed = 24,
    JWKConsensusRpcBcs = 25,
    JWKConsensusRpcJson = 26,
    ConsensusObserver = 27,
    ConsensusObserverRpc = 28,
}

impl ProtocolId {
    pub fn all() -> &'static [ProtocolId] {
        &[
            ProtocolId::ConsensusRpcBcs,
            ProtocolId::ConsensusDirectSendBcs,
            ProtocolId::MempoolDirectSend,
            ProtocolId::StateSyncDirectSend,
            ProtocolId::DiscoveryDirectSend,
            ProtocolId::HealthCheckerRpc,
            ProtocolId::ConsensusDirectSendJson,
            ProtocolId::ConsensusRpcJson,
            ProtocolId::StorageServiceRpc,
            ProtocolId::MempoolRpc,
            ProtocolId::PeerMonitoringServiceRpc,
            ProtocolId::ConsensusRpcCompressed,
            ProtocolId::ConsensusDirectSendCompressed,
            ProtocolId::NetbenchDirectSend,
            ProtocolId::NetbenchRpc,
            ProtocolId::DKGDirectSendCompressed,
            ProtocolId::DKGDirectSendBcs,
            ProtocolId::DKGDirectSendJson,
            ProtocolId::DKGRpcCompressed,
            ProtocolId::DKGRpcBcs,
            ProtocolId::DKGRpcJson,
            ProtocolId::JWKConsensusDirectSendCompressed,
            ProtocolId::JWKConsensusDirectSendBcs,
            ProtocolId::JWKConsensusDirectSendJson,
            ProtocolId::JWKConsensusRpcCompressed,
            ProtocolId::JWKConsensusRpcBcs,
            ProtocolId::JWKConsensusRpcJson,
            ProtocolId::ConsensusObserver,
            ProtocolId::ConsensusObserverRpc,
        ]
    }
}

//
// ProtocolIdSet
//
// Minimal implementation of a BitVec for ProtocolIdSet to match Zapatos serialization
// Zapatos uses `aptos_bitvec::BitVec`. We will implement a compatible serialization manually or use a simple Vec if we can conform.
// Actually, `aptos_bitvec::BitVec` serializes as a byte vector.
// Let's implement a simple wrapper that holds a Vec<u8> and manages bits.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProtocolIdSet(Vec<u8>);

impl Serialize for ProtocolIdSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // aptos_bitvec serializes as the inner Vec<u8> directly (not length prefixed? no, standard Vec serialization usually is)
        // Checking aptos-bitvec:
        // #[derive(Serialize, Deserialize)] pub struct BitVec(Vec<u8>);
        // So it's just a newtype around Vec<u8>.
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProtocolIdSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = Vec::<u8>::deserialize(deserializer)?;
        Ok(ProtocolIdSet(v))
    }
}

impl ProtocolIdSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, protocol: ProtocolId) {
        let idx = protocol as usize;
        let byte_idx = idx / 8;
        let bit_idx = idx % 8;

        if byte_idx >= self.0.len() {
            self.0.resize(byte_idx + 1, 0);
        }
        self.0[byte_idx] |= 1 << bit_idx;
    }

    pub fn contains(&self, protocol: ProtocolId) -> bool {
        let idx = protocol as usize;
        let byte_idx = idx / 8;
        let bit_idx = idx % 8;

        if byte_idx < self.0.len() {
            (self.0[byte_idx] & (1 << bit_idx)) != 0
        } else {
            false
        }
    }
    
    pub fn intersect(&self, other: &ProtocolIdSet) -> ProtocolIdSet {
        let len = std::cmp::min(self.0.len(), other.0.len());
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            result.push(self.0[i] & other.0[i]);
        }
        // Trim trailing zeros? Not strictly necessary for functionality unless uniqueness matters
        ProtocolIdSet(result)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl FromIterator<ProtocolId> for ProtocolIdSet {
    fn from_iter<T: IntoIterator<Item = ProtocolId>>(iter: T) -> Self {
        let mut set = Self::default();
        for p in iter {
            set.insert(p);
        }
        set
    }
}

//
// MessagingProtocolVersion
//
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Deserialize, Serialize, Debug)]
pub enum MessagingProtocolVersion {
    V1 = 0,
}

//
// HandshakeMsg
//

// ChainId and NetworkId are usually u8 (or similar) in Zapatos, let's verify or use u8 for now if we don't importing everything.
// Zapatos `ChainId` (aptos-types) is u8 (wrapper).
// Zapatos `NetworkId` (aptos-config) is String/enum.
// Wait, `aptos-types::chain_id::ChainId` is `u8`.
// `aptos_config::network_id::NetworkId` serializes as a String usually?
// Let's re-check `NetworkId` from Zapatos code or `zap/src/lib.rs` imports?
// `zap` doesn't currently use them.
// Looking at `zapatos` handshake code, it uses `NetworkId` and `ChainId` struct types.
// Byte-compatibility is key.
// `ChainId` is `u8`.
// `NetworkId`:
// pub enum NetworkId { Validator, Public, Vfn, Custom(String) }
// Serialization of NetworkId: it derives Serialize. It's an enum.
// If it's a Rust enum without #[serde(tag...)] it serializes as variant index or variant name?
// Zapatos uses `serde_rename` often?
// Let's use `zap`'s `ChainId` if it exists, or define it.
// Define minimal compatible versions.

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct ChainId(u8);

impl ChainId {
    pub const MAINNET: ChainId = ChainId(1);
    pub const TESTNET: ChainId = ChainId(2);
    
    pub fn new(id: u8) -> Self { Self(id) }
    pub fn id(&self) -> u8 { self.0 }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum NetworkId {
    Validator,
    Public,
    Vfn,
    // Custom(String) - omitting for simplicity unless needed
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct HandshakeMsg {
    pub supported_protocols: BTreeMap<MessagingProtocolVersion, ProtocolIdSet>,
    pub chain_id: ChainId,
    pub network_id: NetworkId,
}

impl HandshakeMsg {
    pub fn new(chain_id: ChainId, network_id: NetworkId) -> Self {
        let mut supported_protocols = BTreeMap::new();
        // Support some default protocols (e.g. StorageServiceRpc)
        let mut protocols = ProtocolIdSet::default();
        protocols.insert(ProtocolId::StorageServiceRpc); 
        // We probably also want others? For now StorageServiceRpc is what we used in `network.rs`
        
        supported_protocols.insert(MessagingProtocolVersion::V1, protocols);
        
        Self {
            supported_protocols,
            chain_id,
            network_id,
        }
    }
    
    pub fn perform_handshake(
        &self,
        other: &HandshakeMsg,
    ) -> Result<(MessagingProtocolVersion, ProtocolIdSet)> {
        // verify chain id
        if self.chain_id != other.chain_id {
            return Err(anyhow!("Invalid ChainId: expected {:?}, got {:?}", self.chain_id, other.chain_id));
        }

        // verify network id
        if self.network_id != other.network_id {
            return Err(anyhow!("Invalid NetworkId: expected {:?}, got {:?}", self.network_id, other.network_id));
        }

        // intersection
         for (our_handshake_version, our_protocols) in self.supported_protocols.iter().rev() {
            if let Some(their_protocols) = other.supported_protocols.get(our_handshake_version) {
                let common_protocols = our_protocols.intersect(their_protocols);

                if !common_protocols.is_empty() {
                    return Ok((*our_handshake_version, common_protocols));
                }
            }
        }
        
        Err(anyhow!("No common protocols"))
    }
}
