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

#[cfg(test)]
mod username_index_tests {
    use super::*;

    #[sqlx::test]
    async fn sql_performance_username_indexes_and_pruning(pool: PgPool) {
        // Many unrelated users make the username-leading access path meaningful.
        sqlx::query(
            "INSERT INTO users(username, password)
            SELECT 'index-' || i, 'test' FROM generate_series(1, 10000) i",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO mihomo(uid, region, name, level, signature, avatar_icon, achievement_count)
            SELECT i, 'na', 'seed', 1, '', '', 0 FROM generate_series(1, 10000) i")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO gi_profiles(uid, name) SELECT uid, name FROM mihomo")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO zzz_uids(uid) SELECT uid FROM mihomo")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO sessions(uuid, username, expiry)
            SELECT gen_random_uuid(), username, now() + interval '1 day' FROM users",
        )
        .execute(&pool)
        .await
        .unwrap();
        for table in ["connections", "gi_connections", "zzz_connections"] {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "INSERT INTO {table}(uid, username, verified, private)
                 SELECT uid, 'index-' || uid, false, false FROM mihomo"
            )))
            .execute(&pool)
            .await
            .unwrap();
        }
        for table in [
            "sessions",
            "connections",
            "gi_connections",
            "zzz_connections",
        ] {
            // These identifiers are a closed test-only list, never request data.
            sqlx::query(sqlx::AssertSqlSafe(format!("ANALYZE {table}")))
                .execute(&pool)
                .await
                .unwrap();
            let plan: serde_json::Value = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                "EXPLAIN (FORMAT JSON) SELECT * FROM {table} WHERE username = 'index-1'"
            )))
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(
                plan.to_string().contains(&format!("{table}_username_idx")),
                "{table}: {plan}"
            );
        }
        for remaining in 2..=12 {
            set(
                &DbSession {
                    uuid: Uuid::new_v4(),
                    username: "index-1".into(),
                    expiry: Utc::now() + chrono::Duration::days(remaining),
                },
                &pool,
            )
            .await
            .unwrap();
        }
        delete_oldest_by_username("index-1", &pool).await.unwrap();
        let expiries: Vec<DateTime<Utc>> =
            sqlx::query_scalar("SELECT expiry FROM sessions WHERE username = 'index-1'")
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(expiries.len(), 9);
        assert!(expiries
            .iter()
            .all(|e| *e > Utc::now() + chrono::Duration::days(3)));
        let other: i64 =
            sqlx::query_scalar("SELECT count(*) FROM sessions WHERE username != 'index-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(other, 9999);
        assert_eq!(
            crate::database::connections::get_by_username("index-1", &pool)
                .await
                .unwrap()[0]
                .uid,
            1
        );
        assert_eq!(
            crate::database::gi::connections::get_by_username("index-1", &pool)
                .await
                .unwrap()[0]
                .uid,
            1
        );
        assert_eq!(
            crate::database::zzz::connections::get_by_username("index-1", &pool)
                .await
                .unwrap()[0]
                .uid,
            1
        );
    }
}
