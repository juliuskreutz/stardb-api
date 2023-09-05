use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct DbWarpLcCharacter {
    pub id: i64,
    pub uid: i64,
    pub character: i32,
    pub name: String,
    pub timestamp: NaiveDateTime,
}

pub async fn set_warp_lc_character(
    warp_lc_character: &DbWarpLcCharacter,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            warp_lc_characters(id, uid, character, timestamp)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id, uid)
        DO UPDATE SET
            character = EXCLUDED.character,
            timestamp = EXCLUDED.timestamp
        ",
        warp_lc_character.id,
        warp_lc_character.uid,
        warp_lc_character.character,
        warp_lc_character.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_warp_lc_characters_by_uid(
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbWarpLcCharacter>> {
    Ok(sqlx::query_as!(
        DbWarpLcCharacter,
        "
        SELECT
            warp_lc_characters.*,
            characters_text.name
        FROM
            warp_lc_characters
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

pub async fn get_warp_lc_character_by_id_and_uid(
    id: i64,
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<DbWarpLcCharacter> {
    Ok(sqlx::query_as!(
        DbWarpLcCharacter,
        "
        SELECT
            warp_lc_characters.*,
            characters_text.name
        FROM
            warp_lc_characters
        INNER JOIN
            characters_text
        ON
            characters_text.id = character AND characters_text.language = $3
        WHERE
            warp_lc_characters.id = $1
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
