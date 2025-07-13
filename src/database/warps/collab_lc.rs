use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::Language;

use super::{DbWarp, DbWarpInfo, SetAll};

pub async fn set_all(set_all: &SetAll, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!(
        "sql/warps/collab_lc/set_all.sql",
        &set_all.id,
        &set_all.uid,
        &set_all.character as &[Option<i32>],
        &set_all.light_cone as &[Option<i32>],
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
) -> anyhow::Result<Vec<DbWarp>> {
    let language = language.to_string();

    Ok(
        sqlx::query_file_as!(DbWarp, "sql/warps/collab_lc/get_by_uid.sql", uid, language)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_infos_by_uid(uid: i32, pool: &PgPool) -> anyhow::Result<Vec<DbWarpInfo>> {
    Ok(
        sqlx::query_file_as!(DbWarpInfo, "sql/warps/collab_lc/get_infos.sql", uid,)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_count_by_uid(uid: i32, pool: &PgPool) -> anyhow::Result<i64> {
    Ok(sqlx::query_file!("sql/warps/collab_lc/get_count_by_uid.sql", uid)
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}

pub async fn get_earliest_timestamp_by_uid(
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    Ok(
        sqlx::query_file!("sql/warps/collab_lc/get_earliest_timestamp_by_uid.sql", uid)
            .fetch_one(pool)
            .await?
            .min,
    )
}

pub async fn get_latest_timestamp_by_uid(
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    Ok(
        sqlx::query_file!("sql/warps/collab_lc/get_latest_timestamp_by_uid.sql", uid)
            .fetch_one(pool)
            .await?
            .max,
    )
}

pub async fn delete_unofficial(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!("sql/warps/collab_lc/delete_unofficial.sql", uid)
        .execute(pool)
        .await?;

    Ok(())
}
