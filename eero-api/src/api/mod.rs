//! API method implementations for [`EeroClient`](crate::EeroClient).
//!
//! Each submodule adds methods to `EeroClient` for a specific domain:
//!
//! - [`auth`] — Login, verify, and logout
//! - [`account`] — Account information and network listing
//! - [`networks`] — Network settings, speed tests, guest network, reboot
//! - [`eeros`] — Eero node listing, details, LED/nightlight settings, reboot
//! - [`devices`] — Connected device listing, details, nickname/block/pause
//! - [`profiles`] — Parental control profiles
//! - [`forwards`] — Port forwarding rules
//! - [`activity`] — Network and per-device activity (eero Plus)
//! - [`diagnostics`] — Network diagnostic reports

pub mod account;
pub mod activity;
pub mod auth;
pub mod devices;
pub mod diagnostics;
pub mod eeros;
pub mod forwards;
pub mod networks;
pub mod profiles;
