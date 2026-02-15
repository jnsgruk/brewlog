use anyhow::{Context, Result};
use reqwest::StatusCode;

use super::BrewlogClient;

pub struct TimelineClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> TimelineClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn rebuild(&self) -> Result<()> {
        let url = self.inner.endpoint("api/v1/timeline/rebuild")?;
        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .send()
            .await
            .context("failed to issue timeline rebuild request")?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            _ => Err(self.inner.response_error(response).await),
        }
    }
}
