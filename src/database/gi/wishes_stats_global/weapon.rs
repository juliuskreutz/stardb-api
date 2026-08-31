use anyhow::Result;
use sqlx::PgPool;

use super::DbWishesStatGlobal;

/// Upserts a batch of global percentile rows in one round trip.
pub async fn set_bulk(stats: &[DbWishesStatGlobal], pool: &PgPool) -> Result<()> {
    if stats.is_empty() {
        return Ok(());
    }
    let uids: Vec<_> = stats.iter().map(|stat| stat.uid).collect();
    let count: Vec<_> = stats.iter().map(|stat| stat.count_percentile).collect();
    let luck_4: Vec<_> = stats.iter().map(|stat| stat.luck_4_percentile).collect();
    let luck_5: Vec<_> = stats.iter().map(|stat| stat.luck_5_percentile).collect();

    sqlx::query_file!(
        "sql/gi/wishes_stats_global/weapon/set_bulk.sql",
        &uids,
        &count,
        &luck_4,
        &luck_5,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWishesStatGlobal>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatGlobal,
        "sql/gi/wishes_stats_global/weapon/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}
