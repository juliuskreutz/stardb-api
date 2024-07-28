use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut achievement_series_id = Vec::new();
    let mut achievement_series_priority = Vec::new();

    for achievement_series in &configs.achievement_goal_data {
        let id = achievement_series.id.unwrap_or_default();
        let priority = achievement_series.order;

        achievement_series_id.push(id);
        achievement_series_priority.push(priority);
    }

    database::gi::achievement_series::set_all(
        &achievement_series_id,
        &achievement_series_priority,
        pool,
    )
    .await?;

    Ok(())
}
