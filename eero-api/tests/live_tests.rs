#![deny(dead_code)]

//! Live integration tests for the eero API.
//!
//! These tests are gated behind the EERO_LIVE_TEST=1 environment variable
//! and require valid credentials (authenticate via eeroctl first).
//!
//! All tests are READ-ONLY to avoid modifying the user's network.

fn is_live() -> bool {
    std::env::var("EERO_LIVE_TEST")
        .map(|v| v == "1")
        .unwrap_or(false)
}

macro_rules! skip_unless_live {
    () => {
        if !is_live() {
            eprintln!("Skipping live test (set EERO_LIVE_TEST=1 to enable)");
            return;
        }
    };
}

async fn get_network_id() -> (eero_api::EeroClient, u64) {
    let client = eero_api::EeroClient::new().unwrap();
    let account = client.get_account().await.unwrap();
    assert!(!account.networks.data.is_empty(), "No networks found");
    let url = &account.networks.data[0].url;
    // Extract network ID from URL like /2.2/networks/12345
    let id: u64 = url
        .rsplit('/')
        .next()
        .unwrap()
        .parse()
        .expect("Could not parse network ID from URL");
    (client, id)
}

#[tokio::test]
async fn live_get_account() {
    skip_unless_live!();
    let client = eero_api::EeroClient::new().unwrap();
    let account = client.get_account().await.unwrap();
    assert!(account.url.is_some() || account.name.is_some());
    eprintln!("Account: {:?}", account.name);
}

#[tokio::test]
async fn live_get_network() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let network = client.get_network(id).await.unwrap();
    eprintln!("Network: {:?}", network.name);
    assert!(network.name.is_some());
}

#[tokio::test]
async fn live_get_eeros() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let eeros = client.get_eeros(id).await.unwrap();
    assert!(!eeros.is_empty());
    eprintln!("Eeros: {}", eeros.len());
    for eero in &eeros {
        eprintln!("  - {:?} ({:?})", eero.location, eero.model);
    }
}

#[tokio::test]
async fn live_get_devices() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let devices = client.get_devices(id).await.unwrap();
    eprintln!("Devices: {}", devices.len());
    assert!(devices.iter().all(|device| device.usage.is_some()));
}

#[tokio::test]
async fn live_get_data_usage() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let end = time::OffsetDateTime::now_utc();
    let start = end - time::Duration::hours(24);
    let query = eero_api::types::data_usage::DataUsageQuery {
        start: start
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
        end: end
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
        cadence: "hourly".into(),
        timezone: "UTC".into(),
    };

    let network_usage = client.get_data_usage(id, &query).await.unwrap();
    assert!(!network_usage.series.is_empty());

    let device_usage = client.get_device_data_usage(id, &query).await.unwrap();
    assert!(!device_usage.values.is_empty());

    let eero_usage = client.get_eero_data_usage(id, &query).await.unwrap();
    assert!(!eero_usage.values.is_empty());
}

#[tokio::test]
async fn live_get_profiles() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let profiles = client.get_profiles(id).await.unwrap();
    eprintln!("Profiles: {}", profiles.len());
}

#[tokio::test]
async fn live_get_port_forwards() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let forwards = client.get_port_forwards(id).await.unwrap();
    eprintln!("Port forwards: {}", forwards.len());
}

#[tokio::test]
async fn live_get_reservations() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let reservations = client.get_reservations(id).await.unwrap();
    eprintln!("Reservations: {}", reservations.len());
}

#[tokio::test]
async fn live_get_speed_test() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let speeds = client.get_speed_test(id).await.unwrap();
    eprintln!("Speed tests: {}", speeds.len());
}

#[tokio::test]
async fn live_get_guest_network() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let guest = client.get_guest_network(id).await.unwrap();
    eprintln!(
        "Guest network: {:?} (enabled: {:?})",
        guest.name, guest.enabled
    );
}

#[tokio::test]
async fn live_get_diagnostics() {
    skip_unless_live!();
    let (client, id) = get_network_id().await;
    let diag = client.get_diagnostics(id).await.unwrap();
    eprintln!("Diagnostics status: {:?}", diag.status);
}
