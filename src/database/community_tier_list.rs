use anyhow::Result;
use sqlx::PgPool;

pub struct DbCommunityTierListEntry {
    pub character: i32,
    pub eidolon: i32,
    pub average: f64,
    pub variance: f64,
    pub votes: i32,
    pub character_name: String,
    pub character_path: String,
    pub character_element: String,
    pub total_votes: i32,
}

pub async fn set_community_tier_list_entry(
    community_tier_list_entry: &DbCommunityTierListEntry,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            community_tier_list_entries(character, eidolon, average, variance, votes, total_votes)
        VALUES
            ($1, $2, $3, $4, $5, $6)
        ON CONFLICT
            (character, eidolon)
        DO UPDATE SET
            average = EXCLUDED.average,
            variance = EXCLUDED.variance,
            votes = EXCLUDED.votes,
            total_votes = EXCLUDED.total_votes
        ",
        community_tier_list_entry.character,
        community_tier_list_entry.eidolon,
        community_tier_list_entry.average,
        community_tier_list_entry.variance,
        community_tier_list_entry.votes,
        community_tier_list_entry.total_votes,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_community_tier_list_entries(
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbCommunityTierListEntry>> {
    Ok(sqlx::query_as!(
        DbCommunityTierListEntry,
        "
        SELECT
            community_tier_list_entries.*,
            characters_text.name character_name,
            characters_text.element character_element,
            characters_text.path character_path
        FROM
            community_tier_list_entries
        INNER JOIN
            characters_text
        ON
            character = characters_text.id AND language = $1
        ORDER BY
            average DESC
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}
