use anyhow::Result;
use sqlx::PgPool;

pub struct DbUid {
    pub uid: i32,
}

pub async fn set(uid: &DbUid, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/zzz/uids/set.sql", uid.uid)
        .execute(pool)
        .await?;

    Ok(())
}
