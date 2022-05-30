use serde::{Deserialize, Serialize};

/// Request for forwarding tx to gas-tank based relay service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ForwardRequest {
    pub type_id: String,
    pub chain_id: usize,
    pub target: String,
    pub data: String,
    pub fee_token: String,
    pub payment_type: usize, // 1 = gas tank
    pub max_fee: String,
    pub gas: String,
    pub sponsor: String,
    pub sponsor_chain_id: usize,     // same as chain_id
    pub nonce: usize,                // can default 0 if next field false
    pub enforce_sponsor_nonce: bool, // default false given replay safe
    pub enforce_sponsor_nonce_ordering: bool,
    pub sponsor_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequest {
    pub dest: String,
    pub data: String,
    pub token: String,
    pub relayer_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EstimatedFeeRequest {
    pub payment_token: String,
    pub gas_limit: usize,
    pub is_high_priority: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelayResponse {
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EstimatedFeeResponse {
    pub estimated_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainsResponse {
    pub relays: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusResponse {
    pub data: Vec<TaskStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub service: String,
    pub chain: String,
    pub task_id: String,
    pub task_state: TaskState,
    #[serde(rename = "created_at")]
    pub created_at: String, // date
    pub last_check: Option<CheckOrDate>,
    pub execution: Option<Execution>,
    pub last_execution: String, // date
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub status: String,
    pub transaction_hash: String,
    pub block_number: usize,
    #[serde(rename = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged, rename_all = "camelCase")]
pub enum CheckOrDate {
    Check(Check),
    Date(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub task_state: TaskState,
    pub message: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub to: String,
    pub data: String,
    pub fee_data: FeeData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeeData {
    pub gas_price: usize,
    pub max_fee_per_gas: usize,
    pub max_priority_fee_per_gas: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskState {
    CheckPending,
    ExecPending,
    ExecSuccess,
    ExecReverted,
    WaitingForConfirmation,
    Blacklisted,
    Cancelled,
    NotFound,
}
