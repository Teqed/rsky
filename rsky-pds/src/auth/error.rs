use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AuthError {
    AccessDenied,
    CookieNotProvided,
    InvalidScope,
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::AccessDenied => (StatusCode::UNAUTHORIZED, "Access denied"),
            AuthError::CookieNotProvided => (StatusCode::BAD_REQUEST, "No auth cookie"),
            AuthError::InvalidScope => (StatusCode::FORBIDDEN, "Invalid auth scope"),
            AuthError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        (status, msg).into_response()
    }
}
