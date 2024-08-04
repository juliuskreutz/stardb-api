use anyhow::Result;
use sqlx::PgPool;

pub struct DbWishesStatGlobalChronicled {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_4_percentile: f64,
    pub luck_5_percentile: f64,
}

pub async fn set(stat: &DbWishesStatGlobalChronicled, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/wishes_stats_global/chronicled/set.sql",
        stat.uid,
        stat.count_percentile,
        stat.luck_4_percentile,
        stat.luck_5_percentile,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWishesStatGlobalChronicled>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatGlobalChronicled,
        "sql/gi/wishes_stats_global/chronicled/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}
