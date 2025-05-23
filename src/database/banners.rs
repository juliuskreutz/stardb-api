use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct DbBanner {
    pub id: i32,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
}

pub async fn set(banner: &DbBanner, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/banners/set.sql", banner.id, banner.start, banner.end)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbBanner>> {
    Ok(sqlx::query_file_as!(DbBanner, "sql/banners/get_all.sql")
        .fetch_all(pool)
        .await?)
}

pub async fn get_by_id(id: i32, pool: &PgPool) -> Result<DbBanner> {
    Ok(
        sqlx::query_file_as!(DbBanner, "sql/banners/get_by_id.sql", id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn get_by_character(character: i32, pool: &PgPool) -> Result<Vec<DbBanner>> {
    Ok(
        sqlx::query_file_as!(DbBanner, "sql/banners/get_by_character.sql", character)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_all_light_cone(light_cone: i32, pool: &PgPool) -> Result<Vec<DbBanner>> {
    Ok(
        sqlx::query_file_as!(DbBanner, "sql/banners/get_by_light_cone.sql", light_cone)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn delete_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file_as!(DbBanner, "sql/banners/delete_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}
