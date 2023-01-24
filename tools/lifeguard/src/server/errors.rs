use chrono::{DateTime, Utc};
use k8s_openapi::http::StatusCode;
use serde::Serialize;

/// Struct that will be an body of a response to a request, which represents one of:
///   * a backoff limit reached - then `next_attempt` will be a date of the next earliest attempt possible
///   * an internal error - then `message` will contain an error message
#[derive(Serialize)]
pub struct ResponseMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// Enum that represents a server rejection in one of the cases:
///   * `InternalError` - if something goes wrong it return 500 with an error message
///   * `TooEarly` - if pod backoff fired it returns 429 with date-time of the next possible successful attempt
#[derive(Debug)]
pub enum ServerRejection {
    InternalError(String),
    // Status(StatusCode),
    TooEarly(DateTime<Utc>),
}

impl warp::reject::Reject for ServerRejection {}

impl ServerRejection {
    /// Metod that converst `ServerRejection` into `StatusCode` and relevant `ResponseMessage`
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
