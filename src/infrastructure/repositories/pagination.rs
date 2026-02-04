use sqlx::{FromRow, QueryBuilder, query_scalar};

use crate::domain::RepositoryError;
use crate::domain::listing::{ListRequest, Page, PageSize, SortKey};
use crate::infrastructure::database::{DatabaseDriver, DatabasePool, DatabaseRow};

/// Describes which columns to search and the term to match.
pub struct SearchFilter {
    pub term: String,
    pub columns: Vec<&'static str>,
}

impl SearchFilter {
    pub fn new(term: &str, columns: Vec<&'static str>) -> Option<Self> {
        let term = term.trim().to_lowercase();
        if term.is_empty() {
            None
        } else {
            Some(Self { term, columns })
        }
    }

    fn like_pattern(&self) -> String {
        format!("%{}%", self.term)
    }
}

pub async fn paginate<K, R, T, MapFn>(
    pool: &DatabasePool,
    request: &ListRequest<K>,
    base_query: &str,
    count_query: &str,
    order_clause: &str,
    search: Option<&SearchFilter>,
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
            let records = fetch_records::<R>(pool, base_query, order_clause, search, None).await?;

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

            let total = fetch_count(pool, count_query, search).await?;

            let mut records = fetch_records::<R>(
                pool,
                base_query,
                order_clause,
                search,
                Some((limit, offset)),
            )
            .await?;

            if page > 1 && records.is_empty() && total > 0 {
                let last_page = ((total + limit - 1) / limit) as u32;
                page = last_page.max(1);
                let offset = i64::from(page - 1).saturating_mul(limit);
                records = fetch_records::<R>(
                    pool,
                    base_query,
                    order_clause,
                    search,
                    Some((limit, offset)),
                )
                .await?;
            }

            let mut items = Vec::with_capacity(records.len());
            for record in records {
                items.push(map_fn(record)?);
            }

            Ok(Page::new(items, page, page_size, total as u64, false))
        }
    }
}

async fn fetch_records<R>(
    pool: &DatabasePool,
    base_query: &str,
    order_clause: &str,
    search: Option<&SearchFilter>,
    limit_offset: Option<(i64, i64)>,
) -> Result<Vec<R>, RepositoryError>
where
    R: for<'r> FromRow<'r, DatabaseRow> + Send + Unpin,
{
    let mut qb = QueryBuilder::new(base_query);
    if let Some(sf) = search {
        append_search_condition(&mut qb, base_query, sf);
    }
    qb.push(" ORDER BY ");
    qb.push(order_clause);
    if let Some((limit, offset)) = limit_offset {
        qb.push(" LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);
    }
    qb.build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))
}

async fn fetch_count(
    pool: &DatabasePool,
    count_query: &str,
    search: Option<&SearchFilter>,
) -> Result<i64, RepositoryError> {
    if let Some(sf) = search {
        let mut qb = QueryBuilder::new(count_query);
        append_search_condition(&mut qb, count_query, sf);
        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        Ok(row.0)
    } else {
        query_scalar(count_query)
            .fetch_one(pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))
    }
}

fn append_search_condition(
    qb: &mut QueryBuilder<'_, DatabaseDriver>,
    base_sql: &str,
    search: &SearchFilter,
) {
    let connector = if base_sql.to_uppercase().contains("WHERE") {
        " AND "
    } else {
        " WHERE "
    };
    qb.push(connector);
    qb.push("(");
    let pattern = search.like_pattern();
    for (i, col) in search.columns.iter().enumerate() {
        if i > 0 {
            qb.push(" OR ");
        }
        qb.push("LOWER(");
        qb.push(*col);
        qb.push(") LIKE ");
        qb.push_bind(pattern.clone());
    }
    qb.push(")");
}
