use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatSpecial {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

pub async fn set(stat: &DbSignalsStatSpecial, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats/special/set.sql",
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

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatSpecial>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatSpecial,
        "sql/zzz/signals_stats/special/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbSignalsStatSpecial>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatSpecial,
        "sql/zzz/signals_stats/special/get_all.sql"
    )
    .fetch_all(pool)
    .await?)
}
