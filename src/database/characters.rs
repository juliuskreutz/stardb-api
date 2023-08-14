use sqlx::PgPool;

use crate::Result;

pub struct DbCharacter {
    pub id: i32,
    pub name: String,
    pub element: String,
    pub path: String,
}

pub async fn set_character(character: &DbCharacter, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            characters(id, name, element, path)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id)
        DO UPDATE SET
            name = EXCLUDED.name,
            element = EXCLUDED.element,
            path = EXCLUDED.path
        ",
        character.id,
        character.name,
        character.element,
        character.path,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_characters(
    element: Option<&str>,
    path: Option<&str>,
    pool: &PgPool,
) -> Result<Vec<DbCharacter>> {
    Ok(sqlx::query_as!(
        DbCharacter,
        "
        SELECT
            *
        FROM
            characters
        WHERE
            ($1::TEXT IS NULL OR LOWER(element) = LOWER($1))
        AND
            ($2::TEXT IS NULL OR LOWER(path) = LOWER($2))
        ",
        element,
        path,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_character_by_id(id: i32, pool: &PgPool) -> Result<DbCharacter> {
    Ok(
        sqlx::query_as!(DbCharacter, "SELECT * FROM characters WHERE id = $1", id)
            .fetch_one(pool)
            .await?,
    )
}
