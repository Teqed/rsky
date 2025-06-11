use crate::auth::types::AccessOutput;

/// Represents an access token with standard permissions, including additional checks.
#[derive(Clone, Debug)]
pub struct AccessStandardIncludeChecks {
    pub access: AccessOutput,
}

use crate::auth::auth_scope::AuthScope;
use crate::auth::error::AuthError;
use crate::auth::validation::validate_access_token;
use crate::auth::validate_access_token_opts::ValidateAccessTokenOpts;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[async_trait]
impl<S> FromRequestParts<S> for AccessStandardIncludeChecks
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let scopes = vec![AuthScope::Access];
        let opts = Some(ValidateAccessTokenOpts {
            check_takedown: Some(true),
            check_deactivated: None,
        });
        let access = validate_access_token(headers, scopes, opts).await?;
        Ok(AccessStandardIncludeChecks { access })
    }
}
