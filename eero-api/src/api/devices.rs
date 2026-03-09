use crate::client::EeroClient;
use crate::error::Result;
use crate::types::device::{Device, DeviceUpdate};

impl EeroClient {
    /// List all devices on a network.
    #[tracing::instrument(skip(self))]
    pub async fn get_devices(&self, network_id: u64) -> Result<Vec<Device>> {
        let url = self.url(&format!("/networks/{network_id}/devices"));
        self.get(&url).await
    }

    /// Get a specific device by its resource URL.
    #[tracing::instrument(skip(self))]
    pub async fn get_device(&self, device_url: &str) -> Result<Device> {
        let url = self.resource_url(device_url);
        self.get(&url).await
    }

    /// Update a device (nickname, blocked, paused, profile assignment).
    #[tracing::instrument(skip(self, update))]
    pub async fn update_device(
        &self,
        device_url: &str,
        update: &DeviceUpdate,
    ) -> Result<Device> {
        let url = self.resource_url(device_url);
        self.put(&url, update).await
    }
}
