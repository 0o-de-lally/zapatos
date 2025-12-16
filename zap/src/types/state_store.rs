use serde::{Deserialize, Serialize};
use move_core_types::account_address::AccountAddress;



// Actually, StateKey in aptos-types:
// impl Deserialize -> StateKeyInner::deserialize -> StateKey::from_deserialized(inner)
// Use #[serde(from = "StateKeyInner", into = "StateKeyInner")] if we want conversion.
// For now, let's just make StateKey a wrapper around StateKeyInner and use #[serde(from...)] logic?
// No, simpler: if I define it as a struct wrapping Inner, standard derive(Deserialize) expects a struct with "inner" field.
// BUT StateKeyInner is the *entire* wire content.
// So I should use:
// #[derive(Deserialize)]
// #[serde(transparent)]
// pub struct StateKey(pub StateKeyInner);

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StateKey {
    pub inner: StateKeyInner,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum StateKeyInner {
    AccessPath(AccessPath),
    TableItem {
        handle: AccountAddress,
        #[serde(with = "serde_bytes")]
        key: Vec<u8>,
    },
    #[serde(with = "serde_bytes")]
    Raw(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AccessPath {
    pub address: AccountAddress,
    #[serde(with = "serde_bytes")]
    pub path: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "StateValueMetadata")]
pub enum PersistedStateValueMetadata {
    V0 {
        deposit: u64,
        creation_time_usecs: u64,
    },
    V1 {
        slot_deposit: u64,
        bytes_deposit: u64,
        creation_time_usecs: u64,
    },
}

impl PersistedStateValueMetadata {
    pub fn into_in_mem_form(self) -> StateValueMetadata {
        match self {
            PersistedStateValueMetadata::V0 {
                deposit,
                creation_time_usecs,
            } => StateValueMetadata::new(deposit, 0, creation_time_usecs),
            PersistedStateValueMetadata::V1 {
                slot_deposit,
                bytes_deposit,
                creation_time_usecs,
            } => StateValueMetadata::new(slot_deposit, bytes_deposit, creation_time_usecs),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateValueMetadata {
    // Simplified: aptos-types uses Option<StateValueMetadataInner>
    // but here we can just store fields?
    // Wait, aptos-types StateValueMetadata can be "none".
    // "StateValueMetadata::none()" -> inner is None.
    pub inner: Option<StateValueMetadataInner>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateValueMetadataInner {
    pub slot_deposit: u64,
    pub bytes_deposit: u64,
    pub creation_time_usecs: u64,
}

impl StateValueMetadata {
    pub fn new(slot_deposit: u64, bytes_deposit: u64, creation_time_usecs: u64) -> Self {
        Self {
            inner: Some(StateValueMetadataInner {
                slot_deposit,
                bytes_deposit,
                creation_time_usecs,
            }),
        }
    }
    pub fn none() -> Self {
        Self { inner: None }
    }

    pub fn into_persistable(self) -> Option<PersistedStateValueMetadata> {
        self.inner.map(|inner| {
            let StateValueMetadataInner {
                slot_deposit,
                bytes_deposit,
                creation_time_usecs,
            } = inner;
            if bytes_deposit == 0 {
                PersistedStateValueMetadata::V0 {
                    deposit: slot_deposit,
                    creation_time_usecs,
                }
            } else {
                PersistedStateValueMetadata::V1 {
                    slot_deposit,
                    bytes_deposit,
                    creation_time_usecs,
                }
            }
        })
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "StateValue")]
pub enum PersistedStateValue {
    V0(Vec<u8>), // aptos-types uses Bytes, Vec<u8> is compatible
    WithMetadata {
        data: Vec<u8>,
        metadata: PersistedStateValueMetadata,
    },
}

impl PersistedStateValue {
    pub fn into_state_value(self) -> StateValue {
        match self {
            PersistedStateValue::V0(data) => StateValue {
                data,
                metadata: StateValueMetadata::none(),
            },
            PersistedStateValue::WithMetadata { data, metadata } => StateValue {
                data,
                metadata: metadata.into_in_mem_form(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateValue {
    pub data: Vec<u8>,
    pub metadata: StateValueMetadata,
}

impl Serialize for StateValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.clone().into_persistable().serialize(serializer)
    }
}

impl StateValue {
    pub fn into_persistable(self) -> PersistedStateValue {
        let metadata = self.metadata.into_persistable();
        match metadata {
            None => PersistedStateValue::V0(self.data),
            Some(metadata) => PersistedStateValue::WithMetadata {
                data: self.data,
                metadata,
            },
        }
    }
}

impl<'de> Deserialize<'de> for StateValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let persisted = PersistedStateValue::deserialize(deserializer)?;
        Ok(persisted.into_state_value())
    }
}

// Add conversions for PersistedStateValueMetadata if necessary
