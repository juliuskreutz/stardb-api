use std::{collections::HashMap, fs::File, io::BufReader};

use sqlx::PgPool;

use crate::{database, Language};

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
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
