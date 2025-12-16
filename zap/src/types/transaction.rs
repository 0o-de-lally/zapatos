use serde::{Deserialize, Serialize};
use crate::crypto::HashValue;
use crate::types::write_set::WriteSet;
use crate::types::contract_event::ContractEvent;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Transaction {
    UserTransaction(Vec<u8>), // Placeholder
    GenesisTransaction(WriteSetPayload),
    BlockMetadata(Vec<u8>),   // Placeholder
    StateCheckpoint(HashValue),
    ValidatorTransaction(Vec<u8>), // Placeholder
    BlockMetadataExt(Vec<u8>), // Placeholder
    BlockEpilogue(Vec<u8>), // Placeholder
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WriteSetPayload {
    Direct(ChangeSet),
    Script {
        execute_as: Vec<u8>, // AccountAddress placeholder
        script: Vec<u8>, // Script placeholder
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChangeSet {
    pub write_set: WriteSet,
    pub events: Vec<ContractEvent>,
}
