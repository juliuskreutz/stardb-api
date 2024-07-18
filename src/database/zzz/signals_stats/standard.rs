use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatStandard {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
}

pub async fn set(stat: &DbSignalsStatStandard, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats/standard/set.sql",
        stat.uid,
        stat.luck_a,
        stat.luck_s,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<DbSignalsStatStandard> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatStandard,
        "sql/zzz/signals_stats/standard/get_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?)
}
