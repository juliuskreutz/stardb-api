use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DbGachaBanner {
    pub id: i32,
    pub game_id: i32,
    pub gacha_type: Option<i32>,
    pub banner_id: Option<i32>,
    pub title: Option<String>,
    pub internal_name: Option<String>,
    pub version: Option<String>,
    pub rate_up_5_stars: Vec<i32>,
    pub rate_up_4_stars: Vec<i32>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub timezone_dependant: bool,
    pub disabled: bool,
    pub created_at: DateTime<Utc>
}

#[derive(Debug)]
pub enum GachaGame {
    StarRail = 1,
    Zenless = 2,
    Genshin = 3,
}

impl GachaGame {
    fn from_game_id(game_id: i32) -> Option<Self> {
        match game_id {
            1 => Some(GachaGame::StarRail),
            2 => Some(GachaGame::Zenless),
            3 => Some(GachaGame::Genshin),
            _ => None,
        }
    }
}

pub async fn create(banner: &DbGachaBanner, pool: &PgPool) -> anyhow::Result<Option<DbGachaBanner>> {
    let rec = sqlx::query_file!(
        "sql/gacha_banners/create.sql",
        banner.game_id,
        banner.gacha_type,
        banner.banner_id,
        banner.title,
        banner.internal_name,
        banner.version,
        &banner.rate_up_5_stars,
        &banner.rate_up_4_stars,
        banner.start_time,
        banner.end_time,
        banner.timezone_dependant,
        banner.disabled,
    )
        .fetch_one(pool)
        .await?;

    let id = rec.id;
    get_by_id(id, pool).await
}

pub async fn update(banner: &DbGachaBanner, pool: &PgPool) -> anyhow::Result<Option<DbGachaBanner>> {
    sqlx::query_file!(
        "sql/gacha_banners/update.sql",
        banner.game_id,
        banner.gacha_type,
        banner.banner_id,
        banner.title,
        banner.internal_name,
        banner.version,
        &banner.rate_up_5_stars,
        &banner.rate_up_4_stars,
        banner.start_time,
        banner.end_time,
        banner.timezone_dependant,
        banner.disabled,
        banner.id,
    )
        .execute(pool)
        .await?;

    get_by_id(banner.id, pool).await
}

pub async fn get_all(pool: &PgPool) -> anyhow::Result<Vec<DbGachaBanner>> {
    Ok(sqlx::query_file_as!(DbGachaBanner, "sql/gacha_banners/get_all.sql")
        .fetch_all(pool)
        .await?)
}

pub async fn get_all_by_game(game: GachaGame, pool: &PgPool) -> anyhow::Result<Vec<DbGachaBanner>> {
    let game_id = game as i32;
    Ok(
        sqlx::query_file_as!(DbGachaBanner, "sql/gacha_banners/get_all_by_game.sql", game_id)
            .fetch_all(pool)
            .await?
    )
}

pub async fn get_by_id(id: i32, pool: &PgPool) -> anyhow::Result<Option<DbGachaBanner>> {
    Ok(
        sqlx::query_file_as!(DbGachaBanner, "sql/gacha_banners/get_by_id.sql", id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn delete_by_id(id: i32, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file_as!(DbGachaBanner, "sql/gacha_banners/delete_by_id.sql", id)
        .execute(pool)
        .await?;

    Ok(())
}
