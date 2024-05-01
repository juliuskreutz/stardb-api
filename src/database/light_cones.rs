use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbLightCone {
    pub id: i32,
    pub name: String,
    pub rarity: i32,
    pub path: String,
    pub path_id: String,
}

pub async fn set_light_cone(light_cone: &DbLightCone, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            light_cones(id, rarity)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            rarity = EXCLUDED.rarity
        ",
        light_cone.id,
        light_cone.rarity,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_light_cones(language: Language, pool: &PgPool) -> Result<Vec<DbLightCone>> {
    Ok(sqlx::query_as!(
        DbLightCone,
        "
        SELECT
            light_cones.*,
            light_cones_text.name,
            light_cones_text.path,
            light_cones_text_en.path as path_id
        FROM
            light_cones
        INNER JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cones.id AND light_cones_text.language = $1
        INNER JOIN
            light_cones_text AS light_cones_text_en
        ON
            light_cones_text_en.id = light_cones.id AND light_cones_text_en.language = 'en'
        ORDER BY
            id
        ",
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_light_cone_by_id(
    id: i32,
    language: Language,
    pool: &PgPool,
) -> Result<DbLightCone> {
    Ok(sqlx::query_as!(
        DbLightCone,
        "
        SELECT
            light_cones.*,
            light_cones_text.name,
            light_cones_text.path,
            light_cones_text_en.path as path_id
        FROM
            light_cones
        INNER JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cones.id AND light_cones_text.language = $2
        INNER JOIN
            light_cones_text AS light_cones_text_en
        ON
            light_cones_text_en.id = light_cones.id AND light_cones_text_en.language = 'en'
        WHERE
            light_cones.id = $1
        ",
        id,
        language as Language,
    )
    .fetch_one(pool)
    .await?)
}
