use anyhow::Result;
use sqlx::PgPool;

pub struct DbWarpsStatCollabLc {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

pub async fn set(stat: &DbWarpsStatCollabLc, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats/collab_lc/set.sql",
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

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWarpsStatCollabLc>> {
    Ok(
        sqlx::query_file_as!(DbWarpsStatCollabLc, "sql/warps_stats/collab_lc/get_all.sql")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStatCollabLc>> {
    Ok(
        sqlx::query_file_as!(DbWarpsStatCollabLc, "sql/warps_stats/collab_lc/get_by_uid.sql", uid)
            .fetch_optional(pool)
            .await?,
    )
}
