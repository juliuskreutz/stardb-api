use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

#[derive(Default)]
pub struct DbSubmissionHeal {
    pub uuid: Uuid,
    pub uid: i64,
    pub heal: i32,
    pub video: String,
    pub created_at: NaiveDateTime,
}

pub async fn set_submission_heal(submission: &DbSubmissionHeal, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            submissions_heal(uid, heal, video)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            heal = EXCLUDED.heal,
            video = EXCLUDED.video,
            created_at = now()
        ",
        submission.uid,
        submission.heal,
        submission.video,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_submissions_heal(
    uid: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbSubmissionHeal>> {
    Ok(sqlx::query_as!(
        DbSubmissionHeal,
        "SELECT * FROM submissions_heal WHERE $1::int8 IS NULL OR $1 = uid ORDER BY created_at",
        uid,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_submission_heal_by_uuid(uuid: Uuid, pool: &PgPool) -> Result<DbSubmissionHeal> {
    Ok(sqlx::query_as!(
        DbSubmissionHeal,
        "SELECT * FROM submissions_heal WHERE uuid = $1",
        uuid
    )
    .fetch_one(pool)
    .await?)
}

pub async fn delete_submission_heal(uuid: Uuid, pool: &PgPool) -> Result<()> {
    sqlx::query_as!(
        DbSubmissionDamage,
        "DELETE FROM submissions_heal WHERE uuid = $1",
        uuid,
    )
    .fetch_all(pool)
    .await?;

    Ok(())
}
