use std::vec::Vec;

#[derive(Clone, Debug)]
pub struct ServiceJwtOpts {
    pub aud: Option<String>,
    pub iss: Option<Vec<String>>,
}
