use crate::crypto::hash::HashValue;
use crate::types::epoch_state::EpochState;
use serde::{Deserialize, Serialize};

pub type Round = u64;
pub type Version = u64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockInfo {
    pub epoch: u64,
    pub round: Round,
    pub id: HashValue,
    pub executed_state_id: HashValue,
    pub version: Version,
    pub timestamp_usecs: u64,
    pub next_epoch_state: Option<EpochState>,
}

impl BlockInfo {
    pub fn empty() -> Self {
        Self {
            epoch: 0,
            round: 0,
            id: HashValue::zero(),
            executed_state_id: HashValue::zero(),
            version: 0,
            timestamp_usecs: 0,
            next_epoch_state: None,
        }
    }
}
