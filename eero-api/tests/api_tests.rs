#![deny(dead_code)]

mod mock_server;

use eero_api::types::data_usage::DataUsageQuery;
use eero_api::types::device::DeviceUpdate;
use eero_api::types::diagnostic::DiagnosticsRequest;
use eero_api::types::eero::EeroUpdate;
use eero_api::types::forward::NewPortForward;
use eero_api::types::network::{GuestNetworkUpdate, NetworkUpdate};
use eero_api::types::profile::ProfileUpdate;
use mock_server::MockServer;

const SESSION: &str = "test_session_token";

#[tokio::test]
async fn test_login() {
    let server = MockServer::start().await;
    let client = server.client();

    let resp = client.login("test@example.com").await.unwrap();
    assert_eq!(resp.user_token, "ut_test_token_12345");

    let stored = client.credentials().get_user_token().await.unwrap();
    assert_eq!(stored, Some("ut_test_token_12345".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_verify() {
    let server = MockServer::start().await;
    let client = server.client();

    // Login first to get user token
    client.login("test@example.com").await.unwrap();

    // Verify with the code
    client.verify("123456").await.unwrap();

    // Session token should be set, user token cleared
    let session = client.credentials().get_session_token().await.unwrap();
    assert_eq!(session, Some("session_test_token".to_string()));
    let user = client.credentials().get_user_token().await.unwrap();
    assert!(user.is_none());

    server.shutdown();
}

#[tokio::test]
async fn test_logout() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    client.logout().await.unwrap();

    let session = client.credentials().get_session_token().await.unwrap();
    assert!(session.is_none());

    server.shutdown();
}

#[tokio::test]
async fn test_get_account() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let account = client.get_account().await.unwrap();
    assert_eq!(account.name, Some("Test User".to_string()));
    assert_eq!(account.email.as_ref().unwrap().value, "test@example.com");
    assert_eq!(account.networks.count, 1);
    assert_eq!(account.networks.data.len(), 1);
    assert_eq!(account.networks.data[0].name, Some("TestNetwork".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_get_account_unauthenticated() {
    let server = MockServer::start().await;
    let client = server.client();

    let result = client.get_account().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("401"), "Expected 401 error, got: {err_str}");

    server.shutdown();
}

#[tokio::test]
async fn test_get_network() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let network = client.get_network(12345).await.unwrap();
    assert_eq!(network.name, Some("TestNetwork".to_string()));
    assert_eq!(network.sqm, Some(serde_json::Value::Bool(true)));
    assert_eq!(network.wpa3, Some(serde_json::Value::Bool(true)));
    assert!(network.health.is_some());
    assert!(network.speed.is_some());

    server.shutdown();
}

#[tokio::test]
async fn test_get_network_by_url() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let network = client
        .get_network_by_url("/2.2/networks/12345")
        .await
        .unwrap();
    assert_eq!(network.name, Some("TestNetwork".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_update_network() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let update = NetworkUpdate {
        name: Some("NewName".to_string()),
        sqm: Some(false),
        ..Default::default()
    };
    let network = client.update_network(12345, &update).await.unwrap();
    assert_eq!(network.name, Some("NewName".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_reboot_network() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let result = client.reboot_network(12345).await;
    assert!(result.is_ok());

    server.shutdown();
}

#[tokio::test]
async fn test_get_speed_test() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let speeds = client.get_speed_test(12345).await.unwrap();
    assert_eq!(speeds.len(), 1);
    assert!(speeds[0].down_mbps.is_some());

    server.shutdown();
}

#[tokio::test]
async fn test_run_speed_test() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let result = client.run_speed_test(12345).await.unwrap();
    assert_eq!(result.as_array().unwrap().len(), 1);

    server.shutdown();
}

#[tokio::test]
async fn test_get_guest_network() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let guest = client.get_guest_network(12345).await.unwrap();
    assert_eq!(guest.name, Some("TestNetwork Guest".to_string()));
    assert_eq!(guest.enabled, Some(true));

    server.shutdown();
}

#[tokio::test]
async fn test_update_guest_network() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let update = GuestNetworkUpdate {
        name: Some("New Guest".to_string()),
        enabled: Some(false),
        ..Default::default()
    };
    let guest = client.update_guest_network(12345, &update).await.unwrap();
    assert_eq!(guest.name, Some("New Guest".to_string()));
    assert_eq!(guest.enabled, Some(false));

    server.shutdown();
}

#[tokio::test]
async fn test_list_eeros() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let eeros = client.get_eeros(12345).await.unwrap();
    assert_eq!(eeros.len(), 2);
    assert_eq!(eeros[0].location, Some("Living Room".to_string()));
    assert_eq!(eeros[1].location, Some("Bedroom".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_get_eero() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let eero = client.get_eero("/2.2/eeros/1001").await.unwrap();
    assert_eq!(eero.model, Some("eero Pro 6E".to_string()));
    assert_eq!(eero.gateway, Some(true));
    assert!(eero.status_light.is_some());

    server.shutdown();
}

#[tokio::test]
async fn test_update_eero() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let update = EeroUpdate {
        location: Some("Office".to_string()),
        ..Default::default()
    };
    let eero = client.update_eero("/2.2/eeros/1001", &update).await.unwrap();
    assert_eq!(eero.location, Some("Office".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_update_eero_led() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let update = EeroUpdate {
        led_on: Some(false),
        ..Default::default()
    };
    let eero = client.update_eero("/2.2/eeros/1001", &update).await.unwrap();
    assert_eq!(eero.extra.get("led_on"), Some(&serde_json::json!(false)));

    server.shutdown();
}

#[tokio::test]
async fn test_update_eero_nightlight() {
    use eero_api::types::eero::NightlightUpdate;

    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let update = EeroUpdate {
        nightlight: Some(NightlightUpdate {
            enabled: Some(true),
            brightness: Some(75),
            ..Default::default()
        }),
        ..Default::default()
    };
    let eero = client.update_eero("/2.2/eeros/1001", &update).await.unwrap();
    let nightlight = eero.nightlight.unwrap();
    assert_eq!(nightlight["enabled"], serde_json::json!(true));
    assert_eq!(nightlight["brightness"], serde_json::json!(75));

    server.shutdown();
}

#[tokio::test]
async fn test_run_speed_test_202() {
    let server = MockServer::start_speed_test_202().await;
    let client = server.client_with_session(SESSION);

    let result = client.run_speed_test(12345).await.unwrap();
    assert_eq!(result, serde_json::Value::Null);

    server.shutdown();
}

#[tokio::test]
async fn test_reboot_eero() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let result = client.reboot_eero("/2.2/eeros/1001").await;
    assert!(result.is_ok());

    server.shutdown();
}

#[tokio::test]
async fn test_list_devices() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let devices = client.get_devices(12345).await.unwrap();
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].display_name, Some("MacBook Pro".to_string()));
    assert_eq!(devices[0].usage.as_ref().unwrap().down_mbps, Some(12.5));
    assert_eq!(devices[0].usage.as_ref().unwrap().up_mbps, Some(3.25));

    server.shutdown();
}

