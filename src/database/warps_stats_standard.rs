use anyhow::Result;
use sqlx::PgPool;

pub struct DbWarpsStatStandard {
    uid: i32,
    count: i32,
    count_rank: i32,
    luck_4: f64,
    luck_4_rank: i32,
    luck_5: f64,
    luck_5_rank: i32,
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
        &set_all.count,
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
