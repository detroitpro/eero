use crate::client::EeroClient;
use crate::error::Result;
use crate::types::diagnostic::{DiagnosticReport, DiagnosticsRequest};

impl EeroClient {
    /// Get the latest diagnostic report for a network.
    #[tracing::instrument(skip(self))]
    pub async fn get_diagnostics(&self, network_id: u64) -> Result<DiagnosticReport> {
        let url = self.url(&format!("/networks/{network_id}/diagnostics"));
        self.get(&url).await
    }

    /// Run a new diagnostic check on the network.
    #[tracing::instrument(skip(self, request))]
    pub async fn run_diagnostics(
        &self,
        network_id: u64,
        request: &DiagnosticsRequest,
    ) -> Result<DiagnosticReport> {
        let url = self.url(&format!("/networks/{network_id}/diagnostics"));
        self.post(&url, request).await
    }
}
