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
// actix-session stores each value as JSON text. Decode the entire string so
// quotes, backslashes and Unicode remain raw username characters in PostgreSQL.
fn decode_username(state: &SessionState) -> anyhow::Result<String> {
    Ok(serde_json::from_str(state.get("username").ok_or_else(
        || anyhow::anyhow!("missing session username"),
    )?)?)
}
// Rebuild the same JSON envelope on load; manual quote wrapping is not symmetric.
fn encode_username(username: &str) -> anyhow::Result<SessionState> {
    Ok(HashMap::from([(
        "username".into(),
        serde_json::to_string(username)?,
    )]))
}
fn expiry(ttl: &Duration) -> anyhow::Result<DateTime<Utc>> {
    chrono::Duration::try_seconds(ttl.whole_seconds())
        .and_then(|ttl| Utc::now().checked_add_signed(ttl))
        .ok_or_else(|| anyhow::anyhow!("invalid session expiry"))
}
fn parse_key(key: &SessionKey) -> anyhow::Result<Uuid> {
    Ok(Uuid::parse_str(key.as_ref())?)
}
impl PgSessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
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
    async fn update_ttl(&self, key: &SessionKey, ttl: &Duration) -> anyhow::Result<()> {
        database::sessions::update_expiry_by_uuid(parse_key(key)?, expiry(ttl)?, &self.pool).await
    }
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
