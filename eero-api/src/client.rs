use reqwest::Method;
use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::credential::CredentialStore;
use crate::credential::file::FileStore;
use crate::error::{Error, Result};
use crate::types::envelope::ApiResponse;

const BASE_URL: &str = "https://api-user.e2ro.com/2.2";
const USER_AGENT: &str = "eero-api-rs/0.1";

/// JSON keys whose values are credentials or the account holder's contact
/// details. [`redact_body`] masks them.
const SECRET_KEYS: &[&str] = &[
    "password",
    "user_token",
    "session_token",
    "token",
    "email",
    "phone",
];

/// Build the `s=<token>` cookie, marked sensitive so `Debug` never prints it.
pub(crate) fn session_cookie(token: &str) -> Result<HeaderValue> {
    let mut value = HeaderValue::from_str(&format!("s={token}"))
        .map_err(|e| Error::CredentialStore(e.to_string()))?;
    value.set_sensitive(true);
    Ok(value)
}

/// The API's error code, plus the per-field reasons it puts in `data` for
/// form errors, e.g. `error.form.errors (symptom: error.form.field.required)`.
fn api_error_message(error: Option<String>, data: Option<&serde_json::Value>) -> String {
    let mut message = error.unwrap_or_else(|| "unknown error".into());
    if let Some(serde_json::Value::Object(fields)) = data {
        let reasons: Vec<String> = fields
            .iter()
            .filter_map(|(field, reason)| Some(format!("{field}: {}", reason.as_str()?)))
            .collect();
        if !reasons.is_empty() {
            message = format!("{message} ({})", reasons.join(", "));
        }
    }
    message
}

/// A request or response body made safe to log: credential fields masked,
/// and non-JSON bodies reduced to their length.
pub(crate) fn redact_body(text: &str) -> String {
    fn mask(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, v) in map.iter_mut() {
                    if SECRET_KEYS.contains(&key.as_str()) && !v.is_null() {
                        *v = serde_json::Value::String("<redacted>".into());
                    } else {
                        mask(v);
                    }
                }
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(mask),
            _ => {}
        }
    }
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(mut value) => {
            mask(&mut value);
            value.to_string()
        }
        Err(_) => format!("<{} bytes>", text.len()),
    }
}

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
            headers.insert(COOKIE, session_cookie(&token)?);
        } else {
            tracing::debug!("no session token available");
        }
        Ok(headers)
    }

    /// Extract response data, returning an error for non-success status codes.
    pub fn unwrap_response<T>(response: ApiResponse<T>) -> Result<T> {
        let code = response.meta.code;
        if (200..300).contains(&code) {
            response.data.ok_or_else(|| Error::Api {
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

    /// Send a request with the session cookie and decode the response envelope.
    ///
    /// Only the method, URL, status and redacted bodies are logged: never
    /// headers, which carry the session cookie.
    pub(crate) async fn send<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: Option<String>,
    ) -> Result<ApiResponse<T>> {
        let headers = self.session_headers().await?;
        match &body {
            Some(b) => tracing::debug!(%method, url, body = %redact_body(b), "request"),
            None => tracing::debug!(%method, url, "request"),
        }
        let mut req = self.http.request(method, url).headers(headers);
        if let Some(b) = body {
            req = req.header("content-type", "application/json").body(b);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        tracing::debug!(%status, body = %redact_body(&text), "response");
        let raw: ApiResponse<serde_json::Value> = serde_json::from_str(&text)?;
        let code = raw.meta.code;
        if !(200..300).contains(&code) {
            return Err(Error::Api {
                code,
                message: api_error_message(raw.meta.error, raw.data.as_ref()),
            });
        }
        let data = match raw.data {
            None | Some(serde_json::Value::Null) => None,
            Some(value) => Some(serde_json::from_value(value)?),
        };
        Ok(ApiResponse {
            meta: raw.meta,
            data,
        })
    }

    /// Perform an authenticated GET request.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        Self::unwrap_response(self.send(Method::GET, url, None).await?)
    }

    /// Perform an authenticated GET request with a JSON body.
    #[tracing::instrument(skip(self, body))]
    pub(crate) async fn get_with_json_body<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let body = Some(serde_json::to_string(body)?);
        Self::unwrap_response(self.send(Method::GET, url, body).await?)
    }

    /// Perform an authenticated POST request with a JSON body.
    #[tracing::instrument(skip(self, body))]
    pub(crate) async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let body = Some(serde_json::to_string(body)?);
        Self::unwrap_response(self.send(Method::POST, url, body).await?)
    }

    /// Perform an authenticated POST request without a body (reboots, logout).
    /// Success may carry no data; that comes back as `Value::Null`.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn post_empty(&self, url: &str) -> Result<serde_json::Value> {
        let resp = self.send(Method::POST, url, None).await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Perform an authenticated PUT request with a JSON body.
    #[tracing::instrument(skip(self, body))]
    pub(crate) async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let body = Some(serde_json::to_string(body)?);
        Self::unwrap_response(self.send(Method::PUT, url, body).await?)
    }

    /// Perform an authenticated DELETE request. The API answers a successful
    /// delete with `"data": null`, which comes back as `Value::Null`.
    #[tracing::instrument(skip(self))]
    pub(crate) async fn delete(&self, url: &str) -> Result<serde_json::Value> {
        let resp = self.send(Method::DELETE, url, None).await?;
        Ok(resp.data.unwrap_or(serde_json::Value::Null))
    }

    /// Access the raw reqwest HTTP client (for advanced use cases).
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http
    }
}
