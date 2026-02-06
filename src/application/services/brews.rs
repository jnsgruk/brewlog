use std::sync::Arc;

use tracing::warn;

use crate::domain::brews::{BrewWithDetails, NewBrew};
use crate::domain::errors::RepositoryError;
use crate::domain::repositories::{BrewRepository, TimelineEventRepository};

#[derive(Clone)]
pub struct BrewService {
    brew_repo: Arc<dyn BrewRepository>,
    timeline_repo: Arc<dyn TimelineEventRepository>,
}

impl BrewService {
    pub fn new(
        brew_repo: Arc<dyn BrewRepository>,
        timeline_repo: Arc<dyn TimelineEventRepository>,
    ) -> Self {
        Self {
            brew_repo,
            timeline_repo,
        }
    }

    /// Insert a brew, enrich it with related entity names, record a timeline
    /// event, and return the enriched result.
    pub async fn create(&self, new: NewBrew) -> Result<BrewWithDetails, RepositoryError> {
        let brew = self.brew_repo.insert(new).await?;
        let enriched = self.brew_repo.get_with_details(brew.id).await?;
        if let Err(err) = self
            .timeline_repo
            .insert(enriched.to_timeline_event())
            .await
        {
            warn!(error = %err, brew_id = %brew.id, "failed to record brew timeline event");
        }
        Ok(enriched)
    }
}
