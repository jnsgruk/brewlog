use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::domain::brews::{BrewWithDetails, QuickNote, UpdateBrew};
use crate::domain::ids::{BagId, BrewId, GearId};

use super::BrewlogClient;

pub struct BrewsClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> BrewsClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        bag_id: BagId,
        coffee_weight: f64,
        grinder_id: GearId,
        grind_setting: f64,
        brewer_id: GearId,
        filter_paper_id: Option<GearId>,
        water_volume: i32,
        water_temp: f64,
        quick_notes: Vec<QuickNote>,
        brew_time: Option<i32>,
        created_at: Option<DateTime<Utc>>,
    ) -> Result<BrewWithDetails> {
        let url = self.inner.endpoint("api/v1/brews")?;
        let mut payload = serde_json::json!({
            "bag_id": bag_id,
            "coffee_weight": coffee_weight,
            "grinder_id": grinder_id,
            "grind_setting": grind_setting,
            "brewer_id": brewer_id,
            "water_volume": water_volume,
            "water_temp": water_temp,
        });
        if let Some(fp_id) = filter_paper_id {
            payload["filter_paper_id"] = serde_json::json!(fp_id);
        }
        if !quick_notes.is_empty() {
            let labels: Vec<&str> = quick_notes.iter().map(|n| n.label()).collect();
            payload["quick_notes"] = serde_json::json!(labels);
        }
        if let Some(bt) = brew_time {
            payload["brew_time"] = serde_json::json!(bt);
        }
        if let Some(ts) = created_at {
            payload["created_at"] = serde_json::json!(ts);
        }

        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(&payload)
            .send()
            .await
            .context("failed to issue create brew request")?;

        self.inner.handle_response(response).await
    }

    pub async fn list(&self, bag_id: Option<BagId>) -> Result<Vec<BrewWithDetails>> {
        let mut url = self.inner.endpoint("api/v1/brews")?;
        if let Some(bag_id) = bag_id {
            url.query_pairs_mut()
                .append_pair("bag_id", &bag_id.to_string());
        }

        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue list brews request")?;

        self.inner.handle_response(response).await
    }

    pub async fn get(&self, id: BrewId) -> Result<BrewWithDetails> {
        let url = self.inner.endpoint(&format!("api/v1/brews/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue get brew request")?;

        self.inner.handle_response(response).await
    }

    pub async fn update(&self, id: BrewId, payload: &UpdateBrew) -> Result<BrewWithDetails> {
        let url = self.inner.endpoint(&format!("api/v1/brews/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::PUT, url)
            .json(payload)
            .send()
            .await
            .context("failed to issue update brew request")?;

        self.inner.handle_response(response).await
    }

    pub async fn delete(&self, id: BrewId) -> Result<()> {
        let url = self.inner.endpoint(&format!("api/v1/brews/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("failed to issue delete brew request")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(self.inner.response_error(response).await)
        }
    }
}
