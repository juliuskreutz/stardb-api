//! Session UUID persistence and expiry filtering; callers own authentication and TTL policy.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct DbSession {
    pub uuid: Uuid,
    pub username: String,
    pub expiry: DateTime<Utc>,
}

/// Inserts or replaces the raw username and expiry for one session UUID.
pub async fn set(session: &DbSession, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/sessions/set.sql",
        session.uuid,
        session.username,
        session.expiry,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Deletes all but the nine sessions with latest expiry for this username.
pub async fn delete_oldest_by_username(username: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/sessions/delete_oldest_by_username.sql", username)
        .execute(pool)
        .await?;

    Ok(())
}

/// Returns an unexpired session, or None for an unknown or expired UUID.
pub async fn get_one_by_uuid(uuid: Uuid, pool: &PgPool) -> Result<Option<DbSession>> {
    Ok(
        sqlx::query_file_as!(DbSession, "sql/sessions/get_one_by_uuid.sql", uuid)
            .fetch_optional(pool)
            .await?,
    )
}

/// Replaces only the UUID's expiry, succeeding if no matching row exists.
pub async fn update_expiry_by_uuid(uuid: Uuid, expiry: DateTime<Utc>, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/sessions/update_expiry_by_uuid.sql", uuid, expiry)
        .execute(pool)
        .await?;

    Ok(())
}

/// Deletes one UUID without requiring a matching row to exist.
pub async fn delete_by_uuid(uuid: Uuid, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/sessions/delete_by_uuid.sql", uuid)
        .execute(pool)
        .await?;

    Ok(())
}
