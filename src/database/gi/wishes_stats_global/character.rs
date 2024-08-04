use anyhow::Result;
use sqlx::PgPool;

pub struct DbWishesStatGlobalCharacter {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_4_percentile: f64,
    pub luck_5_percentile: f64,
}

pub async fn set(stat: &DbWishesStatGlobalCharacter, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/wishes_stats_global/character/set.sql",
        stat.uid,
        stat.count_percentile,
        stat.luck_4_percentile,
        stat.luck_5_percentile,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWishesStatGlobalCharacter>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatGlobalCharacter,
        "sql/gi/wishes_stats_global/character/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}
