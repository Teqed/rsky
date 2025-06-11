use std::fmt;

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
