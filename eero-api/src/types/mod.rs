//! Data types for eero API requests and responses.
//!
//! All response types use `#[serde(flatten)] pub extra: HashMap<String, Value>`
//! to capture unknown fields, making the library forward-compatible with new
//! API fields without requiring code changes.
//!
//! Update types (e.g. [`network::NetworkUpdate`], [`eero::EeroUpdate`]) use
//! `#[serde(skip_serializing_if = "Option::is_none")]` so only explicitly set
//! fields are sent in the request body.

pub mod account;
pub mod activity;
pub mod auth;
pub mod device;
pub mod diagnostic;
pub mod eero;
pub mod envelope;
pub mod forward;
pub mod network;
pub mod profile;
