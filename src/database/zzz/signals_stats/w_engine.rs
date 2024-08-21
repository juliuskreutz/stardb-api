use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatWEngine {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

pub async fn set(stat: &DbSignalsStatWEngine, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats/w_engine/set.sql",
        stat.uid,
        stat.luck_a,
        stat.luck_s,
        stat.win_rate,
        stat.win_streak,
        stat.loss_streak,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatWEngine>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatWEngine,
        "sql/zzz/signals_stats/w_engine/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbSignalsStatWEngine>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatWEngine,
        "sql/zzz/signals_stats/w_engine/get_all.sql"
    )
    .fetch_all(pool)
    .await?)
}
