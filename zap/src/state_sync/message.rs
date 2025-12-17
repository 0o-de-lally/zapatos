use serde::{Deserialize, Serialize};
use crate::types::ledger_info::LedgerInfoWithSignatures;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageServiceRequest {
    pub data_request: DataRequest,
    pub use_compression: bool,
}

impl StorageServiceRequest {
    pub fn new(data_request: DataRequest, use_compression: bool) -> Self {
        Self {
            data_request,
            use_compression,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DataRequest {
    GetStorageServerSummary,
    GetServerProtocolVersion,
    // Add other variants as needed, but for now we only need these
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageServiceResponse {
    pub response: DataResponse,
}

impl StorageServiceResponse {
    pub fn get_data_response(self) -> Result<DataResponse, String> {
        Ok(self.response)
    }
}

// In Zapatos, StorageServiceResponse is an enum:
// pub enum StorageServiceResponse {
//     CompressedResponse(CompressedDataResponse),
//     RawResponse(DataResponse),
// }
// However, since we requested use_compression: false, we expect RawResponse.
// But we must match the Enum serialization.
// 0 -> Compressed
// 1 -> Raw
// We can represent this with an Enum.

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StorageServiceResponseWrapper {
    CompressedResponse(Vec<u8>), // Placeholder
    RawResponse(DataResponse),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DataResponse {
    ServerProtocolVersion(ServerProtocolVersion),
    StorageServerSummary(StorageServerSummary),
    // Others...
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServerProtocolVersion {
    pub protocol_version: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageServerSummary {
    pub protocol_metadata: ProtocolMetadata,
    pub data_summary: DataSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtocolMetadata {
    pub max_epoch_chunk_size: u64,
    pub max_transaction_chunk_size: u64,
    pub max_transaction_output_chunk_size: u64,
    pub max_account_states_chunk_size: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DataSummary {
    pub synced_ledger_info: Option<LedgerInfoWithSignatures>,
    pub epoch_ending_ledger_infos: Option<LedgerInfoWithSignatures>,
    pub transactions: Option<u64>, // TransactionVersion
    pub transaction_outputs: Option<u64>, // TransactionVersion
    pub account_states: Option<u64>, // TransactionVersion
}
