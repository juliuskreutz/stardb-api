use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn spawn(pool: PgPool) {
    super::spawn_periodic(
        "achievements_percent",
        Duration::from_secs(3600),
        Duration::from_secs(30),
        move || update(pool.clone()),
    );
}

async fn update(pool: PgPool) -> Result<()> {
    database::achievements_percent::update(&pool).await?;

    Ok(())
}
