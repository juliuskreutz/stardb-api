use chrono::{DateTime, Utc};
use sqlx::{Executor, PgConnection, PgPool, Postgres};

use crate::Language;

use super::{DbSignal, DbSignalInfo, SetAll};

pub async fn set_all(set_all: &SetAll, connection: &mut PgConnection) -> anyhow::Result<u64> {
    let result = sqlx::query_file!(
        "sql/zzz/signals/standard/set_all.sql",
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

pub async fn get_earliest_timestamp_by_uid(
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    Ok(sqlx::query_file!(
        "sql/zzz/signals/standard/get_earliest_timestamp_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?
    .timestamp)
}

pub async fn get_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> anyhow::Result<Vec<DbSignal>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbSignal,
        "sql/zzz/signals/standard/get_by_uid.sql",
        uid,
        language
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_infos_by_uid<'e, E>(uid: i32, executor: E) -> anyhow::Result<Vec<DbSignalInfo>>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(
        sqlx::query_file_as!(DbSignalInfo, "sql/zzz/signals/standard/get_infos.sql", uid)
            .fetch_all(executor)
            .await?,
    )
}

pub async fn get_count_by_uid(uid: i32, pool: &PgPool) -> anyhow::Result<i64> {
    Ok(
        sqlx::query_file!("sql/zzz/signals/standard/get_count_by_uid.sql", uid)
            .fetch_one(pool)
            .await?
            .count
            .unwrap(),
    )
}

pub async fn delete_all(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!("sql/zzz/signals/standard/delete_all.sql", uid)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_unofficial(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!("sql/zzz/signals/standard/delete_unofficial.sql", uid)
        .execute(pool)
        .await?;

    Ok(())
}
