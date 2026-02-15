use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::domain::bags::bag_timeline_event;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::{BagId, BrewId, CafeId, CupId, GearId, RoastId, RoasterId};
use crate::domain::repositories::{
    BagRepository, BrewRepository, CafeRepository, CupRepository, GearRepository, RoastRepository,
    RoasterRepository, TimelineEventRepository,
};
use crate::domain::roasts::roast_timeline_event;

/// Invalidation signal sent by HTTP handlers to the background rebuild task.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimelineInvalidation {
    /// Refresh a single entity's timeline event(s) and cascade to downstream entities.
    Entity {
        entity_type: EntityType,
        entity_id: i64,
    },
    /// Delete all timeline events and rebuild from scratch.
    Full,
}

/// Sends invalidation signals to the background timeline rebuild task.
/// Non-blocking and fire-and-forget — safe to call from any handler.
#[derive(Clone)]
pub struct TimelineInvalidator {
    tx: mpsc::Sender<TimelineInvalidation>,
}

impl TimelineInvalidator {
    pub fn new(tx: mpsc::Sender<TimelineInvalidation>) -> Self {
        Self { tx }
    }

    /// Signal that a specific entity's timeline event(s) need refreshing.
    pub fn invalidate(&self, entity_type: EntityType, entity_id: i64) {
        let _ = self.tx.try_send(TimelineInvalidation::Entity {
            entity_type,
            entity_id,
        });
    }

    /// Signal a full rebuild of all timeline events.
    pub fn rebuild_all(&self) {
        let _ = self.tx.try_send(TimelineInvalidation::Full);
    }
}

/// All repository dependencies needed to rebuild timeline events.
pub struct TimelineRebuilder {
    pub timeline_repo: Arc<dyn TimelineEventRepository>,
    pub roaster_repo: Arc<dyn RoasterRepository>,
    pub roast_repo: Arc<dyn RoastRepository>,
    pub bag_repo: Arc<dyn BagRepository>,
    pub brew_repo: Arc<dyn BrewRepository>,
    pub cup_repo: Arc<dyn CupRepository>,
    pub gear_repo: Arc<dyn GearRepository>,
    pub cafe_repo: Arc<dyn CafeRepository>,
}

/// Listens for invalidation signals, debounces, and rebuilds affected timeline events.
/// Runs as a long-lived background task — spawn with `tokio::spawn`.
pub async fn timeline_rebuild_task(
    mut rx: mpsc::Receiver<TimelineInvalidation>,
    rebuilder: TimelineRebuilder,
    debounce: Duration,
) {
    loop {
        let Some(first) = rx.recv().await else {
            break;
        };

        // Debounce: wait then drain any accumulated signals
        tokio::time::sleep(debounce).await;

        let mut dirty = HashSet::new();
        let mut full_rebuild = matches!(first, TimelineInvalidation::Full);
        if let TimelineInvalidation::Entity {
            entity_type,
            entity_id,
        } = first
        {
            dirty.insert((entity_type, entity_id));
        }

        while let Ok(signal) = rx.try_recv() {
            match signal {
                TimelineInvalidation::Full => full_rebuild = true,
                TimelineInvalidation::Entity {
                    entity_type,
                    entity_id,
                } => {
                    dirty.insert((entity_type, entity_id));
                }
            }
        }

        if full_rebuild {
            if let Err(err) = rebuild_all(&rebuilder).await {
                error!(error = %err, "timeline full rebuild failed");
            }
        } else {
            // Expand cascade targets before refreshing
            let mut all_targets = HashSet::new();
            for (entity_type, entity_id) in &dirty {
                collect_cascade_targets(&rebuilder, *entity_type, *entity_id, &mut all_targets)
                    .await;
            }

            for (entity_type, entity_id) in &all_targets {
                if let Err(err) = refresh_entity_event(&rebuilder, *entity_type, *entity_id).await {
                    warn!(
                        error = %err,
                        entity_type = entity_type.as_str(),
                        entity_id,
                        "failed to refresh timeline event"
                    );
                }
            }

            if !all_targets.is_empty() {
                info!(count = all_targets.len(), "timeline events refreshed");
            }
        }
    }
}

