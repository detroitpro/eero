use std::path::PathBuf;

use crate::credential::CredentialStore;
use crate::error::{Error, Result};

/// File-based credential store. Stores tokens as plain text files in the
/// platform config directory (e.g. `~/.config/eero/` on Linux).
pub struct FileStore {
    dir: PathBuf,
}

impl FileStore {
    /// Create a new FileStore using the default config directory.
    pub fn new() -> Result<Self> {
        let dir = dirs::config_dir()
            .ok_or_else(|| Error::CredentialStore("could not determine config directory".into()))?
            .join("eero");
        Ok(Self { dir })
    }

    /// Create a new FileStore with a custom directory.
    pub fn with_dir(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn session_path(&self) -> PathBuf {
        self.dir.join("session_token")
    }

    fn user_path(&self) -> PathBuf {
        self.dir.join("user_token")
    }

    async fn read_token(&self, path: PathBuf) -> Result<Option<String>> {
        match tokio::fs::read_to_string(&path).await {
            Ok(contents) => {
                let trimmed = contents.trim().to_string();
                if trimmed.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(trimmed))
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Write a token readable only by its owner: the directory is `0700` and
    /// the file `0600` on Unix. The file is written beside its final path and
    /// renamed, so a reader never sees a partial token or looser permissions.
    async fn write_token(&self, path: PathBuf, token: &str) -> Result<()> {
        tokio::fs::create_dir_all(&self.dir).await?;
        let tmp = path.with_extension("tmp");
        let mut options = tokio::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&self.dir, std::fs::Permissions::from_mode(0o700)).await?;
            options.mode(0o600);
        }
        let mut file = options.open(&tmp).await?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .await?;
        }
        tokio::io::AsyncWriteExt::write_all(&mut file, token.as_bytes()).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&tmp, &path).await?;
        Ok(())
    }

    async fn delete_token(&self, path: PathBuf) -> Result<()> {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait::async_trait]
impl CredentialStore for FileStore {
    async fn get_session_token(&self) -> Result<Option<String>> {
        self.read_token(self.session_path()).await
    }

    async fn set_session_token(&self, token: &str) -> Result<()> {
        self.write_token(self.session_path(), token).await
    }

    async fn delete_session_token(&self) -> Result<()> {
        self.delete_token(self.session_path()).await
    }

    async fn get_user_token(&self) -> Result<Option<String>> {
        self.read_token(self.user_path()).await
    }

    async fn set_user_token(&self, token: &str) -> Result<()> {
        self.write_token(self.user_path(), token).await
    }

    async fn delete_user_token(&self) -> Result<()> {
        self.delete_token(self.user_path()).await
    }
}
