pub mod fixtures;
pub mod routes;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use eero_api::credential::CredentialStore;
use eero_api::{ClientBuilder, EeroClient};
use tokio::net::TcpListener;

pub struct MockServer {
    addr: SocketAddr,
    shutdown: tokio::sync::oneshot::Sender<()>,
}

impl MockServer {
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let app = routes::router();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
                .unwrap();
        });

        MockServer { addr, shutdown: tx }
    }

    pub async fn start_verify_no_cookie() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let app = routes::router_verify_no_cookie();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
                .unwrap();
        });

        MockServer { addr, shutdown: tx }
    }

    pub async fn start_speed_test_202() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let app = routes::router_speed_test_202();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
                .unwrap();
        });

        MockServer { addr, shutdown: tx }
    }

    pub fn base_url(&self) -> String {
        format!("http://{}/2.2", self.addr)
    }

    pub fn client(&self) -> EeroClient {
        ClientBuilder::new()
            .base_url(self.base_url())
            .credential_store(Box::new(InMemoryStore::new()))
            .build()
            .unwrap()
    }

    pub fn client_with_session(&self, token: &str) -> EeroClient {
        let store = InMemoryStore::new();
        store.session.lock().unwrap().replace(token.to_string());
        ClientBuilder::new()
            .base_url(self.base_url())
            .credential_store(Box::new(store))
            .build()
            .unwrap()
    }

    pub fn client_with_user_token(&self, token: &str) -> EeroClient {
        let store = InMemoryStore::new();
        store.user.lock().unwrap().replace(token.to_string());
        ClientBuilder::new()
            .base_url(self.base_url())
            .credential_store(Box::new(store))
            .build()
            .unwrap()
    }

    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
    }
}

#[derive(Clone)]
pub struct InMemoryStore {
    pub session: Arc<Mutex<Option<String>>>,
    pub user: Arc<Mutex<Option<String>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            user: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl CredentialStore for InMemoryStore {
    async fn get_session_token(&self) -> eero_api::Result<Option<String>> {
        Ok(self.session.lock().unwrap().clone())
    }

    async fn set_session_token(&self, token: &str) -> eero_api::Result<()> {
        *self.session.lock().unwrap() = Some(token.to_string());
        Ok(())
    }

    async fn delete_session_token(&self) -> eero_api::Result<()> {
        *self.session.lock().unwrap() = None;
        Ok(())
    }

    async fn get_user_token(&self) -> eero_api::Result<Option<String>> {
        Ok(self.user.lock().unwrap().clone())
    }

    async fn set_user_token(&self, token: &str) -> eero_api::Result<()> {
        *self.user.lock().unwrap() = Some(token.to_string());
        Ok(())
    }

    async fn delete_user_token(&self) -> eero_api::Result<()> {
        *self.user.lock().unwrap() = None;
        Ok(())
    }
}
