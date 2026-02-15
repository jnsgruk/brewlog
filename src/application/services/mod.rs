mod bags;
mod brews;
mod cups;
mod roasts;
pub mod stats;
pub mod timeline_refresh;

pub use bags::BagService;
pub use brews::BrewService;
pub use cups::CupService;
pub use roasts::RoastService;
pub use stats::StatsInvalidator;
pub use timeline_refresh::TimelineInvalidator;

use std::sync::Arc;

use tracing::warn;

use crate::domain::errors::RepositoryError;
use crate::domain::repositories::TimelineEventRepository;

/// Generates a service struct with a `create` method that inserts via the
/// repository and then records a timeline event (fire-and-forget).
///
/// Use this for entities whose `to_timeline_event()` method needs only `&self`
/// (no related-entity lookups). For entities that need enrichment or
/// cross-repo lookups, write the service by hand.
///
/// # Example
/// ```ignore
/// define_simple_service!(RoasterService, RoasterRepository, Roaster, NewRoaster, "roaster");
/// ```
macro_rules! define_simple_service {
    ($service:ident, $repo_trait:path, $entity:ty, $new_entity:ty, $entity_name:literal) => {
        #[derive(Clone)]
        pub struct $service {
            repo: Arc<dyn $repo_trait>,
            timeline_repo: Arc<dyn TimelineEventRepository>,
        }

        impl $service {
            pub fn new(
                repo: Arc<dyn $repo_trait>,
                timeline_repo: Arc<dyn TimelineEventRepository>,
            ) -> Self {
                Self {
                    repo,
                    timeline_repo,
                }
            }

            pub async fn create(
                &self,
                new: $new_entity,
            ) -> Result<$entity, RepositoryError> {
                let entity = self.repo.insert(new).await?;
                if let Err(err) = self
                    .timeline_repo
                    .insert(entity.to_timeline_event())
                    .await
                {
                    warn!(
                        error = %err,
                        id = %entity.id,
                        concat!("failed to record ", $entity_name, " timeline event"),
                    );
                }
                Ok(entity)
            }
        }
    };
}

use crate::domain::cafes::{Cafe, NewCafe};
use crate::domain::gear::{Gear, NewGear};
use crate::domain::repositories::{CafeRepository, GearRepository, RoasterRepository};
use crate::domain::roasters::{NewRoaster, Roaster};

define_simple_service!(
    RoasterService,
    RoasterRepository,
    Roaster,
    NewRoaster,
    "roaster"
);
define_simple_service!(CafeService, CafeRepository, Cafe, NewCafe, "cafe");
define_simple_service!(GearService, GearRepository, Gear, NewGear, "gear");
