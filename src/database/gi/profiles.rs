use sqlx::PgPool;

pub struct DbProfile {
    pub uid: i32,
    pub name: String,
}

pub async fn set(profile: &DbProfile, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query_file!("sql/gi/profiles/set.sql", profile.uid, profile.name)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> anyhow::Result<DbProfile> {
    Ok(
        sqlx::query_file_as!(DbProfile, "sql/gi/profiles/get_by_uid.sql", uid)
            .fetch_one(pool)
            .await?,
    )
}
