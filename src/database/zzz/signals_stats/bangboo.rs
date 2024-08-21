use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatBangboo {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
}

pub async fn set(stat: &DbSignalsStatBangboo, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats/bangboo/set.sql",
        stat.uid,
        stat.luck_a,
        stat.luck_s,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatBangboo>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatBangboo,
        "sql/zzz/signals_stats/bangboo/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbSignalsStatBangboo>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatBangboo,
        "sql/zzz/signals_stats/bangboo/get_all.sql"
    )
    .fetch_all(pool)
    .await?)
}
