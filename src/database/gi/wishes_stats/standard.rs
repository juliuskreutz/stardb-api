use anyhow::Result;
use sqlx::PgPool;

pub struct DbWishesStatStandard {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

pub async fn set(stat: &DbWishesStatStandard, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/wishes_stats/standard/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<DbWishesStatStandard> {
    Ok(sqlx::query_file_as!(
        DbWishesStatStandard,
        "sql/gi/wishes_stats/standard/get_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?)
}
