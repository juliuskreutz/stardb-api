use std::{collections::HashMap, fs::File, io::BufReader};

use regex::{Captures, Regex};
use sqlx::PgPool;

use crate::{database, Language};

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let param_re = Regex::new(r"#(\d+)(\[i\])?(%?)")?;

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
    let mut characters_path = Vec::new();
    let mut characters_element = Vec::new();

    let mut light_cones_id = Vec::new();
    let mut light_cones_language = Vec::new();
    let mut light_cones_name = Vec::new();
    let mut light_cones_path = Vec::new();

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
                "dimbreath/TurnBasedGameData/TextMap/TextMap{language_str}.json",
            ))?))?;

        info!("Starting {} achievement series", language);
        for achievement_series in &configs.achievement_series {
            let id = achievement_series.id;

            let name = html(&text_map[&achievement_series.title.hash.to_string()])?;

            achievement_series_id.push(id);
            achievement_series_language.push(language);
            achievement_series_name.push(name);
        }

        info!("Starting {} achievements", language);
        for achievement_data in &configs.achievement_data {
            let id = achievement_data.id;

            let name_key = &achievement_data.title.hash.to_string();
            let name = match text_map.get(name_key) {
                Some(val) => html(val)?,
                None => {
                    warn!(
                        "Missing achievement name hash: {} (id: {})",
                        name_key, id
                    );
                    "UNKNOWN".to_string()
                }
            };
            let name = gender(&name)?;

            let name = if id == 4074007 && language == Language::En {
                "The Conqueror King".to_string()
            } else {
                name
            };

            let name = if id == 4074020 && language == Language::En {
                "Truth is the Eternal Ultimate".to_string()
            } else {
                name
            };

            let name = if id == 4054010 && language == Language::Fr {
                "Danse avec les vagues et les bêtes".to_string()
            } else {
                name
            };

            let description_key = &achievement_data.description.hash.to_string();
            let description = match text_map.get(description_key) {
                Some(val) => html(val)?,
                None => {
                    warn!(
                        "Missing achievement description hash: {} (id: {})",
                        description_key, id
                    );
                    "UNKNOWN".to_string()
                }
            };
            let description = layout(&description)?;

            // Idk what's happening here. Leave this as is
            let mut description = param_re
                .replace_all(&description, |c: &Captures| {
                    let m = c.get(1).unwrap();
                    let i: usize = m.as_str().parse().unwrap();

                    if let Some(param) = achievement_data.param_list.get(i - 1) {
                        if c.get(2).is_some_and(|m| !m.is_empty())
                            && c.get(3).is_some_and(|m| !m.is_empty())
                        {
                            ((param.value * 100.0) as i32).to_string() + "%"
                        } else if c.get(3).is_some_and(|m| !m.is_empty()) {
                            param.value.to_string() + "%"
                        } else {
                            param.value.to_string()
                        }
                    } else {
                        c.get(0).unwrap().as_str().to_string()
                    }
                })
                // 6354779731002018877 = Trailblazer
                .replace("{NICKNAME}", &text_map["6354779731002018877"]);

            if language == Language::En {
                description = description.replace("{TEXTJOIN#54}", "Chris P. Bacon (Trotter)");
                description = description.replace("{TEXTJOIN#87}", "The Radiant Feldspar");
            }

            achievements_id.push(id);
            achievements_language.push(language);
            achievements_name.push(name);
            achievements_description.push(description);
        }

        info!("Starting {} avatars", language);
        for avatar_config in &configs.avatar_config {
            let element = text_map[&configs
                .damage_type
                .iter()
                .find(|dt| dt.id == avatar_config.element)
                .unwrap()
                .name
                .hash
                .to_string()]
                .clone();

            let name = match avatar_config.id {
                id if id > 8000 => {
                    // 6354779731002018877 = Trailblazer
                    let trail_blazer = text_map["6354779731002018877"].clone();

                    format!("{trail_blazer} ({element})")
                }
                _ => text_map[&avatar_config.name.hash.to_string()].clone(),
            };

            let name = gender(&name)?;
            let name = ruby(&name)?;

            let id = avatar_config.id;
            let path = ruby(
                &text_map[&configs
                    .avatar_base_type
                    .iter()
                    .find(|abt| abt.id.as_ref() == Some(&avatar_config.base_type))
                    .unwrap()
                    .text
                    .hash
                    .to_string()],
            )?;

            characters_id.push(id);
            characters_language.push(language);
            characters_name.push(name);
            characters_path.push(path);
            characters_element.push(element);
        }

        info!("Starting {} light cones", language);
        for equipment_config in &configs.equipment_config {
            let id = equipment_config.id;
            let name = ruby(&text_map[&equipment_config.name.hash.to_string()])?;
            let path = ruby(
                &text_map[&configs
                    .avatar_base_type
                    .iter()
                    .find(|abt| abt.id.as_ref() == Some(&equipment_config.base_type))
                    .unwrap()
                    .text
                    .hash
                    .to_string()],
            )?;

            light_cones_id.push(id);
            light_cones_language.push(language);
            light_cones_name.push(name);
            light_cones_path.push(path);
        }
    }

    info!("Setting all achievement series texts");
    database::achievement_series_text::set_all(
        &achievement_series_id,
        &achievement_series_language,
        &achievement_series_name,
        pool,
    )
    .await?;

    info!("Setting all achievement texts");
    database::achievements_text::set_all(
        &achievements_id,
        &achievements_language,
        &achievements_name,
        &achievements_description,
        pool,
    )
    .await?;

    info!("Setting all character texts");
    database::characters_text::set_all(
        &characters_id,
        &characters_language,
        &characters_name,
        &characters_path,
        &characters_element,
        pool,
    )
    .await?;

    info!("Setting all light cone texts");
    database::light_cones_text::set_all(
        &light_cones_id,
        &light_cones_language,
        &light_cones_name,
        &light_cones_path,
        pool,
    )
    .await?;

    Ok(())
}

fn html(s: &str) -> anyhow::Result<String> {
    Ok(Regex::new(r"<[^>]*>")?
        .replace_all(s, |_: &Captures| "")
        .to_string())
}

fn gender(s: &str) -> anyhow::Result<String> {
    Ok(Regex::new(r"\{(M|F)#([^}]*)\}\{(F|M)#([^}]*)\}")?
        .replace_all(s, |c: &Captures| {
            c.get(2).unwrap().as_str().to_string() + "/" + c.get(4).unwrap().as_str()
        })
        .to_string())
}

fn layout(s: &str) -> anyhow::Result<String> {
    Ok(Regex::new(
        r"\{LAYOUT_MOBILE#([^}]*)\}\{LAYOUT_CONTROLLER#([^}]*)\}\{LAYOUT_KEYBOARD#([^}]*)\}",
    )?
    .replace_all(s, |c: &Captures| {
        c.get(1).unwrap().as_str().to_string()
            + "/"
            + c.get(2).unwrap().as_str()
            + "/"
            + c.get(3).unwrap().as_str()
    })
    .to_string())
}

fn ruby(s: &str) -> anyhow::Result<String> {
    Ok(Regex::new(r"\{RUBY_.#.*?\}")?
        .replace_all(s, |_: &Captures| "")
        .to_string())
}
