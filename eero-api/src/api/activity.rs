use crate::client::EeroClient;
use crate::error::Result;
use crate::types::activity::{ActivitySummary, ClientActivity};

impl EeroClient {
    /// Get network activity summary (requires eero Plus).
    #[tracing::instrument(skip(self))]
    pub async fn get_activity(&self, network_id: u64) -> Result<ActivitySummary> {
        let url = self.url(&format!("/networks/{network_id}/activity"));
        self.get(&url).await
    }

    /// Get activity for a specific client/device (requires eero Plus).
    #[tracing::instrument(skip(self))]
    pub async fn get_client_activity(&self, device_url: &str) -> Result<ClientActivity> {
        let url = format!("{}/activity", self.resource_url(device_url));
        self.get(&url).await
    }
}
