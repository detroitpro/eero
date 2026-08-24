use crate::client::EeroClient;
use crate::error::Result;
use crate::types::data_usage::{
    DataUsage, DataUsageBreakdown, DataUsageQuery, DeviceDataUsage, EeroDataUsage,
};

impl EeroClient {
    /// Get network-wide byte totals for a time window.
    pub async fn get_data_usage(
        &self,
        network_id: u64,
        query: &DataUsageQuery,
    ) -> Result<DataUsage> {
        let url = self.url(&format!("/networks/{network_id}/data_usage"));
        self.get_with_json_body(&url, query).await
    }

    /// Get per-device byte totals for a time window.
    pub async fn get_device_data_usage(
        &self,
        network_id: u64,
        query: &DataUsageQuery,
    ) -> Result<DataUsageBreakdown<DeviceDataUsage>> {
        let url = self.url(&format!("/networks/{network_id}/data_usage/devices"));
        self.get_with_json_body(&url, query).await
    }

    /// Get per-eero-node byte totals for a time window.
    pub async fn get_eero_data_usage(
        &self,
        network_id: u64,
        query: &DataUsageQuery,
    ) -> Result<DataUsageBreakdown<EeroDataUsage>> {
        let url = self.url(&format!("/networks/{network_id}/data_usage/eeros"));
        self.get_with_json_body(&url, query).await
    }
}
