//! Keeps ZZZ achievement completion percentages correct.
//!
//! Completion list writes keep the running totals current (see
//! `database::achievement_completions`); this job only recounts from scratch when the
//! game has never been counted, its threshold changed, or the weekly correction is due.

use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;

use crate::database::{achievement_completions, achievement_lists::Game};

/// Check hourly whether a full recount is due; retry failed recounts after thirty seconds.
pub async fn spawn(pool: PgPool) {
    super::spawn_periodic(
        "zzz_achievements_percent",
        Duration::from_secs(3600),
        Duration::from_secs(30),
        move || update(pool.clone()),
    );
}

/// Recount this game's completion totals when due, as one transaction.
async fn update(pool: PgPool) -> Result<()> {
    if !achievement_completions::recount_due(Game::Zzz, &pool).await? {
        return Ok(());
    }
    let _refresh = super::HEAVY_REFRESH_LOCK.lock().await;
    let mut transaction = pool.begin().await?;
    achievement_completions::recount(Game::Zzz, &mut transaction).await?;
    transaction.commit().await?;
    info!("ZZZ achievement completions recounted");
    Ok(())
}
