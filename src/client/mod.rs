pub mod roasters;
pub mod roasts;
pub mod tokens;

use anyhow::{Context, Result, anyhow};
use reqwest::{Client, Url};

use crate::server::errors::ErrorResponse;

pub struct BrewlogClient {
    base_url: Url,
    http: Client,
    token: Option<String>,
}

impl BrewlogClient {
    pub fn new(base_url: Url) -> Result<Self> {
        let mut normalized = base_url;
        if !normalized.path().ends_with('/') {
            normalized.set_path(&format!("{}/", normalized.path().trim_end_matches('/')));
        }

        let token = std::env::var("BREWLOG_TOKEN").ok();

        let http = Client::builder()
            .user_agent("brewlog-cli/0.1")
            .build()
            .context("failed to configure HTTP client")?;

        Ok(Self {
            base_url: normalized,
            http,
            token,
        })
    }

    pub fn from_base_url(base_url: &str) -> Result<Self> {
        let url = Url::parse(base_url).with_context(|| format!("invalid API url: {base_url}"))?;
        Self::new(url)
    }

    pub fn roasters(&self) -> roasters::RoastersClient<'_> {
        roasters::RoastersClient::new(self)
    }

    pub fn roasts(&self) -> roasts::RoastsClient<'_> {
        roasts::RoastsClient::new(self)
    }

    pub fn tokens(&self) -> tokens::TokensClient<'_> {
        tokens::TokensClient::new(self)
    }

    pub(crate) fn endpoint(&self, path: &str) -> Result<Url> {
        self.base_url
            .join(path)
            .with_context(|| format!("invalid API path: {path}"))
    }

    pub(crate) fn http_client(&self) -> &Client {
        &self.http
    }

    /// Build a request with authentication if token is available
    pub(crate) fn request(&self, method: reqwest::Method, url: Url) -> reqwest::RequestBuilder {
        let mut request = self.http.request(method, url);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        request
    }

    pub(crate) async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        if response.status().is_success() {
            response
                .json::<T>()
                .await
                .context("failed to deserialize response body")
        } else {
            Err(self.response_error(response).await)
        }
    }

    pub(crate) async fn response_error(&self, response: reqwest::Response) -> anyhow::Error {
        let status = response.status();
        let bytes = response.bytes().await.unwrap_or_default();

        if let Ok(err) = serde_json::from_slice::<ErrorResponse>(&bytes) {
            return anyhow!("request failed ({status}): {}", err.message);
        }

        let message = String::from_utf8_lossy(&bytes);
        anyhow!("request failed ({status}): {message}")
    }
}
