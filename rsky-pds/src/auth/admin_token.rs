use crate::auth::types::AccessOutput;
use crate::auth::auth_scope::AuthScope;
use crate::auth::error::AuthError;
use crate::auth::validation::validate_access_token;
use crate::auth::validate_access_token_opts::ValidateAccessTokenOpts;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

/// Represents an admin's access token.
#[derive(Clone, Debug)]
pub struct AdminToken {
    pub access: AccessOutput,
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminToken
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        // In a real implementation, you would check for admin privileges here.
        // For now, we use the Access scope as a placeholder.
        let scopes = vec![AuthScope::Access];
        let opts: Option<ValidateAccessTokenOpts> = None;
        let access = validate_access_token(headers, scopes, opts).await?;
        Ok(AdminToken { access })
    }
}