use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Time window requested from an eero `data_usage` endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DataUsageQuery {
    pub start: String,
    pub end: String,
    pub cadence: String,
    pub timezone: String,
}

/// Network-wide upload and download totals over a requested time window.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataUsage {
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub limit: Option<String>,
    #[serde(default)]
    pub series: Vec<DataUsageSeries>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One upload or download series from a network data-usage response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataUsageSeries {
    #[serde(rename = "type")]
    pub usage_type: String,
    #[serde(default)]
    pub sum: Option<u64>,
    #[serde(default)]
    pub values: Vec<DataUsagePoint>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One time bucket in a network data-usage series.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataUsagePoint {
    pub time: String,
    #[serde(default)]
    pub value: Option<u64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A resource breakdown returned by `data_usage/devices` or `data_usage/eeros`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataUsageBreakdown<T> {
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub values: Vec<T>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Per-device byte totals over a requested data-usage window.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DeviceDataUsage {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub mac: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub download: Option<u64>,
    #[serde(default)]
    pub upload: Option<u64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Per-eero-node byte totals over a requested data-usage window.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EeroDataUsage {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub download: Option<u64>,
    #[serde(default)]
    pub upload: Option<u64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
