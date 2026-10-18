use crate::client::EeroClient;
use crate::error::Result;
use crate::types::network::{
    GuestNetwork, GuestNetworkUpdate, Network, NetworkUpdate, SpeedTest,
};

impl EeroClient {
    /// Get network details by network ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_network(&self, network_id: u64) -> Result<Network> {
        let url = self.url(&format!("/networks/{network_id}"));
        self.get(&url).await
    }

    /// Get network details by resource URL.
    #[tracing::instrument(skip(self))]
    pub async fn get_network_by_url(&self, url: &str) -> Result<Network> {
        let url = self.resource_url(url);
        self.get(&url).await
    }

    /// Update network settings.
    #[tracing::instrument(skip(self, update))]
    pub async fn update_network(
        &self,
        network_id: u64,
        update: &NetworkUpdate,
    ) -> Result<Network> {
        let url = self.url(&format!("/networks/{network_id}"));
        self.put(&url, update).await
    }

    /// Reboot the entire network.
    #[tracing::instrument(skip(self))]
    pub async fn reboot_network(&self, network_id: u64) -> Result<serde_json::Value> {
        let url = self.url(&format!("/networks/{network_id}/reboot"));
        self.post_empty(&url).await
    }

    /// Run a speed test on the network.
    ///
    /// Returns `serde_json::Value` because the API may return 202 Accepted
    /// with no data body.
    #[tracing::instrument(skip(self))]
    pub async fn run_speed_test(&self, network_id: u64) -> Result<serde_json::Value> {
        let url = self.url(&format!("/networks/{network_id}/speedtest"));
        let parsed = self
            .send::<serde_json::Value>(reqwest::Method::POST, &url, None)
            .await?;
        let code = parsed.meta.code;
        if (200..300).contains(&code) {
            Ok(parsed.data.unwrap_or(serde_json::Value::Null))
        } else {
            Err(crate::error::Error::Api {
                code,
                message: parsed.meta.error.unwrap_or_else(|| "unknown error".into()),
            })
        }
    }

    /// Get the latest speed test result.
    #[tracing::instrument(skip(self))]
    pub async fn get_speed_test(&self, network_id: u64) -> Result<Vec<SpeedTest>> {
        let url = self.url(&format!("/networks/{network_id}/speedtest"));
        self.get(&url).await
    }

    /// Get guest network settings.
    #[tracing::instrument(skip(self))]
    pub async fn get_guest_network(&self, network_id: u64) -> Result<GuestNetwork> {
        let url = self.url(&format!("/networks/{network_id}/guestnetwork"));
        self.get(&url).await
    }

    /// Update guest network settings.
    #[tracing::instrument(skip(self, update))]
    pub async fn update_guest_network(
        &self,
        network_id: u64,
        update: &GuestNetworkUpdate,
    ) -> Result<GuestNetwork> {
        let url = self.url(&format!("/networks/{network_id}/guestnetwork"));
        self.put(&url, update).await
    }
}
