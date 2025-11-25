use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::BrewlogClient;
use crate::domain::ids::{TokenId, UserId};

pub struct TokensClient<'a> {
    client: &'a BrewlogClient,
}

impl<'a> TokensClient<'a> {
    pub fn new(client: &'a BrewlogClient) -> Self {
        Self { client }
    }

    pub async fn create(
        &self,
        username: &str,
        password: &str,
        name: &str,
    ) -> Result<TokenResponse> {
        let url = self.client.endpoint("api/v1/tokens")?;
        let body = CreateTokenRequest {
            username: username.to_string(),
            password: password.to_string(),
            name: name.to_string(),
        };

        let response = self
            .client
            .http_client()
            .post(url)
            .json(&body)
            .send()
            .await?;

        self.client.handle_response(response).await
    }

    pub async fn list(&self) -> Result<Vec<TokenInfo>> {
        let url = self.client.endpoint("api/v1/tokens")?;

        let response = self
            .client
            .request(reqwest::Method::GET, url)
            .send()
            .await?;

        self.client.handle_response(response).await
    }

    pub async fn revoke(&self, id: TokenId) -> Result<TokenInfo> {
        let url = self
            .client
            .endpoint(&format!("api/v1/tokens/{id}/revoke"))?;

        let response = self
            .client
            .request(reqwest::Method::POST, url)
            .send()
            .await?;

        self.client.handle_response(response).await
    }
}

#[derive(Debug, Serialize)]
struct CreateTokenRequest {
    username: String,
    password: String,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub id: TokenId,
    pub name: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenInfo {
    pub id: TokenId,
    pub user_id: UserId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}
