use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AuthError {
    AccessDenied,
    CookieNotProvided,
    InvalidScope,
    BadJwt,
    BadJwtAudience,
    UntrustedIss,
    AuthRequired,
    AccountNotFound,
    AccountTakedown,
    AccountDeactivated,
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::AccessDenied => (StatusCode::UNAUTHORIZED, "Access denied"),
            AuthError::CookieNotProvided => (StatusCode::BAD_REQUEST, "No auth cookie"),
            AuthError::InvalidScope => (StatusCode::FORBIDDEN, "Invalid auth scope"),
            AuthError::BadJwt => (StatusCode::UNAUTHORIZED, "Invalid JWT"),
            AuthError::BadJwtAudience => (StatusCode::UNAUTHORIZED, "Invalid JWT audience"),
            AuthError::UntrustedIss => (StatusCode::UNAUTHORIZED, "Untrusted issuer"),
            AuthError::AuthRequired => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AuthError::AccountNotFound => (StatusCode::NOT_FOUND, "Account not found"),
            AuthError::AccountTakedown => (StatusCode::FORBIDDEN, "Account taken down"),
            AuthError::AccountDeactivated => (StatusCode::FORBIDDEN, "Account deactivated"),
            AuthError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        (status, msg).into_response()
    }
}
