use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequest {
    pub dest: String,
    pub data: String,
    pub token: String,
    pub relayer_fee: String,
}

pub struct RelayTransaction {
    pub task_id: String
}