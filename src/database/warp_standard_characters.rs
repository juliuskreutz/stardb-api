use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct DbWarpStandardCharacter {
    pub id: i64,
    pub uid: i64,
    pub character: i32,
    pub name: String,
    pub timestamp: NaiveDateTime,
}

pub async fn set_warp_standard_character(
    warp_standard_character: &DbWarpStandardCharacter,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            warp_standard_characters(id, uid, character, timestamp)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id, uid)
        DO UPDATE SET
            character = EXCLUDED.character,
            timestamp = EXCLUDED.timestamp
        ",
        warp_standard_character.id,
        warp_standard_character.uid,
        warp_standard_character.character,
        warp_standard_character.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_warp_standard_characters_by_uid(
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbWarpStandardCharacter>> {
    Ok(sqlx::query_as!(
        DbWarpStandardCharacter,
        "
        SELECT
            warp_standard_characters.*,
            characters_text.name
        FROM
            warp_standard_characters
        INNER JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $2
        WHERE
            uid = $1
        ORDER BY
            timestamp
        ",
        uid,
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_warp_standard_character_by_id_and_uid(
    id: i64,
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<DbWarpStandardCharacter> {
    Ok(sqlx::query_as!(
        DbWarpStandardCharacter,
        "
        SELECT
            warp_standard_characters.*,
            characters_text.name
        FROM
            warp_standard_characters
        INNER JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $3
        WHERE
            warp_standard_characters.id = $1
        AND
            uid = $2
        ",
        id,
        uid,
        language,
    )
    .fetch_one(pool)
    .await?)
}
