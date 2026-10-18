//! Its own test binary: tracing caches per-callsite interest process-wide, so
//! a subscriber installed by one test among many misses events from callsites
//! another thread registered first. A global subscriber here sees them all.

#[allow(dead_code)]
mod mock_server;

use std::sync::{Arc, Mutex};

use eero_api::types::network::NetworkUpdate;
use mock_server::MockServer;

#[derive(Clone, Default)]
struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for LogBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn trace_logs_never_contain_credentials_or_contact_details() {
    let logs = LogBuffer::default();
    let writer = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let server = MockServer::start().await;
    let client = server.client();
    client.login("test@example.com").await.unwrap();
    client.verify("123456").await.unwrap();
    client.get_account().await.unwrap();
    client.get_network(12345).await.unwrap();
    client.get_guest_network(12345).await.unwrap();
    let update = NetworkUpdate {
        password: Some("new-wifi-password".to_string()),
        ..Default::default()
    };
    client.update_network(12345, &update).await.unwrap();
    server.shutdown();

    let text = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
    for expected in ["login response body", "/2.2/account", "<redacted>"] {
        assert!(text.contains(expected), "expected {expected:?} in:\n{text}");
    }
    for secret in [
        "ut_test_token_12345",
        "session_test_token",
        "new-wifi-password",
        "guestpass",
        "testpass123",
        "test@example.com",
        "+15555550100",
    ] {
        assert!(!text.contains(secret), "{secret:?} leaked into logs:\n{text}");
    }
}
