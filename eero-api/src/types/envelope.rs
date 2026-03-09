//! API response envelope types.
//!
//! Every eero API response is wrapped in an [`ApiResponse`] containing a
//! [`Meta`] header (status code, server time, optional error message) and
//! an optional `data` payload.

use serde::{Deserialize, Serialize};

/// Metadata included in every API response.
#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    #[serde(default)]
    pub code: u16,
    #[serde(default)]
    pub server_time: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Top-level API response wrapper containing metadata and an optional data payload.
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub meta: Meta,
    pub data: Option<T>,
}