fn data_usage_query() -> DataUsageQuery {
    DataUsageQuery {
        start: "2026-08-24T04:00:00Z".into(),
        end: "2026-08-25T03:59:59Z".into(),
        cadence: "hourly".into(),
        timezone: "America/New_York".into(),
    }
}

#[tokio::test]
async fn test_get_data_usage() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let usage = client
        .get_data_usage(12345, &data_usage_query())
        .await
        .unwrap();
    assert_eq!(usage.series.len(), 2);
    assert_eq!(usage.series[0].usage_type, "upload");
    assert_eq!(usage.series[0].sum, Some(300));
    assert_eq!(usage.series[1].values[0].value, Some(1200));

    server.shutdown();
}

#[tokio::test]
async fn test_get_device_data_usage() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let usage = client
        .get_device_data_usage(12345, &data_usage_query())
        .await
        .unwrap();
    assert_eq!(usage.values.len(), 1);
    assert_eq!(usage.values[0].download, Some(1200));
    assert_eq!(usage.values[0].upload, Some(300));

    server.shutdown();
}

#[tokio::test]
async fn test_get_eero_data_usage() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let usage = client
        .get_eero_data_usage(12345, &data_usage_query())
        .await
        .unwrap();
    assert_eq!(usage.values.len(), 1);
    assert_eq!(usage.values[0].location.as_deref(), Some("Living Room"));
    assert_eq!(usage.values[0].download, Some(1200));

    server.shutdown();
}

