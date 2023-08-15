use anyhow::Result;
use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DbSession {
    pub uuid: Uuid,
    pub value: Value,
    pub expiry: NaiveDateTime,
}

pub async fn set_session(session: &DbSession, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            sessions(uuid, value, expiry)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (uuid)
        DO UPDATE SET
            value = EXCLUDED.value,
            expiry = EXCLUDED.expiry
        ",
        session.uuid,
        session.value,
        session.expiry,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_session_by_uuid(uuid: Uuid, pool: &PgPool) -> Result<DbSession> {
    Ok(sqlx::query_as!(
        DbSession,
        "
        SELECT
            *
        FROM
            sessions
        WHERE
            uuid = $1
        AND
            expiry > NOW()
        ",
        uuid,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn update_session_expiry(uuid: Uuid, expiry: NaiveDateTime, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        UPDATE
            sessions
        SET
            expiry = $2
        WHERE
            uuid = $1
        ",
        uuid,
        expiry,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_session_by_uuid(uuid: Uuid, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        DELETE FROM
            sessions
        WHERE
            uuid = $1
        ",
        uuid,
    )
    .execute(pool)
    .await?;

    Ok(())
}
