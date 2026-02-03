use std::sync::Arc;

use brewlog::domain::bags::{Bag, BagFilter, BagSortKey, NewBag};
use brewlog::domain::brews::{Brew, BrewFilter, BrewSortKey, NewBrew};
use brewlog::domain::gear::{Gear, GearCategory, GearFilter, GearSortKey, NewGear};
use brewlog::domain::listing::{ListRequest, PageSize};
use brewlog::domain::repositories::{
    BagRepository, BrewRepository, GearRepository, RoastRepository, RoasterRepository,
    TimelineEventRepository,
};
use brewlog::domain::roasters::{NewRoaster, Roaster, RoasterSortKey};
use brewlog::domain::roasts::{NewRoast, Roast, RoastSortKey};
use brewlog::domain::timeline::TimelineEvent;
use brewlog::infrastructure::backup::{BackupData, BackupService};
use brewlog::infrastructure::database::Database;
use brewlog::infrastructure::repositories::bags::SqlBagRepository;
use brewlog::infrastructure::repositories::brews::SqlBrewRepository;
use brewlog::infrastructure::repositories::gear::SqlGearRepository;
use brewlog::infrastructure::repositories::roasters::SqlRoasterRepository;
use brewlog::infrastructure::repositories::roasts::SqlRoastRepository;
use brewlog::infrastructure::repositories::timeline_events::SqlTimelineEventRepository;

struct TestDb {
    roaster_repo: Arc<dyn RoasterRepository>,
    roast_repo: Arc<dyn RoastRepository>,
    bag_repo: Arc<dyn BagRepository>,
    gear_repo: Arc<dyn GearRepository>,
    brew_repo: Arc<dyn BrewRepository>,
    timeline_repo: Arc<dyn TimelineEventRepository>,
    backup_service: BackupService,
}

async fn create_test_db() -> TestDb {
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    let pool = database.clone_pool();

    TestDb {
        roaster_repo: Arc::new(SqlRoasterRepository::new(pool.clone())),
        roast_repo: Arc::new(SqlRoastRepository::new(pool.clone())),
        bag_repo: Arc::new(SqlBagRepository::new(pool.clone())),
        gear_repo: Arc::new(SqlGearRepository::new(pool.clone())),
        brew_repo: Arc::new(SqlBrewRepository::new(pool.clone())),
        timeline_repo: Arc::new(SqlTimelineEventRepository::new(pool.clone())),
        backup_service: BackupService::new(pool),
    }
}

fn list_all_request<K: brewlog::domain::listing::SortKey>() -> ListRequest<K> {
    ListRequest::new(
        1,
        PageSize::All,
        K::default(),
        K::default().default_direction(),
    )
}

async fn list_all_roasters(repo: &dyn RoasterRepository) -> Vec<Roaster> {
    repo.list(&list_all_request::<RoasterSortKey>(), None)
        .await
        .expect("failed to list roasters")
        .items
}

async fn list_all_roasts(repo: &dyn RoastRepository) -> Vec<Roast> {
    let page = repo
        .list(&list_all_request::<RoastSortKey>(), None)
        .await
        .expect("failed to list roasts");
    page.items.into_iter().map(|rwr| rwr.roast).collect()
}

async fn list_all_bags(repo: &dyn BagRepository) -> Vec<Bag> {
    let page = repo
        .list(BagFilter::all(), &list_all_request::<BagSortKey>(), None)
        .await
        .expect("failed to list bags");
    page.items.into_iter().map(|bwr| bwr.bag).collect()
}

async fn list_all_gear(repo: &dyn GearRepository) -> Vec<Gear> {
    repo.list(GearFilter::all(), &list_all_request::<GearSortKey>(), None)
        .await
        .expect("failed to list gear")
        .items
}

async fn list_all_brews(repo: &dyn BrewRepository) -> Vec<Brew> {
    let page = repo
        .list(BrewFilter::all(), &list_all_request::<BrewSortKey>(), None)
        .await
        .expect("failed to list brews");
    page.items.into_iter().map(|bwd| bwd.brew).collect()
}

async fn list_all_timeline_events(repo: &dyn TimelineEventRepository) -> Vec<TimelineEvent> {
    repo.list_all()
        .await
        .expect("failed to list timeline events")
}

