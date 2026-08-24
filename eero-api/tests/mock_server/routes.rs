use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use super::fixtures;

fn wrap(data: Value) -> Value {
    json!({
        "meta": { "code": 200, "server_time": "2024-06-15T12:00:00Z" },
        "data": data
    })
}

fn wrap_empty() -> Value {
    json!({
        "meta": { "code": 200, "server_time": "2024-06-15T12:00:00Z" },
        "data": {}
    })
}

fn error_response(code: u16, message: &str) -> Response {
    let body = json!({
        "meta": { "code": code, "error": message },
        "data": null
    });
    (StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(body))
        .into_response()
}

fn check_auth(headers: &HeaderMap) -> std::result::Result<(), Response> {
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if cookie.contains("s=") {
        let token = cookie
            .split("s=")
            .nth(1)
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("");
        if !token.is_empty() {
            return Ok(());
        }
    }

    Err(error_response(401, "unauthorized"))
}

async fn login(Json(body): Json<Value>) -> Response {
    let _login = body.get("login").and_then(|v| v.as_str()).unwrap_or("");
    let data = fixtures::login_response();
    let resp = json!({
        "meta": { "code": 200, "server_time": "2024-06-15T12:00:00Z" },
        "data": data
    });
    Json(resp).into_response()
}

async fn verify(headers: HeaderMap, Json(body): Json<Value>) -> Response {
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !cookie.contains("s=") {
        return error_response(401, "unauthorized");
    }

    let code = body.get("code").and_then(|v| v.as_str()).unwrap_or("");
    if code.is_empty() {
        return error_response(400, "invalid verification code");
    }

    let resp = json!({
        "meta": { "code": 200, "server_time": "2024-06-15T12:00:00Z" },
        "data": {}
    });

    let mut response = Json(resp).into_response();
    response.headers_mut().insert(
        "set-cookie",
        format!("s=session_test_token; Path=/; HttpOnly")
            .parse()
            .unwrap(),
    );
    response
}

async fn verify_no_cookie(headers: HeaderMap, Json(body): Json<Value>) -> Response {
    let cookie = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !cookie.contains("s=") {
        return error_response(401, "unauthorized");
    }

    let code = body.get("code").and_then(|v| v.as_str()).unwrap_or("");
    if code.is_empty() {
        return error_response(400, "invalid verification code");
    }

    let resp = json!({
        "meta": { "code": 200, "server_time": "2024-06-15T12:00:00Z" },
        "data": {}
    });

    Json(resp).into_response()
}

async fn logout(headers: HeaderMap) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap_empty()).into_response()
}

async fn get_account(headers: HeaderMap) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::account())).into_response()
}

async fn get_network(headers: HeaderMap, Path(network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    let _ = network_id;
    Json(wrap(fixtures::network())).into_response()
}

async fn update_network(
    headers: HeaderMap,
    Path(network_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    let _ = network_id;
    let mut net = fixtures::network();
    if let Some(obj) = body.as_object() {
        if let Some(name) = obj.get("name") {
            net["name"] = name.clone();
        }
        if let Some(sqm) = obj.get("sqm") {
            net["sqm"] = sqm.clone();
        }
        if let Some(band_steering) = obj.get("band_steering") {
            net["band_steering"] = band_steering.clone();
        }
        if let Some(wpa3) = obj.get("wpa3") {
            net["wpa3"] = wpa3.clone();
        }
    }
    Json(wrap(net)).into_response()
}

async fn reboot_network(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(json!({}))).into_response()
}

async fn speed_test_get(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::speed_test())).into_response()
}

async fn speed_test_post(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::speed_test())).into_response()
}

async fn speed_test_post_202(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    let body = json!({
        "meta": { "code": 202 },
        "data": null
    });
    (StatusCode::ACCEPTED, Json(body)).into_response()
}

async fn get_guest(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::guest_network())).into_response()
}

async fn update_guest(
    headers: HeaderMap,
    Path(_network_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    let mut guest = fixtures::guest_network();
    if let Some(obj) = body.as_object() {
        if let Some(name) = obj.get("name") {
            guest["name"] = name.clone();
        }
        if let Some(enabled) = obj.get("enabled") {
            guest["enabled"] = enabled.clone();
        }
    }
    Json(wrap(guest)).into_response()
}

async fn list_eeros(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(json!([fixtures::eero_node(), fixtures::eero_node_2()]))).into_response()
}

