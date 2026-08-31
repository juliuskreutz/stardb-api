use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use super::DbWishesStatCount;

pub struct DbWishesStatStandard {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

pub async fn set<'e, E>(stat: &DbWishesStatStandard, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/gi/wishes_stats/standard/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWishesStatStandard>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatStandard,
        "sql/gi/wishes_stats/standard/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWishesStatCount>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatCount,
        "sql/gi/wishes_stats/standard/get_all.sql",
    )
    .fetch_all(pool)
    .await?)
}
