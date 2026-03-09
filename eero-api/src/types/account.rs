use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    #[serde(default)]
    pub url: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub phone: Option<ContactInfo>,
    #[serde(default)]
    pub email: Option<ContactInfo>,
    #[serde(default)]
    pub networks: Networks,
    #[serde(default)]
    pub premium_status: Option<String>,
    #[serde(default)]
    pub can_transfer: Option<bool>,
    #[serde(default)]
    pub is_premium: Option<bool>,
    #[serde(default)]
    pub amazon_directed_id: Option<String>,
    #[serde(default)]
    pub log_id: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContactInfo {
    pub value: String,
    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub national_number: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Networks {
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub data: Vec<NetworkRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkRef {
    pub url: String,
    pub name: Option<String>,
}
