use std::{collections::HashMap, str::FromStr};

use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database;

pub struct PgSessionStore {
    pool: PgPool,
}

impl PgSessionStore {
    pub fn new(pool: PgPool) -> Self {
        PgSessionStore { pool }
    }
}

type SessionState = HashMap<String, String>;

impl SessionStore for PgSessionStore {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        let uuid = Uuid::from_str(session_key.as_ref())
            .map_err(anyhow::Error::new)
            .map_err(LoadError::Other)?;

        let db_session = database::get_session_by_uuid(uuid, &self.pool)
            .await
            .map_err(LoadError::Deserialization)?;

        let mut session_state = HashMap::new();
        session_state.insert(
            "username".to_string(),
            format!("\"{}\"", db_session.username),
        );
        Ok(Some(session_state))
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        let uuid = Uuid::new_v4();

        let username = &session_state["username"];
        let username = username[1..username.len() - 1].to_string();

        database::delete_oldest_sessions_by_username(&username, &self.pool)
            .await
            .map_err(SaveError::Other)?;

        let expiry =
            (Utc::now() + chrono::Duration::try_seconds(ttl.whole_seconds()).unwrap()).naive_utc();

        let db_session = database::DbSession {
            uuid,
            username,
            expiry,
        };

        database::set_session(&db_session, &self.pool)
            .await
            .map_err(SaveError::Other)?;

        uuid.to_string()
            .try_into()
            .map_err(Into::into)
            .map_err(SaveError::Other)
    }

    async fn update(
        &self,
        session_key: SessionKey,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, UpdateError> {
        let uuid = Uuid::from_str(session_key.as_ref())
            .map_err(anyhow::Error::new)
            .map_err(UpdateError::Other)?;

        let username = &session_state["username"];
        let username = username[1..username.len() - 1].to_string();

        let expiry =
            (Utc::now() + chrono::Duration::try_seconds(ttl.whole_seconds()).unwrap()).naive_utc();

        let db_session = database::DbSession {
            uuid,
            username,
            expiry,
        };

        database::set_session(&db_session, &self.pool)
            .await
            .map_err(UpdateError::Other)?;

        Ok(session_key)
    }

    async fn update_ttl(&self, session_key: &SessionKey, ttl: &Duration) -> anyhow::Result<()> {
        let uuid = Uuid::from_str(session_key.as_ref())
            .map_err(anyhow::Error::new)
            .map_err(UpdateError::Other)?;

        let expiry =
            (Utc::now() + chrono::Duration::try_seconds(ttl.whole_seconds()).unwrap()).naive_utc();

        database::update_session_expiry(uuid, expiry, &self.pool).await
    }

    async fn delete(&self, session_key: &SessionKey) -> anyhow::Result<()> {
        let uuid = Uuid::from_str(session_key.as_ref())?;

        database::delete_session_by_uuid(uuid, &self.pool).await
    }
}
