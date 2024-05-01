use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{GachaType, Language};

pub struct DbWarp {
    pub id: i64,
    pub uid: i32,
    pub gacha_type: GachaType,
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
}

pub async fn set_warp(warp: &DbWarp, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            warps(id, uid, gacha_type, character, light_cone, timestamp)
        VALUES
            ($1, $2, $3, $4, $5, $6)
        ON CONFLICT
            DO NOTHING
        ",
        warp.id,
        warp.uid,
        warp.gacha_type as GachaType,
        warp.character,
        warp.light_cone,
        warp.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_warp_by_id_and_timestamp(
    id: i64,
    timestamp: DateTime<Utc>,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        DELETE FROM
            warps
        WHERE
            id = $1
        AND
            timestamp = $2
        ",
        id,
        timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_warps_by_uid(uid: i32, language: Language, pool: &PgPool) -> Result<Vec<DbWarp>> {
    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.id,
            warps.uid,
            warps.gacha_type as \"gacha_type: GachaType\",
            warps.character,
            warps.light_cone,
            warps.timestamp,
            COALESCE(characters_text.name, light_cones_text.name) AS name,
            COALESCE(characters.rarity, light_cones.rarity) AS rarity
        FROM
            warps
        LEFT JOIN
            characters
        ON
            characters.id = character
        LEFT JOIN
            light_cones
        ON
            light_cones.id = light_cone
        LEFT JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $2
        LEFT JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cone AND light_cones_text.language = $2
        WHERE
            uid = $1
        ORDER BY
            id
        ",
        uid,
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_warps_by_uid_and_gacha_type(
    uid: i32,
    gacha_type: GachaType,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbWarp>> {
    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.id,
            warps.uid,
            warps.gacha_type as \"gacha_type: GachaType\",
            warps.character,
            warps.light_cone,
            warps.timestamp,
            COALESCE(characters_text.name, light_cones_text.name) AS name,
            COALESCE(characters.rarity, light_cones.rarity) AS rarity
        FROM
            warps
        LEFT JOIN
            characters
        ON
            characters.id = character
        LEFT JOIN
            light_cones
        ON
            light_cones.id = light_cone
        LEFT JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $3
        LEFT JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cone AND light_cones_text.language = $3
        WHERE
            uid = $1
        AND
            gacha_type = $2
        ORDER BY
            id
        ",
        uid,
        gacha_type as GachaType,
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_warp_by_id_and_timestamp(
    id: i64,
    timestamp: DateTime<Utc>,
    language: Language,
    pool: &PgPool,
) -> Result<DbWarp> {
    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.id,
            warps.uid,
            warps.gacha_type as \"gacha_type: GachaType\",
            warps.character,
            warps.light_cone,
            warps.timestamp,
            COALESCE(characters_text.name, light_cones_text.name) AS name,
            COALESCE(characters.rarity, light_cones.rarity) AS rarity
        FROM
            warps
        LEFT JOIN
            characters
        ON
            characters.id = character
        LEFT JOIN
            light_cones
        ON
            light_cones.id = light_cone
        LEFT JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $3
        LEFT JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cone AND light_cones_text.language = $3
        WHERE
            warps.id = $1
        AND
            timestamp = $2
        ",
        id,
        timestamp,
        language as Language,
    )
    .fetch_one(pool)
    .await?)
}

pub struct DbWarpCount {
    pub gacha_type: GachaType,
    pub count: Option<i64>,
}

pub async fn get_warp_counts_by_uid(uid: i32, pool: &PgPool) -> Result<Vec<DbWarpCount>> {
    Ok(sqlx::query_as!(
        DbWarpCount,
        "
        SELECT
            gacha_type AS \"gacha_type: GachaType\",
            count(*)
        FROM
            warps
        WHERE
            uid = $1
        GROUP BY
            gacha_type
        ",
        uid,
    )
    .fetch_all(pool)
    .await?)
}

pub struct DbCharacterCount {
    pub id: i32,
    pub rarity: i32,
    pub name: String,
    pub path: String,
    pub element: String,
    pub path_id: String,
    pub element_id: String,
    pub count: Option<i64>,
}

pub async fn get_characters_count_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbCharacterCount>> {
    Ok(sqlx::query_as!(
        DbCharacterCount,
        "
        SELECT
            characters.id,
            characters.rarity,
            characters_text.name,
            characters_text.path,
            characters_text.element,
            characters_text_en.path path_id,
            characters_text_en.element element_id,
            COUNT(*)
        FROM
            warps
        LEFT JOIN
            characters
        ON
            characters.id = character
        LEFT JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $2
        LEFT JOIN
            characters_text AS characters_text_en
        ON
            characters_text_en.id = character AND characters_text_en.language = 'en'
        WHERE
            uid = $1
        AND
            character IS NOT NULL
        GROUP BY
            characters.id,
            characters.rarity,
            characters_text.name,
            characters_text.path,
            characters_text.element,
            characters_text_en.path,
            characters_text_en.element
        ORDER BY 
            rarity DESC, id DESC
        ",
        uid,
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}

pub struct DbLightConeCount {
    pub id: i32,
    pub rarity: i32,
    pub name: String,
    pub path: String,
    pub path_id: String,
    pub count: Option<i64>,
}

pub async fn get_light_cones_count_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbLightConeCount>> {
    Ok(sqlx::query_as!(
        DbLightConeCount,
        "
        SELECT
            light_cones.id,
            light_cones.rarity,
            light_cones_text.name,
            light_cones_text.path,
            light_cones_text_en.path as path_id,
            COUNT(*)
        FROM
            warps
        LEFT JOIN
            light_cones
        ON
            light_cones.id = light_cone
        LEFT JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cone AND light_cones_text.language = $2
        LEFT JOIN
            light_cones_text AS light_cones_text_en
        ON
            light_cones_text_en.id = light_cone AND light_cones_text_en.language = 'en'
        WHERE
            uid = $1
        AND
            light_cone IS NOT NULL
        GROUP BY
            light_cones.id,
            light_cones.rarity,
            light_cones_text.name,
            light_cones_text.path,
            light_cones_text_en.path
        ORDER BY 
            rarity DESC, id DESC
        ",
        uid,
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}
