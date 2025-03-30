use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut achievements_id = Vec::new();
    let mut achievements_series = Vec::new();
    let mut achievements_primogems = Vec::new();
    let mut achievements_hidden = Vec::new();
    let mut achievements_priority = Vec::new();

    for achievement_data in &configs.achievement_data {
        if achievement_data.disuse == Some(true) {
            continue;
        }

        let id = achievement_data.id;

        let series = achievement_data.goal.unwrap_or_default();

        let primogems = configs
            .reward_data
            .iter()
            .find(|r| r.id == achievement_data.reward)
            .unwrap()
            .rewards
            .iter()
            .find(|r| r.id == Some(201))
            .unwrap()
            .count
            .unwrap();

        let hidden = achievement_data.show.as_deref() == Some("SHOWTYPE_HIDE");

        let priority = achievement_data.order.unwrap();

        achievements_id.push(id);
        achievements_series.push(series);
        achievements_primogems.push(primogems);
        achievements_hidden.push(hidden);
        achievements_priority.push(priority);
    }

    database::gi::achievements::set_all(
        &achievements_id,
        &achievements_series,
        &achievements_primogems,
        &achievements_hidden,
        &achievements_priority,
        pool,
    )
    .await?;

    Ok(())
}
