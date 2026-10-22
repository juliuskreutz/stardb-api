use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

/// Schedule completion-percent refreshes hourly, retrying failed updates after thirty seconds.
pub async fn spawn(pool: PgPool) {
    super::spawn_periodic(
        "achievements_percent",
        Duration::from_secs(3600),
        Duration::from_secs(30),
        move || update(pool.clone()),
    );
}

/// Rebuild this game's persisted achievement percentages and propagate database errors.
async fn update(pool: PgPool) -> Result<()> {
    database::achievements_percent::update(&pool).await?;

    Ok(())
}
