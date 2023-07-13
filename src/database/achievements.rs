use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbAchievement {
    pub id: i64,
    pub series: String,
    pub title: String,
    pub description: String,
    pub hidden: bool,
    pub jades: i32,
    pub comment: Option<String>,
    pub reference: Option<String>,
    pub difficulty: Option<String>,
    pub percent: Option<f64>,
}

impl AsRef<DbAchievement> for DbAchievement {
    fn as_ref(&self) -> &DbAchievement {
        self
    }
}

pub async fn set_achievement(achievement: &DbAchievement, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievements(id, series, title, description, jades, hidden)
        VALUES
            ($1, $2, $3, $4, $5, $6)
        ON CONFLICT
            (id)
        DO UPDATE SET
            series = EXCLUDED.series,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            jades = EXCLUDED.jades,
            hidden = EXCLUDED.hidden
        ",
        achievement.id,
        achievement.series,
        achievement.title,
        achievement.description,
        achievement.jades,
        achievement.hidden,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_achievements(pool: &PgPool) -> Result<Vec<DbAchievement>> {
    Ok(sqlx::query_as!(
        DbAchievement,
        "
        SELECT
            *
        FROM
            achievements
        NATURAL LEFT JOIN
            (SELECT id, (COUNT(*)::float) / (SELECT COUNT(DISTINCT username) from completed) percent FROM completed GROUP BY id) percents
        ORDER BY
            id
        "
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_achievement_by_id(id: i64, pool: &PgPool) -> Result<DbAchievement> {
    Ok(sqlx::query_as!(
        DbAchievement,
        "SELECT
            *
        FROM
            achievements
        NATURAL LEFT JOIN
            (SELECT id, (COUNT(*)::float) / (SELECT COUNT(DISTINCT username) from completed) percent FROM completed GROUP BY id) percents
        WHERE
            id = $1
        ",
        id
    )
    .fetch_one(pool)
    .await?)
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
