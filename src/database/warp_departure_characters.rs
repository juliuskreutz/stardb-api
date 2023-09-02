use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct DbWarpDepartureCharacter {
    pub id: i64,
    pub uid: i64,
    pub character: i32,
    pub name: String,
    pub timestamp: NaiveDateTime,
}

pub async fn set_warp_departure_character(
    warp_departure_character: &DbWarpDepartureCharacter,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            warp_departure_characters(id, uid, character, timestamp)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id, uid)
        DO UPDATE SET
            character = EXCLUDED.character,
            timestamp = EXCLUDED.timestamp
        ",
        warp_departure_character.id,
        warp_departure_character.uid,
        warp_departure_character.character,
        warp_departure_character.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_warp_departure_characters_by_uid(
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbWarpDepartureCharacter>> {
    Ok(sqlx::query_as!(
        DbWarpDepartureCharacter,
        "
        SELECT
            warp_departure_characters.*,
            characters_text.name
        FROM
            warp_departure_characters
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

pub async fn get_warp_departure_character_by_id_and_uid(
    id: i64,
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<DbWarpDepartureCharacter> {
    Ok(sqlx::query_as!(
        DbWarpDepartureCharacter,
        "
        SELECT
            warp_departure_characters.*,
            characters_text.name
        FROM
            warp_departure_characters
        LEFT JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $3
        WHERE
            warp_departure_characters.id = $1
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
