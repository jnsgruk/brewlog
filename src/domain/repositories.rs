use super::RepositoryError;
use crate::domain::listing::{ListRequest, Page, SortDirection, SortKey};

use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasters::{Roaster, UpdateRoaster};
use crate::domain::roasts::RoastSortKey;
use crate::domain::roasts::{Roast, RoastWithRoaster, UpdateRoast};
use crate::domain::timeline::{TimelineEvent, TimelineSortKey};
use crate::domain::tokens::{Token, TokenId};
use crate::domain::users::{User, UserId};
use async_trait::async_trait;

#[async_trait]
pub trait RoasterRepository: Send + Sync {
    async fn insert(&self, roaster: Roaster) -> Result<Roaster, RepositoryError>;
    async fn get(&self, id: String) -> Result<Roaster, RepositoryError>;
    async fn list(
        &self,
        request: &ListRequest<RoasterSortKey>,
    ) -> Result<Page<Roaster>, RepositoryError>;
    async fn update(&self, id: String, changes: UpdateRoaster) -> Result<Roaster, RepositoryError>;
    async fn delete(&self, id: String) -> Result<(), RepositoryError>;

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
    async fn insert(&self, roast: Roast) -> Result<Roast, RepositoryError>;
    async fn get(&self, id: String) -> Result<Roast, RepositoryError>;
    async fn list(
        &self,
        request: &ListRequest<RoastSortKey>,
    ) -> Result<Page<RoastWithRoaster>, RepositoryError>;
    async fn list_by_roaster(
        &self,
        roaster_id: String,
    ) -> Result<Vec<RoastWithRoaster>, RepositoryError>;
    async fn update(&self, id: String, changes: UpdateRoast) -> Result<Roast, RepositoryError>;
    async fn delete(&self, id: String) -> Result<(), RepositoryError>;

    async fn list_all(&self) -> Result<Vec<RoastWithRoaster>, RepositoryError> {
        let sort_key = <RoastSortKey as SortKey>::default();
        let request = ListRequest::<RoastSortKey>::show_all(sort_key, sort_key.default_direction());
        let page = self.list(&request).await?;
        Ok(page.items)
    }
}

#[async_trait]
pub trait TimelineEventRepository: Send + Sync {
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
    async fn insert(&self, user: User) -> Result<User, RepositoryError>;
    async fn get(&self, id: UserId) -> Result<User, RepositoryError>;
    async fn get_by_username(&self, username: &str) -> Result<User, RepositoryError>;
    async fn exists(&self) -> Result<bool, RepositoryError>;
}

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn insert(&self, token: Token) -> Result<Token, RepositoryError>;
    async fn get(&self, id: TokenId) -> Result<Token, RepositoryError>;
    async fn get_by_token_hash(&self, token_hash: &str) -> Result<Token, RepositoryError>;
    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Token>, RepositoryError>;
    async fn revoke(&self, id: TokenId) -> Result<Token, RepositoryError>;
    async fn update_last_used(&self, id: TokenId) -> Result<(), RepositoryError>;
}