/// Collect the edited entity plus all downstream cascade targets into `targets`.
async fn collect_cascade_targets(
    rebuilder: &TimelineRebuilder,
    entity_type: EntityType,
    entity_id: i64,
    targets: &mut HashSet<(EntityType, i64)>,
) {
    targets.insert((entity_type, entity_id));

    match entity_type {
        EntityType::Roaster => {
            // Roaster → Roasts → Bags → Brews, and Roasts → Cups
            let roasts = rebuilder
                .roast_repo
                .list_by_roaster(RoasterId::new(entity_id))
                .await
                .unwrap_or_default();
            for rwr in &roasts {
                let roast_id = rwr.roast.id.into_inner();
                targets.insert((EntityType::Roast, roast_id));
                collect_roast_downstream(rebuilder, rwr.roast.id, targets).await;
            }
        }
        EntityType::Roast => {
            collect_roast_downstream(rebuilder, RoastId::new(entity_id), targets).await;
        }
        EntityType::Bag => {
            collect_bag_downstream(rebuilder, BagId::new(entity_id), targets).await;
        }
        EntityType::Gear => {
            // Gear → Brews that use this gear piece
            let filter = crate::domain::brews::BrewFilter::for_gear(GearId::new(entity_id));
            if let Ok(page) = rebuilder
                .brew_repo
                .list(
                    filter,
                    &show_all_request::<crate::domain::brews::BrewSortKey>(),
                    None,
                )
                .await
            {
                for bwd in &page.items {
                    targets.insert((EntityType::Brew, bwd.brew.id.into_inner()));
                }
            }
        }
        EntityType::Cafe => {
            // Cafe → Cups at this cafe
            let filter = crate::domain::cups::CupFilter {
                cafe_id: Some(CafeId::new(entity_id)),
                ..Default::default()
            };
            if let Ok(page) = rebuilder
                .cup_repo
                .list(
                    filter,
                    &show_all_request::<crate::domain::cups::CupSortKey>(),
                    None,
                )
                .await
            {
                for cwd in &page.items {
                    targets.insert((EntityType::Cup, cwd.cup.id.into_inner()));
                }
            }
        }
        EntityType::Brew | EntityType::Cup => {
            // Leaf entities — no downstream cascade
        }
    }
}

/// Collect bags, brews, and cups downstream of a roast.
async fn collect_roast_downstream(
    rebuilder: &TimelineRebuilder,
    roast_id: RoastId,
    targets: &mut HashSet<(EntityType, i64)>,
) {
    // Bags for this roast
    let bag_filter = crate::domain::bags::BagFilter {
        roast_id: Some(roast_id),
        ..Default::default()
    };
    if let Ok(page) = rebuilder
        .bag_repo
        .list(
            bag_filter,
            &show_all_request::<crate::domain::bags::BagSortKey>(),
            None,
        )
        .await
    {
        for bwr in &page.items {
            targets.insert((EntityType::Bag, bwr.bag.id.into_inner()));
            collect_bag_downstream(rebuilder, bwr.bag.id, targets).await;
        }
    }

    // Cups for this roast
    let cup_filter = crate::domain::cups::CupFilter {
        roast_id: Some(roast_id),
        ..Default::default()
    };
    if let Ok(page) = rebuilder
        .cup_repo
        .list(
            cup_filter,
            &show_all_request::<crate::domain::cups::CupSortKey>(),
            None,
        )
        .await
    {
        for cwd in &page.items {
            targets.insert((EntityType::Cup, cwd.cup.id.into_inner()));
        }
    }
}

/// Collect brews downstream of a bag.
async fn collect_bag_downstream(
    rebuilder: &TimelineRebuilder,
    bag_id: BagId,
    targets: &mut HashSet<(EntityType, i64)>,
) {
    let filter = crate::domain::brews::BrewFilter::for_bag(bag_id);
    if let Ok(page) = rebuilder
        .brew_repo
        .list(
            filter,
            &show_all_request::<crate::domain::brews::BrewSortKey>(),
            None,
        )
        .await
    {
        for bwd in &page.items {
            targets.insert((EntityType::Brew, bwd.brew.id.into_inner()));
        }
    }
}

/// Refresh a single entity's timeline event(s) by regenerating from current data.
async fn refresh_entity_event(
    rebuilder: &TimelineRebuilder,
    entity_type: EntityType,
    entity_id: i64,
) -> Result<(), crate::domain::RepositoryError> {
    let event = match entity_type {
        EntityType::Roaster => {
            let roaster = rebuilder
                .roaster_repo
                .get(RoasterId::new(entity_id))
                .await?;
            roaster.to_timeline_event()
        }
        EntityType::Roast => {
            let rwr = rebuilder
                .roast_repo
                .get_with_roaster(RoastId::new(entity_id))
                .await?;
            let roaster = rebuilder.roaster_repo.get(rwr.roast.roaster_id).await?;
            roast_timeline_event(&rwr.roast, &roaster)
        }
        EntityType::Bag => {
            let bwr = rebuilder
                .bag_repo
                .get_with_roast(BagId::new(entity_id))
                .await?;
            let roast = rebuilder.roast_repo.get(bwr.bag.roast_id).await?;
            let roaster = rebuilder.roaster_repo.get(roast.roaster_id).await?;
            // update_by_entity updates all events for this entity,
            // preserving each event's original action ("added" or "finished")
            bag_timeline_event(&bwr.bag, "added", &roast, &roaster)
        }
        EntityType::Brew => {
            let enriched = rebuilder
                .brew_repo
                .get_with_details(BrewId::new(entity_id))
                .await?;
            enriched.to_timeline_event()
        }
        EntityType::Cup => {
            let enriched = rebuilder
                .cup_repo
                .get_with_details(CupId::new(entity_id))
                .await?;
            enriched.to_timeline_event()
        }
        EntityType::Gear => {
            let gear = rebuilder.gear_repo.get(GearId::new(entity_id)).await?;
            gear.to_timeline_event()
        }
        EntityType::Cafe => {
            let cafe = rebuilder.cafe_repo.get(CafeId::new(entity_id)).await?;
            cafe.to_timeline_event()
        }
    };

    rebuilder
        .timeline_repo
        .update_by_entity(entity_type, entity_id, event)
        .await
}

