pub mod beginner;
pub mod character;
pub mod chronicled;
pub mod standard;
pub mod weapon;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct DbWish {
    pub id: i64,
    pub character: Option<i32>,
    pub weapon: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}

pub struct DbWishInfo {
    pub character: Option<i32>,
    pub weapon: Option<i32>,
    pub rarity: Option<i32>,
}

#[derive(Default)]
pub struct SetAll {
    pub id: Vec<i64>,
    pub uid: Vec<i32>,
    pub character: Vec<Option<i32>>,
    pub weapon: Vec<Option<i32>>,
    pub timestamp: Vec<DateTime<Utc>>,
    pub official: Vec<bool>,
}

pub async fn get_uids(pool: &PgPool) -> anyhow::Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/gi/wishes/get_uids.sql")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| r.uid)
        .collect())
}
