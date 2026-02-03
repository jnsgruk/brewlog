use sqlx::{FromRow, QueryBuilder, query_as, query_scalar};

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
            let records: Vec<R> = if let Some(sf) = search {
                let mut qb = QueryBuilder::new(base_query);
                append_search_condition(&mut qb, base_query, sf);
                qb.push(" ORDER BY ");
                qb.push(order_clause);
                qb.build_query_as()
                    .fetch_all(pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            } else {
                let query = format!("{base_query} ORDER BY {order_clause}");
                query_as::<_, R>(&query)
                    .fetch_all(pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            };

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

            let total: i64 = if let Some(sf) = search {
                let mut count_qb = QueryBuilder::new(count_query);
                append_search_condition(&mut count_qb, count_query, sf);
                let row: (i64,) = count_qb
                    .build_query_as()
                    .fetch_one(pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
                row.0
            } else {
                query_scalar(count_query)
                    .fetch_one(pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            };

            let mut records =
                fetch_page::<R>(pool, base_query, order_clause, search, limit, offset).await?;

            if page > 1 && records.is_empty() && total > 0 {
                let last_page = ((total + limit - 1) / limit) as u32;
                page = last_page.max(1);
                let offset = i64::from(page - 1).saturating_mul(limit);
                records =
                    fetch_page::<R>(pool, base_query, order_clause, search, limit, offset).await?;
            }

            let mut items = Vec::with_capacity(records.len());
            for record in records {
                items.push(map_fn(record)?);
            }

            Ok(Page::new(items, page, page_size, total as u64, false))
        }
    }
}

async fn fetch_page<R>(
    pool: &DatabasePool,
    base_query: &str,
    order_clause: &str,
    search: Option<&SearchFilter>,
    limit: i64,
    offset: i64,
) -> Result<Vec<R>, RepositoryError>
where
    R: for<'r> FromRow<'r, DatabaseRow> + Send + Unpin,
{
    if let Some(sf) = search {
        let mut qb = QueryBuilder::new(base_query);
        append_search_condition(&mut qb, base_query, sf);
        qb.push(" ORDER BY ");
        qb.push(order_clause);
        qb.push(" LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);
        qb.build_query_as()
            .fetch_all(pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))
    } else {
        let query_sql = format!("{base_query} ORDER BY {order_clause} LIMIT ? OFFSET ?");
        query_as::<_, R>(&query_sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
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
