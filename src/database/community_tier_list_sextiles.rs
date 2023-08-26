use anyhow::Result;
use sqlx::PgPool;

pub struct DbCommunityTierListSextile {
    pub value: f64,
}

pub async fn set_community_tier_list_sextile(
    community_tier_list_sextile: &DbCommunityTierListSextile,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            community_tier_list_sextiles(value)
        VALUES
            ($1)
        ",
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
            value
        ",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn delete_community_tier_list_sextiles(pool: &PgPool) -> Result<()> {
    sqlx::query!("DELETE FROM community_tier_list_sextiles")
        .execute(pool)
        .await?;

    Ok(())
}
