use std::{collections::HashMap, str::FromStr};

use actix_session::storage::{LoadError, SaveError, SessionKey, SessionStore, UpdateError};
use actix_web::cookie::time::Duration;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PgSessionStore {
    pool: PgPool,
}

impl PgSessionStore {
    pub fn new(pool: PgPool) -> Self {
        PgSessionStore { pool }
    }
}

type SessionState = HashMap<String, String>;

#[async_trait::async_trait(?Send)]
impl SessionStore for PgSessionStore {
    async fn load(&self, session_key: &SessionKey) -> Result<Option<SessionState>, LoadError> {
        let uuid = Uuid::from_str(session_key.as_ref())
            .map_err(anyhow::Error::new)
            .map_err(LoadError::Other)?;

        let db_session = stardb_database::get_session_by_uuid(uuid, &self.pool)
            .await
            .map_err(LoadError::Deserialization)?;

        serde_json::from_value(db_session.value)
            .map(Some)
            .map_err(anyhow::Error::new)
            .map_err(LoadError::Deserialization)
    }

    async fn save(
        &self,
        session_state: SessionState,
        ttl: &Duration,
    ) -> Result<SessionKey, SaveError> {
        let uuid = Uuid::new_v4();

        let value = serde_json::to_value(&session_state)
            .map_err(anyhow::Error::new)
            .map_err(SaveError::Serialization)?;

        let expiry = (Utc::now() + chrono::Duration::seconds(ttl.whole_seconds())).naive_utc();

        let db_session = stardb_database::DbSession {
            uuid,
            value,
            expiry,
        };

        stardb_database::set_session(&db_session, &self.pool)
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

        let value = serde_json::to_value(&session_state)
            .map_err(anyhow::Error::new)
            .map_err(UpdateError::Serialization)?;

        let expiry = (Utc::now() + chrono::Duration::seconds(ttl.whole_seconds())).naive_utc();

        let db_session = stardb_database::DbSession {
            uuid,
            value,
            expiry,
        };

        stardb_database::set_session(&db_session, &self.pool)
            .await
            .map_err(UpdateError::Other)?;

        Ok(session_key)
    }

    async fn update_ttl(&self, session_key: &SessionKey, ttl: &Duration) -> anyhow::Result<()> {
        let uuid = Uuid::from_str(session_key.as_ref())
            .map_err(anyhow::Error::new)
            .map_err(UpdateError::Other)?;

        let expiry = (Utc::now() + chrono::Duration::seconds(ttl.whole_seconds())).naive_utc();

        stardb_database::update_session_expiry(uuid, expiry, &self.pool).await
    }

    async fn delete(&self, session_key: &SessionKey) -> anyhow::Result<()> {
        let uuid = Uuid::from_str(session_key.as_ref())?;

        stardb_database::delete_session_by_uuid(uuid, &self.pool).await
    }
}
