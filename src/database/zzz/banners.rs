use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

#[derive(FromRow)]
pub struct DbBanner {
    pub id: i32,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub character: Option<i32>,
    pub w_engine: Option<i32>,
    pub bangboo: Option<i32>,
}

pub async fn set(banner: &DbBanner, pool: &PgPool) -> Result<()> {
    sqlx::query(include_str!("../../../sql/zzz/banners/set.sql"))
        .bind(banner.id)
        .bind(&banner.name)
        .bind(banner.start)
        .bind(banner.end)
        .bind(banner.character)
        .bind(banner.w_engine)
        .bind(banner.bangboo)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbBanner>> {
    Ok(
        sqlx::query_as::<_, DbBanner>(include_str!("../../../sql/zzz/banners/get_all.sql"))
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_by_id(id: i32, pool: &PgPool) -> Result<DbBanner> {
    Ok(
        sqlx::query_as::<_, DbBanner>(include_str!("../../../sql/zzz/banners/get_by_id.sql"))
            .bind(id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn delete_by_id(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query(include_str!("../../../sql/zzz/banners/delete_by_id.sql"))
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
