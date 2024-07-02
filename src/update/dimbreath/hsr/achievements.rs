use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut achievements_id = Vec::new();
    let mut achievements_series = Vec::new();
    let mut achievements_jades = Vec::new();
    let mut achievements_hidden = Vec::new();
    let mut achievements_priority = Vec::new();

    for achievement_data in configs.achievement_data.values() {
        let id = achievement_data.id;

        let series = achievement_data.series;

        let jades = configs.reward_data[&configs.quest_data[&id.to_string()].reward_id.to_string()]
            .jades
            .unwrap_or_default();

        let hidden = achievement_data.show_type.as_deref() == Some("ShowAfterFinish");

        let priority = achievement_data.priority;

        achievements_id.push(id);
        achievements_series.push(series);
        achievements_jades.push(jades);
        achievements_hidden.push(hidden);
        achievements_priority.push(priority);
    }

    database::achievements::set_all(
        &achievements_id,
        &achievements_series,
        &achievements_jades,
        &achievements_hidden,
        &achievements_priority,
        pool,
    )
    .await?;

    Ok(())
}
