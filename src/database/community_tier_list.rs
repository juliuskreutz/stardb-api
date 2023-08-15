use sqlx::PgPool;

use anyhow::Result;

pub struct DbCommunityTierListEntry {
    pub character: i32,
    pub eidolon: i32,
    pub average: f64,
    pub variance: f64,
    pub votes: i32,
    pub character_name: String,
    pub character_path: String,
    pub character_element: String,
}

pub async fn set_community_tier_list_entry(
    community_tier_list_entry: &DbCommunityTierListEntry,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            community_tier_list_entries(character, eidolon, average, variance, votes)
        VALUES
            ($1, $2, $3, $4, $5)
        ON CONFLICT
            (character, eidolon)
        DO UPDATE SET
            average = EXCLUDED.average,
            variance = EXCLUDED.variance,
            votes = EXCLUDED.votes
        ",
        community_tier_list_entry.character,
        community_tier_list_entry.eidolon,
        community_tier_list_entry.average,
        community_tier_list_entry.variance,
        community_tier_list_entry.votes,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_community_tier_list_entries(
    pool: &PgPool,
) -> Result<Vec<DbCommunityTierListEntry>> {
    Ok(sqlx::query_as!(
        DbCommunityTierListEntry,
        "
        SELECT
            community_tier_list_entries.*,
            characters.name character_name,
            characters.element character_element,
            characters.path character_path
        FROM
            community_tier_list_entries
        INNER JOIN
            characters
        ON
            character = id
        ORDER BY
            average DESC
        ",
    )
    .fetch_all(pool)
    .await?)
}
