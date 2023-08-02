use sqlx::PgPool;

use crate::Result;

pub struct DbTierListEntry {
    pub character: i32,
    pub character_tag: String,
    pub character_name: String,
    pub character_element: String,
    pub character_path: String,
    pub eidolon: i32,
    pub role: String,
    pub st_dps: Option<i32>,
    pub aoe_dps: Option<i32>,
    pub buffer: Option<i32>,
    pub debuffer: Option<i32>,
    pub healer: Option<i32>,
    pub survivability: Option<i32>,
    pub sp_efficiency: Option<i32>,
    pub avg_break: Option<i32>,
    pub st_break: Option<i32>,
    pub base_speed: Option<i32>,
    pub footnote: Option<String>,
    pub score: i32,
    pub score_number: i32,
}

pub async fn set_tier_list_entry(tier_list_entry: &DbTierListEntry, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            tier_list(character, eidolon, role, st_dps, aoe_dps, buffer, debuffer, healer, survivability, sp_efficiency, avg_break, st_break, base_speed, footnote, score, score_number)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        ON CONFLICT
            (character, eidolon, role)
        DO UPDATE SET
            st_dps = EXCLUDED.st_dps,
            aoe_dps = EXCLUDED.aoe_dps,
            buffer = EXCLUDED.buffer,
            debuffer = EXCLUDED.debuffer,
            healer = EXCLUDED.healer,
            survivability = EXCLUDED.survivability,
            sp_efficiency = EXCLUDED.sp_efficiency,
            avg_break = EXCLUDED.avg_break,
            st_break = EXCLUDED.st_break,
            base_speed = EXCLUDED.base_speed,
            footnote = EXCLUDED.footnote,
            score = EXCLUDED.score,
            score_number = EXCLUDED.score_number
        ",
        tier_list_entry.character,
        tier_list_entry.eidolon,
        tier_list_entry.role,
        tier_list_entry.st_dps,
        tier_list_entry.aoe_dps,
        tier_list_entry.buffer,
        tier_list_entry.debuffer,
        tier_list_entry.healer ,
        tier_list_entry.survivability,
        tier_list_entry.sp_efficiency,
        tier_list_entry.avg_break ,
        tier_list_entry.st_break ,
        tier_list_entry.base_speed,
        tier_list_entry.footnote,
        tier_list_entry.score,
        tier_list_entry.score_number,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_tier_list_entries(pool: &PgPool) -> Result<Vec<DbTierListEntry>> {
    Ok(sqlx::query_as!(
        DbTierListEntry,
        "
        SELECT
            tier_list.*,
            characters.tag character_tag,
            characters.name character_name,
            characters.element character_element,
            characters.path character_path
        FROM
            tier_list
        INNER JOIN
            characters
        ON
            character = characters.id
        ORDER BY
            score_number DESC
        "
    )
    .fetch_all(pool)
    .await?)
}
