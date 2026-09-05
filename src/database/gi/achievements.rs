//! GI achievement catalog persistence, localized reads, and separate patch/CSV metadata contracts.

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

/// Upsert catalog rows from aligned parallel slices; each index must describe the same record.
/// Database errors propagate to the catalog refresh caller.
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

/// Fetch localized catalog rows in SQL display order, including hidden impossible members.
/// Callers apply visibility only after building relationships from the complete result.
pub async fn get_all(language: Language, pool: &PgPool) -> Result<Vec<DbAchievement>> {
    let language = language.to_string();

    Ok(
        sqlx::query_file_as!(DbAchievement, "sql/gi/achievements/get_all.sql", language)
            .fetch_all(pool)
            .await?,
    )
}

/// Fetch one localized catalog row, returning None for an unknown ID; visibility remains caller-owned.
pub async fn get_one_by_id(
    id: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Option<DbAchievement>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbAchievement,
        "sql/gi/achievements/get_one_by_id.sql",
        id,
        language,
    )
    .fetch_optional(pool)
    .await?)
}

/// List other members of the supplied set without visibility filtering or an ordering guarantee.
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

/// List achievement IDs excluding entries that are both hidden and impossible.
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
    pub set: Option<i32>,
}

/// Patch metadata for one ID; omitted/null values preserve stored columns through COALESCE.
pub async fn update_achievement_by_id(
    achievement: &DbUpdateAchievement,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/achievements/update_achievement_by_id.sql",
        achievement.id,
        achievement.version,
        achievement.comment,
        achievement.reference,
        achievement.difficulty,
        achievement.video,
        achievement.gacha,
        achievement.timegated,
        achievement.missable,
        achievement.impossible,
        achievement.set
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub struct DbImportAchievement {
    pub id: i32,
    pub version: Option<String>,
    pub difficulty: Option<String>,
    pub comment: Option<String>,
    pub impossible: bool,
    pub timegated: Option<String>,
}

/// Replace the CSV-owned metadata columns in one statement; empty CSV cells clear nullable fields.
/// This replacement contract differs deliberately from the metadata PATCH-style update.
pub async fn import_metadata(achievement: &DbImportAchievement, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE gi_achievements SET version = $2, difficulty = $3, comment = $4, impossible = $5, timegated = $6 WHERE id = $1", achievement.id, achievement.version, achievement.difficulty, achievement.comment, achievement.impossible, achievement.timegated).execute(pool).await?;
    Ok(())
}
