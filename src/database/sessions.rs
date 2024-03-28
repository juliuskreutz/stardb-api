use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct DbSession {
    pub uuid: Uuid,
    pub username: String,
    pub expiry: DateTime<Utc>,
}

pub async fn set_session(session: &DbSession, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            sessions(uuid, username, expiry)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (uuid)
        DO UPDATE SET
            username = EXCLUDED.username,
            expiry = EXCLUDED.expiry
        ",
        session.uuid,
        session.username,
        session.expiry,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_oldest_sessions_by_username(username: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        DELETE FROM
            sessions
        WHERE
            uuid
        IN
            (SELECT uuid FROM sessions WHERE username = $1 ORDER BY expiry DESC OFFSET 9)
        ",
        username,
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

pub async fn update_session_expiry(uuid: Uuid, expiry: DateTime<Utc>, pool: &PgPool) -> Result<()> {
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
