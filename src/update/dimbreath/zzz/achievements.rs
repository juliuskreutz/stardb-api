use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut achievements_id = Vec::new();
    let mut achievements_series = Vec::new();
    let mut achievements_polychromes = Vec::new();
    let mut achievements_hidden = Vec::new();
    let mut achievements_priority = Vec::new();
    let mut achievements_arcade = Vec::new();

    for achievement_data in &configs.achievement["MIJCMCEDADM"] {
        let id = achievement_data.id;

        let series = achievement_data.series;

        let rewards = configs.once_reward["MIJCMCEDADM"]
            .iter()
            .find(|r| r.id == achievement_data.reward)
            .unwrap();

        let polychromes = rewards.rewards.iter().find(|r| r.id == 100).unwrap().amount;
        let hidden = achievement_data.hidden == 1;

        let priority = achievement_data.priority;

        achievements_id.push(id);
        achievements_series.push(series);
        achievements_polychromes.push(polychromes);
        achievements_hidden.push(hidden);
        achievements_priority.push(priority);
        achievements_arcade.push(false);
    }

    for arcade_achievement_data in &configs.arcade_achievement["MIJCMCEDADM"] {
        let id = arcade_achievement_data.id;

        let series = arcade_achievement_data.series;

        let rewards = configs.once_reward["MIJCMCEDADM"]
            .iter()
            .find(|r| r.id == arcade_achievement_data.reward)
            .unwrap();

        let polychromes = rewards.rewards.iter().find(|r| r.id == 100).unwrap().amount;

        achievements_id.push(id);
        achievements_series.push(series);
        achievements_polychromes.push(polychromes);
        achievements_hidden.push(false);
        achievements_priority.push(id);
        achievements_arcade.push(true);
    }

    database::zzz::achievements::set_all(
        &achievements_id,
        &achievements_series,
        &achievements_polychromes,
        &achievements_hidden,
        &achievements_priority,
        &achievements_arcade,
        pool,
    )
    .await?;

    Ok(())
}