#[tokio::test]
async fn test_get_device() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let device = client.get_device("/2.2/devices/2001").await.unwrap();
    assert_eq!(device.mac, Some("11:22:33:44:55:66".to_string()));
    assert_eq!(device.connected, Some(true));

    server.shutdown();
}

#[tokio::test]
async fn test_update_device() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    // The mock, like eero, ignores device PUTs on 2.2 and applies them on 2.3.
    let update = DeviceUpdate {
        nickname: Some("My Laptop".to_string()),
        blocked: Some(true),
        paused: Some(true),
        ..Default::default()
    };
    let device = client
        .update_device("/2.2/devices/2001", &update)
        .await
        .unwrap();
    assert_eq!(device.nickname, Some("My Laptop".to_string()));
    assert_eq!(device.blocked, Some(true));
    assert_eq!(device.paused, Some(true));

    server.shutdown();
}

#[tokio::test]
async fn test_list_profiles() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let profiles = client.get_profiles(12345).await.unwrap();
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0].name, Some("Default".to_string()));
    assert_eq!(profiles[1].name, Some("Kids".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_get_profile() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let profile = client.get_profile("/2.2/profiles/3001").await.unwrap();
    assert_eq!(profile.name, Some("Default".to_string()));
    assert_eq!(profile.paused, Some(false));

    server.shutdown();
}

#[tokio::test]
async fn test_update_profile() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let update = ProfileUpdate {
        name: Some("Adults".to_string()),
        paused: Some(true),
        ..Default::default()
    };
    let profile = client
        .update_profile("/2.2/profiles/3001", &update)
        .await
        .unwrap();
    assert_eq!(profile.name, Some("Adults".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_list_port_forwards() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let forwards = client.get_port_forwards(12345).await.unwrap();
    assert_eq!(forwards.len(), 1);
    assert_eq!(forwards[0].port, Some("8080".to_string()));
    assert_eq!(forwards[0].protocol, Some("tcp".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_create_port_forward() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let forward = NewPortForward {
        ip_address: "192.168.4.100".to_string(),
        port: "3000".to_string(),
        external_port: Some("3000".to_string()),
        protocol: Some("tcp".to_string()),
    };
    let result = client.create_port_forward(12345, &forward).await.unwrap();
    assert_eq!(result.enabled, Some(true));

    server.shutdown();
}

#[tokio::test]
async fn test_delete_port_forward() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let result = client.delete_port_forward("/2.2/forwards/4001").await;
    assert!(result.is_ok());

    server.shutdown();
}

#[tokio::test]
async fn test_get_activity() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let activity = client.get_activity(12345).await.unwrap();
    assert!(activity.insights.is_some());
    let insights = activity.insights.unwrap();
    assert_eq!(insights.len(), 2);
    assert!(activity.counts.is_some());
    let counts = activity.counts.unwrap();
    assert_eq!(counts.blocked, Some(42));

    server.shutdown();
}

#[tokio::test]
async fn test_get_client_activity() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let activity = client
        .get_client_activity("/2.2/devices/2001")
        .await
        .unwrap();
    assert_eq!(activity.display_name, Some("MacBook Pro".to_string()));
    assert!(activity.insights.is_some());

    server.shutdown();
}

#[tokio::test]
async fn test_get_diagnostics() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let diag = client.get_diagnostics(12345).await.unwrap();
    assert_eq!(diag.status, Some("completed".to_string()));
    assert!(diag.results.is_some());
    let results = diag.results.unwrap();
    assert_eq!(results.len(), 2);

    server.shutdown();
}