/// Populate a database with representative test data and return the key entities.
async fn populate_test_data(db: &TestDb) -> (Roaster, Roast, Bag, Gear, Gear, Gear, Brew) {
    // Create roaster
    let roaster = db
        .roaster_repo
        .insert(NewRoaster {
            name: "Square Mile".to_string(),
            country: "UK".to_string(),
            city: Some("London".to_string()),
            homepage: Some("https://shop.squaremilecoffee.com".to_string()),
            notes: Some("Great seasonal espresso".to_string()),
        })
        .await
        .expect("failed to create roaster");

    // Create roast
    let roast = db
        .roast_repo
        .insert(NewRoast {
            roaster_id: roaster.id,
            name: "Red Brick".to_string(),
            origin: "Brazil".to_string(),
            region: "Cerrado".to_string(),
            producer: "Fazenda Passeio".to_string(),
            tasting_notes: vec![
                "Milk Chocolate".to_string(),
                "Hazelnut".to_string(),
                "Caramel".to_string(),
            ],
            process: "Natural".to_string(),
        })
        .await
        .expect("failed to create roast");

    // Create bag (250g)
    let bag = db
        .bag_repo
        .insert(NewBag {
            roast_id: roast.id,
            roast_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
            amount: 250.0,
        })
        .await
        .expect("failed to create bag");

    // Create gear
    let grinder = db
        .gear_repo
        .insert(NewGear {
            category: GearCategory::Grinder,
            make: "Comandante".to_string(),
            model: "C40 MK4".to_string(),
        })
        .await
        .expect("failed to create grinder");

    let brewer = db
        .gear_repo
        .insert(NewGear {
            category: GearCategory::Brewer,
            make: "Hario".to_string(),
            model: "V60 02".to_string(),
        })
        .await
        .expect("failed to create brewer");

    let filter_paper = db
        .gear_repo
        .insert(NewGear {
            category: GearCategory::FilterPaper,
            make: "Hario".to_string(),
            model: "V60 Tabbed 02".to_string(),
        })
        .await
        .expect("failed to create filter paper");

    // Create a brew (deducts 15g from bag, remaining becomes 235)
    let brew = db
        .brew_repo
        .insert(NewBrew {
            bag_id: bag.id,
            coffee_weight: 15.0,
            grinder_id: grinder.id,
            grind_setting: 24.0,
            brewer_id: brewer.id,
            filter_paper_id: Some(filter_paper.id),
            water_volume: 250,
            water_temp: 93.5,
        })
        .await
        .expect("failed to create brew");

    // Re-fetch bag to get updated remaining
    let bag = db
        .bag_repo
        .get(bag.id)
        .await
        .expect("failed to re-fetch bag");

    assert_eq!(
        bag.remaining, 235.0,
        "bag remaining should be 235 after brew"
    );

    (roaster, roast, bag, grinder, brewer, filter_paper, brew)
}

