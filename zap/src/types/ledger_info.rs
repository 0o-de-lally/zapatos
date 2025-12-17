use crate::crypto::hash::HashValue;
use crate::types::block_info::BlockInfo;
use crate::types::validator_verifier::AggregateSignature;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfo {
    pub commit_info: BlockInfo,
    pub consensus_data_hash: HashValue,
}

impl LedgerInfo {
    pub fn version(&self) -> u64 {
        self.commit_info.version
    }

    pub fn epoch(&self) -> u64 {
        self.commit_info.epoch
    }
    
    pub fn timestamp_usecs(&self) -> u64 {
        self.commit_info.timestamp_usecs
    }
    
    pub fn consensus_block_id(&self) -> HashValue {
        self.commit_info.id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfoWithV0 {
    pub ledger_info: LedgerInfo,
    pub signatures: AggregateSignature,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LedgerInfoWithSignatures {
    V0(LedgerInfoWithV0),
}

impl LedgerInfoWithSignatures {
    pub fn ledger_info(&self) -> &LedgerInfo {
        match self {
            LedgerInfoWithSignatures::V0(l) => &l.ledger_info,
        }
    }
}

impl Display for LedgerInfoWithSignatures {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let info = self.ledger_info();
        write!(f, "LedgerInfo(v={}, epoch={}, ts={})", info.version(), info.epoch(), info.timestamp_usecs())
    }
}
