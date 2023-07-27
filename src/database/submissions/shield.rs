use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

#[derive(Default)]
pub struct DbSubmissionShield {
    pub uuid: Uuid,
    pub uid: i64,
    pub shield: i32,
    pub video: String,
    pub created_at: NaiveDateTime,
}

pub async fn set_submission_shield(submission: &DbSubmissionShield, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            submissions_shield(uid, shield, video)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            shield = EXCLUDED.shield,
            video = EXCLUDED.video,
            created_at = now()
        ",
        submission.uid,
        submission.shield,
        submission.video,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_submissions_shield(
    uid: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbSubmissionShield>> {
    Ok(sqlx::query_as!(
        DbSubmissionShield,
        "SELECT * FROM submissions_shield WHERE $1::int8 IS NULL OR $1 = uid ORDER BY created_at",
        uid,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_submission_shield_by_uuid(
    uuid: Uuid,
    pool: &PgPool,
) -> Result<DbSubmissionShield> {
    Ok(sqlx::query_as!(
        DbSubmissionShield,
        "SELECT * FROM submissions_shield WHERE uuid = $1 ORDER BY created_at",
        uuid
    )
    .fetch_one(pool)
    .await?)
}

pub async fn delete_submission_shield(uuid: Uuid, pool: &PgPool) -> Result<()> {
    sqlx::query_as!(
        DbSubmissionDamage,
        "DELETE FROM submissions_shield WHERE uuid = $1",
        uuid,
    )
    .fetch_all(pool)
    .await?;

    Ok(())
}
