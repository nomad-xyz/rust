use serde::{Deserialize, Serialize};

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
    pub last_check: Option<String>,
    pub execution: Option<Execution>,
    pub last_execution: String, // date
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
    pub status: String,
    pub transaction_hash: String,
    pub block_number: usize,
    pub created_at: String,
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
