use std::sync::Arc;

use tracing::warn;

use crate::domain::bags::{Bag, NewBag, UpdateBag, bag_timeline_event};
use crate::domain::errors::RepositoryError;
use crate::domain::ids::BagId;
use crate::domain::repositories::{
    BagRepository, RoastRepository, RoasterRepository, TimelineEventRepository,
};

#[allow(clippy::struct_field_names)]
#[derive(Clone)]
pub struct BagService {
    bag_repo: Arc<dyn BagRepository>,
    roast_repo: Arc<dyn RoastRepository>,
    roaster_repo: Arc<dyn RoasterRepository>,
    timeline_repo: Arc<dyn TimelineEventRepository>,
}

impl BagService {
    pub fn new(
        bag_repo: Arc<dyn BagRepository>,
        roast_repo: Arc<dyn RoastRepository>,
        roaster_repo: Arc<dyn RoasterRepository>,
        timeline_repo: Arc<dyn TimelineEventRepository>,
    ) -> Self {
        Self {
            bag_repo,
            roast_repo,
            roaster_repo,
            timeline_repo,
        }
    }

    pub async fn create(&self, new: NewBag) -> Result<Bag, RepositoryError> {
        let bag = self.bag_repo.insert(new).await?;
        self.record_timeline_event(&bag, "added").await;
        Ok(bag)
    }

    /// Close a bag: sets `finished_at` (if not provided), updates via the
    /// repository, and records a "finished" timeline event.
    pub async fn finish(&self, id: BagId, mut update: UpdateBag) -> Result<Bag, RepositoryError> {
        if update.finished_at.is_none() {
            update.finished_at = Some(chrono::Utc::now().date_naive());
        }
        let bag = self.bag_repo.update(id, update).await?;
        self.record_timeline_event(&bag, "finished").await;
        Ok(bag)
    }

    async fn record_timeline_event(&self, bag: &Bag, action: &str) {
        let roast = match self.roast_repo.get(bag.roast_id).await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, bag_id = %bag.id, "failed to fetch roast for bag timeline event");
                return;
            }
        };
        let roaster = match self.roaster_repo.get(roast.roaster_id).await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, bag_id = %bag.id, "failed to fetch roaster for bag timeline event");
                return;
            }
        };
        if let Err(err) = self
            .timeline_repo
            .insert(bag_timeline_event(bag, action, &roast, &roaster))
            .await
        {
            warn!(error = %err, bag_id = %bag.id, "failed to record bag timeline event");
        }
    }
}
