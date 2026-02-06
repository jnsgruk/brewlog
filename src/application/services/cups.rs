use std::sync::Arc;

use tracing::warn;

use crate::domain::cups::{Cup, NewCup};
use crate::domain::errors::RepositoryError;
use crate::domain::repositories::{CupRepository, TimelineEventRepository};

#[derive(Clone)]
pub struct CupService {
    cup_repo: Arc<dyn CupRepository>,
    timeline_repo: Arc<dyn TimelineEventRepository>,
}

impl CupService {
    pub fn new(
        cup_repo: Arc<dyn CupRepository>,
        timeline_repo: Arc<dyn TimelineEventRepository>,
    ) -> Self {
        Self {
            cup_repo,
            timeline_repo,
        }
    }

    pub async fn create(&self, new: NewCup) -> Result<Cup, RepositoryError> {
        let cup = self.cup_repo.insert(new).await?;
        match self.cup_repo.get_with_details(cup.id).await {
            Ok(enriched) => {
                if let Err(err) = self
                    .timeline_repo
                    .insert(enriched.to_timeline_event())
                    .await
                {
                    warn!(error = %err, cup_id = %cup.id, "failed to record cup timeline event");
                }
            }
            Err(err) => {
                warn!(error = %err, cup_id = %cup.id, "failed to enrich cup for timeline event");
            }
        }
        Ok(cup)
    }
}
