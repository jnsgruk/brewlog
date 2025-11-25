use anyhow::{Context, Result};
use reqwest::StatusCode;

use crate::domain::roasts::{NewRoast, Roast, RoastWithRoaster};

use super::BrewlogClient;

pub struct RoastsClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> RoastsClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn create(&self, payload: &NewRoast) -> Result<Roast> {
        let url = self.inner.endpoint("api/v1/roasts")?;
        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(payload)
            .send()
            .await
            .context("failed to issue create roast request")?;

        self.inner.handle_response(response).await
    }

    pub async fn list(&self, roaster_id: Option<&str>) -> Result<Vec<RoastWithRoaster>> {
        let mut url = self.inner.endpoint("api/v1/roasts")?;
        if let Some(roaster_id) = roaster_id {
            url.query_pairs_mut().append_pair("roaster_id", roaster_id);
        }

        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue list roasts request")?;

        self.inner.handle_response(response).await
    }

    pub async fn get(&self, id: &str) -> Result<Roast> {
        let url = self.inner.endpoint(&format!("api/v1/roasts/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue get roast request")?;

        self.inner.handle_response(response).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let url = self.inner.endpoint(&format!("api/v1/roasts/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("failed to issue delete roast request")?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            _ => Err(self.inner.response_error(response).await),
        }
    }
}
