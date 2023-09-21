use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn achievements_percent(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
                log::error!(
                    "Achievements Percent update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                log::info!(
                    "Achievements Percent update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let total_count = database::get_users_achievements_user_count(pool).await? as f64;

    let achievements_users_count = database::get_achievements_users_count(pool).await?;

    let mut achievements_users_count_map = HashMap::new();

    for id in database::get_achievements_id(pool).await? {
        achievements_users_count_map.insert(id, 0.0);
    }

    for achievement_users_count in achievements_users_count {
        let id = achievement_users_count.id;
        let percent = achievement_users_count.count.unwrap_or_default() as f64 / total_count;

        achievements_users_count_map.insert(id, percent);
    }

    for (id, percent) in achievements_users_count_map {
        let achievement_percent = database::DbAchievementPercent { id, percent };

        database::set_achievement_percent(&achievement_percent, pool).await?;
    }

    Ok(())
}