/// Delete all timeline events and rebuild from current entity data.
pub async fn rebuild_all(
    rebuilder: &TimelineRebuilder,
) -> Result<(), crate::domain::RepositoryError> {
    let start = std::time::Instant::now();

    rebuilder.timeline_repo.delete_all().await?;

    // Roasters
    let roasters = rebuilder.roaster_repo.list_all().await?;
    for roaster in &roasters {
        if let Err(err) = rebuilder
            .timeline_repo
            .insert(roaster.to_timeline_event())
            .await
        {
            warn!(error = %err, id = %roaster.id, "failed to rebuild roaster timeline event");
        }
    }

    // Cafes
    let cafes = rebuilder.cafe_repo.list_all().await?;
    for cafe in &cafes {
        if let Err(err) = rebuilder
            .timeline_repo
            .insert(cafe.to_timeline_event())
            .await
        {
            warn!(error = %err, id = %cafe.id, "failed to rebuild cafe timeline event");
        }
    }

    // Gear
    let gear_list = rebuilder.gear_repo.list_all().await?;
    for gear in &gear_list {
        if let Err(err) = rebuilder
            .timeline_repo
            .insert(gear.to_timeline_event())
            .await
        {
            warn!(error = %err, id = %gear.id, "failed to rebuild gear timeline event");
        }
    }

    // Roasts (need roaster for each)
    let roasts = rebuilder.roast_repo.list_all().await?;
    for rwr in &roasts {
        let roaster = match rebuilder.roaster_repo.get(rwr.roast.roaster_id).await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, id = %rwr.roast.id, "failed to fetch roaster for roast timeline rebuild");
                continue;
            }
        };
        if let Err(err) = rebuilder
            .timeline_repo
            .insert(roast_timeline_event(&rwr.roast, &roaster))
            .await
        {
            warn!(error = %err, id = %rwr.roast.id, "failed to rebuild roast timeline event");
        }
    }

    rebuild_bag_events(rebuilder).await?;

    // Brews (get_with_details for enrichment)
    let brews = rebuilder.brew_repo.list_all().await?;
    for bwd in &brews {
        if let Err(err) = rebuilder
            .timeline_repo
            .insert(bwd.to_timeline_event())
            .await
        {
            warn!(error = %err, id = %bwd.brew.id, "failed to rebuild brew timeline event");
        }
    }

    // Cups (list_all returns CupWithDetails)
    let cups = rebuilder.cup_repo.list_all().await?;
    for cwd in &cups {
        if let Err(err) = rebuilder
            .timeline_repo
            .insert(cwd.to_timeline_event())
            .await
        {
            warn!(error = %err, id = %cwd.cup.id, "failed to rebuild cup timeline event");
        }
    }

    info!(
        duration_ms = start.elapsed().as_millis(),
        "timeline events rebuilt"
    );
    Ok(())
}

/// Rebuild timeline events for all bags (need roast + roaster lookups for each).
async fn rebuild_bag_events(
    rebuilder: &TimelineRebuilder,
) -> Result<(), crate::domain::RepositoryError> {
    let bags = rebuilder.bag_repo.list_all().await?;
    for bwr in &bags {
        let roast = match rebuilder.roast_repo.get(bwr.bag.roast_id).await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, id = %bwr.bag.id, "failed to fetch roast for bag timeline rebuild");
                continue;
            }
        };
        let roaster = match rebuilder.roaster_repo.get(roast.roaster_id).await {
            Ok(r) => r,
            Err(err) => {
                warn!(error = %err, id = %bwr.bag.id, "failed to fetch roaster for bag timeline rebuild");
                continue;
            }
        };
        if let Err(err) = rebuilder
            .timeline_repo
            .insert(bag_timeline_event(&bwr.bag, "added", &roast, &roaster))
            .await
        {
            warn!(error = %err, id = %bwr.bag.id, "failed to rebuild bag 'added' timeline event");
        }
        if bwr.bag.closed {
            let mut finished_event = bag_timeline_event(&bwr.bag, "finished", &roast, &roaster);
            if let Some(finished_at) = bwr.bag.finished_at {
                finished_event.occurred_at = finished_at.and_time(chrono::NaiveTime::MIN).and_utc();
            }
            if let Err(err) = rebuilder.timeline_repo.insert(finished_event).await {
                warn!(error = %err, id = %bwr.bag.id, "failed to rebuild bag 'finished' timeline event");
            }
        }
    }
    Ok(())
}

/// Helper to create a show-all `ListRequest` for any `SortKey` type.
fn show_all_request<S: crate::domain::listing::SortKey>() -> crate::domain::listing::ListRequest<S>
{
    let sort_key = S::default();
    crate::domain::listing::ListRequest::show_all(sort_key, sort_key.default_direction())
}
