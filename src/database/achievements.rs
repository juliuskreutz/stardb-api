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
    pub jades: i32,
    pub hidden: bool,
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
    pub percent: Option<f64>,
}

pub async fn select_all(username: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/select_all.sql", username)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_all(
    id: &[i32],
    series: &[i32],
    jades: &[i32],
    hidden: &[bool],
    priority: &[i32],
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/achievements/set_all.sql",
        id,
        series,
        jades,
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
        sqlx::query_file_as!(DbAchievement, "sql/achievements/get_all.sql", language)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_all_ids_shown(pool: &PgPool) -> Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/achievements/get_all_ids_shown.sql")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|r| r.id)
        .collect())
}

pub async fn get_all_related_ids(id: i32, set: i32, pool: &PgPool) -> Result<Vec<i32>> {
    Ok(
        sqlx::query_file!("sql/achievements/get_all_related_ids.sql", id, set)
            .fetch_all(pool)
            .await?
            .iter_mut()
            .map(|id| id.id)
            .collect(),
    )
}

pub async fn get_one_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbAchievement> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbAchievement,
        "sql/achievements/get_one_by_id.sql",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}

pub struct DbUpdateAchievement {
    pub id: i32,
    pub version: Option<String>,
    pub comment: Option<String>,
    pub reference: Option<String>,
    pub difficulty: Option<String>,
    pub video: Option<String>,
    pub gacha: Option<bool>,
    pub timegated: Option<String>,
    pub missable: Option<bool>,
    pub impossible: Option<bool>,
}

pub async fn update_achievement_by_id(
    achievement: &DbUpdateAchievement,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/achievements/update_achievement_by_id.sql",
        achievement.id,
        achievement.version,
        achievement.comment,
        achievement.reference,
        achievement.difficulty,
        achievement.video,
        achievement.gacha,
        achievement.timegated,
        achievement.missable,
        achievement.impossible
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_version_by_id(id: i32, version: Option<&str>, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/update_version_by_id.sql", id, version)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_comment_by_id(id: i32, comment: Option<&str>, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/update_comment_by_id.sql", id, comment)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_reference_by_id(id: i32, reference: Option<&str>, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/update_reference_by_id.sql", id, reference)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_difficulty_by_id(
    id: i32,
    difficulty: Option<&str>,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/achievements/update_difficulty_by_id.sql",
        id,
        difficulty,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_gacha_by_id(id: i32, gacha: bool, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/update_gacha_by_id.sql", id, gacha)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_timegated_by_id(id: i32, timegated: Option<&str>, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/update_timegated_by_id.sql", id, timegated)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_missable_by_id(id: i32, missable: bool, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/update_missable_by_id.sql", id, missable)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_impossible_by_id(id: i32, impossible: bool, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/achievements/update_impossible_by_id.sql",
        id,
        impossible,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_video_by_id(id: i32, video: Option<&str>, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements/update_video_by_id.sql", id, video)
        .execute(pool)
        .await?;

    Ok(())
}
