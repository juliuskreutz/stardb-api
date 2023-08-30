use anyhow::Result;
use sqlx::PgPool;

pub struct DbCommunityTierListSextile {
    pub id: i32,
    pub value: f64,
}

pub async fn set_community_tier_list_sextile(
    community_tier_list_sextile: &DbCommunityTierListSextile,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            community_tier_list_sextiles(id, value)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            value = EXCLUDED.value
        ",
        community_tier_list_sextile.id,
        community_tier_list_sextile.value,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_community_tier_list_sextiles(
    pool: &PgPool,
) -> Result<Vec<DbCommunityTierListSextile>> {
    Ok(sqlx::query_as!(
        DbCommunityTierListSextile,
        "
        SELECT
            *
        FROM
            community_tier_list_sextiles
        ORDER BY
            id
        ",
    )
    .fetch_all(pool)
    .await?)
}
