use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Profile {
    pub url: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub paused: Option<bool>,
    #[serde(default)]
    pub blocked_applications: Option<serde_json::Value>,
    #[serde(default)]
    pub dns_policies: Option<serde_json::Value>,
    #[serde(default)]
    pub devices: Option<Vec<ProfileDevice>>,
    #[serde(default)]
    pub schedule: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DnsPolicies {
    #[serde(default)]
    pub block: Option<Vec<String>>,
    #[serde(default)]
    pub safe_search_enabled: Option<bool>,
    #[serde(default)]
    pub youtube_restricted: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileDevice {
    pub url: Option<String>,
    pub mac: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileSchedule {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub time_ranges: Option<Vec<serde_json::Value>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProfileUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_applications: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_policies: Option<DnsPoliciesUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<ProfileScheduleUpdate>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DnsPoliciesUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_search_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube_restricted: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProfileScheduleUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_ranges: Option<Vec<serde_json::Value>>,
}
