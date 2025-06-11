use crate::auth::types::AccessOutput;
use crate::auth::auth_scope::AuthScope;
use crate::auth::error::AuthError;
use crate::auth::validation::validate_access_token;
use crate::auth::validate_access_token_opts::ValidateAccessTokenOpts;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

/// Represents an access token for a user whose signup is queued.
#[derive(Clone, Debug)]
pub struct AccessStandardSignupQueued {
    pub access: AccessOutput,
}

#[async_trait]
impl<S> FromRequestParts<S> for AccessStandardSignupQueued
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let scopes = vec![AuthScope::SignupQueued];
        let opts: Option<ValidateAccessTokenOpts> = None;
        let access = validate_access_token(headers, scopes, opts).await?;
        Ok(AccessStandardSignupQueued { access })
    }
}
