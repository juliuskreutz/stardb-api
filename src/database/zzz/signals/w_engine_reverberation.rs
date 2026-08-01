//! Pull persistence and read models for W-Engine Reverberation signals.

use chrono::{DateTime, Utc};
use sqlx::{Executor, PgConnection, PgPool, Postgres};

use crate::Language;

use super::{DbSignal, DbSignalInfo, SetAll};

/// Bulk-upserts normalized signals and returns changed row count.
pub async fn set_all(set_all: &SetAll, connection: &mut PgConnection) -> anyhow::Result<u64> {
    let result = sqlx::query_file!(
        "sql/zzz/signals/w_engine_reverberation/set_all.sql",
        &set_all.id,
        &set_all.uid,
        &set_all.character as &[Option<i32>],
        &set_all.w_engine as &[Option<i32>],
        &set_all.timestamp as &[DateTime<Utc>],
        &set_all.official,
    )
    .execute(&mut *connection)
    .await?;

    Ok(result.rows_affected())
}

/// Returns the oldest stored signal timestamp used by import cutoff policy.
pub async fn get_earliest_timestamp_by_uid(
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    Ok(sqlx::query_file!(
        "sql/zzz/signals/w_engine_reverberation/get_earliest_timestamp_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?
    .timestamp)
}

/// Returns localized tracker rows for one UID.
pub async fn get_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> anyhow::Result<Vec<DbSignal>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbSignal,
        "sql/zzz/signals/w_engine_reverberation/get_by_uid.sql",
        uid,
        language
    )
    .fetch_all(pool)
    .await?)
}

/// Returns calculation-only rows on a caller-owned executor.
pub async fn get_infos_by_uid<'e, E>(uid: i32, executor: E) -> anyhow::Result<Vec<DbSignalInfo>>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query_file_as!(
        DbSignalInfo,
        "sql/zzz/signals/w_engine_reverberation/get_infos.sql",
        uid
    )
    .fetch_all(executor)
    .await?)
}

/// Counts one UID's signals for global-stat eligibility.
pub async fn get_count_by_uid(uid: i32, pool: &PgPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_file!(
        "sql/zzz/signals/w_engine_reverberation/get_count_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap())
}

/// Deletes every signal stored for one UID.
pub async fn delete_all(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!("sql/zzz/signals/w_engine_reverberation/delete_all.sql", uid)
        .execute(pool)
        .await?;

    Ok(())
}

/// Deletes only non-official signals for one UID.
pub async fn delete_unofficial(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals/w_engine_reverberation/delete_unofficial.sql",
        uid
    )
    .execute(pool)
    .await?;

    Ok(())
}