#[tokio::test]
async fn test_run_diagnostics() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let request = DiagnosticsRequest {
        symptom: Some("slow".to_string()),
        device: None,
    };
    let diag = client.run_diagnostics(12345, &request).await.unwrap();
    assert_eq!(diag.status, Some("completed".to_string()));

    server.shutdown();
}

#[tokio::test]
async fn test_run_diagnostics_without_symptom_names_the_field() {
    let server = MockServer::start().await;
    let client = server.client_with_session(SESSION);

    let err = client
        .run_diagnostics(12345, &DiagnosticsRequest::default())
        .await
        .unwrap_err();
    match err {
        eero_api::Error::Api { code, message } => {
            assert_eq!(code, 400);
            assert_eq!(
                message,
                "error.form.errors (symptom: error.form.field.required)"
            );
        }
        other => panic!("expected an API error, got {other:?}"),
    }

    server.shutdown();
}

#[tokio::test]
async fn test_api_error_response() {
    let server = MockServer::start().await;
    let client = server.client(); // no session token

    let result = client.get_network(12345).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        eero_api::Error::Api { code, message } => {
            assert_eq!(code, 401);
            assert_eq!(message, "unauthorized");
        }
        other => panic!("Expected Api error, got: {other:?}"),
    }

    server.shutdown();
}

#[tokio::test]
async fn test_verify_no_cookie() {
    let server = MockServer::start_verify_no_cookie().await;
    let client = server.client();

    // Login stores user_token
    client.login("test@example.com").await.unwrap();

    // Verify against a server that returns no Set-Cookie header.
    // The client should promote user_token to session_token.
    client.verify("123456").await.unwrap();

    let session = client.credentials().get_session_token().await.unwrap();
    assert_eq!(session, Some("ut_test_token_12345".to_string()));
    let user = client.credentials().get_user_token().await.unwrap();
    assert!(user.is_none());

    server.shutdown();
}

#[tokio::test]
async fn test_get_account_with_user_token() {
    let server = MockServer::start().await;
    let client = server.client_with_user_token("fallback_token");

    // session_headers() should fall back to user_token when no session_token
    let account = client.get_account().await.unwrap();
    assert_eq!(account.name, Some("Test User".to_string()));

    server.shutdown();
}

#[test]
fn test_account_extra_fields() {
    let json = r#"{
        "url": "/2.2/account",
        "name": "Test",
        "networks": { "count": 0, "data": [] },
        "unknown_field": "surprise",
        "another": 42
    }"#;
    let account: eero_api::types::account::Account = serde_json::from_str(json).unwrap();
    assert_eq!(account.name, Some("Test".to_string()));
    assert!(account.extra.contains_key("unknown_field"));
    assert_eq!(account.extra["another"], serde_json::json!(42));
}

#[test]
fn test_network_extra_fields() {
    let json = r#"{
        "url": "/2.2/networks/1",
        "name": "Net",
        "new_eero_feature": true,
        "future_field": "hello"
    }"#;
    let network: eero_api::types::network::Network = serde_json::from_str(json).unwrap();
    assert_eq!(network.name, Some("Net".to_string()));
    assert_eq!(network.extra["new_eero_feature"], serde_json::json!(true));
    assert_eq!(network.extra["future_field"], serde_json::json!("hello"));
}

#[test]
fn test_eero_extra_fields() {
    let json = r#"{
        "url": "/2.2/eeros/1",
        "model": "eero Pro 6E",
        "location": "Office",
        "some_new_field": [1, 2, 3]
    }"#;
    let eero: eero_api::types::eero::EeroNode = serde_json::from_str(json).unwrap();
    assert_eq!(eero.model, Some("eero Pro 6E".to_string()));
    assert_eq!(eero.extra["some_new_field"], serde_json::json!([1, 2, 3]));
}