async fn get_eero(headers: HeaderMap, Path(_eero_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::eero_node())).into_response()
}

async fn update_eero(
    headers: HeaderMap,
    Path(_eero_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    let mut eero = fixtures::eero_node();
    if let Some(obj) = body.as_object() {
        if let Some(location) = obj.get("location") {
            eero["location"] = location.clone();
        }
        if let Some(led_on) = obj.get("led_on") {
            eero["led_on"] = led_on.clone();
        }
        if let Some(nightlight) = obj.get("nightlight") {
            if let (Some(existing), Some(incoming)) = (eero["nightlight"].as_object_mut(), nightlight.as_object()) {
                for (k, v) in incoming {
                    existing.insert(k.clone(), v.clone());
                }
            }
        }
    }
    Json(wrap(eero)).into_response()
}

async fn reboot_eero(headers: HeaderMap, Path(_eero_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(json!({}))).into_response()
}

async fn list_devices(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(json!([fixtures::device(), fixtures::device_2()]))).into_response()
}

fn validate_data_usage_query(body: &Value) -> Option<Response> {
    for field in ["start", "end", "cadence", "timezone"] {
        if body.get(field).and_then(Value::as_str).is_none() {
            return Some(error_response(400, &format!("missing {field}")));
        }
    }
    None
}

async fn get_data_usage(
    headers: HeaderMap,
    Path(_network_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    if let Some(e) = validate_data_usage_query(&body) {
        return e;
    }
    Json(wrap(fixtures::data_usage())).into_response()
}

async fn get_device_data_usage(
    headers: HeaderMap,
    Path(_network_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    if let Some(e) = validate_data_usage_query(&body) {
        return e;
    }
    Json(wrap(fixtures::device_data_usage())).into_response()
}

async fn get_eero_data_usage(
    headers: HeaderMap,
    Path(_network_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    if let Some(e) = validate_data_usage_query(&body) {
        return e;
    }
    Json(wrap(fixtures::eero_data_usage())).into_response()
}

async fn get_device(headers: HeaderMap, Path(_device_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::device())).into_response()
}

async fn update_device(
    headers: HeaderMap,
    Path(_device_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    let mut device = fixtures::device();
    if let Some(obj) = body.as_object() {
        if let Some(nickname) = obj.get("nickname") {
            device["nickname"] = nickname.clone();
        }
        if let Some(blocked) = obj.get("blocked") {
            device["blocked"] = blocked.clone();
        }
        if let Some(paused) = obj.get("paused") {
            device["paused"] = paused.clone();
        }
    }
    Json(wrap(device)).into_response()
}

async fn list_profiles(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(json!([fixtures::profile(), fixtures::profile_2()]))).into_response()
}

async fn get_profile(headers: HeaderMap, Path(_profile_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::profile())).into_response()
}

async fn update_profile(
    headers: HeaderMap,
    Path(_profile_id): Path<u64>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    let mut profile = fixtures::profile();
    if let Some(obj) = body.as_object() {
        if let Some(name) = obj.get("name") {
            profile["name"] = name.clone();
        }
        if let Some(paused) = obj.get("paused") {
            profile["paused"] = paused.clone();
        }
    }
    Json(wrap(profile)).into_response()
}

async fn list_forwards(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(json!([fixtures::port_forward()]))).into_response()
}

async fn create_forward(
    headers: HeaderMap,
    Path(_network_id): Path<u64>,
    Json(_body): Json<Value>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::port_forward())).into_response()
}

async fn delete_forward(headers: HeaderMap, Path(_forward_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(json!({}))).into_response()
}

async fn get_activity(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::activity())).into_response()
}

async fn get_client_activity(
    headers: HeaderMap,
    Path((_device_id,)): Path<(u64,)>,
) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::client_activity())).into_response()
}

async fn get_diagnostics(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::diagnostics())).into_response()
}

