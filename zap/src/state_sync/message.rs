use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct StorageServiceRequest {
    pub data_request: DataRequest, 
    pub use_compression: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DataRequest {
    GetServerProtocolVersion,
    GetStorageServerSummary,
    GetEpochEndingLedgerInfos(EpochEndingLedgerInfoRequest),
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct EpochEndingLedgerInfoRequest {
    pub start_epoch: u64,
    pub expected_end_epoch: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum StorageServiceResponse {
    CompressedResponse(String, Vec<u8>), 
    RawResponse(DataResponse),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DataResponse {
    ServerProtocolVersion(ServerProtocolVersion),
    StorageServerSummary(StorageServerSummary),
    EpochEndingLedgerInfos(Vec<u8>), // Placeholder for EpochChangeProof
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ServerProtocolVersion {
    pub protocol_version: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageServerSummary {
    // Placeholder fields
}
