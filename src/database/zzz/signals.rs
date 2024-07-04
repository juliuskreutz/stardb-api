use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{Language, ZzzGachaType};

pub struct DbSignal {
    pub id: i64,
    pub uid: i32,
    pub gacha_type: String,
    pub character: Option<i32>,
    pub w_engine: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}

pub async fn set_all(
    id: &[i64],
    uid: &[i32],
    gacha_type: &[ZzzGachaType],
    character: &[Option<i32>],
    w_engine: &[Option<i32>],
    timestamp: &[DateTime<Utc>],
    official: &[bool],
    pool: &PgPool,
) -> Result<()> {
    let gacha_type = &gacha_type
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    sqlx::query_file!(
        "sql/zzz/signals/set_all.sql",
        id,
        uid,
        gacha_type,
        character as &[Option<i32>],
        w_engine as &[Option<i32>],
        timestamp as &[DateTime<Utc>],
        official,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_uids(pool: &PgPool) -> Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/zzz/signals/get_uids.sql")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|r| r.uid)
        .collect())
}

pub async fn get_by_uid(uid: i32, language: Language, pool: &PgPool) -> Result<Vec<DbSignal>> {
    let language = language.to_string();

    Ok(
        sqlx::query_file_as!(DbSignal, "sql/zzz/signals/get_by_uid.sql", uid, language)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_by_uid_and_gacha_type(
    uid: i32,
    gacha_type: ZzzGachaType,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbSignal>> {
    let gacha_type = gacha_type.to_string();
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbSignal,
        "sql/zzz/signals/get_by_uid_and_gacha_type.sql",
        uid,
        gacha_type,
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_id_and_timestamp(
    id: i64,
    timestamp: DateTime<Utc>,
    language: Language,
    pool: &PgPool,
) -> Result<DbSignal> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbSignal,
        "sql/zzz/signals/get_by_id_and_timestamp.sql",
        id,
        timestamp,
        language,
    )
    .fetch_one(pool)
    .await?)
}
