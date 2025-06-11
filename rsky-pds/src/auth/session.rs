use async_trait::async_trait;
use axum::extract::{FromRequestParts};
use axum::http::request::Parts;
use crate::auth::error::AuthError;

/// Represents an authenticated session, e.g. from a bearer token or cookie.
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    // Add more fields as needed, e.g. user id, roles, etc.
}

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Example: extract token from "Authorization" header as "Bearer <token>"
        if let Some(auth_header) = parts.headers.get("authorization") {
            if let Ok(header_str) = auth_header.to_str() {
                if let Some(token) = header_str.strip_prefix("Bearer ").map(str::to_string) {
                    // Here you would validate the token, look up session, etc.
                    return Ok(Session { token });
                }
            }
        }
        Err(AuthError::AccessDenied)
    }
}