use crate::client::EeroClient;
use crate::error::Result;
use crate::types::eero::{EeroNode, EeroUpdate};

impl EeroClient {
    /// List all eero nodes on a network.
    #[tracing::instrument(skip(self))]
    pub async fn get_eeros(&self, network_id: u64) -> Result<Vec<EeroNode>> {
        let url = self.url(&format!("/networks/{network_id}/eeros"));
        self.get(&url).await
    }

    /// Get a specific eero node by its resource URL.
    #[tracing::instrument(skip(self))]
    pub async fn get_eero(&self, eero_url: &str) -> Result<EeroNode> {
        let url = self.resource_url(eero_url);
        self.get(&url).await
    }

    /// Update an eero node (location, LED, nightlight settings).
    #[tracing::instrument(skip(self, update))]
    pub async fn update_eero(
        &self,
        eero_url: &str,
        update: &EeroUpdate,
    ) -> Result<EeroNode> {
        let url = self.resource_url(eero_url);
        self.put(&url, update).await
    }

    /// Reboot a specific eero node.
    #[tracing::instrument(skip(self))]
    pub async fn reboot_eero(&self, eero_url: &str) -> Result<serde_json::Value> {
        let url = format!("{}/reboot", self.resource_url(eero_url));
        self.post_empty(&url).await
    }
}
