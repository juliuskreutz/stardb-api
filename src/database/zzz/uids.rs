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

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<DbUid> {
    Ok(
        sqlx::query_file_as!(DbUid, "sql/zzz/uids/get_by_uid.sql", uid)
            .fetch_one(pool)
            .await?,
    )
}
