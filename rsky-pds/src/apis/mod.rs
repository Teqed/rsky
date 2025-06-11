use crate::auth_verifier_rocket::AccessStandard;
use crate::handle;
use crate::handle::errors::ErrorKind;
use crate::pipethrough::{pipethrough_procedure, pipethrough_procedure_post, ProxyRequest};
use anyhow::{Error, Result};
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Proxy Responder for Axum
pub struct ProxyResponder {
    buffer: Vec<u8>,
    headers: HeaderMap,
}

impl ProxyResponder {
    pub fn new(buffer: Vec<u8>, headers: HeaderMap) -> Self {
        Self { buffer, headers }
    }
}

impl IntoResponse for ProxyResponder {
    fn into_response(self) -> Response {
        let mut response = Response::builder().status(StatusCode::OK);

        // Add headers from the proxy response
        let headers = response.headers_mut().unwrap();
        for (key, value) in self.headers.iter() {
            if let key_str = key.as_str() {
                headers.insert(key, value.clone());
            }
        }

        // Ensure content-type and content-length are set
        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/octet-stream"),
            );
        }

        if !headers.contains_key(header::CONTENT_LENGTH) {
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&self.buffer.len().to_string()).unwrap(),
            );
        }

        // Build the response with the body
        response.body(Body::from(self.buffer)).unwrap()
    }
}

#[allow(dead_code)]
pub struct Nsid(String);

impl Nsid {
    pub fn from_param(param: &str) -> Result<Self, String> {
        // This is how we make sure we allowlist lexicons and what gets proxied
        if param.starts_with("app.bsky.") || param.starts_with("chat.bsky") {
            Ok(Nsid(param.to_string()))
        } else {
            Err(format!("Invalid NSID: {}", param))
        }
    }
}

// Routes will be implemented with Axum in the main application

#[derive(Clone, Debug)]
pub enum ApiError {
    RuntimeError,
    InvalidLogin,
    AccountTakendown,
    InvalidRequest(String),
    ExpiredToken,
    InvalidToken,
    RecordNotFound,
    InvalidHandle,
    InvalidEmail,
    InvalidPassword,
    InvalidInviteCode,
    HandleNotAvailable,
    EmailNotAvailable,
    UnsupportedDomain,
    UnresolvableDid,
    IncompatibleDidDoc,
    WellKnownNotFound,
    AccountNotFound,
    BlobNotFound,
    BadRequest(String, String),
    AuthRequiredError(String),
}

#[derive(Serialize)]
pub struct ErrorBody {
    error: String,
    message: String,
}

impl ApiError {
    /// Get the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::RuntimeError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidLogin
            | Self::ExpiredToken
            | Self::InvalidToken
            | Self::AuthRequiredError(_) => StatusCode::UNAUTHORIZED,
            Self::AccountTakendown => StatusCode::FORBIDDEN,
            Self::RecordNotFound
            | Self::WellKnownNotFound
            | Self::AccountNotFound
            | Self::BlobNotFound => StatusCode::NOT_FOUND,
            // All bad requests grouped together
            _ => StatusCode::BAD_REQUEST,
        }
    }

    /// Get the error type string for API responses
    fn error_type(&self) -> String {
        match self {
            Self::RuntimeError => "InternalServerError",
            Self::InvalidLogin => "InvalidLogin",
            Self::AccountTakendown => "AccountTakendown",
            Self::InvalidRequest(_) => "InvalidRequest",
            Self::ExpiredToken => "ExpiredToken",
            Self::InvalidToken => "InvalidToken",
            Self::RecordNotFound => "RecordNotFound",
            Self::InvalidHandle => "InvalidHandle",
            Self::InvalidEmail => "InvalidEmail",
            Self::InvalidPassword => "InvalidPassword",
            Self::InvalidInviteCode => "InvalidInviteCode",
            Self::HandleNotAvailable => "HandleNotAvailable",
            Self::EmailNotAvailable => "EmailNotAvailable",
            Self::UnsupportedDomain => "UnsupportedDomain",
            Self::UnresolvableDid => "UnresolvableDid",
            Self::IncompatibleDidDoc => "IncompatibleDidDoc",
            Self::WellKnownNotFound => "WellKnownNotFound",
            Self::AccountNotFound => "AccountNotFound",
            Self::BlobNotFound => "BlobNotFound",
            Self::BadRequest(error, _) => error,
            Self::AuthRequiredError(_) => "AuthRequiredError",
        }
        .to_owned()
    }

    /// Get the user-facing error message
    fn message(&self) -> String {
        match self {
            Self::RuntimeError => "Something went wrong",
            Self::InvalidLogin => "Invalid identifier or password",
            Self::AccountTakendown => "Account has been taken down",
            Self::InvalidRequest(msg) => msg,
            Self::ExpiredToken => "Token is expired",
            Self::InvalidToken => "Token is invalid",
            Self::RecordNotFound => "Record could not be found",
            Self::InvalidHandle => "Handle is invalid",
            Self::InvalidEmail => "Invalid email",
            Self::InvalidPassword => "Invalid Password",
            Self::InvalidInviteCode => "Invalid invite code",
            Self::HandleNotAvailable => "Handle not available",
            Self::EmailNotAvailable => "Email not available",
            Self::UnsupportedDomain => "Unsupported domain",
            Self::UnresolvableDid => "Unresolved Did",
            Self::IncompatibleDidDoc => "IncompatibleDidDoc",
            Self::WellKnownNotFound => "User not found",
            Self::AccountNotFound => "Account could not be found",
            Self::BlobNotFound => "Blob could not be found",
            Self::BadRequest(_, msg) => msg,
            Self::AuthRequiredError(msg) => msg,
        }
        .to_owned()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_body = ErrorBody {
            error: self.error_type(),
            message: self.message(),
        };

        // If this is a debug build, log the error
        if cfg!(debug_assertions) {
            tracing::error!("API Error: {}: {}", error_body.error, error_body.message);
        }

        // Serialize to JSON and create response
        let body = match serde_json::to_string(&error_body) {
            Ok(json) => json,
            Err(_) => r#"{"error":"InternalServerError","message":"Error serializing response"}"#
                .to_owned(),
        };

        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap_or_else(|_| Response::new(Body::from("Internal Server Error")))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type(), self.message())
    }
}

impl From<Error> for ApiError {
    fn from(_value: Error) -> Self {
        ApiError::RuntimeError
    }
}

impl From<handle::errors::Error> for ApiError {
    fn from(value: handle::errors::Error) -> Self {
        match value.kind {
            ErrorKind::InvalidHandle => ApiError::InvalidHandle,
            ErrorKind::HandleNotAvailable => ApiError::HandleNotAvailable,
            ErrorKind::UnsupportedDomain => ApiError::UnsupportedDomain,
            ErrorKind::InternalError => ApiError::RuntimeError,
        }
    }
}

pub mod app;
pub mod com;
