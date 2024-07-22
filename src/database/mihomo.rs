use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Default)]
pub struct DbMihomo {
    pub uid: i32,
    pub region: String,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: DateTime<Utc>,
}

pub async fn set(mihomo: &DbMihomo, pool: &PgPool) -> Result<DbMihomo> {
    sqlx::query_file!(
        "sql/mihomo/set.sql",
        mihomo.uid,
        mihomo.region,
        mihomo.name,
        mihomo.level,
        mihomo.signature,
        mihomo.avatar_icon,
        mihomo.achievement_count,
        mihomo.updated_at,
    )
    .execute(pool)
    .await?;

    get_one_by_uid(mihomo.uid, pool).await
}

pub async fn get_one_by_uid(uid: i32, pool: &PgPool) -> Result<DbMihomo> {
    Ok(
        sqlx::query_file_as!(DbMihomo, "sql/mihomo/get_one_by_uid.sql", uid)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn exists(uid: i32, pool: &PgPool) -> Result<bool> {
    Ok(sqlx::query_file!("sql/mihomo/exists.sql", uid)
        .fetch_optional(pool)
        .await?
        .is_some())
}

pub async fn get_all_uids(pool: &PgPool) -> Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/mihomo/get_all_uids.sql")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|r| r.uid)
        .collect())
}
