#![deny(dead_code)]

use eero_api::types::device::DeviceUpdate;
use eero_api::types::eero::EeroUpdate;
use eero_api::types::envelope::{ApiResponse, Meta};
use eero_api::types::forward::NewPortForward;
use eero_api::types::network::{GuestNetworkUpdate, NetworkUpdate};
use eero_api::types::profile::ProfileUpdate;
use eero_api::EeroClient;

#[test]
fn test_network_update_empty() {
    let update = NetworkUpdate::default();
    let json = serde_json::to_string(&update).unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn test_network_update_partial() {
    let update = NetworkUpdate {
        name: Some("NewName".to_string()),
        sqm: Some(false),
        ..Default::default()
    };
    let json = serde_json::to_value(&update).unwrap();
    assert_eq!(json.get("name").unwrap(), "NewName");
    assert_eq!(json.get("sqm").unwrap(), false);
    assert!(json.get("password").is_none());
    assert!(json.get("band_steering").is_none());
}

#[test]
fn test_eero_update_empty() {
    let update = EeroUpdate::default();
    let json = serde_json::to_string(&update).unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn test_eero_update_partial() {
    let update = EeroUpdate {
        location: Some("Kitchen".to_string()),
        led_on: Some(false),
        ..Default::default()
    };
    let json = serde_json::to_value(&update).unwrap();
    assert_eq!(json.get("location").unwrap(), "Kitchen");
    assert_eq!(json.get("led_on").unwrap(), false);
    assert!(json.get("nightlight").is_none());
}

#[test]
fn test_device_update_empty() {
    let update = DeviceUpdate::default();
    let json = serde_json::to_string(&update).unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn test_device_update_partial() {
    let update = DeviceUpdate {
        nickname: Some("Phone".to_string()),
        blocked: Some(true),
        ..Default::default()
    };
    let json = serde_json::to_value(&update).unwrap();
    assert_eq!(json.get("nickname").unwrap(), "Phone");
    assert_eq!(json.get("blocked").unwrap(), true);
    assert!(json.get("paused").is_none());
    assert!(json.get("profile").is_none());
}

#[test]
fn test_profile_update_empty() {
    let update = ProfileUpdate::default();
    let json = serde_json::to_string(&update).unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn test_profile_update_partial() {
    let update = ProfileUpdate {
        name: Some("Teens".to_string()),
        paused: Some(true),
        ..Default::default()
    };
    let json = serde_json::to_value(&update).unwrap();
    assert_eq!(json.get("name").unwrap(), "Teens");
    assert_eq!(json.get("paused").unwrap(), true);
    assert!(json.get("blocked_applications").is_none());
}

#[test]
fn test_guest_network_update_empty() {
    let update = GuestNetworkUpdate::default();
    let json = serde_json::to_string(&update).unwrap();
    assert_eq!(json, "{}");
}

#[test]
fn test_new_port_forward_required_fields() {
    let forward = NewPortForward {
        ip_address: "192.168.1.100".to_string(),
        port: "8080".to_string(),
        external_port: None,
        protocol: None,
    };
    let json = serde_json::to_value(&forward).unwrap();
    assert_eq!(json.get("ip_address").unwrap(), "192.168.1.100");
    assert_eq!(json.get("port").unwrap(), "8080");
    assert!(json.get("external_port").is_none());
    assert!(json.get("protocol").is_none());
}

#[test]
fn test_new_port_forward_all_fields() {
    let forward = NewPortForward {
        ip_address: "192.168.1.100".to_string(),
        port: "8080".to_string(),
        external_port: Some("80".to_string()),
        protocol: Some("tcp".to_string()),
    };
    let json = serde_json::to_value(&forward).unwrap();
    assert_eq!(json.get("ip_address").unwrap(), "192.168.1.100");
    assert_eq!(json.get("port").unwrap(), "8080");
    assert_eq!(json.get("external_port").unwrap(), "80");
    assert_eq!(json.get("protocol").unwrap(), "tcp");
}

#[test]
fn test_api_response_unwrap_success() {
    let response: ApiResponse<String> = ApiResponse {
        meta: Meta {
            code: 200,
            server_time: Some("2024-01-01T00:00:00Z".to_string()),
            error: None,
        },
        data: Some("hello".to_string()),
    };
    let result = EeroClient::unwrap_response(response).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_api_response_unwrap_error() {
    let response: ApiResponse<String> = ApiResponse {
        meta: Meta {
            code: 401,
            server_time: None,
            error: Some("unauthorized".to_string()),
        },
        data: None,
    };
    let result = EeroClient::unwrap_response(response);
    assert!(result.is_err());
    match result.unwrap_err() {
        eero_api::Error::Api { code, message } => {
            assert_eq!(code, 401);
            assert_eq!(message, "unauthorized");
        }
        other => panic!("Expected Api error, got: {other:?}"),
    }
}

#[test]
fn test_api_response_unwrap_no_data() {
    let response: ApiResponse<String> = ApiResponse {
        meta: Meta {
            code: 200,
            server_time: None,
            error: None,
        },
        data: None,
    };
    let result = EeroClient::unwrap_response(response);
    assert!(result.is_err());
    match result.unwrap_err() {
        eero_api::Error::Api { code, message } => {
            assert_eq!(code, 200);
            assert!(message.contains("no data"));
        }
        other => panic!("Expected Api error, got: {other:?}"),
    }
}
