use super::RepositoryError;
use crate::domain::listing::{ListRequest, Page, SortDirection, SortKey};

use crate::domain::bags::{Bag, BagSortKey, BagWithRoast, NewBag, UpdateBag};
use crate::domain::ids::{BagId, RoastId, RoasterId, SessionId, TokenId, UserId};
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
        let page = self.list(&request).await?;
        Ok(page.items)
    }

    async fn list_all_sorted(
        &self,
        sort_key: RoasterSortKey,
        direction: SortDirection,
    ) -> Result<Vec<Roaster>, RepositoryError> {
        let request = ListRequest::show_all(sort_key, direction);
        let page = self.list(&request).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait RoastRepository: Send + Sync {
    async fn insert(&self, roast: NewRoast) -> Result<Roast, RepositoryError>;
    async fn get(&self, id: RoastId) -> Result<Roast, RepositoryError>;
    async fn get_by_slug(
        &self,
        roaster_id: RoasterId,
        slug: &str,
    ) -> Result<Roast, RepositoryError>;
    async fn list(
        &self,
        request: &ListRequest<RoastSortKey>,
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
        let page = self.list(&request).await?;
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
    async fn list(
        &self,
        request: &ListRequest<BagSortKey>,
    ) -> Result<Page<BagWithRoast>, RepositoryError>;
    async fn list_by_roast(&self, roast_id: RoastId) -> Result<Vec<BagWithRoast>, RepositoryError>;
    async fn update(&self, id: BagId, changes: UpdateBag) -> Result<Bag, RepositoryError>;
    async fn delete(&self, id: BagId) -> Result<(), RepositoryError>;
    async fn list_open(&self) -> Result<Vec<BagWithRoast>, RepositoryError>;
    async fn list_closed(
        &self,
        request: &ListRequest<BagSortKey>,
    ) -> Result<Page<BagWithRoast>, RepositoryError>;
    async fn list_all(&self) -> Result<Vec<BagWithRoast>, RepositoryError>;
}
