use std::sync::{Arc, Mutex};

use crate::error::Result;

use super::CredentialStore;

/// An in-memory credential store backed by `Arc<Mutex<…>>`.
///
/// Useful when a session token is provided externally (e.g. via CLI flag or
/// environment variable) and should not be persisted to disk.
#[derive(Clone)]
pub struct InMemoryStore {
    session: Arc<Mutex<Option<String>>>,
    user: Arc<Mutex<Option<String>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            user: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CredentialStore for InMemoryStore {
    async fn get_session_token(&self) -> Result<Option<String>> {
        Ok(self.session.lock().unwrap().clone())
    }

    async fn set_session_token(&self, token: &str) -> Result<()> {
        *self.session.lock().unwrap() = Some(token.to_string());
        Ok(())
    }

    async fn delete_session_token(&self) -> Result<()> {
        *self.session.lock().unwrap() = None;
        Ok(())
    }

    async fn get_user_token(&self) -> Result<Option<String>> {
        Ok(self.user.lock().unwrap().clone())
    }

    async fn set_user_token(&self, token: &str) -> Result<()> {
        *self.user.lock().unwrap() = Some(token.to_string());
        Ok(())
    }

    async fn delete_user_token(&self) -> Result<()> {
        *self.user.lock().unwrap() = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_token_crud() {
        let store = InMemoryStore::new();
        assert_eq!(store.get_session_token().await.unwrap(), None);

        store.set_session_token("abc").await.unwrap();
        assert_eq!(
            store.get_session_token().await.unwrap(),
            Some("abc".to_string())
        );

        store.delete_session_token().await.unwrap();
        assert_eq!(store.get_session_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_user_token_crud() {
        let store = InMemoryStore::new();
        assert_eq!(store.get_user_token().await.unwrap(), None);

        store.set_user_token("xyz").await.unwrap();
        assert_eq!(
            store.get_user_token().await.unwrap(),
            Some("xyz".to_string())
        );

        store.delete_user_token().await.unwrap();
        assert_eq!(store.get_user_token().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_tokens_independent() {
        let store = InMemoryStore::new();

        store.set_session_token("session").await.unwrap();
        assert_eq!(store.get_user_token().await.unwrap(), None);

        store.set_user_token("user").await.unwrap();
        assert_eq!(
            store.get_session_token().await.unwrap(),
            Some("session".to_string())
        );
        assert_eq!(
            store.get_user_token().await.unwrap(),
            Some("user".to_string())
        );

        store.delete_session_token().await.unwrap();
        assert_eq!(
            store.get_user_token().await.unwrap(),
            Some("user".to_string())
        );
    }

    #[tokio::test]
    async fn test_default() {
        let store = InMemoryStore::default();
        assert_eq!(store.get_session_token().await.unwrap(), None);
        assert_eq!(store.get_user_token().await.unwrap(), None);
    }
}
