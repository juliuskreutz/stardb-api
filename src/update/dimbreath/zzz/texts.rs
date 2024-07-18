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

    let mut w_engines_id = Vec::new();
    let mut w_engines_language = Vec::new();
    let mut w_engines_name = Vec::new();

    let mut bangboos_id = Vec::new();
    let mut bangboos_language = Vec::new();
    let mut bangboos_name = Vec::new();

    for (language_str, language) in [
        ("", Language::ZhCn),
        ("_CHT", Language::ZhTw),
        ("_DE", Language::De),
        ("_EN", Language::En),
        ("_ES", Language::EsEs),
        ("_FR", Language::Fr),
        ("_ID", Language::Id),
        ("_JA", Language::Ja),
        ("_KO", Language::Ko),
        ("_PT", Language::PtPt),
        ("_RU", Language::Ru),
        ("_TH", Language::Th),
        ("_VI", Language::Vi),
    ] {
        actix_web::rt::task::yield_now().await;

        info!("Starting {}", language);

        let text_map: HashMap<String, String> =
            serde_json::from_reader(BufReader::new(File::open(format!(
                "dimbreath/ZenlessData/TextMap/TextMap{language_str}TemplateTb.json",
            ))?))?;

        info!("Starting {} achievement series", language);
        for achievement_second_class in &configs.achievement_second_class["GMNCBMLIHPE"] {
            let name = text_map[&achievement_second_class.name].clone();

            let id = achievement_second_class.id;

            achievement_series_id.push(id);
            achievement_series_language.push(language);
            achievement_series_name.push(name);
        }

        info!("Starting {} achievements", language);
        for achievement in &configs.achievement["GMNCBMLIHPE"] {
            let name = text_map[&achievement.name].clone();
            let description = text_map[&achievement.description].clone();

            let id = achievement.id;

            achievements_id.push(id);
            achievements_language.push(language);
            achievements_name.push(name);
            achievements_description.push(description);
        }

        info!("Starting {} avatars", language);
        for avatar in &configs.avatar["GMNCBMLIHPE"] {
            let name = text_map[&avatar.name].clone();

            let id = avatar.id;

            characters_id.push(id);
            characters_language.push(language);
            characters_name.push(name);
        }

        info!("Starting {} weapons", language);
        for weapon in &configs.weapon["GMNCBMLIHPE"] {
            let id = weapon.id;

            let name = &configs.item["GMNCBMLIHPE"]
                .iter()
                .find(|i| i.id == weapon.id)
                .unwrap()
                .name;
            let name = text_map[name].clone();

            w_engines_id.push(id);
            w_engines_language.push(language);
            w_engines_name.push(name);
        }

        info!("Starting {} buddys", language);
        for buddy in &configs.buddy["GMNCBMLIHPE"] {
            let id = buddy.id;

            let name = &configs.item["GMNCBMLIHPE"]
                .iter()
                .find(|i| i.id == buddy.id)
                .unwrap()
                .name;
            let name = text_map[name].clone();

            bangboos_id.push(id);
            bangboos_language.push(language);
            bangboos_name.push(name);
        }
    }

    info!("Setting all achievement series texts");
    database::zzz::achievement_series_text::set_all(
        &achievement_series_id,
        &achievement_series_language,
        &achievement_series_name,
        pool,
    )
    .await?;

    info!("Setting all achievements texts");
    database::zzz::achievements_text::set_all(
        &achievements_id,
        &achievements_language,
        &achievements_name,
        &achievements_description,
        pool,
    )
    .await?;

    info!("Setting all character texts");
    database::zzz::characters_text::set_all(
        &characters_id,
        &characters_language,
        &characters_name,
        pool,
    )
    .await?;

    info!("Setting all w-engines texts");
    database::zzz::w_engines_text::set_all(
        &w_engines_id,
        &w_engines_language,
        &w_engines_name,
        pool,
    )
    .await?;

    info!("Setting all bangboos texts");
    database::zzz::bangboos_text::set_all(&bangboos_id, &bangboos_language, &bangboos_name, pool)
        .await?;

    Ok(())
}
