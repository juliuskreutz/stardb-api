use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use super::DbWishesStatCount;

pub struct DbWishesStatChronicled {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

pub async fn set<'e, E>(stat: &DbWishesStatChronicled, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/gi/wishes_stats/chronicled/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWishesStatChronicled>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatChronicled,
        "sql/gi/wishes_stats/chronicled/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWishesStatCount>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatCount,
        "sql/gi/wishes_stats/chronicled/get_all.sql",
    )
    .fetch_all(pool)
    .await?)
}
