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
