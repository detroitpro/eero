use crate::client::EeroClient;
use crate::error::Result;
use crate::types::forward::{NewPortForward, PortForward};

impl EeroClient {
    /// List all port forwards on a network.
    #[tracing::instrument(skip(self))]
    pub async fn get_port_forwards(&self, network_id: u64) -> Result<Vec<PortForward>> {
        let url = self.url(&format!("/networks/{network_id}/forwards"));
        self.get(&url).await
    }

    /// Create a new port forward.
    #[tracing::instrument(skip(self, forward))]
    pub async fn create_port_forward(
        &self,
        network_id: u64,
        forward: &NewPortForward,
    ) -> Result<PortForward> {
        let url = self.url(&format!("/networks/{network_id}/forwards"));
        self.post(&url, forward).await
    }

    /// Delete a port forward by its resource URL.
    #[tracing::instrument(skip(self))]
    pub async fn delete_port_forward(&self, forward_url: &str) -> Result<serde_json::Value> {
        let url = self.resource_url(forward_url);
        self.delete(&url).await
    }
}
