use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct DbWarpSpecialCharacter {
    pub id: i64,
    pub uid: i64,
    pub character: i32,
    pub name: String,
    pub timestamp: NaiveDateTime,
}

pub async fn set_warp_special_character(
    warp_special_character: &DbWarpSpecialCharacter,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            warp_special_characters(id, uid, character, timestamp)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id, uid)
        DO UPDATE SET
            character = EXCLUDED.character,
            timestamp = EXCLUDED.timestamp
        ",
        warp_special_character.id,
        warp_special_character.uid,
        warp_special_character.character,
        warp_special_character.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_warp_special_characters_by_uid(
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbWarpSpecialCharacter>> {
    Ok(sqlx::query_as!(
        DbWarpSpecialCharacter,
        "
        SELECT
            warp_special_characters.*,
            characters_text.name
        FROM
            warp_special_characters
        LEFT JOIN
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

pub async fn get_warp_special_character_by_id_and_uid(
    id: i64,
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<DbWarpSpecialCharacter> {
    Ok(sqlx::query_as!(
        DbWarpSpecialCharacter,
        "
        SELECT
            warp_special_characters.*,
            characters_text.name
        FROM
            warp_special_characters
        LEFT JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $3
        WHERE
            warp_special_characters.id = $1
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
