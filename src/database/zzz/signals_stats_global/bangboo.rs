use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatGlobalBangboo {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_a_percentile: f64,
    pub luck_s_percentile: f64,
}

pub async fn set(stat: &DbSignalsStatGlobalBangboo, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/bangboo/set.sql",
        stat.uid,
        stat.count_percentile,
        stat.luck_a_percentile,
        stat.luck_s_percentile,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatGlobalBangboo>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatGlobalBangboo,
        "sql/zzz/signals_stats_global/bangboo/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_by_uid(uid: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/bangboo/delete_by_uid.sql",
        uid
    )
    .execute(pool)
    .await?;

    Ok(())
}
