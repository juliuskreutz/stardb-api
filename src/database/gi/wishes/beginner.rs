use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::Language;

use super::{DbWish, SetAll};

pub async fn set_all(set_all: &SetAll, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!(
        "sql/gi/wishes/beginner/set_all.sql",
        &set_all.id,
        &set_all.uid,
        &set_all.character as &[Option<i32>],
        &set_all.weapon as &[Option<i32>],
        &set_all.timestamp as &[DateTime<Utc>],
        &set_all.official,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> anyhow::Result<Vec<DbWish>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbWish,
        "sql/gi/wishes/beginner/get_by_uid.sql",
        uid,
        language
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_earliest_timestamp_by_uid(
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    Ok(sqlx::query_file!(
        "sql/gi/wishes/beginner/get_earliest_timestamp_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?
    .min)
}

pub async fn get_latest_timestamp_by_uid(
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    Ok(sqlx::query_file!(
        "sql/gi/wishes/beginner/get_latest_timestamp_by_uid.sql",
        uid
    )
    .fetch_one(pool)
    .await?
    .max)
}

pub async fn delete_unofficial(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!("sql/gi/wishes/beginner/delete_unofficial.sql", uid)
        .execute(pool)
        .await?;

    Ok(())
}
