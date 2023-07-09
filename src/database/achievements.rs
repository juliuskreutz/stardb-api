use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbAchievement {
    pub id: i64,
    pub series: String,
    pub title: String,
    pub description: String,
    pub comment: Option<String>,
    pub reference: Option<String>,
    pub difficulty: Option<String>,
}

impl AsRef<DbAchievement> for DbAchievement {
    fn as_ref(&self) -> &DbAchievement {
        self
    }
}

pub async fn set_achievement(achievement: &DbAchievement, pool: &PgPool) -> Result<DbAchievement> {
    Ok(sqlx::query_as!(
        DbAchievement,
        "
        INSERT INTO
            achievements(id, series, title, description, comment, reference, difficulty)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT
            (id)
        DO UPDATE SET
            series = EXCLUDED.series,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            comment = COALESCE(EXCLUDED.comment, achievements.comment),
            reference = COALESCE(EXCLUDED.reference, achievements.reference),
            difficulty = COALESCE(EXCLUDED.difficulty, achievements.difficulty)
        RETURNING
            *
        ",
        achievement.id,
        achievement.series,
        achievement.title,
        achievement.description,
        achievement.comment,
        achievement.reference,
        achievement.difficulty,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_achievements(pool: &PgPool) -> Result<Vec<DbAchievement>> {
    Ok(
        sqlx::query_as!(DbAchievement, "SELECT * FROM achievements ORDER BY id")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_achievement_by_id(id: i64, pool: &PgPool) -> Result<DbAchievement> {
    Ok(sqlx::query_as!(
        DbAchievement,
        "SELECT * FROM achievements WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await?)
}
