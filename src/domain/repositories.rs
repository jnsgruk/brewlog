use super::RepositoryError;
use crate::domain::ai_usage::{AiUsage, AiUsageSummary, NewAiUsage};
use crate::domain::entity_type::EntityType;
use crate::domain::listing::{ListRequest, Page, SortDirection, SortKey};

use crate::domain::bags::{Bag, BagFilter, BagSortKey, BagWithRoast, NewBag, UpdateBag};
use crate::domain::brews::{Brew, BrewFilter, BrewSortKey, BrewWithDetails, NewBrew, UpdateBrew};
use crate::domain::cafes::{Cafe, CafeSortKey, NewCafe, UpdateCafe};
use crate::domain::cups::{Cup, CupFilter, CupSortKey, CupWithDetails, NewCup, UpdateCup};
use crate::domain::gear::{Gear, GearFilter, GearSortKey, NewGear, UpdateGear};
use crate::domain::ids::{
    BagId, BrewId, CafeId, CupId, GearId, PasskeyCredentialId, RegistrationTokenId, RoastId,
    RoasterId, SessionId, TokenId, UserId,
};
use crate::domain::images::EntityImage;
use crate::domain::passkey_credentials::{NewPasskeyCredential, PasskeyCredential};
use crate::domain::registration_tokens::{NewRegistrationToken, RegistrationToken};
use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasters::{NewRoaster, Roaster, UpdateRoaster};
use crate::domain::roasts::RoastSortKey;
use crate::domain::roasts::{NewRoast, Roast, RoastWithRoaster, UpdateRoast};
use crate::domain::sessions::{NewSession, Session};
use crate::domain::timeline::{NewTimelineEvent, TimelineEvent, TimelineSortKey};
use crate::domain::tokens::{NewToken, Token};
use crate::domain::users::{NewUser, User};
use async_trait::async_trait;

#[async_trait]
pub trait RoasterRepository: Send + Sync {
    async fn insert(&self, roaster: NewRoaster) -> Result<Roaster, RepositoryError>;
    async fn get(&self, id: RoasterId) -> Result<Roaster, RepositoryError>;
    async fn get_by_slug(&self, slug: &str) -> Result<Roaster, RepositoryError>;
    async fn list(
        &self,
        request: &ListRequest<RoasterSortKey>,
        search: Option<&str>,
    ) -> Result<Page<Roaster>, RepositoryError>;
    async fn update(
        &self,
        id: RoasterId,
        changes: UpdateRoaster,
    ) -> Result<Roaster, RepositoryError>;
    async fn delete(&self, id: RoasterId) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<Roaster>, RepositoryError> {
        let sort_key = <RoasterSortKey as SortKey>::default();
        let request =
            ListRequest::<RoasterSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(&request, None).await?;
        Ok(page.items)
    }

