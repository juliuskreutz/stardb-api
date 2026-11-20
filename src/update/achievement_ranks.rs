//! Refreshes the precomputed achievement leaderboard ranks every few minutes.

use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

/// Rank every five minutes; leaderboard pages read the stored result instead of ranking.
pub async fn spawn(pool: PgPool) {
    super::spawn_periodic(
        "achievement_ranks",
        Duration::from_secs(5 * 60),
        Duration::from_secs(30),
        move || update(pool.clone()),
    );
}

/// Recompute ranks in one transaction so pages never see a half-updated ranking.
///
/// Deliberately skips `HEAVY_REFRESH_LOCK`: the gacha stats rebuilds hold it for their
/// whole run, which would leave the leaderboard stale far longer than five minutes.
async fn update(pool: PgPool) -> Result<()> {
    let mut transaction = pool.begin().await?;
    let changed = database::achievement_scores::refresh_ranks(&mut transaction).await?;
    transaction.commit().await?;
    info!("Achievement ranks refreshed: {changed} rows changed");
    Ok(())
}
