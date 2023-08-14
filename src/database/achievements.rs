use sqlx::PgPool;

use crate::Result;

#[derive(Clone)]
pub struct DbAchievement {
    pub id: i64,
    pub series: i32,
    pub series_tag: String,
    pub series_name: String,
    pub tag: String,
    pub name: String,
    pub description: String,
    pub jades: i32,
    pub hidden: bool,
    pub priority: i32,
    pub version: Option<String>,
    pub comment: Option<String>,
    pub reference: Option<String>,
    pub difficulty: Option<String>,
    pub gacha: bool,
    pub set: Option<i32>,
    pub percent: Option<f64>,
}

pub async fn set_achievement(achievement: &DbAchievement, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievements(id, series, tag, jades, hidden, priority)
        VALUES
            ($1, $2, $3, $4, $5, $6)
        ON CONFLICT
            (id)
        DO UPDATE SET
        series = EXCLUDED.series,
            tag = EXCLUDED.tag,
            jades = EXCLUDED.jades,
            hidden = EXCLUDED.hidden,
            priority = EXCLUDED.priority
        ",
        achievement.id,
        achievement.series,
        achievement.tag,
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
            series.tag series_tag,
            series_text.name series_name
        FROM
            achievements
        NATURAL LEFT JOIN
            (SELECT id, (COUNT(*)::float) / (SELECT COUNT(DISTINCT username) from completed) percent FROM completed GROUP BY id) percents
        INNER JOIN
            achievements_text
        ON
            achievements.id = achievements_text.id AND achievements_text.language = $1
        INNER JOIN
            series
        ON
            series = series.id
        INNER JOIN
            series_text
        ON
            series = series_text.id AND series_text.language = $1
        ORDER BY
            series.priority DESC, series, priority DESC
        ",
        language
    )
    .fetch_all(pool)
    .await?)
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
            series.tag series_tag,
            series_text.name series_name
        FROM
            achievements
        NATURAL LEFT JOIN
            (SELECT id, (COUNT(*)::float) / (SELECT COUNT(DISTINCT username) from completed) percent FROM completed GROUP BY id) percents
        INNER JOIN
            achievements_text
        ON
            achievements.id = achievements_text.id AND achievements_text.language = $2
        INNER JOIN
            series
        ON
            series = series.id
        INNER JOIN
            series_text
        ON
            series = series.id AND series_text.language = $2
        WHERE
            achievements.id = $1
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