#[tokio::test]
async fn backup_and_restore_round_trip() {
    // 1. Create source database and populate with test data
    let source = create_test_db().await;
    let (roaster, roast, bag, grinder, brewer, filter_paper, brew) =
        populate_test_data(&source).await;

    // Verify timeline events were created (roaster + roast inserts create them)
    let source_timeline = list_all_timeline_events(source.timeline_repo.as_ref()).await;
    assert!(
        source_timeline.len() >= 2,
        "expected at least 2 timeline events from roaster+roast creation"
    );

    // 2. Export backup
    let backup_data = source
        .backup_service
        .export()
        .await
        .expect("failed to export backup");

    assert_eq!(backup_data.version, 1);
    assert_eq!(backup_data.roasters.len(), 1);
    assert_eq!(backup_data.roasts.len(), 1);
    assert_eq!(backup_data.bags.len(), 1);
    assert_eq!(backup_data.gear.len(), 3);
    assert_eq!(backup_data.brews.len(), 1);
    assert_eq!(backup_data.timeline_events.len(), source_timeline.len());

    // 3. Serialize to JSON and deserialize back (verify serde round-trip)
    let json = serde_json::to_string_pretty(&backup_data).expect("failed to serialize backup");
    let restored_data: BackupData =
        serde_json::from_str(&json).expect("failed to deserialize backup");

    assert_eq!(restored_data.version, 1);
    assert_eq!(restored_data.roasters.len(), 1);
    assert_eq!(restored_data.roasts.len(), 1);
    assert_eq!(restored_data.bags.len(), 1);
    assert_eq!(restored_data.gear.len(), 3);
    assert_eq!(restored_data.brews.len(), 1);

    // 4. Restore to a fresh database
    let target = create_test_db().await;
    target
        .backup_service
        .restore(restored_data)
        .await
        .expect("failed to restore backup");

    // 5. Verify all data matches

    // Roasters
    let target_roasters = list_all_roasters(target.roaster_repo.as_ref()).await;
    assert_eq!(target_roasters.len(), 1);
    let restored_roaster = &target_roasters[0];
    assert_eq!(restored_roaster.id, roaster.id);
    assert_eq!(restored_roaster.name, roaster.name);
    assert_eq!(restored_roaster.slug, roaster.slug);
    assert_eq!(restored_roaster.country, roaster.country);
    assert_eq!(restored_roaster.city, roaster.city);
    assert_eq!(restored_roaster.homepage, roaster.homepage);
    assert_eq!(restored_roaster.notes, roaster.notes);
    assert_eq!(restored_roaster.created_at, roaster.created_at);

    // Roasts
    let target_roasts = list_all_roasts(target.roast_repo.as_ref()).await;
    assert_eq!(target_roasts.len(), 1);
    let restored_roast = &target_roasts[0];
    assert_eq!(restored_roast.id, roast.id);
    assert_eq!(restored_roast.roaster_id, roast.roaster_id);
    assert_eq!(restored_roast.name, roast.name);
    assert_eq!(restored_roast.slug, roast.slug);
    assert_eq!(restored_roast.origin, roast.origin);
    assert_eq!(restored_roast.region, roast.region);
    assert_eq!(restored_roast.producer, roast.producer);
    assert_eq!(restored_roast.process, roast.process);
    assert_eq!(restored_roast.tasting_notes, roast.tasting_notes);

    // Bags - critically verify remaining was NOT re-deducted
    let target_bags = list_all_bags(target.bag_repo.as_ref()).await;
    assert_eq!(target_bags.len(), 1);
    let restored_bag = &target_bags[0];
    assert_eq!(restored_bag.id, bag.id);
    assert_eq!(restored_bag.roast_id, bag.roast_id);
    assert_eq!(restored_bag.roast_date, bag.roast_date);
    assert_eq!(restored_bag.amount, 250.0);
    assert_eq!(
        restored_bag.remaining, 235.0,
        "bag remaining should be preserved at 235, not re-deducted by brew restore"
    );
    assert_eq!(restored_bag.closed, bag.closed);

    // Gear
    let target_gear = list_all_gear(target.gear_repo.as_ref()).await;
    assert_eq!(target_gear.len(), 3);
    let grinder_restored = target_gear.iter().find(|g| g.id == grinder.id).unwrap();
    assert_eq!(grinder_restored.category, GearCategory::Grinder);
    assert_eq!(grinder_restored.make, "Comandante");
    assert_eq!(grinder_restored.model, "C40 MK4");
    let brewer_restored = target_gear.iter().find(|g| g.id == brewer.id).unwrap();
    assert_eq!(brewer_restored.category, GearCategory::Brewer);
    let fp_restored = target_gear
        .iter()
        .find(|g| g.id == filter_paper.id)
        .unwrap();
    assert_eq!(fp_restored.category, GearCategory::FilterPaper);

    // Brews
    let target_brews = list_all_brews(target.brew_repo.as_ref()).await;
    assert_eq!(target_brews.len(), 1);
    let restored_brew = &target_brews[0];
    assert_eq!(restored_brew.id, brew.id);
    assert_eq!(restored_brew.bag_id, brew.bag_id);
    assert_eq!(restored_brew.coffee_weight, 15.0);
    assert_eq!(restored_brew.grinder_id, brew.grinder_id);
    assert_eq!(restored_brew.grind_setting, 24.0);
    assert_eq!(restored_brew.brewer_id, brew.brewer_id);
    assert_eq!(restored_brew.filter_paper_id, Some(filter_paper.id));
    assert_eq!(restored_brew.water_volume, 250);
    assert_eq!(restored_brew.water_temp, 93.5);

    // Timeline events
    let target_timeline = list_all_timeline_events(target.timeline_repo.as_ref()).await;
    assert_eq!(target_timeline.len(), source_timeline.len());
    for (source_event, target_event) in source_timeline.iter().zip(target_timeline.iter()) {
        assert_eq!(target_event.id, source_event.id);
        assert_eq!(target_event.entity_type, source_event.entity_type);
        assert_eq!(target_event.entity_id, source_event.entity_id);
        assert_eq!(target_event.action, source_event.action);
        assert_eq!(target_event.title, source_event.title);
        assert_eq!(target_event.details.len(), source_event.details.len());
        assert_eq!(target_event.tasting_notes, source_event.tasting_notes);
        assert_eq!(target_event.slug, source_event.slug);
        assert_eq!(target_event.roaster_slug, source_event.roaster_slug);
    }
}

#[tokio::test]
async fn restore_to_non_empty_database_fails() {
    let db = create_test_db().await;

    // Add a roaster to make the database non-empty
    db.roaster_repo
        .insert(NewRoaster {
            name: "Existing Roaster".to_string(),
            country: "UK".to_string(),
            city: None,
            homepage: None,
            notes: None,
        })
        .await
        .expect("failed to create roaster");

    // Create a minimal backup
    let backup_data = BackupData {
        version: 1,
        created_at: chrono::Utc::now(),
        roasters: vec![],
        gear: vec![],
        roasts: vec![],
        bags: vec![],
        brews: vec![],
        timeline_events: vec![],
    };

    // Restore should fail because the database is not empty
    let result = db.backup_service.restore(backup_data).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not empty"),
        "expected 'not empty' error, got: {err_msg}"
    );
}

#[tokio::test]
async fn backup_empty_database() {
    let db = create_test_db().await;

    let backup_data = db
        .backup_service
        .export()
        .await
        .expect("failed to export empty database");

    assert_eq!(backup_data.version, 1);
    assert!(backup_data.roasters.is_empty());
    assert!(backup_data.roasts.is_empty());
    assert!(backup_data.bags.is_empty());
    assert!(backup_data.gear.is_empty());
    assert!(backup_data.brews.is_empty());
    assert!(backup_data.timeline_events.is_empty());

    // Should serialize to valid JSON
    let json = serde_json::to_string_pretty(&backup_data).expect("failed to serialize");
    let parsed: BackupData = serde_json::from_str(&json).expect("failed to deserialize");
    assert_eq!(parsed.version, 1);
}
