use std::str::FromStr;

use anyhow::{Context, bail};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use crate::domain::bags::Bag;
use crate::domain::brews::{Brew, QuickNote};
use crate::domain::cafes::Cafe;
use crate::domain::cups::Cup;
use crate::domain::entity_type::EntityType;
use crate::domain::gear::{Gear, GearCategory};
use crate::domain::ids::{
    BagId, BrewId, CafeId, CupId, GearId, RoastId, RoasterId, TimelineEventId,
};
use crate::domain::roasters::Roaster;
use crate::domain::roasts::Roast;
use crate::domain::timeline::TimelineEvent;
use crate::infrastructure::database::{DatabasePool, DatabaseTransaction};

fn decode_json_vec<T: serde::de::DeserializeOwned>(
    raw: Option<String>,
    label: &str,
) -> anyhow::Result<Vec<T>> {
    match raw {
        Some(s) if !s.is_empty() => {
            from_str(&s).with_context(|| format!("failed to decode {label}: {s}"))
        }
        _ => Ok(Vec::new()),
    }
}

fn decode_json_opt<T: serde::de::DeserializeOwned>(
    raw: Option<String>,
    label: &str,
) -> anyhow::Result<Option<T>> {
    match raw {
        Some(s) if !s.is_empty() => from_str(&s)
            .map(Some)
            .with_context(|| format!("failed to decode {label}: {s}")),
        _ => Ok(None),
    }
}

