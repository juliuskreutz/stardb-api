use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

pub struct DbSignalsStatWEngineReverberation {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

pub async fn set<'e, E>(stat: &DbSignalsStatWEngineReverberation, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/zzz/signals_stats/w_engine_reverberation/set.sql",
        stat.uid,
        stat.luck_a,
        stat.luck_s,
        stat.win_rate,
        stat.win_streak,
        stat.loss_streak,
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_by_uid(
    uid: i32,
    pool: &PgPool,
) -> Result<Option<DbSignalsStatWEngineReverberation>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatWEngineReverberation,
        "sql/zzz/signals_stats/w_engine_reverberation/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbSignalsStatWEngineReverberation>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatWEngineReverberation,
        "sql/zzz/signals_stats/w_engine_reverberation/get_all.sql"
    )
    .fetch_all(pool)
    .await?)
}
