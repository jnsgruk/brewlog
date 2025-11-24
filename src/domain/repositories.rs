use super::RepositoryError;
use crate::domain::listing::{ListRequest, Page, SortDirection, SortKey};

use crate::domain::roasters::RoasterSortKey;
use crate::domain::roasters::{Roaster, UpdateRoaster};
use crate::domain::roasts::RoastSortKey;
use crate::domain::roasts::{Roast, RoastWithRoaster, UpdateRoast};
use crate::domain::timeline::TimelineEvent;
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
    async fn list_all(&self) -> Result<Vec<TimelineEvent>, RepositoryError>;
}
