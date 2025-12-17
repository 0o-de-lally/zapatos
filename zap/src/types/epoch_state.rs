use crate::types::validator_verifier::ValidatorVerifier;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpochState {
    pub epoch: u64,
    pub verifier: ValidatorVerifier,
}
