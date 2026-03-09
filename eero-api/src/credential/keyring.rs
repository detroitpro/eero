use crate::credential::CredentialStore;
use crate::error::{Error, Result};

const SERVICE: &str = "eero-api";
const SESSION_KEY: &str = "session_token";
const USER_KEY: &str = "user_token";

/// Credential store backed by the OS keychain (macOS Keychain, Linux Secret
/// Service, Windows Credential Manager).
pub struct KeyringStore;

impl KeyringStore {
    pub fn new() -> Self {
        Self
    }

    fn get(&self, key: &'static str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(SERVICE, key)
            .map_err(|e| Error::CredentialStore(e.to_string()))?;
        match entry.get_password() {
            Ok(val) => Ok(Some(val)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::CredentialStore(e.to_string())),
        }
    }

    fn set(&self, key: &'static str, value: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, key)
            .map_err(|e| Error::CredentialStore(e.to_string()))?;
        entry
            .set_password(value)
            .map_err(|e| Error::CredentialStore(e.to_string()))
    }

    fn delete(&self, key: &'static str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE, key)
            .map_err(|e| Error::CredentialStore(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::CredentialStore(e.to_string())),
        }
    }
}

#[async_trait::async_trait]
impl CredentialStore for KeyringStore {
    async fn get_session_token(&self) -> Result<Option<String>> {
        let this = Self;
        tokio::task::spawn_blocking(move || this.get(SESSION_KEY))
            .await
            .map_err(|e| Error::CredentialStore(e.to_string()))?
    }

    async fn set_session_token(&self, token: &str) -> Result<()> {
        let token = token.to_string();
        let this = Self;
        tokio::task::spawn_blocking(move || this.set(SESSION_KEY, &token))
            .await
            .map_err(|e| Error::CredentialStore(e.to_string()))?
    }

    async fn delete_session_token(&self) -> Result<()> {
        let this = Self;
        tokio::task::spawn_blocking(move || this.delete(SESSION_KEY))
            .await
            .map_err(|e| Error::CredentialStore(e.to_string()))?
    }

    async fn get_user_token(&self) -> Result<Option<String>> {
        let this = Self;
        tokio::task::spawn_blocking(move || this.get(USER_KEY))
            .await
            .map_err(|e| Error::CredentialStore(e.to_string()))?
    }

    async fn set_user_token(&self, token: &str) -> Result<()> {
        let token = token.to_string();
        let this = Self;
        tokio::task::spawn_blocking(move || this.set(USER_KEY, &token))
            .await
            .map_err(|e| Error::CredentialStore(e.to_string()))?
    }

    async fn delete_user_token(&self) -> Result<()> {
        let this = Self;
        tokio::task::spawn_blocking(move || this.delete(USER_KEY))
            .await
            .map_err(|e| Error::CredentialStore(e.to_string()))?
    }
}
