use serde::{Deserialize, Serialize};

/// Represents a request to revoke a refresh token by its ID.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevokeRefreshToken {
    pub id: String,
}
