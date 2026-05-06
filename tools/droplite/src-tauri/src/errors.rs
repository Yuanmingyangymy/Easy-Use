use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::{fmt, io};

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    FileTooLarge { max_bytes: u64 },
    Io(io::Error),
    LockFailed(&'static str),
    Multipart(String),
    SessionExpired,
    TokenInvalid,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::FileTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::SessionExpired | AppError::TokenInvalid => StatusCode::UNAUTHORIZED,
            AppError::Io(_) | AppError::LockFailed(_) | AppError::Multipart(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest(message) => write!(formatter, "{message}"),
            AppError::FileTooLarge { max_bytes } => {
                write!(
                    formatter,
                    "File is too large. Maximum size is {max_bytes} bytes."
                )
            }
            AppError::Io(error) => write!(formatter, "{error}"),
            AppError::LockFailed(name) => write!(formatter, "Internal state lock failed: {name}"),
            AppError::Multipart(message) => write!(formatter, "{message}"),
            AppError::SessionExpired => write!(formatter, "Session expired. Please scan again."),
            AppError::TokenInvalid => write!(formatter, "Invalid session token."),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(ErrorBody {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}
