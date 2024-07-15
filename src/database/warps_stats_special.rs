use anyhow::Result;
use sqlx::PgPool;

pub struct DbWarpsStatSpecial {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_4: f64,
    pub luck_4_percentile: f64,
    pub luck_5: f64,
    pub luck_5_percentile: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

pub struct SetData {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

pub async fn update_percentiles_by_uid(
    uid: i32,
    count_percentile: f64,
    luck_4_percentile: f64,
    luck_5_percentile: f64,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats_special/update_percentiles_by_uid.sql",
        uid,
        count_percentile,
        luck_4_percentile,
        luck_5_percentile
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_data(set_data: &SetData, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats_special/set_data.sql",
        set_data.uid,
        set_data.luck_4,
        set_data.luck_5,
        set_data.win_rate,
        set_data.win_streak,
        set_data.loss_streak,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStatSpecial>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatSpecial,
        "sql/warps_stats_special/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn count(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_file!("sql/warps_stats_special/count.sql")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}
