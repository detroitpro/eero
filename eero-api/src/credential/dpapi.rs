use std::path::PathBuf;

use crate::credential::CredentialStore;
use crate::error::{Error, Result};

/// Credential store using Windows DPAPI for encryption.
///
/// Tokens are encrypted using CryptProtectData and stored as files in the
/// config directory. Only the current Windows user can decrypt them.
pub struct DpapiStore {
    dir: PathBuf,
}

impl DpapiStore {
    pub fn new() -> Result<Self> {
        let dir = dirs::config_dir()
            .ok_or_else(|| Error::CredentialStore("could not determine config directory".into()))?
            .join("eero");
        Ok(Self { dir })
    }

    fn session_path(&self) -> PathBuf {
        self.dir.join("session_token.dpapi")
    }

    fn user_path(&self) -> PathBuf {
        self.dir.join("user_token.dpapi")
    }

    fn encrypt(data: &[u8]) -> Result<Vec<u8>> {
        use windows_sys::Win32::Security::Cryptography::*;

        let mut input = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        let result = unsafe {
            CryptProtectData(
                &mut input,
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut output,
            )
        };

        if result == 0 {
            return Err(Error::CredentialStore("CryptProtectData failed".into()));
        }

        let encrypted = unsafe {
            std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec()
        };
        unsafe {
            windows_sys::Win32::System::Memory::LocalFree(output.pbData as _);
        }

        Ok(encrypted)
    }

    fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
        use windows_sys::Win32::Security::Cryptography::*;

        let mut input = CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        let result = unsafe {
            CryptUnprotectData(
                &mut input,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut output,
            )
        };

        if result == 0 {
            return Err(Error::CredentialStore("CryptUnprotectData failed".into()));
        }

        let decrypted = unsafe {
            std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec()
        };
        unsafe {
            windows_sys::Win32::System::Memory::LocalFree(output.pbData as _);
        }

        Ok(decrypted)
    }

    async fn read_token(&self, path: PathBuf) -> Result<Option<String>> {
        match tokio::fs::read(&path).await {
            Ok(encrypted) => {
                let decrypted = tokio::task::spawn_blocking(move || Self::decrypt(&encrypted))
                    .await
                    .map_err(|e| Error::CredentialStore(e.to_string()))??;
                let token = String::from_utf8(decrypted)
                    .map_err(|e| Error::CredentialStore(e.to_string()))?;
                if token.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(token))
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn write_token(&self, path: PathBuf, token: &str) -> Result<()> {
        tokio::fs::create_dir_all(&self.dir).await?;
        let data = token.as_bytes().to_vec();
        let encrypted = tokio::task::spawn_blocking(move || Self::encrypt(&data))
            .await
            .map_err(|e| Error::CredentialStore(e.to_string()))??;
        tokio::fs::write(&path, &encrypted).await?;
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
impl CredentialStore for DpapiStore {
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
