use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Network {
    pub url: String,
    pub name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub health: Option<NetworkHealth>,
    #[serde(default)]
    pub speed: Option<SpeedInfo>,
    #[serde(default)]
    pub clients: Option<ClientCount>,
    #[serde(default)]
    pub eeros: Option<EeroCount>,
    #[serde(default)]
    pub profiles: Option<ProfileCount>,
    #[serde(default)]
    pub dns: Option<serde_json::Value>,
    #[serde(default)]
    pub guest_network: Option<GuestNetwork>,
    #[serde(default)]
    pub sqm: Option<serde_json::Value>,
    #[serde(default)]
    pub band_steering: Option<serde_json::Value>,
    #[serde(default)]
    pub thread: Option<serde_json::Value>,
    #[serde(default)]
    pub upnp: Option<serde_json::Value>,
    #[serde(default)]
    pub ipv6: Option<serde_json::Value>,
    #[serde(default)]
    pub wpa3: Option<serde_json::Value>,
    #[serde(default)]
    pub target_firmware: Option<String>,
    #[serde(default)]
    pub updates: Option<serde_json::Value>,
    #[serde(default)]
    pub timezone: Option<Timezone>,
    #[serde(default)]
    pub premium_dns: Option<serde_json::Value>,
    #[serde(default)]
    pub premium_status: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkHealth {
    #[serde(default)]
    pub internet: Option<HealthStatus>,
    #[serde(default)]
    pub eero_network: Option<HealthStatus>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthStatus {
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpeedInfo {
    pub down: Option<SpeedValue>,
    pub up: Option<SpeedValue>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpeedValue {
    pub value: Option<f64>,
    pub units: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientCount {
    pub count: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EeroCount {
    pub count: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileCount {
    pub count: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuestNetwork {
    pub name: Option<String>,
    pub password: Option<String>,
    pub enabled: Option<bool>,
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Timezone {
    pub value: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NetworkUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sqm: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_steering: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upnp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wpa3: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<DnsUpdate>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DnsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caching: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GuestNetworkUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpeedTest {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub up_mbps: Option<f64>,
    #[serde(default)]
    pub down_mbps: Option<f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
