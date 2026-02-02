use sqlx::{FromRow, query_as, query_scalar};

use crate::domain::RepositoryError;
use crate::domain::listing::{ListRequest, Page, PageSize, SortKey};
use crate::infrastructure::database::{DatabasePool, DatabaseRow};

pub async fn paginate<K, R, T, MapFn>(
    pool: &DatabasePool,
    request: &ListRequest<K>,
    base_query: &str,
    count_query: &str,
    order_clause: &str,
    map_fn: MapFn,
) -> Result<Page<T>, RepositoryError>
where
    K: SortKey,
    R: for<'r> FromRow<'r, DatabaseRow> + Send + Unpin,
    T: Send,
    MapFn: Fn(R) -> Result<T, RepositoryError> + Send + Sync,
{
    match request.page_size() {
        PageSize::All => {
            let query = format!("{base_query} ORDER BY {order_clause}");
            let records = query_as::<_, R>(&query)
                .fetch_all(pool)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

            let mut items = Vec::with_capacity(records.len());
            for record in records {
                items.push(map_fn(record)?);
            }
            let total = items.len() as u64;
            let page_size = total.min(u64::from(u32::MAX)) as u32;
            Ok(Page::new(items, 1, page_size.max(1), total, true))
        }
        PageSize::Limited(page_size) => {
            let limit = i64::from(page_size);
            let mut page = request.page();
            let offset = i64::from(page - 1).saturating_mul(limit);

            let query_sql = format!("{base_query} ORDER BY {order_clause} LIMIT ? OFFSET ?");

            let mut records = query_as::<_, R>(&query_sql)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

            let total: i64 = query_scalar(count_query)
                .fetch_one(pool)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

            if page > 1 && records.is_empty() && total > 0 {
                let last_page = ((total + limit - 1) / limit) as u32;
                page = last_page.max(1);
                let offset = i64::from(page - 1).saturating_mul(limit);
                records = query_as::<_, R>(&query_sql)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
            }

            let mut items = Vec::with_capacity(records.len());
            for record in records {
                items.push(map_fn(record)?);
            }

            Ok(Page::new(items, page, page_size, total as u64, false))
        }
    }
}
