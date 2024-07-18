use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatGlobalSpecial {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_a_percentile: f64,
    pub luck_s_percentile: f64,
}

pub async fn set(stat: &DbSignalsStatGlobalSpecial, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/special/set.sql",
        stat.uid,
        stat.count_percentile,
        stat.luck_a_percentile,
        stat.luck_s_percentile,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatGlobalSpecial>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatGlobalSpecial,
        "sql/zzz/signals_stats_global/special/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_by_uid(uid: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/special/delete_by_uid.sql",
        uid
    )
    .execute(pool)
    .await?;

    Ok(())
}
