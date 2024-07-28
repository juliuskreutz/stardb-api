use std::{collections::HashMap, fs::File, io::BufReader};

use sqlx::PgPool;

use crate::{database, Language};

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut achievement_series_id = Vec::new();
    let mut achievement_series_language = Vec::new();
    let mut achievement_series_name = Vec::new();

    let mut achievements_id = Vec::new();
    let mut achievements_language = Vec::new();
    let mut achievements_name = Vec::new();
    let mut achievements_description = Vec::new();

    let mut characters_id = Vec::new();
    let mut characters_language = Vec::new();
    let mut characters_name = Vec::new();

    let mut weapons_id = Vec::new();
    let mut weapons_language = Vec::new();
    let mut weapons_name = Vec::new();

    for (language_str, language) in [
        ("CHS", Language::ZhCn),
        ("CHT", Language::ZhTw),
        ("DE", Language::De),
        ("EN", Language::En),
        ("ES", Language::EsEs),
        ("FR", Language::Fr),
        ("ID", Language::Id),
        ("JP", Language::Ja),
        ("KR", Language::Ko),
        ("PT", Language::PtPt),
        ("RU", Language::Ru),
        ("TH", Language::Th),
        ("VI", Language::Vi),
    ] {
        actix_web::rt::task::yield_now().await;

        info!("Starting {}", language);

        let text_map: HashMap<String, String> =
            serde_json::from_reader(BufReader::new(File::open(format!(
                "dimbreath/AnimeGameData/TextMap/TextMap{language_str}.json",
            ))?))?;

        info!("Starting {} achievement series", language);
        for achievement_goal in &configs.achievement_goal_data {
            let name = text_map[&achievement_goal.name.to_string()].clone();

            let id = achievement_goal.id.unwrap_or_default();

            achievement_series_id.push(id);
            achievement_series_language.push(language);
            achievement_series_name.push(name);
        }

        info!("Starting {} achievements", language);
        for achievement in &configs.achievement_data {
            if achievement.disuse == Some(true) {
                continue;
            }

            let name = text_map[&achievement.name.to_string()].clone();
            let description = text_map[&achievement.description.to_string()].clone();

            let id = achievement.id;

            achievements_id.push(id);
            achievements_language.push(language);
            achievements_name.push(name);
            achievements_description.push(description);
        }

        info!("Starting {} avatars", language);
        for avatar in &configs.avatar_data {
            let name = text_map[&avatar.name.to_string()].clone();

            let id = avatar.id;

            characters_id.push(id);
            characters_language.push(language);
            characters_name.push(name);
        }

        info!("Starting {} weapons", language);
        for weapon in &configs.weapon_data {
            let id = weapon.id;

            let Some(name) = text_map.get(&weapon.name.to_string()).cloned() else {
                continue;
            };

            weapons_id.push(id);
            weapons_language.push(language);
            weapons_name.push(name);
        }
    }

    info!("Setting all achievement series texts");
    database::gi::achievement_series_text::set_all(
        &achievement_series_id,
        &achievement_series_language,
        &achievement_series_name,
        pool,
    )
    .await?;

    info!("Setting all achievements texts");
    database::gi::achievements_text::set_all(
        &achievements_id,
        &achievements_language,
        &achievements_name,
        &achievements_description,
        pool,
    )
    .await?;

    info!("Setting all character texts");
    database::gi::characters_text::set_all(
        &characters_id,
        &characters_language,
        &characters_name,
        pool,
    )
    .await?;

    info!("Setting all w-engines texts");
    database::gi::weapons_text::set_all(&weapons_id, &weapons_language, &weapons_name, pool)
        .await?;

    Ok(())
}
