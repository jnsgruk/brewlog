use brewlog::domain::RepositoryError;
use brewlog::domain::bags::UpdateBag;
use brewlog::domain::brews::{NewBrew, QuickNote, UpdateBrew};
use brewlog::domain::ids::{BagId, BrewId, GearId};
use brewlog::domain::repositories::{BagRepository, BrewRepository};
use brewlog::infrastructure::database::Database;
use brewlog::infrastructure::repositories::{bags::SqlBagRepository, brews::SqlBrewRepository};

struct Inventory {
    db: Database,
    bags: SqlBagRepository,
    brews: SqlBrewRepository,
}

impl Inventory {
    async fn new(stock: f64) -> Self {
        Self::with_database(Database::connect("sqlite::memory:").await.unwrap(), stock).await
    }

    async fn with_database(db: Database, stock: f64) -> Self {
        for statement in [
            "INSERT INTO roasters (name, country) VALUES ('Test', 'GB')",
            "INSERT INTO roasts (roaster_id, name) VALUES (1, 'Test')",
            "INSERT INTO gear (category, make, model) VALUES ('grinder', 'Test', 'Grinder'), ('brewer', 'Test', 'Brewer')",
        ] {
            sqlx::query(statement).execute(db.pool()).await.unwrap();
        }
        for _ in 0..2 {
            sqlx::query("INSERT INTO bags (roast_id, amount, remaining) VALUES (1, 250, ?)")
                .bind(stock)
                .execute(db.pool())
                .await
                .unwrap();
        }
        Self {
            bags: SqlBagRepository::new(db.clone_pool()),
            brews: SqlBrewRepository::new(db.clone_pool()),
            db,
        }
    }

    fn new_brew(dose: f64) -> NewBrew {
        NewBrew {
            bag_id: BagId::new(1),
            coffee_weight: dose,
            grinder_id: GearId::new(1),
            grind_setting: 24.0,
            brewer_id: GearId::new(2),
            filter_paper_id: None,
            water_volume: 250,
            water_temp: 92.0,
            quick_notes: vec![],
            brew_time: None,
            created_at: None,
        }
    }

    async fn stock(&self) -> (f64, f64) {
        (
            self.bags.get(BagId::new(1)).await.unwrap().remaining,
            self.bags.get(BagId::new(2)).await.unwrap().remaining,
        )
    }

