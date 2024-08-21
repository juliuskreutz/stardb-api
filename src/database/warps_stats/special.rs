use anyhow::Result;
use sqlx::PgPool;

pub struct DbWarpsStatSpecial {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

pub async fn set(stat: &DbWarpsStatSpecial, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats/special/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
        stat.win_rate,
        stat.win_streak,
        stat.loss_streak,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWarpsStatSpecial>> {
    Ok(
        sqlx::query_file_as!(DbWarpsStatSpecial, "sql/warps_stats/special/get_all.sql")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStatSpecial>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatSpecial,
        "sql/warps_stats/special/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}
