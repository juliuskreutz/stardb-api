use anyhow::Result;
use sqlx::PgPool;

pub struct DbCharacter {
    pub id: i32,
    pub rarity: i32,
}

pub async fn set_all(id: &[i32], rarity: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/characters/set_all.sql", id, rarity)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_by_paimon_moe_id(id: &str, pool: &PgPool) -> Result<DbCharacter> {
    Ok(sqlx::query_file_as!(
        DbCharacter,
        "sql/gi/characters/get_by_paimon_moe_id.sql",
        id,
    )
    .fetch_one(pool)
    .await?)
}
