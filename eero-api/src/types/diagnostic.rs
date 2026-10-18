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

/// Body for starting a diagnostic run. Newer firmware (eero Pro 7, 7.17)
/// rejects a run without a `symptom`, mirroring the eero app's health check,
/// which always asks what is wrong first.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DiagnosticsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symptom: Option<String>,
    /// Device (MAC or device ID) to focus the run on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticResult {
    #[serde(rename = "type")]
    pub result_type: Option<String>,
    pub status: Option<String>,
    pub details: Option<serde_json::Value>,
}
