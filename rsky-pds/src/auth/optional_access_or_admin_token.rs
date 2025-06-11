use crate::auth::types::AccessOutput;
use crate::auth::error::AuthError;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

/// Represents an optional access token or admin token for flexible authentication.
#[derive(Clone, Debug)]
pub struct OptionalAccessOrAdminToken {
    pub access: Option<AccessOutput>,
}

use crate::auth::auth_scope::AuthScope;
use crate::auth::validation::validate_access_token;
use crate::auth::validate_access_token_opts::ValidateAccessTokenOpts;

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAccessOrAdminToken
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let scopes = vec![AuthScope::Access];
        let opts: Option<ValidateAccessTokenOpts> = None;
        match validate_access_token(headers, scopes, opts).await {
            Ok(access) => Ok(OptionalAccessOrAdminToken { access: Some(access) }),
            Err(AuthError::AccessDenied) => Ok(OptionalAccessOrAdminToken { access: None }),
            Err(e) => Err(e),
        }
    }
}
