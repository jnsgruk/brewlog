use anyhow::{Context, Result};
use reqwest::StatusCode;

use crate::domain::cafes::{Cafe, NewCafe, UpdateCafe};
use crate::domain::ids::CafeId;

use super::BrewlogClient;

pub struct CafesClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> CafesClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn create(&self, payload: &NewCafe) -> Result<Cafe> {
        let url = self.inner.endpoint("api/v1/cafes")?;
        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(payload)
            .send()
            .await
            .context("failed to issue create cafe request")?;

        self.inner.handle_response(response).await
    }

    pub async fn list(&self) -> Result<Vec<Cafe>> {
        let url = self.inner.endpoint("api/v1/cafes")?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue list cafes request")?;

        self.inner.handle_response(response).await
    }

    pub async fn get(&self, id: CafeId) -> Result<Cafe> {
        let url = self.inner.endpoint(&format!("api/v1/cafes/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue get cafe request")?;

        self.inner.handle_response(response).await
    }

    pub async fn update(&self, id: CafeId, payload: &UpdateCafe) -> Result<Cafe> {
        let url = self.inner.endpoint(&format!("api/v1/cafes/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::PUT, url)
            .json(payload)
            .send()
            .await
            .context("failed to issue update cafe request")?;

        self.inner.handle_response(response).await
    }

    pub async fn delete(&self, id: CafeId) -> Result<()> {
        let url = self.inner.endpoint(&format!("api/v1/cafes/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("failed to issue delete cafe request")?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            _ => Err(self.inner.response_error(response).await),
        }
    }
}