mod base64_serde {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&STANDARD.encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        STANDARD.decode(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupImage {
    pub entity_type: String,
    pub entity_id: i64,
    pub content_type: String,
    #[serde(with = "base64_serde")]
    pub image_data: Vec<u8>,
    #[serde(with = "base64_serde")]
    pub thumbnail_data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub roasters: Vec<Roaster>,
    pub gear: Vec<Gear>,
    pub roasts: Vec<Roast>,
    pub bags: Vec<Bag>,
    pub brews: Vec<Brew>,
    #[serde(default)]
    pub cafes: Vec<Cafe>,
    #[serde(default)]
    pub cups: Vec<Cup>,
    pub timeline_events: Vec<TimelineEvent>,
    #[serde(default)]
    pub images: Vec<BackupImage>,
}

pub struct BackupService {
    pool: DatabasePool,
}

impl BackupService {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    pub async fn export(&self) -> anyhow::Result<BackupData> {
        let roasters = self.export_roasters().await?;
        let gear = self.export_gear().await?;
        let roasts = self.export_roasts().await?;
        let bags = self.export_bags().await?;
        let brews = self.export_brews().await?;
        let cafes = self.export_cafes().await?;
        let cups = self.export_cups().await?;
        let timeline_events = self.export_timeline_events().await?;
        let images = self.export_images().await?;

        Ok(BackupData {
            version: 2,
            created_at: Utc::now(),
            roasters,
            gear,
            roasts,
            bags,
            brews,
            cafes,
            cups,
            timeline_events,
            images,
        })
    }

    pub async fn restore(&self, data: BackupData) -> anyhow::Result<()> {
        self.verify_empty_database().await?;

        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to begin transaction")?;

        self.restore_roasters(&mut tx, &data.roasters).await?;
        self.restore_gear(&mut tx, &data.gear).await?;
        self.restore_roasts(&mut tx, &data.roasts).await?;
        self.restore_bags(&mut tx, &data.bags).await?;
        self.restore_brews(&mut tx, &data.brews).await?;
        self.restore_cafes(&mut tx, &data.cafes).await?;
        self.restore_cups(&mut tx, &data.cups).await?;
        self.restore_timeline_events(&mut tx, &data.timeline_events)
            .await?;
        self.restore_images(&mut tx, &data.images).await?;

        tx.commit().await.context("failed to commit transaction")?;

        Ok(())
    }

    /// Delete all coffee data, leaving auth tables intact.
    ///
    /// After a successful reset the database is in the same state that
    /// `verify_empty_database()` requires, so a subsequent `restore()` will
    /// succeed.
    pub async fn reset(&self) -> anyhow::Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to begin transaction")?;

        // Delete in FK-safe order: children before parents.
        // brews has RESTRICT FK → gear; cups has RESTRICT FK → roasts, cafes.
        let tables = [
            "entity_images",
            "brews",
            "cups",
            "bags",
            "roasts",
            "timeline_events",
            "gear",
            "cafes",
            "roasters",
            "stats_cache",
        ];

        for table in tables {
            let query = format!("DELETE FROM {table}");
            sqlx::query(&query)
                .execute(&mut *tx)
                .await
                .with_context(|| format!("failed to delete from {table}"))?;
        }

        tx.commit().await.context("failed to commit transaction")?;

        Ok(())
    }

    // --- Export methods ---

    async fn export_roasters(&self) -> anyhow::Result<Vec<Roaster>> {
        let records = sqlx::query_as::<_, RoasterRecord>(
            "SELECT id, name, slug, country, city, homepage, created_at FROM roasters ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export roasters")?;

        Ok(records
            .into_iter()
            .map(RoasterRecord::into_domain)
            .collect())
    }

    async fn export_gear(&self) -> anyhow::Result<Vec<Gear>> {
        let records = sqlx::query_as::<_, GearRecord>(
            "SELECT id, category, make, model, created_at, updated_at FROM gear ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export gear")?;

        records
            .into_iter()
            .map(GearRecord::into_domain)
            .collect::<anyhow::Result<Vec<_>>>()
    }

    async fn export_roasts(&self) -> anyhow::Result<Vec<Roast>> {
        let records = sqlx::query_as::<_, RoastRecord>(
            "SELECT id, roaster_id, name, slug, origin, region, producer, process, tasting_notes, created_at FROM roasts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export roasts")?;

        records
            .into_iter()
            .map(RoastRecord::into_domain)
            .collect::<anyhow::Result<Vec<_>>>()
    }

    async fn export_bags(&self) -> anyhow::Result<Vec<Bag>> {
        let records = sqlx::query_as::<_, BagRecord>(
            "SELECT id, roast_id, roast_date, amount, remaining, closed, finished_at, created_at, updated_at FROM bags ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export bags")?;

        Ok(records.into_iter().map(BagRecord::into_domain).collect())
    }

    async fn export_brews(&self) -> anyhow::Result<Vec<Brew>> {
        let records = sqlx::query_as::<_, BrewRecord>(
            "SELECT id, bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, brew_time, created_at, updated_at FROM brews ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export brews")?;

        Ok(records.into_iter().map(BrewRecord::into_domain).collect())
    }

    async fn export_cafes(&self) -> anyhow::Result<Vec<Cafe>> {
        let records = sqlx::query_as::<_, CafeRecord>(
            "SELECT id, name, slug, city, country, latitude, longitude, website, created_at, updated_at FROM cafes ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export cafes")?;

        Ok(records.into_iter().map(CafeRecord::into_domain).collect())
    }

    async fn export_cups(&self) -> anyhow::Result<Vec<Cup>> {
        let records = sqlx::query_as::<_, CupRecord>(
            "SELECT id, roast_id, cafe_id, created_at, updated_at FROM cups ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export cups")?;

        Ok(records.into_iter().map(CupRecord::into_domain).collect())
    }

    async fn export_timeline_events(&self) -> anyhow::Result<Vec<TimelineEvent>> {
        let records = sqlx::query_as::<_, TimelineEventRecord>(
            "SELECT id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json FROM timeline_events ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export timeline events")?;

        records
            .into_iter()
            .map(TimelineEventRecord::into_domain)
            .collect::<anyhow::Result<Vec<_>>>()
    }

    async fn export_images(&self) -> anyhow::Result<Vec<BackupImage>> {
        let records = sqlx::query_as::<_, ImageRecord>(
            "SELECT entity_type, entity_id, content_type, image_data, thumbnail_data FROM entity_images ORDER BY entity_type, entity_id",
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to export images")?;

        Ok(records.into_iter().map(ImageRecord::into_backup).collect())
    }

    // --- Restore methods ---

    async fn verify_empty_database(&self) -> anyhow::Result<()> {
        let tables = [
            "roasters",
            "roasts",
            "bags",
            "gear",
            "brews",
            "cafes",
            "cups",
            "timeline_events",
            "entity_images",
        ];

        for table in tables {
            let query = format!("SELECT COUNT(*) as count FROM {table}");
            let row: (i64,) = sqlx::query_as(&query)
                .fetch_one(&self.pool)
                .await
                .with_context(|| format!("failed to check table {table}"))?;

            if row.0 > 0 {
                bail!(
                    "Cannot restore: table '{table}' is not empty ({} rows). Restore requires an empty database.",
                    row.0
                );
            }
        }

        Ok(())
    }

    async fn restore_roasters(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        roasters: &[Roaster],
    ) -> anyhow::Result<()> {
        for roaster in roasters {
            sqlx::query(
                "INSERT INTO roasters (id, name, slug, country, city, homepage, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(i64::from(roaster.id))
            .bind(&roaster.name)
            .bind(&roaster.slug)
            .bind(&roaster.country)
            .bind(roaster.city.as_deref())
            .bind(roaster.homepage.as_deref())
            .bind(roaster.created_at)
            .execute(&mut **tx)
            .await
            .context("failed to restore roaster")?;
        }

        Ok(())
    }

    async fn restore_gear(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        gear: &[Gear],
    ) -> anyhow::Result<()> {
        for item in gear {
            sqlx::query(
                "INSERT INTO gear (id, category, make, model, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(i64::from(item.id))
            .bind(item.category.as_str())
            .bind(&item.make)
            .bind(&item.model)
            .bind(item.created_at)
            .bind(item.updated_at)
            .execute(&mut **tx)
            .await
            .context("failed to restore gear")?;
        }

        Ok(())
    }

    async fn restore_roasts(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        roasts: &[Roast],
    ) -> anyhow::Result<()> {
        for roast in roasts {
            let tasting_notes_json = if roast.tasting_notes.is_empty() {
                None
            } else {
                Some(
                    to_string(&roast.tasting_notes)
                        .context("failed to encode tasting notes for restore")?,
                )
            };

            sqlx::query(
                "INSERT INTO roasts (id, roaster_id, name, slug, origin, region, producer, process, tasting_notes, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(i64::from(roast.id))
            .bind(i64::from(roast.roaster_id))
            .bind(&roast.name)
            .bind(&roast.slug)
            .bind(roast.origin.as_deref())
            .bind(roast.region.as_deref())
            .bind(roast.producer.as_deref())
            .bind(roast.process.as_deref())
            .bind(tasting_notes_json.as_deref())
            .bind(roast.created_at)
            .execute(&mut **tx)
            .await
            .context("failed to restore roast")?;
        }

        Ok(())
    }

    async fn restore_bags(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        bags: &[Bag],
    ) -> anyhow::Result<()> {
        for bag in bags {
            sqlx::query(
                "INSERT INTO bags (id, roast_id, roast_date, amount, remaining, closed, finished_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(i64::from(bag.id))
            .bind(i64::from(bag.roast_id))
            .bind(bag.roast_date)
            .bind(bag.amount)
            .bind(bag.remaining)
            .bind(bag.closed)
            .bind(bag.finished_at)
            .bind(bag.created_at)
            .bind(bag.updated_at)
            .execute(&mut **tx)
            .await
            .context("failed to restore bag")?;
        }

        Ok(())
    }

    async fn restore_brews(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        brews: &[Brew],
    ) -> anyhow::Result<()> {
        for brew in brews {
            let quick_notes_json = if brew.quick_notes.is_empty() {
                None
            } else {
                let values: Vec<&str> = brew.quick_notes.iter().map(|n| n.form_value()).collect();
                Some(to_string(&values).context("failed to encode quick notes for restore")?)
            };

            sqlx::query(
                "INSERT INTO brews (id, bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, brew_time, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(i64::from(brew.id))
            .bind(i64::from(brew.bag_id))
            .bind(brew.coffee_weight)
            .bind(i64::from(brew.grinder_id))
            .bind(brew.grind_setting)
            .bind(i64::from(brew.brewer_id))
            .bind(brew.filter_paper_id.map(i64::from))
            .bind(brew.water_volume)
            .bind(brew.water_temp)
            .bind(quick_notes_json.as_deref())
            .bind(brew.brew_time)
            .bind(brew.created_at)
            .bind(brew.updated_at)
            .execute(&mut **tx)
            .await
            .context("failed to restore brew")?;
        }

        Ok(())
    }

    async fn restore_cafes(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        cafes: &[Cafe],
    ) -> anyhow::Result<()> {
        for cafe in cafes {
            sqlx::query(
                "INSERT INTO cafes (id, name, slug, city, country, latitude, longitude, website, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(i64::from(cafe.id))
            .bind(&cafe.name)
            .bind(&cafe.slug)
            .bind(&cafe.city)
            .bind(&cafe.country)
            .bind(cafe.latitude)
            .bind(cafe.longitude)
            .bind(cafe.website.as_deref())
            .bind(cafe.created_at)
            .bind(cafe.updated_at)
            .execute(&mut **tx)
            .await
            .context("failed to restore cafe")?;
        }

        Ok(())
    }

    async fn restore_cups(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        cups: &[Cup],
    ) -> anyhow::Result<()> {
        for cup in cups {
            sqlx::query(
                "INSERT INTO cups (id, roast_id, cafe_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(i64::from(cup.id))
            .bind(i64::from(cup.roast_id))
            .bind(i64::from(cup.cafe_id))
            .bind(cup.created_at)
            .bind(cup.updated_at)
            .execute(&mut **tx)
            .await
            .context("failed to restore cup")?;
        }

        Ok(())
    }

    async fn restore_timeline_events(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        events: &[TimelineEvent],
    ) -> anyhow::Result<()> {
        for event in events {
            let details_json = to_string(&event.details)
                .context("failed to encode timeline event details for restore")?;

            let tasting_notes_json = to_string(&event.tasting_notes)
                .context("failed to encode timeline event tasting notes for restore")?;

            let brew_data_json = event
                .brew_data
                .as_ref()
                .map(to_string)
                .transpose()
                .context("failed to encode timeline brew data for restore")?;

            sqlx::query(
                "INSERT INTO timeline_events (id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(i64::from(event.id))
            .bind(event.entity_type.as_str())
            .bind(event.entity_id)
            .bind(&event.action)
            .bind(event.occurred_at)
            .bind(&event.title)
            .bind(&details_json)
            .bind(&tasting_notes_json)
            .bind(event.slug.as_deref())
            .bind(event.roaster_slug.as_deref())
            .bind(brew_data_json.as_deref())
            .execute(&mut **tx)
            .await
            .context("failed to restore timeline event")?;
        }

        Ok(())
    }

    async fn restore_images(
        &self,
        tx: &mut DatabaseTransaction<'_>,
        images: &[BackupImage],
    ) -> anyhow::Result<()> {
        for image in images {
            sqlx::query(
                "INSERT INTO entity_images (entity_type, entity_id, content_type, image_data, thumbnail_data) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&image.entity_type)
            .bind(image.entity_id)
            .bind(&image.content_type)
            .bind(&image.image_data)
            .bind(&image.thumbnail_data)
            .execute(&mut **tx)
            .await
            .context("failed to restore image")?;
        }

        Ok(())
    }
}

// --- Record types for export queries ---

#[derive(sqlx::FromRow)]
struct RoasterRecord {
    id: i64,
    name: String,
    slug: String,
    country: String,
    city: Option<String>,
    homepage: Option<String>,
    created_at: DateTime<Utc>,
}

impl RoasterRecord {
    fn into_domain(self) -> Roaster {
        Roaster {
            id: RoasterId::from(self.id),
            name: self.name,
            slug: self.slug,
            country: self.country,
            city: self.city,
            homepage: self.homepage,
            created_at: self.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct GearRecord {
    id: i64,
    category: String,
    make: String,
    model: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl GearRecord {
    fn into_domain(self) -> anyhow::Result<Gear> {
        let category = GearCategory::from_str(&self.category)
            .map_err(|()| anyhow::anyhow!("invalid gear category: {}", self.category))?;

        Ok(Gear {
            id: GearId::new(self.id),
            category,
            make: self.make,
            model: self.model,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RoastRecord {
    id: i64,
    roaster_id: i64,
    name: String,
    slug: String,
    origin: Option<String>,
    region: Option<String>,
    producer: Option<String>,
    process: Option<String>,
    tasting_notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl RoastRecord {
    fn into_domain(self) -> anyhow::Result<Roast> {
        let tasting_notes = decode_json_vec(self.tasting_notes, "tasting notes")?;

        Ok(Roast {
            id: RoastId::from(self.id),
            roaster_id: RoasterId::from(self.roaster_id),
            name: self.name,
            slug: self.slug,
            origin: self.origin,
            region: self.region,
            producer: self.producer,
            process: self.process,
            tasting_notes,
            created_at: self.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct BagRecord {
    id: i64,
    roast_id: i64,
    roast_date: Option<NaiveDate>,
    amount: f64,
    remaining: f64,
    closed: bool,
    finished_at: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl BagRecord {
    fn into_domain(self) -> Bag {
        Bag {
            id: BagId::new(self.id),
            roast_id: RoastId::new(self.roast_id),
            roast_date: self.roast_date,
            amount: self.amount,
            remaining: self.remaining,
            closed: self.closed,
            finished_at: self.finished_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BrewRecord {
    id: i64,
    bag_id: i64,
    coffee_weight: f64,
    grinder_id: i64,
    grind_setting: f64,
    brewer_id: i64,
    filter_paper_id: Option<i64>,
    water_volume: i32,
    water_temp: f64,
    quick_notes: Option<String>,
    brew_time: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl BrewRecord {
    fn into_domain(self) -> Brew {
        let quick_notes = match self.quick_notes {
            Some(s) if !s.is_empty() => serde_json::from_str::<Vec<String>>(&s)
                .unwrap_or_default()
                .iter()
                .filter_map(|v| QuickNote::from_str_value(v))
                .collect(),
            _ => Vec::new(),
        };
        Brew {
            id: BrewId::new(self.id),
            bag_id: BagId::new(self.bag_id),
            coffee_weight: self.coffee_weight,
            grinder_id: GearId::new(self.grinder_id),
            grind_setting: self.grind_setting,
            brewer_id: GearId::new(self.brewer_id),
            filter_paper_id: self.filter_paper_id.map(GearId::new),
            water_volume: self.water_volume,
            water_temp: self.water_temp,
            quick_notes,
            brew_time: self.brew_time,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CafeRecord {
    id: i64,
    name: String,
    slug: String,
    city: String,
    country: String,
    latitude: f64,
    longitude: f64,
    website: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CafeRecord {
    fn into_domain(self) -> Cafe {
        Cafe {
            id: CafeId::from(self.id),
            name: self.name,
            slug: self.slug,
            city: self.city,
            country: self.country,
            latitude: self.latitude,
            longitude: self.longitude,
            website: self.website,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CupRecord {
    id: i64,
    roast_id: i64,
    cafe_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CupRecord {
    fn into_domain(self) -> Cup {
        Cup {
            id: CupId::from(self.id),
            roast_id: RoastId::from(self.roast_id),
            cafe_id: CafeId::from(self.cafe_id),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TimelineEventRecord {
    id: i64,
    entity_type: String,
    entity_id: i64,
    action: String,
    occurred_at: DateTime<Utc>,
    title: String,
    details_json: Option<String>,
    tasting_notes_json: Option<String>,
    slug: Option<String>,
    roaster_slug: Option<String>,
    brew_data_json: Option<String>,
}

impl TimelineEventRecord {
    fn into_domain(self) -> anyhow::Result<TimelineEvent> {
        let details = decode_json_vec(self.details_json, "timeline event details")?;
        let tasting_notes = decode_json_vec(self.tasting_notes_json, "timeline tasting notes")?;
        let brew_data = decode_json_opt(self.brew_data_json, "timeline brew data")?;

        let entity_type: EntityType = self
            .entity_type
            .parse()
            .map_err(|()| anyhow::anyhow!("unknown entity type: {}", self.entity_type))?;

        Ok(TimelineEvent {
            id: TimelineEventId::from(self.id),
            entity_type,
            entity_id: self.entity_id,
            action: self.action,
            occurred_at: self.occurred_at,
            title: self.title,
            details,
            tasting_notes,
            slug: self.slug,
            roaster_slug: self.roaster_slug,
            brew_data,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ImageRecord {
    entity_type: String,
    entity_id: i64,
    content_type: String,
    image_data: Vec<u8>,
    thumbnail_data: Vec<u8>,
}

impl ImageRecord {
    fn into_backup(self) -> BackupImage {
        BackupImage {
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            content_type: self.content_type,
            image_data: self.image_data,
            thumbnail_data: self.thumbnail_data,
        }
    }
}
