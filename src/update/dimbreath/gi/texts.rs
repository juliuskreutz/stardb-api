use std::{collections::HashMap, fs::File, io::BufReader};

use regex::{Captures, Regex};
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
    let mut characters_path = Vec::new();
    let mut characters_element = Vec::new();

    let mut skills_id = Vec::new();
    let mut skills_language = Vec::new();
    let mut skills_name = Vec::new();

    let mut book_series_worlds_id = Vec::new();
    let mut book_series_worlds_language = Vec::new();
    let mut book_series_worlds_name = Vec::new();

    let mut book_series_id = Vec::new();
    let mut book_series_language = Vec::new();
    let mut book_series_name = Vec::new();

    let mut books_id = Vec::new();
    let mut books_language = Vec::new();
    let mut books_name = Vec::new();

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
                "dimbreath/StarRailData/TextMap/TextMap{language_str}.json",
            ))?))?;

        info!("Starting {} achievement series", language);
        for achievement_series in configs.achievement_series.values() {
            let id = achievement_series.id;

            let name = html(&text_map[&achievement_series.title.hash.to_string()])?;
            let name = gender(&name)?;

            achievement_series_id.push(id);
            achievement_series_language.push(language);
            achievement_series_name.push(name);
        }

        info!("Starting {} achievements", language);
        for achievement_data in configs.achievement_data.values() {
            let id = achievement_data.id;

            let name = html(&text_map[&achievement_data.title.hash.to_string()])?;
            let name = gender(&name)?;

            let description = html(&text_map[&achievement_data.description.hash.to_string()])?;
            let description = gender(&description)?;
            let description = layout(&description)?;

            // Idk what's happening here. Leave this as is
            let param_re = Regex::new(r"#(\d+)(\[i\])?(%?)")?;
            let mut description = param_re
                .replace_all(&description, |c: &Captures| {
                    let m = c.get(1).unwrap();
                    let i: usize = m.as_str().parse().unwrap();

                    if let Some(param) = achievement_data.param_list.get(i - 1) {
                        if c.get(2).map_or(false, |m| !m.is_empty())
                            && c.get(3).map_or(false, |m| !m.is_empty())
                        {
                            ((param.value * 100.0) as i32).to_string() + "%"
                        } else if c.get(3).map_or(false, |m| !m.is_empty()) {
                            param.value.to_string() + "%"
                        } else {
                            param.value.to_string()
                        }
                    } else {
                        c.get(0).unwrap().as_str().to_string()
                    }
                })
                // -2090701432 = Trailblazer
                .replace("{NICKNAME}", &text_map["-2090701432"]);

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
        for avatar_config in configs.avatar_config.values() {
            let element = text_map[&configs.damage_type[&avatar_config.element]
                .name
                .hash
                .to_string()]
                .clone();

            let name = match avatar_config.id {
                id if id > 8000 => {
                    // -2090701432 = Trailblazer
                    let trail_blazer = text_map["-2090701432"].clone();

                    format!("{trail_blazer} ({element})")
                }
                _ => text_map[&avatar_config.name.hash.to_string()].clone(),
            };

            let name = gender(&name)?;
            let name = ruby(&name)?;

            let id = avatar_config.id;
            let path = ruby(
                &text_map[&configs.avatar_base_type[&avatar_config.base_type]
                    .text
                    .hash
                    .to_string()],
            )?;

            characters_id.push(id);
            characters_language.push(language);
            characters_name.push(name);
            characters_path.push(path);
            characters_element.push(element);

            for skill in avatar_config.skills.iter() {
                let skill = &configs.avatar_skill_config[&skill.to_string()].one;

                let id = skill.id;

                let name = ruby(&text_map[&skill.name.hash.to_string()])?;

                skills_id.push(id);
                skills_language.push(language);
                skills_name.push(name);
            }
        }

        info!("Starting {} book series worlds", language);
        for book_series_world in configs.book_series_world.values() {
            let id = book_series_world.id;

            let name = html(&text_map[&book_series_world.name.hash.to_string()])?;

            book_series_worlds_id.push(id);
            book_series_worlds_language.push(language);
            book_series_worlds_name.push(name);
        }

        info!("Starting {} book series", language);
        for book_series_config in configs.book_series_config.values() {
            let id = book_series_config.id;

            let name = html(&text_map[&book_series_config.name.hash.to_string()])?;

            book_series_id.push(id);
            book_series_language.push(language);
            book_series_name.push(name);
        }

        info!("Starting {} books", language);
        for localbook_config in configs.localbook_config.values() {
            let id = localbook_config.id;

            let name = html(&text_map[&localbook_config.name.hash.to_string()])?;

            books_id.push(id);
            books_language.push(language);
            books_name.push(name);
        }

        info!("Starting {} light cones", language);
        for equipment_config in configs.equipment_config.values() {
            let id = equipment_config.id;
            let name = ruby(&text_map[&equipment_config.name.hash.to_string()])?;
            let path = ruby(
                &text_map[&configs.avatar_base_type[&equipment_config.base_type]
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
    database::set_all_achievement_series_texts(
        &achievement_series_id,
        &achievement_series_language,
        &achievement_series_name,
        pool,
    )
    .await?;

    info!("Setting all achievement texts");
    database::set_all_achievement_texts(
        &achievements_id,
        &achievements_language,
        &achievements_name,
        &achievements_description,
        pool,
    )
    .await?;

    info!("Setting all character texts");
    database::set_all_character_texts(
        &characters_id,
        &characters_language,
        &characters_name,
        &characters_path,
        &characters_element,
        pool,
    )
    .await?;

    info!("Setting all skill texts");
    database::set_all_skill_texts(&skills_id, &skills_language, &skills_name, pool).await?;

    info!("Setting all book series world texts");
    database::set_all_book_series_world_texts(
        &book_series_worlds_id,
        &book_series_worlds_language,
        &book_series_worlds_name,
        pool,
    )
    .await?;

    info!("Setting all book series texts");
    database::set_all_book_series_texts(
        &book_series_id,
        &book_series_language,
        &book_series_name,
        pool,
    )
    .await?;

    info!("Setting all book texts");
    database::set_all_book_texts(&books_id, &books_language, &books_name, pool).await?;

    info!("Setting all light cone texts");
    database::set_all_light_cone_texts(
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
