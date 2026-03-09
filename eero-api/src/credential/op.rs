use crate::credential::CredentialStore;
use crate::error::{Error, Result};
use tokio::process::Command;

const VAULT: &str = "eero-api";
const SESSION_ITEM: &str = "session_token";
const USER_ITEM: &str = "user_token";

/// Credential store backed by the 1Password CLI (`op`).
///
/// Requires the `op` binary to be installed and authenticated.
/// Tokens are stored as password fields in the specified vault.
pub struct OpStore {
    vault: String,
}

impl OpStore {
    /// Create a new OpStore using the default vault name.
    pub fn new() -> Self {
        Self {
            vault: VAULT.to_string(),
        }
    }

    /// Create a new OpStore with a custom vault name.
    pub fn with_vault(vault: String) -> Self {
        Self { vault }
    }

    async fn get(&self, item: &str) -> Result<Option<String>> {
        let output = Command::new("op")
            .args(["item", "get", item, "--vault", &self.vault, "--fields", "password", "--format", "json"])
            .output()
            .await
            .map_err(|e| Error::CredentialStore(format!("failed to run op: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("isn't an item") || stderr.contains("not found") {
                return Ok(None);
            }
            return Err(Error::CredentialStore(format!("op error: {stderr}")));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // op --format json returns a JSON object with a "value" field
        let val: serde_json::Value = serde_json::from_str(stdout.trim())
            .map_err(|e| Error::CredentialStore(format!("failed to parse op output: {e}")))?;

        match val.get("value").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => Ok(Some(s.to_string())),
            _ => Ok(None),
        }
    }

    async fn set(&self, item: &str, value: &str) -> Result<()> {
        // Try to edit first, then create if it doesn't exist
        let output = Command::new("op")
            .args([
                "item", "edit", item,
                "--vault", &self.vault,
                &format!("password={value}"),
            ])
            .output()
            .await
            .map_err(|e| Error::CredentialStore(format!("failed to run op: {e}")))?;

        if !output.status.success() {
            // Item doesn't exist, create it
            let output = Command::new("op")
                .args([
                    "item", "create",
                    "--category", "password",
                    "--title", item,
                    "--vault", &self.vault,
                    &format!("password={value}"),
                ])
                .output()
                .await
                .map_err(|e| Error::CredentialStore(format!("failed to run op: {e}")))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::CredentialStore(format!("op create error: {stderr}")));
            }
        }

        Ok(())
    }

    async fn delete(&self, item: &str) -> Result<()> {
        let output = Command::new("op")
            .args(["item", "delete", item, "--vault", &self.vault])
            .output()
            .await
            .map_err(|e| Error::CredentialStore(format!("failed to run op: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Not an error if the item doesn't exist
            if !stderr.contains("isn't an item") && !stderr.contains("not found") {
                return Err(Error::CredentialStore(format!("op delete error: {stderr}")));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl CredentialStore for OpStore {
    async fn get_session_token(&self) -> Result<Option<String>> {
        self.get(SESSION_ITEM).await
    }

    async fn set_session_token(&self, token: &str) -> Result<()> {
        self.set(SESSION_ITEM, token).await
    }

    async fn delete_session_token(&self) -> Result<()> {
        self.delete(SESSION_ITEM).await
    }

    async fn get_user_token(&self) -> Result<Option<String>> {
        self.get(USER_ITEM).await
    }

    async fn set_user_token(&self, token: &str) -> Result<()> {
        self.set(USER_ITEM, token).await
    }

    async fn delete_user_token(&self) -> Result<()> {
        self.delete(USER_ITEM).await
    }
}
