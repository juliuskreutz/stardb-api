use anyhow::Result;
use sqlx::PgPool;

pub struct DbWarpsStatStandard {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

pub async fn set(stat: &DbWarpsStatStandard, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats/standard/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWarpsStatStandard>> {
    Ok(
        sqlx::query_file_as!(DbWarpsStatStandard, "sql/warps_stats/standard/get_all.sql")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStatStandard>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatStandard,
        "sql/warps_stats/standard/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}
