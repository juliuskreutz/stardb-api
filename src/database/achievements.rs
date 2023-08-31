use anyhow::Result;
use sqlx::PgPool;

#[derive(Clone)]
pub struct DbAchievement {
    pub id: i64,
    pub series: i32,
    pub series_name: String,
    pub name: String,
    pub description: String,
    pub jades: i32,
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

pub async fn set_achievement(achievement: &DbAchievement, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievements(id, series, jades, hidden, priority)
        VALUES
            ($1, $2, $3, $4, $5)
        ON CONFLICT
            (id)
        DO UPDATE SET
            series = EXCLUDED.series,
            jades = EXCLUDED.jades,
            hidden = EXCLUDED.hidden,
            priority = EXCLUDED.priority
        ",
        achievement.id,
        achievement.series,
        achievement.jades,
        achievement.hidden,
        achievement.priority,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_achievements(language: &str, pool: &PgPool) -> Result<Vec<DbAchievement>> {
    Ok(sqlx::query_as!(
        DbAchievement,
        "
        SELECT
            achievements.*,
            achievements_text.name,
            achievements_text.description,
            percent,
            achievement_series_text.name series_name
        FROM
            achievements
        NATURAL INNER JOIN
            achievements_percent
        INNER JOIN
            achievements_text
        ON
            achievements.id = achievements_text.id AND achievements_text.language = $1
        INNER JOIN
            achievement_series
        ON
            series = achievement_series.id
        INNER JOIN
            achievement_series_text
        ON
            series = achievement_series_text.id AND achievement_series_text.language = $1
        WHERE NOT
            (hidden AND impossible)
        ORDER BY
            achievement_series.priority DESC, series, priority DESC, id
        ",
        language
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_achievements_id(pool: &PgPool) -> Result<Vec<i64>> {
    Ok(sqlx::query!("SELECT id FROM achievements")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect())
}

pub async fn get_related(id: i64, set: i32, pool: &PgPool) -> Result<Vec<i64>> {
    Ok(sqlx::query!(
        "
        SELECT
            id
        FROM
            achievements
        WHERE
            id != $1
        AND
            set = $2
        ",
        id,
        set,
    )
    .fetch_all(pool)
    .await?
    .iter_mut()
    .map(|id| id.id)
    .collect())
}

pub async fn get_achievement_by_id(
    id: i64,
    language: &str,
    pool: &PgPool,
) -> Result<DbAchievement> {
    Ok(sqlx::query_as!(
        DbAchievement,
        "SELECT
            achievements.*,
            achievements_text.name,
            achievements_text.description,
            percent,
            achievement_series_text.name series_name
        FROM
            achievements
        NATURAL INNER JOIN
            achievements_percent
        INNER JOIN
            achievements_text
        ON
            achievements.id = achievements_text.id AND achievements_text.language = $2
        INNER JOIN
            achievement_series
        ON
            series = achievement_series.id
        INNER JOIN
            achievement_series_text
        ON
            series = achievement_series_text.id AND achievement_series_text.language = $2
        WHERE
            achievements.id = $1
        AND NOT
            (hidden AND impossible)
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn update_achievement_version(id: i64, version: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET version = $2 WHERE id = $1",
        id,
        version,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_achievement_comment(id: i64, comment: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET comment = $2 WHERE id = $1",
        id,
        comment,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_achievement_reference(id: i64, reference: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET reference = $2 WHERE id = $1",
        id,
        reference,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_achievement_difficulty(id: i64, difficulty: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET difficulty = $2 WHERE id = $1",
        id,
        difficulty,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_achievement_gacha(id: i64, gacha: bool, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET gacha = $2 WHERE id = $1",
        id,
        gacha,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_achievement_impossible(id: i64, impossible: bool, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET impossible = $2 WHERE id = $1",
        id,
        impossible,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_achievement_video(id: i64, video: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET video = $2 WHERE id = $1",
        id,
        video,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_achievement_version(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE achievements SET version = NULL WHERE id = $1", id,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_achievement_comment(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE achievements SET comment = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_achievement_reference(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE achievements SET reference = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_achievement_difficulty(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE achievements SET difficulty = NULL WHERE id = $1",
        id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_achievement_video(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE achievements SET video = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}
