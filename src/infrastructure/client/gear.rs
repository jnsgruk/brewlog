use anyhow::{Context, Result};

use crate::domain::gear::{Gear, UpdateGear};
use crate::domain::ids::GearId;

use super::BrewlogClient;

pub struct GearClient<'a> {
    inner: &'a BrewlogClient,
}

impl<'a> GearClient<'a> {
    pub(crate) fn new(inner: &'a BrewlogClient) -> Self {
        Self { inner }
    }

    pub async fn create(
        &self,
        category: &str,
        make: String,
        model: String,
        notes: Option<String>,
    ) -> Result<Gear> {
        let url = self.inner.endpoint("api/v1/gear")?;
        let payload = serde_json::json!({
            "category": category,
            "make": make,
            "model": model,
            "notes": notes,
        });

        let response = self
            .inner
            .request(reqwest::Method::POST, url)
            .json(&payload)
            .send()
            .await
            .context("failed to issue create gear request")?;

        self.inner.handle_response(response).await
    }

    pub async fn list(&self, category: Option<String>) -> Result<Vec<Gear>> {
        let mut url = self.inner.endpoint("api/v1/gear")?;
        if let Some(category) = category {
            url.query_pairs_mut().append_pair("category", &category);
        }

        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue list gear request")?;

        self.inner.handle_response(response).await
    }

    pub async fn get(&self, id: GearId) -> Result<Gear> {
        let url = self.inner.endpoint(&format!("api/v1/gear/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .context("failed to issue get gear request")?;

        self.inner.handle_response(response).await
    }

    pub async fn update(
        &self,
        id: GearId,
        make: Option<String>,
        model: Option<String>,
        notes: Option<String>,
    ) -> Result<Gear> {
        let url = self.inner.endpoint(&format!("api/v1/gear/{id}"))?;
        let payload = UpdateGear { make, model, notes };

        let response = self
            .inner
            .request(reqwest::Method::PUT, url)
            .json(&payload)
            .send()
            .await
            .context("failed to issue update gear request")?;

        self.inner.handle_response(response).await
    }

    pub async fn delete(&self, id: GearId) -> Result<()> {
        let url = self.inner.endpoint(&format!("api/v1/gear/{id}"))?;
        let response = self
            .inner
            .request(reqwest::Method::DELETE, url)
            .send()
            .await
            .context("failed to issue delete gear request")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(self.inner.response_error(response).await)
        }
    }
}
