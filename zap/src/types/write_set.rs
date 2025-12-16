use crate::types::state_store::{StateKey, StateValue, StateValueMetadata, PersistedStateValueMetadata};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WriteSet {
    value: ValueWriteSet,
    hotness: BTreeMap<StateKey, HotStateOp>, 
}

impl Serialize for WriteSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WriteSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let value = ValueWriteSet::deserialize(deserializer)?;
        Ok(Self {
            value,
            hotness: BTreeMap::new(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ValueWriteSet {
    V0(WriteSetV0),
}
impl Default for ValueWriteSet { fn default() -> Self { Self::V0(WriteSetV0::default()) } }

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct WriteSetV0(pub WriteSetMut);

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct WriteSetMut {
    pub write_set: BTreeMap<StateKey, WriteOp>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteOp(pub BaseStateOp);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BaseStateOp {
    Creation(StateValue),
    Modification(StateValue),
    Deletion(StateValueMetadata),
    MakeHot,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HotStateOp(pub BaseStateOp);

#[derive(Serialize, Deserialize)]
#[serde(rename = "WriteOp")]
pub enum PersistedWriteOp {
    Creation(Vec<u8>),
    Modification(Vec<u8>),
    Deletion,
    CreationWithMetadata {
        data: Vec<u8>,
        metadata: PersistedStateValueMetadata,
    },
    ModificationWithMetadata {
        data: Vec<u8>,
        metadata: PersistedStateValueMetadata,
    },
    DeletionWithMetadata {
        metadata: PersistedStateValueMetadata,
    },
}

impl WriteOp {
    pub fn metadata(&self) -> &StateValueMetadata {
        match &self.0 {
            BaseStateOp::Creation(v) | BaseStateOp::Modification(v) => &v.metadata,
            BaseStateOp::Deletion(meta) => meta,
            BaseStateOp::MakeHot => unreachable!("malformed write op"),
        }
    }

    pub fn to_persistable(&self) -> PersistedWriteOp {
         let metadata = self.metadata().clone().into_persistable();
         match metadata {
             None => match &self.0 {
                 BaseStateOp::Creation(v) => PersistedWriteOp::Creation(v.data.clone()),
                 BaseStateOp::Modification(v) => PersistedWriteOp::Modification(v.data.clone()),
                 BaseStateOp::Deletion(_) => PersistedWriteOp::Deletion,
                 BaseStateOp::MakeHot => unreachable!("malformed write op"),
             },
             Some(metadata) => match &self.0 {
                 BaseStateOp::Creation(v) => PersistedWriteOp::CreationWithMetadata {
                     data: v.data.clone(),
                     metadata,
                 },
                 BaseStateOp::Modification(v) => PersistedWriteOp::ModificationWithMetadata {
                     data: v.data.clone(),
                     metadata,
                 },
                 BaseStateOp::Deletion(_) => PersistedWriteOp::DeletionWithMetadata { metadata },
                 BaseStateOp::MakeHot => unreachable!("malformed write op"),
             },
         }
    }

    pub fn from_persisted(op: PersistedWriteOp) -> Self {
        use PersistedWriteOp::*;
        match op {
            Creation(data) => Self(BaseStateOp::Creation(StateValue { data, metadata: StateValueMetadata::none() })),
            Modification(data) => Self(BaseStateOp::Modification(StateValue { data, metadata: StateValueMetadata::none() })),
            Deletion => Self(BaseStateOp::Deletion(StateValueMetadata::none())),
            CreationWithMetadata { data, metadata } => Self(BaseStateOp::Creation(StateValue { 
                data, 
                metadata: metadata.into_in_mem_form() 
            })),
            ModificationWithMetadata { data, metadata } => Self(BaseStateOp::Modification(StateValue { 
                data, 
                metadata: metadata.into_in_mem_form() 
            })),
            DeletionWithMetadata { metadata } => Self(BaseStateOp::Deletion(metadata.into_in_mem_form())),
        }
    }
}

impl Serialize for WriteOp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.to_persistable().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WriteOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let persisted = PersistedWriteOp::deserialize(deserializer)?;
        Ok(Self::from_persisted(persisted))
    }
}
