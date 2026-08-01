use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use super::DbSignalsStatCount;

pub struct DbSignalsStatStandard {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
}

pub async fn set<'e, E>(stat: &DbSignalsStatStandard, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/zzz/signals_stats/standard/set.sql",
        stat.uid,
        stat.luck_a,
        stat.luck_s,
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatStandard>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatStandard,
        "sql/zzz/signals_stats/standard/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbSignalsStatCount>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatCount,
        "sql/zzz/signals_stats/standard/get_all.sql"
    )
    .fetch_all(pool)
    .await?)
}
