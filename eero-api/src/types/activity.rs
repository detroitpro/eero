use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivitySummary {
    #[serde(default)]
    pub insights: Option<Vec<Insight>>,
    #[serde(default)]
    pub counts: Option<ActivityCounts>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Insight {
    #[serde(rename = "type")]
    pub insight_type: Option<String>,
    pub category: Option<String>,
    pub count: Option<u64>,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityCounts {
    pub blocked: Option<u64>,
    pub allowed: Option<u64>,
    pub inspected: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientActivity {
    pub url: Option<String>,
    pub display_name: Option<String>,
    #[serde(default)]
    pub insights: Option<Vec<Insight>>,
    #[serde(default)]
    pub counts: Option<ActivityCounts>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
