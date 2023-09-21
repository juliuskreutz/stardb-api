use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct DbWarp {
    pub id: i64,
    pub uid: i64,
    pub gacha_type: String,
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub name: Option<String>,
    pub timestamp: NaiveDateTime,
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
        warp.gacha_type,
        warp.character,
        warp.light_cone,
        warp.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_warps_by_uid(uid: i64, language: &str, pool: &PgPool) -> Result<Vec<DbWarp>> {
    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.*,
            COALESCE(characters_text.name, light_cones_text.name) AS name
        FROM
            warps
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
    uid: i64,
    gacha_type: &str,
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbWarp>> {
    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.*,
            COALESCE(characters_text.name, light_cones_text.name) AS name
        FROM
            warps
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

pub async fn get_warp_by_id_and_gacha_type(
    id: i64,
    gacha_type: &str,
    language: &str,
    pool: &PgPool,
) -> Result<DbWarp> {
    Ok(sqlx::query_as!(
        DbWarp,
        "
        SELECT
            warps.*,
            COALESCE(characters_text.name, light_cones_text.name) AS name
        FROM
            warps
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
            gacha_type = $2
        ",
        id,
        gacha_type,
        language,
    )
    .fetch_one(pool)
    .await?)
}
