pub mod bangboo;
pub mod special;
pub mod standard;
pub mod w_engine;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct DbSignal {
    pub id: i64,
    pub uid: i32,
    pub character: Option<i32>,
    pub bangboo: Option<i32>,
    pub w_engine: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}

pub struct DbSignalInfo {
    pub uid: i32,
    pub character: Option<i32>,
    pub bangboo: Option<i32>,
    pub w_engine: Option<i32>,
    pub rarity: Option<i32>,
}

#[derive(Default)]
pub struct SetAll {
    pub id: Vec<i64>,
    pub uid: Vec<i32>,
    pub character: Vec<Option<i32>>,
    pub bangboo: Vec<Option<i32>>,
    pub w_engine: Vec<Option<i32>>,
    pub timestamp: Vec<DateTime<Utc>>,
    pub official: Vec<bool>,
}

pub async fn get_uids(pool: &PgPool) -> anyhow::Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/zzz/signals/get_uids.sql")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| r.uid)
        .collect())
}

pub async fn count_uids(pool: &PgPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_file!("sql/zzz/signals/count_uids.sql")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}
