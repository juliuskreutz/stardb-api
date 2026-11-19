use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres};

pub struct DbBanner {
    pub id: i32,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub character: Option<i32>,
    pub character_gacha_type: Option<i32>,
    pub weapon: Option<i32>,
    pub weapon_gacha_type: Option<i32>,
}

pub async fn set<'e, E>(banner: &DbBanner, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/gi/banners/set.sql",
        banner.id,
        banner.name,
        banner.start,
        banner.end,
        banner.character,
        banner.character_gacha_type,
        banner.weapon,
        banner.weapon_gacha_type,
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbBanner>> {
    get_all_with_executor(pool).await
}

pub async fn get_all_with_executor<'e, E>(executor: E) -> Result<Vec<DbBanner>>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(sqlx::query_file_as!(DbBanner, "sql/gi/banners/get_all.sql")
        .fetch_all(executor)
        .await?)
}

pub async fn get_by_id(id: i32, pool: &PgPool) -> Result<DbBanner> {
    Ok(
        sqlx::query_file_as!(DbBanner, "sql/gi/banners/get_by_id.sql", id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn delete_by_id<'e, E>(id: i32, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file_as!(DbBanner, "sql/gi/banners/delete_by_id.sql", id)
        .execute(executor)
        .await?;

    Ok(())
}
