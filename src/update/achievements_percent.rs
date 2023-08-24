use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::rt::{self, time};
use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn achievements_percent(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));

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
    let completed = database::get_completed(pool).await?;

    let mut usernames_achievements: HashMap<String, Vec<i64>> = HashMap::new();

    for completed in &completed {
        usernames_achievements
            .entry(completed.username.clone())
            .or_default()
            .push(completed.id)
    }

    let mut achievements_count: HashMap<i64, usize> = HashMap::new();

    for achievements in usernames_achievements.values().filter(|v| v.len() >= 50) {
        for &achievement in achievements {
            *achievements_count.entry(achievement).or_default() += 1;
        }
    }

    let achievements_id = database::get_achievements_id(pool).await?;

    for id in achievements_id {
        let percent = if let Some(&count) = achievements_count.get(&id) {
            count as f64 / usernames_achievements.len() as f64
        } else {
            0.0
        };

        let achievement_percent = database::DbAchievementPercent { id, percent };

        database::set_achievement_percent(&achievement_percent, pool).await?;
    }

    Ok(())
}
