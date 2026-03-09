#![deny(dead_code)]

use std::sync::{Arc, Mutex};

use eero_api::credential::file::FileStore;
use eero_api::credential::CredentialStore;
use tempfile::TempDir;

#[derive(Clone)]
struct InMemoryStore {
    session: Arc<Mutex<Option<String>>>,
    user: Arc<Mutex<Option<String>>>,
}

impl InMemoryStore {
    fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            user: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl CredentialStore for InMemoryStore {
    async fn get_session_token(&self) -> eero_api::Result<Option<String>> {
        Ok(self.session.lock().unwrap().clone())
    }

    async fn set_session_token(&self, token: &str) -> eero_api::Result<()> {
        *self.session.lock().unwrap() = Some(token.to_string());
        Ok(())
    }

    async fn delete_session_token(&self) -> eero_api::Result<()> {
        *self.session.lock().unwrap() = None;
        Ok(())
    }

    async fn get_user_token(&self) -> eero_api::Result<Option<String>> {
        Ok(self.user.lock().unwrap().clone())
    }

    async fn set_user_token(&self, token: &str) -> eero_api::Result<()> {
        *self.user.lock().unwrap() = Some(token.to_string());
        Ok(())
    }

    async fn delete_user_token(&self) -> eero_api::Result<()> {
        *self.user.lock().unwrap() = None;
        Ok(())
    }
}

#[tokio::test]
async fn test_in_memory_store_lifecycle() {
    let store = InMemoryStore::new();

    // Initially empty
    assert!(store.get_session_token().await.unwrap().is_none());
    assert!(store.get_user_token().await.unwrap().is_none());

    // Set tokens
    store.set_session_token("session123").await.unwrap();
    store.set_user_token("user456").await.unwrap();

    assert_eq!(
        store.get_session_token().await.unwrap(),
        Some("session123".to_string())
    );
    assert_eq!(
        store.get_user_token().await.unwrap(),
        Some("user456".to_string())
    );

    // Delete tokens
    store.delete_session_token().await.unwrap();
    assert!(store.get_session_token().await.unwrap().is_none());
    assert!(store.get_user_token().await.unwrap().is_some());

    store.delete_user_token().await.unwrap();
    assert!(store.get_user_token().await.unwrap().is_none());
}

#[tokio::test]
async fn test_in_memory_store_overwrite() {
    let store = InMemoryStore::new();

    store.set_session_token("first").await.unwrap();
    assert_eq!(
        store.get_session_token().await.unwrap(),
        Some("first".to_string())
    );

    store.set_session_token("second").await.unwrap();
    assert_eq!(
        store.get_session_token().await.unwrap(),
        Some("second".to_string())
    );
}

#[tokio::test]
async fn test_file_store_lifecycle() {
    let dir = TempDir::new().unwrap();
    let store = FileStore::with_dir(dir.path().to_path_buf());

    // Initially empty
    assert!(store.get_session_token().await.unwrap().is_none());
    assert!(store.get_user_token().await.unwrap().is_none());

    // Set tokens
    store.set_session_token("file_session").await.unwrap();
    store.set_user_token("file_user").await.unwrap();

    assert_eq!(
        store.get_session_token().await.unwrap(),
        Some("file_session".to_string())
    );
    assert_eq!(
        store.get_user_token().await.unwrap(),
        Some("file_user".to_string())
    );

    // Delete
    store.delete_session_token().await.unwrap();
    assert!(store.get_session_token().await.unwrap().is_none());

    store.delete_user_token().await.unwrap();
    assert!(store.get_user_token().await.unwrap().is_none());
}

#[tokio::test]
async fn test_file_store_nonexistent_token() {
    let dir = TempDir::new().unwrap();
    let store = FileStore::with_dir(dir.path().to_path_buf());

    // Getting a non-existent token should return None, not error
    assert!(store.get_session_token().await.unwrap().is_none());
}

#[tokio::test]
async fn test_file_store_delete_nonexistent() {
    let dir = TempDir::new().unwrap();
    let store = FileStore::with_dir(dir.path().to_path_buf());

    // Deleting a non-existent token should succeed
    store.delete_session_token().await.unwrap();
    store.delete_user_token().await.unwrap();
}

#[tokio::test]
async fn test_file_store_directory_auto_creation() {
    let dir = TempDir::new().unwrap();
    let nested = dir.path().join("deep").join("nested").join("dir");
    let store = FileStore::with_dir(nested.clone());

    // Directory doesn't exist yet
    assert!(!nested.exists());

    // Writing should create the directory
    store.set_session_token("auto_created").await.unwrap();
    assert!(nested.exists());
    assert_eq!(
        store.get_session_token().await.unwrap(),
        Some("auto_created".to_string())
    );
}

#[tokio::test]
async fn test_file_store_empty_file() {
    let dir = TempDir::new().unwrap();
    let store = FileStore::with_dir(dir.path().to_path_buf());

    // Write an empty file manually
    tokio::fs::create_dir_all(dir.path()).await.unwrap();
    tokio::fs::write(dir.path().join("session_token"), "")
        .await
        .unwrap();

    // Should return None for empty content
    assert!(store.get_session_token().await.unwrap().is_none());
}

#[tokio::test]
async fn test_file_store_whitespace_trimming() {
    let dir = TempDir::new().unwrap();
    let store = FileStore::with_dir(dir.path().to_path_buf());

    // Write token with whitespace
    tokio::fs::create_dir_all(dir.path()).await.unwrap();
    tokio::fs::write(dir.path().join("session_token"), "  my_token  \n")
        .await
        .unwrap();

    assert_eq!(
        store.get_session_token().await.unwrap(),
        Some("my_token".to_string())
    );
}
