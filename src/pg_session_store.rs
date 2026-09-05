//! PostgreSQL session storage for one authenticated username per session.
//! State values use actix-session's JSON envelope; database usernames remain raw strings.

use crate::database;
use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PgSessionStore {
    pool: PgPool,
}
type SessionState = HashMap<String, String>;
/// Decodes actix-session's JSON string into a raw database username, preserving
/// quotes, backslashes and Unicode. Missing keys and non-string or malformed JSON
/// return errors without slicing the serialized value.
fn decode_username(state: &SessionState) -> anyhow::Result<String> {
    Ok(serde_json::from_str(state.get("username").ok_or_else(
        || anyhow::anyhow!("missing session username"),
    )?)?)
}
/// Serializes a raw username into actix-session's JSON envelope on load.
/// JSON serialization preserves the escaping that manual quote wrapping would lose.
fn encode_username(username: &str) -> anyhow::Result<SessionState> {
    Ok(HashMap::from([(
        "username".into(),
        serde_json::to_string(username)?,
    )]))
}
/// Adds whole TTL seconds to the current UTC time, rejecting unrepresentable expiry values.
fn expiry(ttl: &Duration) -> anyhow::Result<DateTime<Utc>> {
    chrono::Duration::try_seconds(ttl.whole_seconds())
        .and_then(|ttl| Utc::now().checked_add_signed(ttl))
        .ok_or_else(|| anyhow::anyhow!("invalid session expiry"))
}
/// Parses an opaque session key as a UUID, returning an error for malformed keys.
fn parse_key(key: &SessionKey) -> anyhow::Result<Uuid> {
    Ok(Uuid::parse_str(key.as_ref())?)
}
impl PgSessionStore {
    /// Creates a session store backed by the supplied PostgreSQL pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    /// Decodes the username and upserts its UUID with a checked expiry.
    /// When pruning, keeps the nine longest-lived prior sessions before insertion;
    /// pruning and upsert are separate statements and either database error propagates.
    async fn persist(
        &self,
        uuid: Uuid,
        state: &SessionState,
        ttl: &Duration,
        prune: bool,
    ) -> anyhow::Result<()> {
        let username = decode_username(state)?;
        let expiry = expiry(ttl)?;
        if prune {
            database::sessions::delete_oldest_by_username(&username, &self.pool).await?;
        }
        database::sessions::set(
            &database::sessions::DbSession {
                uuid,
                username,
                expiry,
            },
            &self.pool,
        )
        .await
    }
}
impl SessionStore for PgSessionStore {
    /// Loads an unexpired session as a JSON state map, returning None for unknown or expired UUIDs.
    /// Malformed keys and database failures map to Other; state encoding failures map to Deserialization.
    async fn load(&self, key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        let uuid = parse_key(key).map_err(LoadError::Other)?;
        let Some(row) = database::sessions::get_one_by_uuid(uuid, &self.pool)
            .await
            .map_err(LoadError::Other)?
        else {
            return Ok(None);
        };
        encode_username(&row.username)
            .map(Some)
            .map_err(LoadError::Deserialization)
    }
    /// Creates a UUID session, pruning older entries before persistence, and returns its key.
    async fn save(&self, state: SessionState, ttl: &Duration) -> Result<SessionKey, SaveError> {
        let uuid = Uuid::new_v4();
        self.persist(uuid, &state, ttl, true)
            .await
            .map_err(SaveError::Other)?;
        uuid.to_string()
            .try_into()
            .map_err(Into::into)
            .map_err(SaveError::Other)
    }
    /// Replaces the state and expiry under the existing UUID without pruning other sessions.
    async fn update(
        &self,
        key: SessionKey,
        state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        let uuid = parse_key(&key).map_err(UpdateError::Other)?;
        self.persist(uuid, &state, ttl, false)
            .await
            .map_err(UpdateError::Other)?;
        Ok(key)
    }
    /// Updates only the session expiry; an absent UUID is a successful no-op.
    async fn update_ttl(&self, key: &SessionKey, ttl: &Duration) -> anyhow::Result<()> {
        database::sessions::update_expiry_by_uuid(parse_key(key)?, expiry(ttl)?, &self.pool).await
    }
    /// Deletes the UUID session, succeeding when its row is already absent.
    async fn delete(&self, key: &SessionKey) -> anyhow::Result<()> {
        database::sessions::delete_by_uuid(parse_key(key)?, &self.pool).await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn username_codec_roundtrips() {
        for name in ["ordinary", "", "a\"b", "a\\b", "雪", "a\nb"] {
            assert_eq!(
                decode_username(&encode_username(name).unwrap()).unwrap(),
                name
            );
        }
        assert!(decode_username(&SessionState::new()).is_err());
        assert!(decode_username(&HashMap::from([("username".into(), "null".into())])).is_err());
    }
}

#[cfg(test)]
mod database_tests {
    use super::*;
    #[actix_web::test]
    async fn unknown_and_expired_sessions_load_as_absent() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let store = PgSessionStore::new(pool.clone());
        let uuid = Uuid::new_v4();
        let key: SessionKey = uuid.to_string().try_into().unwrap();
        assert!(store.load(&key).await.unwrap().is_none());
        let username = format!("s{}", &uuid.simple().to_string()[..20]);
        sqlx::query("INSERT INTO users (username, password) VALUES ($1, '')")
            .bind(&username)
            .execute(&pool)
            .await
            .unwrap();
        database::sessions::set(
            &database::sessions::DbSession {
                uuid,
                username: username.clone(),
                expiry: Utc::now() - chrono::Duration::hours(1),
            },
            &pool,
        )
        .await
        .unwrap();
        assert!(store.load(&key).await.unwrap().is_none());
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&pool)
            .await
            .unwrap();
    }
}
