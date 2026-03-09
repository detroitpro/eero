use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EeroNode {
    pub url: String,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub joined: Option<String>,
    #[serde(default)]
    pub os_version: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub status_light: Option<StatusLight>,
    #[serde(default)]
    pub connected_clients_count: Option<u32>,
    #[serde(default)]
    pub nightlight: Option<serde_json::Value>,
    #[serde(default)]
    pub gateway: Option<bool>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub mesh_quality_bars: Option<u32>,
    #[serde(default)]
    pub wired: Option<bool>,
    #[serde(default)]
    pub ethernet_addresses: Option<serde_json::Value>,
    #[serde(default)]
    pub update_available: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusLight {
    pub enabled: Option<bool>,
    pub brightness: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Nightlight {
    pub enabled: Option<bool>,
    pub schedule: Option<NightlightSchedule>,
    pub brightness: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NightlightSchedule {
    pub on: Option<String>,
    pub off: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EthernetAddress {
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct EeroUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub led_on: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nightlight: Option<NightlightUpdate>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NightlightUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<NightlightScheduleUpdate>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NightlightScheduleUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub off: Option<String>,
}
