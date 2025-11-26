use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut achievement_series_id = Vec::new();
    let mut achievement_series_priority = Vec::new();

    for achievement_series in &configs.achievement_second_class["OOFFGGKCDID"] {
        let id = achievement_series.id;
        let priority = achievement_series.priority;

        achievement_series_id.push(id);
        achievement_series_priority.push(priority);
    }

    for arcade_achievement_series in &configs.arcade_achievement_group["OOFFGGKCDID"] {
        let id = arcade_achievement_series.id;

        achievement_series_id.push(id);
        achievement_series_priority.push(id);
    }

    database::zzz::achievement_series::set_all(
        &achievement_series_id,
        &achievement_series_priority,
        pool,
    )
    .await?;

    Ok(())
}
