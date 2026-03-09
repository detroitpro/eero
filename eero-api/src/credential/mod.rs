//! Pluggable credential storage for eero session tokens.
//!
//! The [`CredentialStore`] trait defines how session and user tokens are
//! persisted. Multiple implementations are provided:
//!
//! - [`file::FileStore`] — Plain text files (default)
//! - [`memory::InMemoryStore`] — Non-persistent, for testing
//! - `keyring::KeyringStore` — OS keychain (feature `keyring`)
//! - `op::OpStore` — 1Password CLI (feature `op`)
//! - `dpapi::DpapiStore` — Windows DPAPI (feature `dpapi`, Windows only)

pub mod file;
pub mod memory;
#[cfg(feature = "keyring")]
pub mod keyring;
#[cfg(feature = "op")]
pub mod op;
#[cfg(all(feature = "dpapi", target_os = "windows"))]
pub mod dpapi;

use crate::error::Result;

/// Trait for storing and retrieving eero authentication tokens.
///
/// Implementations must be `Send + Sync` for use with async code.
/// Two token types are managed:
///
/// - **Session token** — Long-lived token used for API requests
/// - **User token** — Temporary token from login, exchanged during verify
#[async_trait::async_trait]
pub trait CredentialStore: Send + Sync {
    /// Retrieve the stored session token, if any.
    async fn get_session_token(&self) -> Result<Option<String>>;
    /// Store a session token.
    async fn set_session_token(&self, token: &str) -> Result<()>;
    /// Delete the stored session token.
    async fn delete_session_token(&self) -> Result<()>;
    /// Retrieve the stored user token, if any.
    async fn get_user_token(&self) -> Result<Option<String>>;
    /// Store a user token.
    async fn set_user_token(&self, token: &str) -> Result<()>;
    /// Delete the stored user token.
    async fn delete_user_token(&self) -> Result<()>;
}
