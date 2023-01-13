use k8s_openapi::http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorMessage {
    pub code: u16, // TODO: remove pubs
    pub message: String,
}

#[derive(Debug)]
pub enum ServerRejection {
    InternalError(String),
    Status(StatusCode),
    PodLimitElapsed,
}

impl warp::reject::Reject for ServerRejection {}
