use crate::auth::credentials::Credentials;

#[derive(Clone, Debug)]
pub struct AccessOutput {
    pub credentials: Option<Credentials>,
    pub artifacts: Option<String>,
}
