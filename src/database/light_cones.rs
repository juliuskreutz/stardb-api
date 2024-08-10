use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbLightCone {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub path_id: String,
}

pub async fn set_all(id: &[i32], rarity: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            light_cones(id, rarity)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::integer[])
        ON CONFLICT
            (id)
        DO UPDATE SET
            rarity = EXCLUDED.rarity
        ",
        id,
        rarity,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(language: Language, pool: &PgPool) -> Result<Vec<DbLightCone>> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbLightCone,
        "
        SELECT
            light_cones.id,
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
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbLightCone> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbLightCone,
        "
        SELECT
            light_cones.id,
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
        language,
    )
    .fetch_one(pool)
    .await?)
}
