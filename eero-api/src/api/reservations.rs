use crate::client::EeroClient;
use crate::error::Result;
use crate::types::reservation::{NewReservation, Reservation};

impl EeroClient {
    /// List DHCP reservations on a network.
    #[tracing::instrument(skip(self))]
    pub async fn get_reservations(&self, network_id: u64) -> Result<Vec<Reservation>> {
        let url = self.url(&format!("/networks/{network_id}/reservations"));
        self.get(&url).await
    }

    /// Reserve an IP address for a MAC address.
    #[tracing::instrument(skip(self, reservation))]
    pub async fn create_reservation(
        &self,
        network_id: u64,
        reservation: &NewReservation,
    ) -> Result<Reservation> {
        let url = self.url(&format!("/networks/{network_id}/reservations"));
        self.post(&url, reservation).await
    }

    /// Delete a DHCP reservation by its resource URL.
    #[tracing::instrument(skip(self))]
    pub async fn delete_reservation(&self, reservation_url: &str) -> Result<serde_json::Value> {
        let url = self.resource_url(reservation_url);
        self.delete(&url).await
    }
}
