use serde::{Deserialize, Serialize};
use move_core_types::language_storage::TypeTag;
use move_core_types::account_address::AccountAddress;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContractEvent {
    V1(ContractEventV1),
    V2(ContractEventV2),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContractEventV1 {
    pub key: EventKey,
    pub sequence_number: u64,
    pub type_tag: TypeTag,
    #[serde(with = "serde_bytes")]
    pub event_data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContractEventV2 {
    pub type_tag: TypeTag,
    #[serde(with = "serde_bytes")]
    pub event_data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct EventKey {
    creation_number: u64,
    account_address: AccountAddress,
}

impl EventKey {
    pub fn new(creation_number: u64, account_address: AccountAddress) -> Self {
        Self {
            creation_number,
            account_address,
        }
    }
}


