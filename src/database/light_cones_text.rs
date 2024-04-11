use anyhow::Result;
use sqlx::PgPool;

pub struct DbLightConeText {
    pub id: i32,
    pub language: String,
    pub name: String,
    pub path: String,
}

pub async fn set_light_cone_text(light_cone_text: &DbLightConeText, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            light_cones_text(id, language, name, path)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name,
            path = EXCLUDED.path
        ",
        light_cone_text.id,
        light_cone_text.language,
        light_cone_text.name,
        light_cone_text.path,
    )
    .execute(pool)
    .await?;

    Ok(())
}
