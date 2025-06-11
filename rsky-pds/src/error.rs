//! Error handling for the application.
use crate::handle::{self, errors::ErrorKind};
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

/// `axum`-compatible error handler.
#[derive(Error)]
#[expect(clippy::error_impl_error, reason = "just one")]
pub struct Error {
    /// The actual error that occurred.
    err: anyhow::Error,
    /// The error message to be returned as JSON body.
    message: ErrorMessage,
    /// The HTTP status code to be returned.
    status: StatusCode,
}

#[derive(Default, serde::Serialize)]
/// A JSON error message.
pub(crate) struct ErrorMessage {
    /// The error type.
    /// This is used to identify the error in the client.
    /// E.g. `InvalidRequest`, `ExpiredToken`, `InvalidToken`, `HandleNotFound`.
    error: String,
    /// The error message.
    message: String,
}
impl std::fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{"error":"{}","message":"{}"}}"#,
            self.error, self.message
        )
    }
}
impl ErrorMessage {
    /// Create a new error message to be returned as JSON body.
    pub(crate) fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}

impl Error {
    /// Returned when a route is not yet implemented.
    pub fn unimplemented<T: Into<anyhow::Error>>(err: T) -> Self {
        Self {
            status: StatusCode::NOT_IMPLEMENTED,
            err: err.into(),
            message: ErrorMessage::new("NotImplemented", "This route is not yet implemented."),
        }
    }
    pub fn new(status: StatusCode, err: impl Into<anyhow::Error>, message: ErrorMessage) -> Self {
        Self {
            status,
            err: err.into(),
            message,
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            err,
            message: ErrorMessage::new("InternalServerError", "An internal server error occurred."),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.err)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{:?}", self.err);
        Response::builder()
            .status(self.status)
            .header("Content-Type", "application/json")
            .body(Body::new(self.message.to_string()))
            .expect("should be a valid response")
    }
}
