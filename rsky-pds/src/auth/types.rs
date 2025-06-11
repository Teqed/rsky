use crate::auth::auth_scope::AuthScope;
use jwt_simple::claims::Audiences;
use serde::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct ServiceJwtOpts {
    pub aud: Option<String>,
    pub iss: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateAccessTokenOpts {
    pub check_takedown: Option<bool>,
    pub check_deactivated: Option<bool>,
}

/// Credentials for authentication, including type, DID, scope, and other metadata.
#[derive(Clone, Debug)]
pub struct Credentials {
    pub r#type: String,
    pub did: Option<String>,
    pub scope: Option<AuthScope>,
    pub audience: Option<String>,
    pub token_id: Option<String>,
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub is_privileged: Option<bool>,
}

/// Output of an access check, including credentials and optional artifacts.
#[derive(Clone, Debug)]
pub struct AccessOutput {
    pub credentials: Option<Credentials>,
    pub artifacts: Option<String>,
}

/// Status of a role in the authentication system.
#[derive(PartialEq, Clone, Debug)]
pub enum RoleStatus {
    Valid,
    Invalid,
    Missing,
}

/// Holds DIDs for various services used in authentication verification.
#[derive(Clone, Debug)]
pub struct AuthVerifierDids {
    pub pds: String,
    pub entryway: Option<String>,
    pub mod_service: Option<String>,
}

impl fmt::Display for AuthVerifierDids {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuthVerifierDids {{ pds: {}, entryway: {:?}, mod_service: {:?} }}",
            self.pds, self.entryway, self.mod_service
        )
    }
}

/// Represents a verified service JWT with its audience and issuer.
#[derive(Clone, Debug)]
pub struct VerifiedServiceJwt {
    pub aud: String,
    pub iss: String,
}

impl fmt::Display for VerifiedServiceJwt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VerifiedServiceJwt {{ aud: {}, iss: {} }}",
            self.aud, self.iss
        )
    }
}

/// JWT payload with scope, subject, audience, and timing information.
#[derive(Clone, Debug)]
pub struct JwtPayload {
    pub scope: AuthScope,
    pub sub: Option<String>,
    pub aud: Option<Audiences>,
    pub exp: Option<Duration>,
    pub iat: Option<Duration>,
    pub jti: Option<String>,
}

/// Basic authentication credentials.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

/// Represents a request to revoke a refresh token by its ID.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevokeRefreshToken {
    pub id: String,
}