async fn post_diagnostics(headers: HeaderMap, Path(_network_id): Path<u64>) -> Response {
    if let Err(e) = check_auth(&headers) {
        return e;
    }
    Json(wrap(fixtures::diagnostics())).into_response()
}

fn common_routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/account", get(get_account))
        .route("/networks/{network_id}", get(get_network).put(update_network))
        .route("/networks/{network_id}/reboot", post(reboot_network))
        .route(
            "/networks/{network_id}/speedtest",
            get(speed_test_get).post(speed_test_post),
        )
        .route(
            "/networks/{network_id}/guestnetwork",
            get(get_guest).put(update_guest),
        )
        .route("/networks/{network_id}/eeros", get(list_eeros))
        .route("/eeros/{eero_id}", get(get_eero).put(update_eero))
        .route("/eeros/{eero_id}/reboot", post(reboot_eero))
        .route("/networks/{network_id}/devices", get(list_devices))
        .route("/networks/{network_id}/data_usage", get(get_data_usage))
        .route(
            "/networks/{network_id}/data_usage/devices",
            get(get_device_data_usage),
        )
        .route(
            "/networks/{network_id}/data_usage/eeros",
            get(get_eero_data_usage),
        )
        .route(
            "/devices/{device_id}",
            get(get_device).put(update_device),
        )
        .route("/devices/{device_id}/activity", get(get_client_activity))
        .route("/networks/{network_id}/profiles", get(list_profiles))
        .route(
            "/profiles/{profile_id}",
            get(get_profile).put(update_profile),
        )
        .route(
            "/networks/{network_id}/forwards",
            get(list_forwards).post(create_forward),
        )
        .route("/forwards/{forward_id}", delete(delete_forward))
        .route(
            "/networks/{network_id}/activity",
            get(get_activity),
        )
        .route(
            "/networks/{network_id}/diagnostics",
            get(get_diagnostics).post(post_diagnostics),
        )
}

pub fn router() -> Router {
    let api = common_routes()
        .route("/login/verify", post(verify))
        .route("/login/verify_no_cookie", post(verify_no_cookie));

    Router::new().nest("/2.2", api)
}

fn common_routes_speed_test_202() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/account", get(get_account))
        .route("/networks/{network_id}", get(get_network).put(update_network))
        .route("/networks/{network_id}/reboot", post(reboot_network))
        .route(
            "/networks/{network_id}/speedtest",
            get(speed_test_get).post(speed_test_post_202),
        )
        .route(
            "/networks/{network_id}/guestnetwork",
            get(get_guest).put(update_guest),
        )
        .route("/networks/{network_id}/eeros", get(list_eeros))
        .route("/eeros/{eero_id}", get(get_eero).put(update_eero))
        .route("/eeros/{eero_id}/reboot", post(reboot_eero))
        .route("/networks/{network_id}/devices", get(list_devices))
        .route("/networks/{network_id}/data_usage", get(get_data_usage))
        .route(
            "/networks/{network_id}/data_usage/devices",
            get(get_device_data_usage),
        )
        .route(
            "/networks/{network_id}/data_usage/eeros",
            get(get_eero_data_usage),
        )
        .route(
            "/devices/{device_id}",
            get(get_device).put(update_device),
        )
        .route("/devices/{device_id}/activity", get(get_client_activity))
        .route("/networks/{network_id}/profiles", get(list_profiles))
        .route(
            "/profiles/{profile_id}",
            get(get_profile).put(update_profile),
        )
        .route(
            "/networks/{network_id}/forwards",
            get(list_forwards).post(create_forward),
        )
        .route("/forwards/{forward_id}", delete(delete_forward))
        .route(
            "/networks/{network_id}/activity",
            get(get_activity),
        )
        .route(
            "/networks/{network_id}/diagnostics",
            get(get_diagnostics).post(post_diagnostics),
        )
}

pub fn router_speed_test_202() -> Router {
    let api = common_routes_speed_test_202()
        .route("/login/verify", post(verify));

    Router::new().nest("/2.2", api)
}

pub fn router_verify_no_cookie() -> Router {
    let api = common_routes()
        .route("/login/verify", post(verify_no_cookie));

    Router::new().nest("/2.2", api)
}
