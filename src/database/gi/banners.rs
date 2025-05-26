use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct DbBanner {
    pub id: i32,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub character: Option<i32>,
    pub weapon: Option<i32>,
}

pub async fn set(banner: &DbBanner, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/banners/set.sql",
        banner.id,
        banner.name,
        banner.start,
        banner.end,
        banner.character,
        banner.weapon,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbBanner>> {
    Ok(sqlx::query_file_as!(DbBanner, "sql/gi/banners/get_all.sql")
        .fetch_all(pool)
        .await?)
}

pub async fn get_by_id(id: i32, pool: &PgPool) -> Result<DbBanner> {
    Ok(
        sqlx::query_file_as!(DbBanner, "sql/gi/banners/get_by_id.sql", id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn delete_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file_as!(DbBanner, "sql/gi/banners/delete_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}
