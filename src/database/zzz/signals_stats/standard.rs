use anyhow::Result;
use sqlx::PgPool;

pub struct DbSignalsStatStandard {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_a: f64,
    pub luck_a_percentile: f64,
    pub luck_s: f64,
    pub luck_s_percentile: f64,
}

pub struct SetData {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
}

pub async fn update_percentiles_by_uid(
    uid: i32,
    count_percentile: f64,
    luck_a_percentile: f64,
    luck_s_percentile: f64,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats/standard/update_percentiles_by_uid.sql",
        uid,
        count_percentile,
        luck_a_percentile,
        luck_s_percentile
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_data(set_data: &SetData, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats/standard/set_data.sql",
        set_data.uid,
        set_data.luck_a,
        set_data.luck_s,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatStandard>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatStandard,
        "sql/zzz/signals_stats/standard/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn count(pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query_file!("sql/zzz/signals_stats/standard/count.sql")
            .fetch_one(pool)
            .await?
            .count
            .unwrap(),
    )
}
