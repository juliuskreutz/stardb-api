use anyhow::Result;
use sqlx::PgPool;

pub struct DbWarpsStatStandard {
    pub uid: i32,
    pub count_rank: i32,
    pub luck_4: f64,
    pub luck_4_rank: i32,
    pub luck_5: f64,
    pub luck_5_rank: i32,
}

#[derive(Default)]
pub struct SetAll {
    pub uid: Vec<i32>,
    pub count: Vec<i32>,
    pub count_rank: Vec<i32>,
    pub luck_4: Vec<f64>,
    pub luck_4_rank: Vec<i32>,
    pub luck_5: Vec<f64>,
    pub luck_5_rank: Vec<i32>,
}

pub async fn set_all(set_all: &SetAll, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats_standard/set_all.sql",
        &set_all.uid,
        &set_all.count_rank,
        &set_all.luck_4,
        &set_all.luck_4_rank,
        &set_all.luck_5,
        &set_all.luck_5_rank
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStatStandard>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatStandard,
        "sql/warps_stats_standard/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn count(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_file!("sql/warps_stats_standard/count.sql")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}
