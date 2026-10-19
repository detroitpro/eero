use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

/// A port forward rule. Field names follow the live API: `ip`, `client_port`
/// (the port on the device) and `gateway_port` (the port on the WAN side).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortForward {
    pub url: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default, deserialize_with = "port")]
    pub client_port: Option<u16>,
    #[serde(default, deserialize_with = "port")]
    pub gateway_port: Option<u16>,
    /// `tcp`, `udp` or `both`.
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// URL of the DHCP reservation the forward is attached to.
    #[serde(default)]
    pub reservation: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewPortForward {
    pub ip: String,
    pub client_port: u16,
    pub gateway_port: u16,
    /// `tcp`, `udp` or `both`.
    pub protocol: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Ports arrive as numbers, but accept strings too.
fn port<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u16>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Port {
        Number(u16),
        Text(String),
    }
    match Option::<Port>::deserialize(d)? {
        None => Ok(None),
        Some(Port::Number(n)) => Ok(Some(n)),
        Some(Port::Text(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
    }
}
