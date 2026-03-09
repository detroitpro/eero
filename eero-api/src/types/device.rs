use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Device {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub mac: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub ips: Option<Vec<String>>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub connected: Option<bool>,
    #[serde(default)]
    pub wireless: Option<bool>,
    #[serde(default)]
    pub connection_type: Option<String>,
    #[serde(default)]
    pub source: Option<DeviceSource>,
    #[serde(default)]
    pub profile: Option<DeviceProfile>,
    #[serde(default)]
    pub paused: Option<bool>,
    #[serde(default)]
    pub blocked: Option<bool>,
    #[serde(default)]
    pub is_guest: Option<bool>,
    #[serde(default)]
    pub is_private: Option<bool>,
    #[serde(default)]
    pub channel: Option<serde_json::Value>,
    #[serde(default)]
    pub frequency: Option<String>,
    #[serde(default)]
    pub frequency_band: Option<String>,
    #[serde(default)]
    pub auth: Option<String>,
    #[serde(default)]
    pub usage: Option<DeviceUsage>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceSource {
    pub location: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceProfile {
    pub url: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceUsage {
    pub down: Option<f64>,
    pub up: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DeviceUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}
