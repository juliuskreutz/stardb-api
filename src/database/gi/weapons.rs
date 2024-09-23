use anyhow::Result;
use sqlx::PgPool;

pub struct DbWeapon {
    pub id: i32,
    pub rarity: i32,
}

pub async fn set_all(id: &[i32], rarity: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/weapons/set_all.sql", id, rarity)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWeapon>> {
    Ok(sqlx::query_file_as!(DbWeapon, "sql/gi/weapons/get_all.sql")
        .fetch_all(pool)
        .await?)
}

pub async fn get_by_paimon_moe_id(id: &str, pool: &PgPool) -> Result<DbWeapon> {
    Ok(
        sqlx::query_file_as!(DbWeapon, "sql/gi/weapons/get_by_paimon_moe_id.sql", id)
            .fetch_one(pool)
            .await?,
    )
}
