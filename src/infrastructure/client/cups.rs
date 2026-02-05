use anyhow::{Context, Result};
use reqwest::StatusCode;

use crate::domain::cups::{Cup, CupWithDetails, NewCup};
use crate::domain::ids::CupId;

use super::BrewlogClient;

pub struct CupsClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> CupsClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn create(&self, payload: &NewCup) -> Result<Cup> {
        let url = self.inner.endpoint("api/v1/cups")?;
        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(payload)
            .send()
            .await
            .context("failed to issue create cup request")?;

        self.inner.handle_response(response).await
    }

    pub async fn list(&self) -> Result<Vec<CupWithDetails>> {
        let url = self.inner.endpoint("api/v1/cups")?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue list cups request")?;

        self.inner.handle_response(response).await
    }

    pub async fn get(&self, id: CupId) -> Result<CupWithDetails> {
        let url = self.inner.endpoint(&format!("api/v1/cups/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue get cup request")?;

        self.inner.handle_response(response).await
    }

    pub async fn delete(&self, id: CupId) -> Result<()> {
        let url = self.inner.endpoint(&format!("api/v1/cups/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("failed to issue delete cup request")?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            _ => Err(self.inner.response_error(response).await),
        }
    }
}
