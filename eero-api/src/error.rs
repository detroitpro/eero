//! Error types for the eero API client.

use std::io;

/// Errors that can occur when interacting with the eero API.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("API error {code}: {message}")]
    Api { code: u16, message: String },

    #[error("not authenticated - call login() and verify() first")]
    NotAuthenticated,

    #[error("verification required - call verify() with the code sent to your device")]
    VerificationRequired,

    #[error("credential store error: {0}")]
    CredentialStore(String),
}

pub type Result<T> = std::result::Result<T, Error>;
