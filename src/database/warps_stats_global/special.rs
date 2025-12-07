use anyhow::Result;
use sqlx::PgPool;

use crate::database::warps_stats_global::DbWarpsStatGlobal;

pub async fn set(stat: &DbWarpsStatGlobal, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats_global/special/set.sql",
        stat.uid,
        stat.count_percentile,
        stat.luck_4_percentile,
        stat.luck_5_percentile,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_bulk(stats: &[DbWarpsStatGlobal], pool: &PgPool) -> Result<()> {
    if stats.is_empty() {
        return Ok(());
    }

    let uids: Vec<i32> = stats.iter().map(|s| s.uid).collect();
    let count_percentiles: Vec<f64> = stats.iter().map(|s| s.count_percentile).collect();
    let luck_4_percentiles: Vec<f64> = stats.iter().map(|s| s.luck_4_percentile).collect();
    let luck_5_percentiles: Vec<f64> = stats.iter().map(|s| s.luck_5_percentile).collect();

    sqlx::query_file!(
        "sql/warps_stats_global/special/set_bulk.sql",
        &uids,
        &count_percentiles,
        &luck_4_percentiles,
        &luck_5_percentiles,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStatGlobal>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatGlobal,
        "sql/warps_stats_global/special/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}
