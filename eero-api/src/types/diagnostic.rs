use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticReport {
    pub url: Option<String>,
    pub status: Option<String>,
    pub created: Option<String>,
    pub completed: Option<String>,
    #[serde(default)]
    pub results: Option<Vec<DiagnosticResult>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticResult {
    #[serde(rename = "type")]
    pub result_type: Option<String>,
    pub status: Option<String>,
    pub details: Option<serde_json::Value>,
}
