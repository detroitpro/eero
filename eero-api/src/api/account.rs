use crate::client::EeroClient;
use crate::error::Result;
use crate::types::account::Account;

impl EeroClient {
    /// Get the authenticated user's account information.
    #[tracing::instrument(skip(self))]
    pub async fn get_account(&self) -> Result<Account> {
        let url = self.url("/account");
        self.get(&url).await
    }
}
