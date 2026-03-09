use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortForward {
    pub url: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub port: Option<String>,
    #[serde(default)]
    pub external_port: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewPortForward {
    pub ip_address: String,
    pub port: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
}
