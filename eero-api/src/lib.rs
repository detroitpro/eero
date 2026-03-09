#![deny(dead_code)]
//! Rust client library for the eero WiFi router API.
//!
//! This crate provides an async client for interacting with eero mesh WiFi
//! routers. It covers authentication, network management, device tracking,
//! eero node control, profiles (parental controls), port forwarding,
//! activity monitoring, diagnostics, and speed tests.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use eero_api::{EeroClient, InMemoryStore, ClientBuilder};
//!
//! # async fn example() -> eero_api::Result<()> {
//! // Authenticate with a known session token
//! let store = InMemoryStore::new();
//! let client = ClientBuilder::new()
//!     .credential_store(Box::new(store))
//!     .build()?;
//! client.credentials().set_session_token("your_token").await?;
//!
//! // List networks
//! let account = client.get_account().await?;
//! for net in &account.networks.data {
//!     println!("{}: {}", net.url, net.name.as_deref().unwrap_or("unnamed"));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Authentication flow
//!
//! The eero API uses a two-step authentication flow:
//!
//! 1. Call [`EeroClient::login`] with an email or phone number. This sends a
//!    verification code to that address and returns a temporary user token.
//! 2. Call [`EeroClient::verify`] with the code. This exchanges the user token
//!    for a persistent session token stored in the credential store.
//!
//! # Credential backends
//!
//! Session tokens can be stored in several backends via the
//! [`CredentialStore`] trait:
//!
//! | Backend | Feature flag | Description |
//! |---------|-------------|-------------|
//! | [`credential::file::FileStore`] | *(default)* | Plain text files in `~/.config/eero/` |
//! | [`credential::memory::InMemoryStore`] | *(default)* | Non-persistent, for testing or CLI token flags |
//! | `credential::keyring::KeyringStore` | `keyring` | OS keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager) |
//! | `credential::op::OpStore` | `op` | 1Password CLI (`op`) |
//! | `credential::dpapi::DpapiStore` | `dpapi` | Windows DPAPI encrypted files (Windows only) |

pub mod api;
pub mod client;
pub mod credential;
pub mod error;
pub mod types;

pub use client::{ClientBuilder, EeroClient};
pub use credential::memory::InMemoryStore;
pub use credential::CredentialStore;
pub use error::{Error, Result};