    async fn list_all_sorted(
        &self,
        sort_key: RoasterSortKey,
        direction: SortDirection,
    ) -> Result<Vec<Roaster>, RepositoryError> {
        let request = ListRequest::show_all(sort_key, direction);
        let page = self.list(&request, None).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait RoastRepository: Send + Sync {
    async fn insert(&self, roast: NewRoast) -> Result<Roast, RepositoryError>;
    async fn get(&self, id: RoastId) -> Result<Roast, RepositoryError>;
    async fn get_with_roaster(&self, id: RoastId) -> Result<RoastWithRoaster, RepositoryError>;
    async fn get_by_slug(
        &self,
        roaster_id: RoasterId,
        slug: &str,
    ) -> Result<Roast, RepositoryError>;
    async fn list(
        &self,
        request: &ListRequest<RoastSortKey>,
        search: Option<&str>,
    ) -> Result<Page<RoastWithRoaster>, RepositoryError>;
    async fn list_by_roaster(
        &self,
        roaster_id: RoasterId,
    ) -> Result<Vec<RoastWithRoaster>, RepositoryError>;
    async fn update(&self, id: RoastId, changes: UpdateRoast) -> Result<Roast, RepositoryError>;
    async fn delete(&self, id: RoastId) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<RoastWithRoaster>, RepositoryError> {
        let sort_key = <RoastSortKey as SortKey>::default();
        let request = ListRequest::<RoastSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(&request, None).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait TimelineEventRepository: Send + Sync {
    async fn insert(&self, event: NewTimelineEvent) -> Result<TimelineEvent, RepositoryError>;
    async fn list(
        &self,
        request: &ListRequest<TimelineSortKey>,
    ) -> Result<Page<TimelineEvent>, RepositoryError>;

    async fn update_by_entity(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        event: NewTimelineEvent,
    ) -> Result<(), RepositoryError>;

    async fn delete_by_entity(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<(), RepositoryError>;

    async fn delete_all(&self) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<TimelineEvent>, RepositoryError> {
        let sort_key = <TimelineSortKey as SortKey>::default();
        let request =
            ListRequest::<TimelineSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(&request).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert(&self, user: NewUser) -> Result<User, RepositoryError>;
    async fn get(&self, id: UserId) -> Result<User, RepositoryError>;
    async fn get_by_username(&self, username: &str) -> Result<User, RepositoryError>;
    async fn get_by_uuid(&self, uuid: &str) -> Result<User, RepositoryError>;
    async fn exists(&self) -> Result<bool, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<User>, RepositoryError>;
}

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn insert(&self, token: NewToken) -> Result<Token, RepositoryError>;
    async fn get(&self, id: TokenId) -> Result<Token, RepositoryError>;
    async fn get_by_token_hash(&self, token_hash: &str) -> Result<Token, RepositoryError>;
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Token>, RepositoryError>;
    async fn revoke(&self, id: TokenId) -> Result<Token, RepositoryError>;
    async fn update_last_used(&self, id: TokenId) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn insert(&self, session: NewSession) -> Result<Session, RepositoryError>;
    async fn get(&self, id: SessionId) -> Result<Session, RepositoryError>;
    async fn get_by_token_hash(&self, token_hash: &str) -> Result<Session, RepositoryError>;
    async fn delete(&self, id: SessionId) -> Result<(), RepositoryError>;
    async fn delete_expired(&self) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait BagRepository: Send + Sync {
    async fn insert(&self, bag: NewBag) -> Result<Bag, RepositoryError>;
    async fn get(&self, id: BagId) -> Result<Bag, RepositoryError>;
    async fn get_with_roast(&self, id: BagId) -> Result<BagWithRoast, RepositoryError>;
    async fn list(
        &self,
        filter: BagFilter,
        request: &ListRequest<BagSortKey>,
        search: Option<&str>,
    ) -> Result<Page<BagWithRoast>, RepositoryError>;
    async fn update(&self, id: BagId, changes: UpdateBag) -> Result<Bag, RepositoryError>;
    async fn delete(&self, id: BagId) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<BagWithRoast>, RepositoryError> {
        let sort_key = <BagSortKey as SortKey>::default();
        let request = ListRequest::<BagSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(BagFilter::default(), &request, None).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait GearRepository: Send + Sync {
    async fn insert(&self, gear: NewGear) -> Result<Gear, RepositoryError>;
    async fn get(&self, id: GearId) -> Result<Gear, RepositoryError>;
    async fn list(
        &self,
        filter: GearFilter,
        request: &ListRequest<GearSortKey>,
        search: Option<&str>,
    ) -> Result<Page<Gear>, RepositoryError>;
    async fn update(&self, id: GearId, changes: UpdateGear) -> Result<Gear, RepositoryError>;
    async fn delete(&self, id: GearId) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<Gear>, RepositoryError> {
        let sort_key = <GearSortKey as SortKey>::default();
        let request = ListRequest::<GearSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(GearFilter::default(), &request, None).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait BrewRepository: Send + Sync {
    /// Insert a new brew and deduct `coffee_weight` from the bag's remaining amount.
    /// This is a transactional operation.
    async fn insert(&self, brew: NewBrew) -> Result<Brew, RepositoryError>;
    async fn get(&self, id: BrewId) -> Result<Brew, RepositoryError>;
    async fn get_with_details(&self, id: BrewId) -> Result<BrewWithDetails, RepositoryError>;
    async fn list(
        &self,
        filter: BrewFilter,
        request: &ListRequest<BrewSortKey>,
        search: Option<&str>,
    ) -> Result<Page<BrewWithDetails>, RepositoryError>;
    async fn update(&self, id: BrewId, changes: UpdateBrew) -> Result<Brew, RepositoryError>;
    async fn delete(&self, id: BrewId) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<BrewWithDetails>, RepositoryError> {
        let sort_key = <BrewSortKey as SortKey>::default();
        let request = ListRequest::<BrewSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(BrewFilter::default(), &request, None).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait CafeRepository: Send + Sync {
    async fn insert(&self, cafe: NewCafe) -> Result<Cafe, RepositoryError>;
    async fn get(&self, id: CafeId) -> Result<Cafe, RepositoryError>;
    async fn get_by_slug(&self, slug: &str) -> Result<Cafe, RepositoryError>;
    async fn list(
        &self,
        request: &ListRequest<CafeSortKey>,
        search: Option<&str>,
    ) -> Result<Page<Cafe>, RepositoryError>;
    async fn update(&self, id: CafeId, changes: UpdateCafe) -> Result<Cafe, RepositoryError>;
    async fn delete(&self, id: CafeId) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<Cafe>, RepositoryError> {
        let sort_key = <CafeSortKey as SortKey>::default();
        let request = ListRequest::<CafeSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(&request, None).await?;
        Ok(page.items)
    }

    async fn list_all_sorted(
        &self,
        sort_key: CafeSortKey,
        direction: SortDirection,
    ) -> Result<Vec<Cafe>, RepositoryError> {
        let request = ListRequest::show_all(sort_key, direction);
        let page = self.list(&request, None).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait CupRepository: Send + Sync {
    async fn insert(&self, cup: NewCup) -> Result<Cup, RepositoryError>;
    async fn get(&self, id: CupId) -> Result<Cup, RepositoryError>;
    async fn get_with_details(&self, id: CupId) -> Result<CupWithDetails, RepositoryError>;
    async fn list(
        &self,
        filter: CupFilter,
        request: &ListRequest<CupSortKey>,
        search: Option<&str>,
    ) -> Result<Page<CupWithDetails>, RepositoryError>;
    async fn update(&self, id: CupId, changes: UpdateCup) -> Result<Cup, RepositoryError>;
    async fn delete(&self, id: CupId) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<CupWithDetails>, RepositoryError> {
        let sort_key = <CupSortKey as SortKey>::default();
        let request = ListRequest::<CupSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(CupFilter::default(), &request, None).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait PasskeyCredentialRepository: Send + Sync {
    async fn insert(
        &self,
        credential: NewPasskeyCredential,
    ) -> Result<PasskeyCredential, RepositoryError>;
    async fn get(&self, id: PasskeyCredentialId) -> Result<PasskeyCredential, RepositoryError>;
    async fn list_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<PasskeyCredential>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<PasskeyCredential>, RepositoryError>;
    async fn update_credential_json(
        &self,
        id: PasskeyCredentialId,
        credential_json: &str,
    ) -> Result<(), RepositoryError>;
    async fn update_last_used(&self, id: PasskeyCredentialId) -> Result<(), RepositoryError>;
    async fn delete(&self, id: PasskeyCredentialId) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait RegistrationTokenRepository: Send + Sync {
    async fn insert(
        &self,
        token: NewRegistrationToken,
    ) -> Result<RegistrationToken, RepositoryError>;
    async fn get_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<RegistrationToken, RepositoryError>;
    async fn mark_used(
        &self,
        id: RegistrationTokenId,
        user_id: UserId,
    ) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait AiUsageRepository: Send + Sync {
    async fn insert(&self, usage: NewAiUsage) -> Result<AiUsage, RepositoryError>;
    async fn summary_for_user(&self, user_id: UserId) -> Result<AiUsageSummary, RepositoryError>;
}

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn upsert(&self, image: EntityImage) -> Result<(), RepositoryError>;
    async fn get(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<EntityImage, RepositoryError>;
    async fn get_thumbnail(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<EntityImage, RepositoryError>;
    async fn delete(&self, entity_type: EntityType, entity_id: i64) -> Result<(), RepositoryError>;
    async fn has_image(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<bool, RepositoryError>;
}

#[async_trait]
pub trait StatsRepository: Send + Sync {
    async fn roaster_country_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError>;
    async fn roast_origin_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError>;
    async fn cup_country_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError>;
    async fn cafe_country_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError>;
    async fn roast_summary(
        &self,
    ) -> Result<crate::domain::stats::RoastSummaryStats, RepositoryError>;
    async fn consumption_summary(
        &self,
    ) -> Result<crate::domain::stats::ConsumptionStats, RepositoryError>;
    async fn brewing_summary(
        &self,
    ) -> Result<crate::domain::stats::BrewingSummaryStats, RepositoryError>;
    async fn entity_counts(&self) -> Result<crate::domain::stats::EntityCounts, RepositoryError>;
    async fn get_cached(
        &self,
    ) -> Result<Option<crate::domain::stats::CachedStats>, RepositoryError>;
    async fn store_cached(
        &self,
        stats: &crate::domain::stats::CachedStats,
    ) -> Result<(), RepositoryError>;
}
