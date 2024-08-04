use anyhow::Result;
use sqlx::PgPool;

pub struct DbWishesStatChronicled {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

pub async fn set(stat: &DbWishesStatChronicled, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/wishes_stats/chronicled/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<DbWishesStatChronicled> {
    Ok(sqlx::query_file_as!(
        DbWishesStatChronicled,
        "sql/gi/wishes_stats/chronicled/get_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?)
}
