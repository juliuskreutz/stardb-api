use anyhow::Result;
use sqlx::PgPool;

pub struct DbLightCone {
    pub id: i32,
    pub name: String,
}

pub async fn set_light_cone(light_cone: &DbLightCone, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            light_cones(id)
        VALUES
            ($1)
        ON CONFLICT DO NOTHING
        ",
        light_cone.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_light_cones(language: &str, pool: &PgPool) -> Result<Vec<DbLightCone>> {
    Ok(sqlx::query_as!(
        DbLightCone,
        "
        SELECT
            light_cones.id,
            light_cones_text.name
        FROM
            light_cones
        INNER JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cones.id AND light_cones_text.language = $1
        ORDER BY
            id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_light_cone_by_id(id: i32, language: &str, pool: &PgPool) -> Result<DbLightCone> {
    Ok(sqlx::query_as!(
        DbLightCone,
        "
        SELECT
            light_cones.id,
            light_cones_text.name
        FROM
            light_cones
        INNER JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cones.id AND light_cones_text.language = $2
        WHERE
            light_cones.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}
