use crate::client::EeroClient;
use crate::error::Result;
use crate::types::device::{Device, DeviceUpdate};

/// Device writes only take effect on API 2.3. The device URLs the API hands
/// out are 2.2, where a PUT returns 200 and is silently ignored.
fn device_write_url(device_url: &str) -> String {
    device_url.replacen("/2.2/", "/2.3/", 1)
}

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
        let url = self.resource_url(&device_write_url(device_url));
        self.put(&url, update).await
    }
}

#[cfg(test)]
mod tests {
    use super::device_write_url;

    #[test]
    fn rewrites_only_the_version_segment() {
        assert_eq!(
            device_write_url("/2.2/networks/12345/devices/2001"),
            "/2.3/networks/12345/devices/2001"
        );
        assert_eq!(
            device_write_url("https://api-user.e2ro.com/2.2/networks/12345/devices/2001"),
            "https://api-user.e2ro.com/2.3/networks/12345/devices/2001"
        );
        assert_eq!(
            device_write_url("/2.3/networks/12345/devices/2001"),
            "/2.3/networks/12345/devices/2001"
        );
    }
}
