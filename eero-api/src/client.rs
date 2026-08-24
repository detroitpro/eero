use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::credential::file::FileStore;
use crate::credential::CredentialStore;
use crate::error::{Error, Result};
use crate::types::envelope::ApiResponse;

const BASE_URL: &str = "https://api-user.e2ro.com/2.2";
const USER_AGENT: &str = "eero-api-rs/0.1";

/// Client for the eero WiFi router API.
pub struct EeroClient {
    pub(crate) http: reqwest::Client,
    base_url: String,
    credentials: Box<dyn CredentialStore>,
}

/// Builder for constructing an [`EeroClient`].
pub struct ClientBuilder {
    base_url: String,
    credentials: Option<Box<dyn CredentialStore>>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            base_url: BASE_URL.to_string(),
            credentials: None,
        }
    }

    /// Set the API base URL (for testing or alternative endpoints).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the credential store implementation.
    pub fn credential_store(mut self, store: Box<dyn CredentialStore>) -> Self {
        self.credentials = Some(store);
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<EeroClient> {
        let credentials = match self.credentials {
            Some(c) => c,
            None => Box::new(FileStore::new()?),
        };

        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(Error::Http)?;

        Ok(EeroClient {
            http,
            base_url: self.base_url,
            credentials,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EeroClient {
    /// Create a new client with default settings.
    pub fn new() -> Result<Self> {
        ClientBuilder::new().build()
    }

    /// Access the credential store.
    pub fn credentials(&self) -> &dyn CredentialStore {
        &*self.credentials
    }

    /// Build a full URL from a path (e.g. `/login`).
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Build a full URL from an eero resource URL.
    ///
    /// Resource URLs from the API look like `/2.2/networks/12345`. If the url
    /// already starts with `http`, return it as-is. Otherwise prepend the base
    /// host.
    pub(crate) fn resource_url(&self, resource: &str) -> String {
        if resource.starts_with("http") {
            resource.to_string()
        } else {
            let scheme_end = self.base_url.find("://").map(|i| i + 3).unwrap_or(0);
            let origin_end = self.base_url[scheme_end..]
                .find('/')
                .map(|i| scheme_end + i)
                .unwrap_or(self.base_url.len());
            format!("{}{resource}", &self.base_url[..origin_end])
        }
    }

    pub(crate) async fn session_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let token = self.credentials.get_session_token().await?;
        let token = match token {
            Some(t) => {
                tracing::debug!("using session token");
                Some(t)
            }
            None => {
                let ut = self.credentials.get_user_token().await?;
                if ut.is_some() {
                    tracing::debug!("no session token, falling back to user token");
                } else {
                    tracing::debug!("no session token or user token found");
                }
                ut
            }
        };
        if let Some(token) = token {
            tracing::debug!("attaching cookie");
            headers.insert(
                COOKIE,
                HeaderValue::from_str(&format!("s={token}"))
                    .map_err(|e| Error::CredentialStore(e.to_string()))?,
            );
        } else {
            tracing::debug!("no session token available");
        }
        Ok(headers)
    }

    /// Extract response data, returning an error for non-success status codes.
    pub fn unwrap_response<T>(response: ApiResponse<T>) -> Result<T> {
        let code = response.meta.code;
        if (200..300).contains(&code) {
            response
                .data
                .ok_or_else(|| Error::Api {
                    code,
                    message: "response contained no data".into(),
                })
        } else {
            Err(Error::Api {
                code,
                message: response
                    .meta
                    .error
                    .unwrap_or_else(|| "unknown error".into()),
            })
        }
    }

    /// Perform an authenticated GET request.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let headers = self.session_headers().await?;
        tracing::debug!(request_headers = ?headers, "GET request");
        let resp = self.http.get(url).headers(headers).send().await?;
        tracing::debug!(status = %resp.status(), response_headers = ?resp.headers(), "GET response");
        let text = resp.text().await?;
        tracing::debug!(body = %text, "GET response body");
        let parsed: ApiResponse<T> = serde_json::from_str(&text)?;
        Self::unwrap_response(parsed)
    }

    /// Perform an authenticated GET request with a JSON body.
    #[tracing::instrument(skip(self, body))]
    pub(crate) async fn get_with_json_body<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let headers = self.session_headers().await?;
        let body_json = serde_json::to_string(body)?;
        tracing::debug!(request_headers = ?headers, body = %body_json, "GET request with JSON body");
        let resp = self
            .http
            .get(url)
            .headers(headers)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;
        tracing::debug!(status = %resp.status(), response_headers = ?resp.headers(), "GET response");
        let text = resp.text().await?;
        tracing::debug!(body = %text, "GET response body");
        let parsed: ApiResponse<T> = serde_json::from_str(&text)?;
        Self::unwrap_response(parsed)
    }

    /// Perform an authenticated POST request with a JSON body.
    #[tracing::instrument(skip(self, body))]
    pub(crate) async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let headers = self.session_headers().await?;
        let body_json = serde_json::to_string(body)?;
        tracing::debug!(request_headers = ?headers, body = %body_json, "POST request");
        let resp = self
            .http
            .post(url)
            .headers(headers)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;
        tracing::debug!(status = %resp.status(), response_headers = ?resp.headers(), "POST response");
        let text = resp.text().await?;
        tracing::debug!(body = %text, "POST response body");
        let parsed: ApiResponse<T> = serde_json::from_str(&text)?;
        Self::unwrap_response(parsed)
    }

    /// Perform an authenticated POST request without a body.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn post_empty<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let headers = self.session_headers().await?;
        tracing::debug!(request_headers = ?headers, "POST (empty) request");
        let resp = self
            .http
            .post(url)
            .headers(headers)
            .send()
            .await?;
        tracing::debug!(status = %resp.status(), response_headers = ?resp.headers(), "POST (empty) response");
        let text = resp.text().await?;
        tracing::debug!(body = %text, "POST (empty) response body");
        let parsed: ApiResponse<T> = serde_json::from_str(&text)?;
        Self::unwrap_response(parsed)
    }

    /// Perform an authenticated PUT request with a JSON body.
    #[tracing::instrument(skip(self, body))]
    pub(crate) async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let headers = self.session_headers().await?;
        let body_json = serde_json::to_string(body)?;
        tracing::debug!(request_headers = ?headers, body = %body_json, "PUT request");
        let resp = self
            .http
            .put(url)
            .headers(headers)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;
        tracing::debug!(status = %resp.status(), response_headers = ?resp.headers(), "PUT response");
        let text = resp.text().await?;
        tracing::debug!(body = %text, "PUT response body");
        let parsed: ApiResponse<T> = serde_json::from_str(&text)?;
        Self::unwrap_response(parsed)
    }

    /// Perform an authenticated DELETE request.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn delete<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let headers = self.session_headers().await?;
        tracing::debug!(request_headers = ?headers, "DELETE request");
        let resp = self
            .http
            .delete(url)
            .headers(headers)
            .send()
            .await?;
        tracing::debug!(status = %resp.status(), response_headers = ?resp.headers(), "DELETE response");
        let text = resp.text().await?;
        tracing::debug!(body = %text, "DELETE response body");
        let parsed: ApiResponse<T> = serde_json::from_str(&text)?;
        Self::unwrap_response(parsed)
    }

    /// Access the raw reqwest HTTP client (for advanced use cases).
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http
    }
}
