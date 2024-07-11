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

#[derive(Default)]
pub struct SetAll {
    pub uid: Vec<i32>,
    pub count: Vec<i32>,
    pub count_percentile: Vec<f64>,
    pub luck_4: Vec<f64>,
    pub luck_4_percentile: Vec<f64>,
    pub luck_5: Vec<f64>,
    pub luck_5_percentile: Vec<f64>,
    pub win_rate: Vec<f64>,
    pub win_streak: Vec<i32>,
    pub loss_streak: Vec<i32>,
}

pub async fn set_all(set_all: &SetAll, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats_special/set_all.sql",
        &set_all.uid,
        &set_all.count_percentile,
        &set_all.luck_4,
        &set_all.luck_4_percentile,
        &set_all.luck_5,
        &set_all.luck_5_percentile,
        &set_all.win_rate,
        &set_all.win_streak,
        &set_all.loss_streak,
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