    async fn correct(&self, id: BrewId, dose: f64, bag: i64) {
        self.brews
            .update(
                id,
                UpdateBrew {
                    bag_id: Some(BagId::new(bag)),
                    coffee_weight: Some(dose),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn dose_and_bag_corrections_reconcile_inventory_and_retries_are_idempotent() {
    let inventory = Inventory::new(250.0).await;
    let brew = inventory
        .brews
        .insert(Inventory::new_brew(15.0))
        .await
        .unwrap();
    assert_eq!(inventory.stock().await, (235.0, 250.0));
    inventory.correct(brew.id, 18.0, 1).await;
    assert_eq!(inventory.stock().await, (232.0, 250.0));
    // Concurrent identical retries must observe the latest committed dose/bag.
    tokio::join!(
        inventory.correct(brew.id, 18.0, 2),
        inventory.correct(brew.id, 18.0, 2)
    );
    assert_eq!(inventory.stock().await, (250.0, 232.0));
    inventory.correct(brew.id, 12.0, 2).await;
    assert_eq!(inventory.stock().await, (250.0, 238.0));
    inventory.correct(brew.id, 10.0, 1).await;
    assert_eq!(inventory.stock().await, (240.0, 250.0));
    inventory.brews.delete(brew.id).await.unwrap();
    assert_eq!(inventory.stock().await, (240.0, 250.0));
}

#[tokio::test]
async fn clamped_deductions_never_refund_untracked_coffee() {
    let inventory = Inventory::new(250.0).await;
    let brew = inventory
        .brews
        .insert(Inventory::new_brew(300.0))
        .await
        .unwrap();
    inventory.correct(brew.id, 280.0, 1).await;
    assert_eq!(inventory.stock().await, (0.0, 250.0));
    inventory.correct(brew.id, 200.0, 1).await;
    assert_eq!(inventory.stock().await, (50.0, 250.0));
    inventory.correct(brew.id, 200.0, 2).await;
    assert_eq!(inventory.stock().await, (250.0, 50.0));

    let empty = Inventory::new(0.0).await;
    let brew = empty.brews.insert(Inventory::new_brew(15.0)).await.unwrap();
    empty.correct(brew.id, 5.0, 2).await;
    assert_eq!(empty.stock().await, (0.0, 0.0));
}

#[tokio::test]
async fn manual_stock_counts_supersede_old_deductions() {
    for stock in [0.0, 100.0, 235.0, 250.0] {
        let inventory = Inventory::new(250.0).await;
        let brew = inventory
            .brews
            .insert(Inventory::new_brew(15.0))
            .await
            .unwrap();
        inventory
            .bags
            .update(
                BagId::new(1),
                UpdateBag {
                    remaining: Some(stock),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        inventory.correct(brew.id, 12.0, 1).await;
        assert_eq!(inventory.stock().await, (stock, 250.0));
        // A later increase tracks only its fresh, actual deduction.
        inventory.correct(brew.id, 18.0, 1).await;
        inventory.correct(brew.id, 18.0, 2).await;
        assert_eq!(inventory.stock().await, (stock, 232.0));
    }
}

#[tokio::test]
async fn closed_bags_allow_sensory_edits_reductions_and_moves_without_reopening() {
    let inventory = Inventory::new(250.0).await;
    let brew = inventory
        .brews
        .insert(Inventory::new_brew(15.0))
        .await
        .unwrap();
    inventory
        .bags
        .update(
            BagId::new(1),
            UpdateBag {
                closed: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    inventory
        .brews
        .update(
            brew.id,
            UpdateBrew {
                bag_id: Some(brew.bag_id),
                coffee_weight: Some(15.0),
                quick_notes: Some(vec![QuickNote::Good]),
                water_temp: Some(94.0),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(inventory.stock().await, (235.0, 250.0));
    assert!(matches!(
        inventory
            .brews
            .update(
                brew.id,
                UpdateBrew {
                    coffee_weight: Some(18.0),
                    ..Default::default()
                }
            )
            .await,
        Err(RepositoryError::Conflict(_))
    ));
    inventory.correct(brew.id, 12.0, 1).await;
    assert_eq!(inventory.stock().await, (238.0, 250.0));
    inventory.correct(brew.id, 12.0, 2).await;
    assert_eq!(inventory.stock().await, (250.0, 238.0));
    assert!(inventory.bags.get(BagId::new(1)).await.unwrap().closed);
}

#[tokio::test]
async fn failed_corrections_roll_back_both_bags_and_the_brew() {
    let inventory = Inventory::new(250.0).await;
    let brew = inventory
        .brews
        .insert(Inventory::new_brew(15.0))
        .await
        .unwrap();
    inventory
        .bags
        .update(
            BagId::new(2),
            UpdateBag {
                closed: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    for target in [2, 999] {
        assert!(matches!(
            inventory
                .brews
                .update(
                    brew.id,
                    UpdateBrew {
                        bag_id: Some(BagId::new(target)),
                        coffee_weight: Some(18.0),
                        ..Default::default()
                    }
                )
                .await,
            Err(RepositoryError::Conflict(_))
        ));
        assert_eq!(inventory.stock().await, (235.0, 250.0));
        assert_eq!(
            inventory.brews.get(brew.id).await.unwrap().bag_id,
            brew.bag_id
        );
    }
    inventory
        .bags
        .update(
            BagId::new(2),
            UpdateBag {
                closed: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    // Fail the brew write after both inventory writes via a foreign-key error.
    assert!(
        inventory
            .brews
            .update(
                brew.id,
                UpdateBrew {
                    bag_id: Some(BagId::new(2)),
                    coffee_weight: Some(18.0),
                    grinder_id: Some(GearId::new(999)),
                    ..Default::default()
                }
            )
            .await
            .is_err()
    );
    assert_eq!(inventory.stock().await, (235.0, 250.0));
    assert_eq!(
        inventory.brews.get(brew.id).await.unwrap().coffee_weight,
        15.0
    );
}

#[tokio::test]
async fn invalid_doses_are_rejected_even_for_direct_repository_calls() {
    let inventory = Inventory::new(250.0).await;
    let brew = inventory
        .brews
        .insert(Inventory::new_brew(15.0))
        .await
        .unwrap();
    for dose in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            inventory
                .brews
                .insert(Inventory::new_brew(dose))
                .await
                .is_err()
        );
        assert!(
            inventory
                .brews
                .update(
                    brew.id,
                    UpdateBrew {
                        coffee_weight: Some(dose),
                        ..Default::default()
                    }
                )
                .await
                .is_err()
        );
        assert_eq!(inventory.stock().await, (235.0, 250.0));
    }
    assert!(matches!(
        inventory
            .brews
            .update(
                BrewId::new(999),
                UpdateBrew {
                    coffee_weight: Some(18.0),
                    ..Default::default()
                }
            )
            .await,
        Err(RepositoryError::NotFound)
    ));
}

#[tokio::test]
async fn legacy_or_imported_brews_do_not_infer_refundable_stock() {
    let inventory = Inventory::new(235.0).await;
    sqlx::query("INSERT INTO brews (bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, water_volume, water_temp) VALUES (1, 15, 1, 24, 2, 250, 92)")
        .execute(inventory.db.pool()).await.unwrap();
    inventory.correct(BrewId::new(1), 12.0, 1).await;
    assert_eq!(inventory.stock().await, (235.0, 235.0));
    inventory.correct(BrewId::new(1), 12.0, 2).await;
    assert_eq!(inventory.stock().await, (235.0, 223.0));
}

#[tokio::test]
async fn retries_from_separate_connections_do_not_apply_inventory_twice() {
    let directory = tempfile::tempdir().unwrap();
    let url = format!(
        "sqlite://{}",
        directory.path().join("inventory.db").display()
    );
    let inventory = Inventory::with_database(Database::connect(&url).await.unwrap(), 250.0).await;
    let other = SqlBrewRepository::new(Database::connect(&url).await.unwrap().clone_pool());
    let brew = inventory
        .brews
        .insert(Inventory::new_brew(15.0))
        .await
        .unwrap();
    let correction = UpdateBrew {
        bag_id: Some(BagId::new(2)),
        coffee_weight: Some(18.0),
        ..Default::default()
    };
    let (first, retry) = tokio::join!(
        inventory.brews.update(brew.id, correction.clone()),
        other.update(brew.id, correction)
    );
    first.unwrap();
    retry.unwrap();
    assert_eq!(inventory.stock().await, (250.0, 232.0));
}
