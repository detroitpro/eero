//! Authentication request and response types.

use serde::{Deserialize, Serialize};

/// Request body for the login endpoint.
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub login: String,
}

/// Response from the login endpoint containing a temporary user token.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginResponse {
    pub user_token: String,
}

/// Request body for the verify endpoint.
#[derive(Debug, Serialize)]
pub struct VerifyRequest {
    pub code: String,
}
