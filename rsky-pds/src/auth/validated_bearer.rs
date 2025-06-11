use crate::auth::auth_scope::AuthScope;
use crate::auth::types::{Credentials, JwtPayload};

/// Represents a validated bearer token and its associated data.
#[derive(Debug, Clone)]
pub struct ValidatedBearer {
    pub did: String,
    pub scope: AuthScope,
    pub token: String,
    pub payload: JwtPayload,
    pub audience: Option<String>,
}
