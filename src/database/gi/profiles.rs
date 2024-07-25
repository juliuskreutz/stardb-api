use sqlx::PgPool;

pub struct DbProfile {
    pub uid: i32,
    pub name: String,
}

pub async fn set(profile: &DbProfile, pool: &PgPool) -> anyhow::Result<()> {
    Ok(())
}
