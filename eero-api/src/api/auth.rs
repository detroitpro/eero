use reqwest::header::COOKIE;

use crate::client::{redact_body, session_cookie, EeroClient};
use crate::error::{Error, Result};
use crate::types::auth::{LoginRequest, LoginResponse, VerifyRequest};
use crate::types::envelope::ApiResponse;

/// Parse the session token from Set-Cookie headers manually.
///
/// Handles formats like `s=TOKEN; Path=/; HttpOnly` by extracting the
/// value of the cookie named `s`.
fn extract_session_cookie(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|val| val.to_str().ok())
        .find_map(|cookie_str| {
            let pair = cookie_str.split(';').next()?;
            let (name, value) = pair.split_once('=')?;
            if name.trim() == "s" {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
}

impl EeroClient {
    /// Begin login by sending a verification code to the user's email or phone.
    ///
    /// After calling this, the user will receive a verification code. Call
    /// [`verify`](Self::verify) with that code to complete authentication.
    #[tracing::instrument(skip(self, login))]
    pub async fn login(&self, login: &str) -> Result<LoginResponse> {
        let url = self.url("/login");
        let body = LoginRequest {
            login: login.to_string(),
        };

        let body_json = serde_json::to_string(&body)?;
        tracing::debug!("login request");
        let resp = self
            .http_client()
            .post(&url)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;
        tracing::debug!(status = %resp.status(), "login response");
        let text = resp.text().await?;
        tracing::debug!(body = %redact_body(&text), "login response body");
        let api_resp: ApiResponse<LoginResponse> = serde_json::from_str(&text)?;

        let data = Self::unwrap_response(api_resp)?;
        self.credentials()
            .set_user_token(&data.user_token)
            .await?;
        Ok(data)
    }

    /// Complete login by verifying the code sent to the user's device.
    ///
    /// On success, the session token is extracted from the response cookies
    /// and stored in the credential store.
    #[tracing::instrument(skip(self, code))]
    pub async fn verify(&self, code: &str) -> Result<()> {
        let user_token = self
            .credentials()
            .get_user_token()
            .await?
            .ok_or(Error::NotAuthenticated)?;

        let url = self.url("/login/verify");
        let body = VerifyRequest {
            code: code.to_string(),
        };

        let cookie = session_cookie(&user_token)?;
        let body_json = serde_json::to_string(&body)?;
        tracing::debug!("verify request");
        let resp = self
            .http_client()
            .post(&url)
            .header(COOKIE, cookie)
            .header("content-type", "application/json")
            .body(body_json)
            .send()
            .await?;

        // Extract session token from Set-Cookie header, parsing manually
        // rather than using resp.cookies() which can miss cookies from
        // intermediate redirect responses.
        tracing::debug!(status = %resp.status(), "verify response");
        let session_token = extract_session_cookie(resp.headers());

        let text = resp.text().await?;
        tracing::debug!(body = %redact_body(&text), "verify response body");
        let api_resp: ApiResponse<serde_json::Value> = serde_json::from_str(&text)?;
        let code = api_resp.meta.code;
        if !(200..300).contains(&code) {
            return Err(Error::Api {
                code,
                message: api_resp
                    .meta
                    .error
                    .unwrap_or_else(|| "verification failed".into()),
            });
        }

        // If the server returns a new session cookie, use it. Otherwise
        // the user_token from login doubles as the session token.
        if let Some(token) = session_token {
            self.credentials().set_session_token(&token).await?;
        } else {
            let user_token = self.credentials().get_user_token().await?;
            if let Some(token) = user_token {
                self.credentials().set_session_token(&token).await?;
            }
        }

        // Clean up user token
        self.credentials().delete_user_token().await?;

        Ok(())
    }

    /// Log out and clear stored credentials.
    #[tracing::instrument(skip(self))]
    pub async fn logout(&self) -> Result<()> {
        let url = self.url("/logout");
        // Best-effort POST to logout endpoint
        let _ = self.post_empty::<serde_json::Value>(&url).await;

        self.credentials().delete_session_token().await?;
        self.credentials().delete_user_token().await?;
        Ok(())
    }
}
