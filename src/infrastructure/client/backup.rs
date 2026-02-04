use anyhow::{Context, Result};
use reqwest::StatusCode;

use crate::infrastructure::backup::BackupData;

use super::BrewlogClient;

pub struct BackupClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> BackupClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn export(&self) -> Result<BackupData> {
        let url = self.inner.endpoint("api/v1/backup")?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue backup export request")?;

        self.inner.handle_response(response).await
    }

    pub async fn restore(&self, data: &BackupData) -> Result<()> {
        let url = self.inner.endpoint("api/v1/backup/restore")?;
        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(data)
            .send()
            .await
            .context("failed to issue backup restore request")?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            _ => Err(self.inner.response_error(response).await),
        }
    }
}
