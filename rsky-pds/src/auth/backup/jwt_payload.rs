use crate::auth::auth_scope::AuthScope;
use jwt_simple::claims::Audiences;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct JwtPayload {
    pub scope: AuthScope,
    pub sub: Option<String>,
    pub aud: Option<Audiences>,
    pub exp: Option<Duration>,
    pub iat: Option<Duration>,
    pub jti: Option<String>,
}
