use chrono::{DateTime, Utc};
use k8s_openapi::http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseMessage {
    pub message: Option<String>,
    pub next_attempt: Option<DateTime<Utc>>,
}

impl ResponseMessage {
    fn new() -> Self {
        Self {
            next_attempt: None,
            message: None,
        }
    }
}

#[derive(Debug)]
pub enum ServerRejection {
    InternalError(String),
    // Status(StatusCode),
    TooEarly(DateTime<Utc>),
}

impl warp::reject::Reject for ServerRejection {}

impl ServerRejection {
    pub fn code_and_message(&self) -> (StatusCode, ResponseMessage) {
        let mut code = StatusCode::INTERNAL_SERVER_ERROR;
        let mut message = ResponseMessage::new();
        match self {
            Self::InternalError(s) => message.message = Some(s.into()),
            // Self::Status(status_code) => code = *status_code,
            Self::TooEarly(t) => {
                code = StatusCode::TOO_MANY_REQUESTS;
                message.next_attempt = Some(*t);
            }
        }

        (code, message)
    }
}
