pub mod departure;
pub mod lc;
pub mod special;
pub mod standard;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::Language;

pub struct DbWarp {
    pub id: i64,
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
}

pub struct DbWarpInfo {
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub rarity: Option<i32>,
}

#[derive(Default)]
pub struct SetAll {
    pub id: Vec<i64>,
    pub uid: Vec<i32>,
    pub character: Vec<Option<i32>>,
    pub light_cone: Vec<Option<i32>>,
    pub timestamp: Vec<DateTime<Utc>>,
    pub official: Vec<bool>,
}

pub async fn get_uids(pool: &PgPool) -> anyhow::Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/warps/get_uids.sql")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| r.uid)
        .collect())
}

pub async fn count_uids(pool: &PgPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_file!("sql/warps/count_uids.sql")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}

pub struct DbCharacterCount {
    pub id: i32,
    pub rarity: i32,
    pub name: String,
    pub path: String,
    pub element: String,
    pub path_id: String,
    pub element_id: String,
    pub count: Option<i64>,
}

pub async fn get_characters_count_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbCharacterCount>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbCharacterCount,
        "sql/warps/get_characters_count_by_uid.sql",
        uid,
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub struct DbLightConeCount {
    pub id: i32,
    pub rarity: i32,
    pub name: String,
    pub path: String,
    pub path_id: String,
    pub count: Option<i64>,
}

pub async fn get_light_cones_count_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbLightConeCount>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbLightConeCount,
        "sql/warps/get_light_cones_count_by_uid.sql",
        uid,
        language,
    )
    .fetch_all(pool)
    .await?)
}
