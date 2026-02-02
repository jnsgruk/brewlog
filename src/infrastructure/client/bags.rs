use anyhow::{Context, Result};
use chrono::NaiveDate;

use crate::domain::bags::{BagWithRoast, UpdateBag};
use crate::domain::ids::{BagId, RoastId};

use super::BrewlogClient;

pub struct BagsClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> BagsClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn create(
        &self,
        roast_id: RoastId,
        roast_date: Option<NaiveDate>,
        amount: f64,
    ) -> Result<BagWithRoast> {
        let url = self.inner.endpoint("api/v1/bags")?;
        let payload = serde_json::json!({
            "roast_id": roast_id,
            "roast_date": roast_date.map(|d| d.to_string()),
            "amount": amount,
        });

        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(&payload)
            .send()
            .await
            .context("failed to issue create bag request")?;

        self.inner.handle_response(response).await
    }

    pub async fn list(&self, roast_id: Option<RoastId>) -> Result<Vec<BagWithRoast>> {
        let mut url = self.inner.endpoint("api/v1/bags")?;
        if let Some(roast_id) = roast_id {
            url.query_pairs_mut()
                .append_pair("roast_id", &roast_id.to_string());
        }

        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue list bags request")?;

        self.inner.handle_response(response).await
    }

    pub async fn get(&self, id: BagId) -> Result<BagWithRoast> {
        let url = self.inner.endpoint(&format!("api/v1/bags/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue get bag request")?;

        self.inner.handle_response(response).await
    }

    pub async fn update(
        &self,
        id: BagId,
        remaining: Option<f64>,
        closed: Option<bool>,
        finished_at: Option<NaiveDate>,
    ) -> Result<BagWithRoast> {
        let url = self.inner.endpoint(&format!("api/v1/bags/{id}"))?;
        let payload = UpdateBag {
            remaining,
            closed,
            finished_at,
        };

        let response = self
            .inner
            .request(reqwest::Method::PUT, url)
            .json(&payload)
            .send()
            .await
            .context("failed to issue update bag request")?;

        self.inner.handle_response(response).await
    }

    pub async fn delete(&self, id: BagId) -> Result<()> {
        let url = self.inner.endpoint(&format!("api/v1/bags/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("failed to issue delete bag request")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(self.inner.response_error(response).await)
        }
    }
}
