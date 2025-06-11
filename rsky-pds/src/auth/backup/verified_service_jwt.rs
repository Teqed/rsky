use std::fmt;

/// Represents a verified service JWT with its audience and issuer.
#[derive(Clone, Debug)]
pub struct VerifiedServiceJwt {
    pub aud: String,
    pub iss: String,
}

impl fmt::Display for VerifiedServiceJwt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VerifiedServiceJwt {{ aud: {}, iss: {} }}", self.aud, self.iss)
    }
}
