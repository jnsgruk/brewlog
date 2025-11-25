use anyhow::{Context, Result};
use reqwest::StatusCode;

use crate::domain::roasters::{NewRoaster, Roaster, UpdateRoaster};

use super::BrewlogClient;

pub struct RoastersClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> RoastersClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn create(&self, payload: &NewRoaster) -> Result<Roaster> {
        let url = self.inner.endpoint("api/v1/roasters")?;
        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(payload)
            .send()
            .await
            .context("failed to issue create roaster request")?;

        self.inner.handle_response(response).await
    }

    pub async fn list(&self) -> Result<Vec<Roaster>> {
        let url = self.inner.endpoint("api/v1/roasters")?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue list roasters request")?;

        self.inner.handle_response(response).await
    }

    pub async fn get(&self, id: &str) -> Result<Roaster> {
        let url = self.inner.endpoint(&format!("api/v1/roasters/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue get roaster request")?;

        self.inner.handle_response(response).await
    }

    pub async fn update(&self, id: &str, payload: &UpdateRoaster) -> Result<Roaster> {
        let url = self.inner.endpoint(&format!("api/v1/roasters/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::PUT, url)
            .json(payload)
            .send()
            .await
            .context("failed to issue update roaster request")?;

        self.inner.handle_response(response).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let url = self.inner.endpoint(&format!("api/v1/roasters/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("failed to issue delete roaster request")?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            _ => Err(self.inner.response_error(response).await),
        }
    }
}
