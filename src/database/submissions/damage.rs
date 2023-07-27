use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

#[derive(Default)]
pub struct DbSubmissionDamage {
    pub uuid: Uuid,
    pub uid: i64,
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub video: String,
    pub created_at: NaiveDateTime,
}

pub async fn set_submission_damage(submission: &DbSubmissionDamage, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            submissions_damage(uid, character, support, damage, video)
        VALUES
            ($1, $2, $3, $4, $5)
        ON CONFLICT
            (uid, character, support)
        DO UPDATE SET
            uuid = gen_random_uuid(),
            damage = EXCLUDED.damage,
            video = EXCLUDED.video,
            created_at = now()
        ",
        submission.uid,
        submission.character,
        submission.support,
        submission.damage,
        submission.video,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_submissions_damage(
    uid: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbSubmissionDamage>> {
    Ok(sqlx::query_as!(
        DbSubmissionDamage,
        "SELECT * FROM submissions_damage WHERE $1::int8 IS NULL OR $1 = uid ORDER BY created_at",
        uid,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_submission_damage_by_uuid(
    uuid: Uuid,
    pool: &PgPool,
) -> Result<DbSubmissionDamage> {
    Ok(sqlx::query_as!(
        DbSubmissionDamage,
        "SELECT * FROM submissions_damage WHERE uuid = $1",
        uuid
    )
    .fetch_one(pool)
    .await?)
}

pub async fn delete_submission_damage(uuid: Uuid, pool: &PgPool) -> Result<()> {
    sqlx::query_as!(
        DbSubmissionDamage,
        "DELETE FROM submissions_damage WHERE uuid = $1",
        uuid,
    )
    .fetch_all(pool)
    .await?;

    Ok(())
}
