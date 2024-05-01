use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbCommunityTierListEntry {
    pub character: i32,
    pub eidolon: i32,
    pub average: f64,
    pub variance: f64,
    pub quartile_1: f64,
    pub quartile_3: f64,
    pub confidence_interval_95: f64,
    pub votes: i32,
    pub total_votes: i32,
    pub character_rarity: i32,
    pub character_name: String,
    pub character_path: String,
    pub character_element: String,
    pub character_path_id: String,
    pub character_element_id: String,
}

pub async fn set_community_tier_list_entry(
    community_tier_list_entry: &DbCommunityTierListEntry,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            community_tier_list_entries(character, eidolon, average, variance, quartile_1, quartile_3, confidence_interval_95, votes, total_votes)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT
            (character, eidolon)
        DO UPDATE SET
            average = EXCLUDED.average,
            variance = EXCLUDED.variance,
            quartile_1 = EXCLUDED.quartile_1,
            quartile_3 = EXCLUDED.quartile_3,
            confidence_interval_95 = EXCLUDED.confidence_interval_95,
            votes = EXCLUDED.votes,
            total_votes = EXCLUDED.total_votes
        ",
        community_tier_list_entry.character,
        community_tier_list_entry.eidolon,
        community_tier_list_entry.average,
        community_tier_list_entry.variance,
        community_tier_list_entry.quartile_1,
        community_tier_list_entry.quartile_3,
        community_tier_list_entry.confidence_interval_95,
        community_tier_list_entry.votes,
        community_tier_list_entry.total_votes,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_community_tier_list_entries(
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbCommunityTierListEntry>> {
    Ok(sqlx::query_as!(
        DbCommunityTierListEntry,
        "
        SELECT
            community_tier_list_entries.*,
            characters.rarity character_rarity,
            characters_text.name character_name,
            characters_text.element character_element,
            characters_text.path character_path,
            characters_text_en.path character_path_id,
            characters_text_en.element character_element_id
        FROM
            community_tier_list_entries
        INNER JOIN
            characters
        ON
            character = characters.id
        INNER JOIN
            characters_text
        ON
            character = characters_text.id AND characters_text.language = $1
        INNER JOIN
            characters_text AS characters_text_en
        ON
            character = characters_text_en.id AND characters_text_en.language = 'en'
        ORDER BY
            average DESC
        ",
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}
