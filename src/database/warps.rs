use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{GachaType, Language};

pub struct DbWarp {
    pub id: i64,
    pub uid: i32,
    pub gacha_type: String,
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}

pub async fn set_all_warps(
    id: &[i64],
    uid: &[i32],
    gacha_type: &[GachaType],
    character: &[Option<i32>],
    light_cone: &[Option<i32>],
    timestamp: &[DateTime<Utc>],
    official: &[bool],
    pool: &PgPool,
) -> Result<()> {
    let gacha_type = &gacha_type
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    sqlx::query!(
        "
        INSERT INTO
            warps(id, uid, gacha_type, character, light_cone, timestamp, official)
        SELECT
            *
        FROM
            UNNEST($1::bigint[], $2::integer[], $3::text[], $4::integer[], $5::integer[], $6::timestamp[], $7::boolean[])
        ON CONFLICT
            DO NOTHING
        ",
        id,
        uid,
        gacha_type,
        character as &[Option<i32>],
        light_cone as &[Option<i32>],
        timestamp as &[DateTime<Utc>],
        official,
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

pub async fn get_warp_uids(pool: &PgPool) -> Result<Vec<i32>> {
    Ok(sqlx::query!(
        "select uid from mihomo where exists (select * from warps where mihomo.uid = warps.uid) and not exists (select * from connections where mihomo.uid = connections.uid and connections.private)"
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| r.uid)
    .collect())
}

pub async fn get_warps_by_uid(uid: i32, language: Language, pool: &PgPool) -> Result<Vec<DbWarp>> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.id,
            warps.uid,
            warps.gacha_type,
            warps.character,
            warps.light_cone,
            warps.timestamp,
            warps.official,
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
        language,
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
    let gacha_type = gacha_type.to_string();
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.id,
            warps.uid,
            warps.gacha_type,
            warps.character,
            warps.light_cone,
            warps.timestamp,
            warps.official,
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
        gacha_type,
        language,
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
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.id,
            warps.uid,
            warps.gacha_type,
            warps.character,
            warps.light_cone,
            warps.timestamp,
            warps.official,
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
        language,
    )
    .fetch_one(pool)
    .await?)
}

pub struct DbWarpCount {
    pub gacha_type: String,
    pub count: Option<i64>,
}

pub async fn get_warp_counts_by_uid(uid: i32, pool: &PgPool) -> Result<Vec<DbWarpCount>> {
    Ok(sqlx::query_as!(
        DbWarpCount,
        "
        SELECT
            gacha_type,
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
    let language = language.to_string();

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
        language,
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
    let language = language.to_string();

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
        language,
    )
    .fetch_all(pool)
    .await?)
}
