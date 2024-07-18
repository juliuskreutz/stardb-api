use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatGlobalWEngine {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_a_percentile: f64,
    pub luck_s_percentile: f64,
}

pub async fn set(stat: &DbSignalsStatGlobalWEngine, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/w_engine/set.sql",
        stat.uid,
        stat.count_percentile,
        stat.luck_a_percentile,
        stat.luck_s_percentile,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatGlobalWEngine>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatGlobalWEngine,
        "sql/zzz/signals_stats_global/w_engine/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_by_uid(uid: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/w_engine/delete_by_uid.sql",
        uid
    )
    .execute(pool)
    .await?;

    Ok(())
}
