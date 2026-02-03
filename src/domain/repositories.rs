use super::RepositoryError;
use crate::domain::listing::{ListRequest, Page, SortDirection, SortKey};

use crate::domain::bags::{Bag, BagFilter, BagSortKey, BagWithRoast, NewBag, UpdateBag};
use crate::domain::brews::{Brew, BrewFilter, BrewSortKey, BrewWithDetails, NewBrew};
use crate::domain::cafes::{Cafe, CafeSortKey, NewCafe, UpdateCafe};
use crate::domain::gear::{Gear, GearFilter, GearSortKey, NewGear, UpdateGear};
use crate::domain::ids::{
    BagId, BrewId, CafeId, GearId, RoastId, RoasterId, SessionId, TokenId, UserId,
};
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
    async fn exists(&self) -> Result<bool, RepositoryError>;
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
    async fn delete(&self, id: BrewId) -> Result<(), RepositoryError>;
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
