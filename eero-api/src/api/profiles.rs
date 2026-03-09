use crate::client::EeroClient;
use crate::error::Result;
use crate::types::profile::{Profile, ProfileUpdate};

impl EeroClient {
    /// List all profiles on a network.
    #[tracing::instrument(skip(self))]
    pub async fn get_profiles(&self, network_id: u64) -> Result<Vec<Profile>> {
        let url = self.url(&format!("/networks/{network_id}/profiles"));
        self.get(&url).await
    }

    /// Get a specific profile by its resource URL.
    #[tracing::instrument(skip(self))]
    pub async fn get_profile(&self, profile_url: &str) -> Result<Profile> {
        let url = self.resource_url(profile_url);
        self.get(&url).await
    }

    /// Update a profile (parental controls, schedule, DNS policies).
    #[tracing::instrument(skip(self, update))]
    pub async fn update_profile(
        &self,
        profile_url: &str,
        update: &ProfileUpdate,
    ) -> Result<Profile> {
        let url = self.resource_url(profile_url);
        self.put(&url, update).await
    }
}
