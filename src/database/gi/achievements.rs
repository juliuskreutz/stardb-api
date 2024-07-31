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
    pub primogems: i32,
    pub hidden: bool,
    pub priority: i32,
    pub version: Option<String>,
    pub comment: Option<String>,
    pub reference: Option<String>,
    pub difficulty: Option<String>,
    pub video: Option<String>,
    pub gacha: bool,
    pub timegated: Option<String>,
    pub missable: bool,
    pub impossible: bool,
    pub set: Option<i32>,
    pub percent: f64,
}

pub async fn set_all(
    id: &[i32],
    series: &[i32],
    primogems: &[i32],
    hidden: &[bool],
    priority: &[i32],
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/achievements/set_all.sql",
        id,
        series,
        primogems,
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
        sqlx::query_file_as!(DbAchievement, "sql/gi/achievements/get_all.sql", language)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_one_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbAchievement> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbAchievement,
        "sql/gi/achievements/get_one_by_id.sql",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_all_related_ids(id: i32, set: i32, pool: &PgPool) -> Result<Vec<i32>> {
    Ok(
        sqlx::query_file!("sql/gi/achievements/get_all_related_ids.sql", id, set)
            .fetch_all(pool)
            .await?
            .iter_mut()
            .map(|id| id.id)
            .collect(),
    )
}

pub async fn get_all_ids_shown(pool: &PgPool) -> Result<Vec<i32>> {
    Ok(
        sqlx::query_file!("sql/gi/achievements/get_all_ids_shown.sql")
            .fetch_all(pool)
            .await?
            .iter()
            .map(|r| r.id)
            .collect(),
    )
}

pub async fn update_version_by_id(id: i32, version: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/update_version_by_id.sql", id, version)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_comment_by_id(id: i32, comment: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/update_comment_by_id.sql", id, comment)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_reference_by_id(id: i32, reference: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/achievements/update_reference_by_id.sql",
        id,
        reference
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_difficulty_by_id(id: i32, difficulty: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/achievements/update_difficulty_by_id.sql",
        id,
        difficulty,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_gacha_by_id(id: i32, gacha: bool, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/update_gacha_by_id.sql", id, gacha)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_timegated_by_id(id: i32, timegated: Option<&str>, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/achievements/update_timegated_by_id.sql",
        id,
        timegated
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_missable_by_id(id: i32, missable: bool, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/achievements/update_missable_by_id.sql",
        id,
        missable
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_impossible_by_id(id: i32, impossible: bool, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/achievements/update_impossible_by_id.sql",
        id,
        impossible,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_video_by_id(id: i32, video: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/update_video_by_id.sql", id, video)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_version_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/delete_version_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_comment_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/delete_comment_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_reference_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/delete_reference_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_difficulty_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/delete_difficulty_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_video_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievements/delete_video_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}
