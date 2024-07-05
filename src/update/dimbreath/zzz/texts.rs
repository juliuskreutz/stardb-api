use std::{collections::HashMap, fs::File, io::BufReader};

use regex::{Captures, Regex};
use sqlx::PgPool;

use crate::{database, Language};

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
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
                "ZenlessData/TextMap/TextMap{language_str}TemplateTb.json",
            ))?))?;

        info!("Starting {} avatars", language);
        for avatar in &configs.avatar["GMNCBMLIHPE"] {
            let name = text_map[&avatar.name].clone();
            let name = ruby(&name)?;

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
            let name = ruby(&text_map[name])?;

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
            let name = ruby(&text_map[name])?;

            bangboos_id.push(id);
            bangboos_language.push(language);
            bangboos_name.push(name);
        }
    }

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
