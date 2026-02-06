use std::sync::Arc;

use tracing::warn;

use crate::domain::errors::RepositoryError;
use crate::domain::repositories::{RoastRepository, RoasterRepository, TimelineEventRepository};
use crate::domain::roasts::{NewRoast, Roast, roast_timeline_event};

#[allow(clippy::struct_field_names)]
#[derive(Clone)]
pub struct RoastService {
    roast_repo: Arc<dyn RoastRepository>,
    roaster_repo: Arc<dyn RoasterRepository>,
    timeline_repo: Arc<dyn TimelineEventRepository>,
}

impl RoastService {
    pub fn new(
        roast_repo: Arc<dyn RoastRepository>,
        roaster_repo: Arc<dyn RoasterRepository>,
        timeline_repo: Arc<dyn TimelineEventRepository>,
    ) -> Self {
        Self {
            roast_repo,
            roaster_repo,
            timeline_repo,
        }
    }

    pub async fn create(&self, new: NewRoast) -> Result<Roast, RepositoryError> {
        let roast = self.roast_repo.insert(new).await?;
        match self.roaster_repo.get(roast.roaster_id).await {
            Ok(roaster) => {
                if let Err(err) = self
                    .timeline_repo
                    .insert(roast_timeline_event(&roast, &roaster))
                    .await
                {
                    warn!(error = %err, roast_id = %roast.id, "failed to record roast timeline event");
                }
            }
            Err(err) => {
                warn!(error = %err, roast_id = %roast.id, "failed to fetch roaster for roast timeline event");
            }
        }
        Ok(roast)
    }
}
