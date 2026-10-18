use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A DHCP reservation: a fixed IP address for a MAC address.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reservation {
    pub url: String,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub mac: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewReservation {
    pub ip: String,
    pub mac: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
