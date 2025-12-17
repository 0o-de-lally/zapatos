use crate::crypto::bls12381::{PublicKey, Signature};
use crate::types::bitvec::BitVec;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AggregateSignature {
    validator_bitmask: BitVec,
    sig: Option<Signature>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatorConsensusInfo {
    pub address: AccountAddress,
    pub public_key: PublicKey,
    pub voting_power: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ValidatorVerifier {
    pub validator_infos: Vec<ValidatorConsensusInfo>,
}
// Note: The real implementation has internal maps and voting power calculations.
// We only implement the data container for deserialization purposes.
