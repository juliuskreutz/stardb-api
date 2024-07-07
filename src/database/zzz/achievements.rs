use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

#[derive(Clone)]
pub struct DbAchievement {
    pub id: i32,
    pub series: i32,
    pub series_name: String,
    pub name: String,
    pub description: String,
    pub polychromes: i32,
    pub hidden: bool,
    pub priority: i32,
    pub version: Option<String>,
    pub comment: Option<String>,
    pub reference: Option<String>,
    pub difficulty: Option<String>,
    pub video: Option<String>,
    pub gacha: bool,
    pub impossible: bool,
    pub set: Option<i32>,
    pub percent: f64,
}

pub async fn set_all(
    id: &[i32],
    series: &[i32],
    polychromes: &[i32],
    hidden: &[bool],
    priority: &[i32],
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/achievements/set_all.sql",
        id,
        series,
        polychromes,
        hidden,
        priority,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(language: Language, pool: &PgPool) -> Result<Vec<DbAchievement>> {
    let language = language.to_string();

    Ok(
        sqlx::query_file_as!(DbAchievement, "sql/zzz/achievements/get_all.sql", language)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_all_related_ids(id: i32, set: i32, pool: &PgPool) -> Result<Vec<i32>> {
    Ok(
        sqlx::query_file!("sql/zzz/achievements/get_all_related_ids.sql", id, set)
            .fetch_all(pool)
            .await?
            .iter_mut()
            .map(|id| id.id)
            .collect(),
    )
}
