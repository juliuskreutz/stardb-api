use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use async_process::Command;
use regex::{Captures, Regex};
use serde::Deserialize;
use sqlx::PgPool;
use strum::IntoEnumIterator;

use crate::{database, Language};

#[derive(Deserialize)]
struct AchievementData {
    #[serde(rename = "AchievementID")]
    id: i32,
    #[serde(rename = "SeriesID")]
    series: i32,
    #[serde(rename = "AchievementTitle")]
    title: TextHash,
    #[serde(rename = "AchievementDesc")]
    description: TextHash,
    #[serde(rename = "ParamList")]
    param_list: Vec<Param>,
    #[serde(rename = "ShowType")]
    show_type: Option<String>,
    #[serde(rename = "Priority")]
    priority: i32,
}

#[derive(Deserialize)]
struct Param {
    #[serde(rename = "Value")]
    value: f64,
}

#[derive(Deserialize)]
struct AchievementSeries {
    #[serde(rename = "SeriesID")]
    id: i32,
    #[serde(rename = "SeriesTitle")]
    title: TextHash,
    #[serde(rename = "Priority")]
    priority: i32,
}

#[derive(Deserialize)]
struct QuestData {
    #[serde(rename = "RewardID")]
    reward_id: i64,
}

#[derive(Deserialize)]
struct RewardData {
    #[serde(rename = "Hcoin")]
    jades: Option<i32>,
}

#[derive(Deserialize)]
struct AvatarConfig {
    #[serde(rename = "AvatarID")]
    id: i32,
    #[serde(rename = "Rarity")]
    rarity: String,
    #[serde(rename = "AvatarName")]
    name: TextHash,
    #[serde(rename = "DamageType")]
    element: String,
    #[serde(rename = "SkillList")]
    skills: Vec<i32>,
    #[serde(rename = "AvatarBaseType")]
    base_type: String,
}

#[derive(Deserialize)]
struct AvatarSkillConfigWrapper {
    #[serde(rename = "1")]
    one: AvatarSkillConfig,
}

#[derive(Deserialize)]
struct AvatarSkillConfig {
    #[serde(rename = "SkillID")]
    id: i32,
    #[serde(rename = "SkillName")]
    name: TextHash,
}

#[derive(Deserialize)]
struct AvatarBaseType {
    #[serde(rename = "BaseTypeText")]
    text: TextHash,
}

#[derive(Deserialize)]
struct DamageType {
    #[serde(rename = "DamageTypeName")]
    name: TextHash,
}

#[derive(Deserialize)]
struct LocalbookConfig {
    #[serde(rename = "BookID")]
    id: i32,
    #[serde(rename = "BookSeriesID")]
    series: i32,
    #[serde(rename = "BookSeriesInsideID")]
    series_inside: i32,
    #[serde(rename = "BookInsideName")]
    name: TextHash,
}

#[derive(Deserialize)]
struct BookSeriesConfig {
    #[serde(rename = "BookSeriesID")]
    id: i32,
    #[serde(rename = "BookSeriesWorld")]
    world: i32,
    #[serde(rename = "BookSeries")]
    name: TextHash,
    #[serde(rename = "IsShowInBookshelf")]
    bookshelf: Option<bool>,
}

#[derive(Deserialize)]
struct BookSeriesWorld {
    #[serde(rename = "BookSeriesWorld")]
    id: i32,
    #[serde(rename = "BookSeriesWorldTextmapID")]
    name: TextHash,
}

#[derive(Deserialize)]
struct ItemConfig {
    #[serde(rename = "ItemIconPath")]
    icon_path: String,
}

#[derive(Deserialize)]
struct EquipmentConfig {
    #[serde(rename = "EquipmentID")]
    id: i32,
    #[serde(rename = "Rarity")]
    rarity: String,
    #[serde(rename = "EquipmentName")]
    name: TextHash,
    #[serde(rename = "AvatarBaseType")]
    base_type: String,
}

#[derive(Deserialize)]
struct TextHash {
    #[serde(rename = "Hash")]
    hash: i64,
}

pub async fn spawn(pool: PgPool) {
    actix::Arbiter::new().spawn(async move {
        let mut interval = rt::time::interval(Duration::from_secs(60 * 10));

        let mut up_to_date = false;

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&mut up_to_date, pool.clone()).await {
                error!(
                    "Dimbreath update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                info!(
                    "Dimbreath update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(up_to_date: &mut bool, pool: PgPool) -> Result<()> {
    if !Path::new("StarRailData").exists() {
        Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/Dimbreath/StarRailData",
            ])
            .output()
            .await?;

        *up_to_date = false;
    }

    let output = String::from_utf8(
        Command::new("git")
            .arg("pull")
            .current_dir("StarRailData")
            .output()
            .await?
            .stdout,
    )?;

    if !output.contains("Already up to date.") {
        *up_to_date = false;
    }

    if *up_to_date {
        return Ok(());
    }

    let html_re = Regex::new(r"<[^>]*>")?;
    let gender_re = Regex::new(r"\{(M|F)#([^}]*)\}\{(F|M)#([^}]*)\}")?;
    let layout_re = Regex::new(
        r"\{LAYOUT_MOBILE#([^}]*)\}\{LAYOUT_CONTROLLER#([^}]*)\}\{LAYOUT_KEYBOARD#([^}]*)\}",
    )?;
    let rarity_re = Regex::new(r"CombatPowerAvatarRarityType(\d+)")?;
    let ruby_re = Regex::new(r"\{RUBY_.#.*?\}")?;

    let achievement_data: HashMap<String, AchievementData> = serde_json::from_reader(
        BufReader::new(File::open("StarRailData/ExcelOutput/AchievementData.json")?),
    )?;

    let achievement_series: HashMap<String, AchievementSeries> =
        serde_json::from_reader(BufReader::new(File::open(
            "StarRailData/ExcelOutput/AchievementSeries.json",
        )?))?;

    let quest_data: HashMap<String, QuestData> = serde_json::from_reader(BufReader::new(
        File::open("StarRailData/ExcelOutput/QuestData.json")?,
    ))?;

    let reward_data: HashMap<String, RewardData> = serde_json::from_reader(BufReader::new(
        File::open("StarRailData/ExcelOutput/RewardData.json")?,
    ))?;

    let avatar_config: HashMap<String, AvatarConfig> = serde_json::from_reader(BufReader::new(
        File::open("StarRailData/ExcelOutput/AvatarConfig.json")?,
    ))?;

    let avatar_skill_config: HashMap<String, AvatarSkillConfigWrapper> =
        serde_json::from_reader(BufReader::new(File::open(
            "StarRailData/ExcelOutput/AvatarSkillConfig.json",
        )?))?;

    let avatar_base_type: HashMap<String, AvatarBaseType> = serde_json::from_reader(
        BufReader::new(File::open("StarRailData/ExcelOutput/AvatarBaseType.json")?),
    )?;

    let damage_type: HashMap<String, DamageType> = serde_json::from_reader(BufReader::new(
        File::open("StarRailData/ExcelOutput/DamageType.json")?,
    ))?;

    let localbook_config: HashMap<String, LocalbookConfig> = serde_json::from_reader(
        BufReader::new(File::open("StarRailData/ExcelOutput/LocalbookConfig.json")?),
    )?;

    let book_series_config: HashMap<String, BookSeriesConfig> =
        serde_json::from_reader(BufReader::new(File::open(
            "StarRailData/ExcelOutput/BookSeriesConfig.json",
        )?))?;

    let book_series_world: HashMap<String, BookSeriesWorld> = serde_json::from_reader(
        BufReader::new(File::open("StarRailData/ExcelOutput/BookSeriesWorld.json")?),
    )?;

    let item_config: HashMap<String, ItemConfig> = serde_json::from_reader(BufReader::new(
        File::open("StarRailData/ExcelOutput/ItemConfig.json")?,
    ))?;

    let item_config_book: HashMap<String, ItemConfig> = serde_json::from_reader(BufReader::new(
        File::open("StarRailData/ExcelOutput/ItemConfigBook.json")?,
    ))?;

    let equipment_config: HashMap<String, EquipmentConfig> = serde_json::from_reader(
        BufReader::new(File::open("StarRailData/ExcelOutput/EquipmentConfig.json")?),
    )?;

    info!("Parsed all json");

    info!("Starting achievement_series");

    //for achievement_series in achievement_series.values() {
    //    let id = achievement_series.id;
    //
    //    let priority = achievement_series.priority;
    //
    //    let db_series = database::DbAchievementSeries {
    //        id,
    //        priority,
    //        name: String::new(),
    //    };
    //    database::set_achievement_series(&db_series, &pool).await?;
    //}

    info!("Starting achievements");

    //for achievement_data in achievement_data.values() {
    //    let id = achievement_data.id;
    //
    //    let series = achievement_data.series;
    //
    //    let jades = reward_data[&quest_data[&id.to_string()].reward_id.to_string()]
    //        .jades
    //        .unwrap_or_default();
    //
    //    let hidden = achievement_data.show_type.as_deref() == Some("ShowAfterFinish");
    //
    //    let priority = achievement_data.priority;
    //
    //    let db_achievement = database::achievements::DbAchievement {
    //        id,
    //        series,
    //        series_name: String::new(),
    //        name: String::new(),
    //        description: String::new(),
    //        jades,
    //        hidden,
    //        priority,
    //        version: None,
    //        comment: None,
    //        reference: None,
    //        difficulty: None,
    //        video: None,
    //        gacha: false,
    //        impossible: false,
    //        set: None,
    //        percent: 0.0,
    //    };
    //
    //    database::achievements::set(&db_achievement, &pool).await?;
    //}

    info!("Starting avatars");

    for avatar_config in avatar_config.values() {
        let id = avatar_config.id;

        let rarity = rarity_re
            .captures(&avatar_config.rarity)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or_default();

        let db_character = database::DbCharacter {
            id,
            rarity,
            name: String::new(),
            path: String::new(),
            element: String::new(),
            path_id: String::new(),
            element_id: String::new(),
        };

        database::set_character(&db_character, &pool).await?;

        for skill in avatar_config.skills.iter() {
            let skill = &avatar_skill_config[&skill.to_string()].one;

            let id = skill.id;

            let db_skill = database::DbSkill {
                id,
                character: avatar_config.id,
                name: String::new(),
            };

            database::set_skill(&db_skill, &pool).await?;
        }
    }

    info!("Starting book series worlds");

    for book_series_world in book_series_world.values() {
        let id = book_series_world.id;

        let db_series = database::DbBookSeriesWorld {
            id,
            name: String::new(),
        };

        database::set_book_series_world(&db_series, &pool).await?;
    }

    info!("Starting book series");

    for book_series_config in book_series_config.values() {
        let id = book_series_config.id;
        let world = book_series_config.world;
        let bookshelf = book_series_config.bookshelf.unwrap_or_default();

        let db_series = database::DbBookSeries {
            id,
            world,
            bookshelf,
            name: String::new(),
            world_name: String::new(),
        };

        database::set_book_series(&db_series, &pool).await?;
    }

    info!("Starting books");

    for localbook_config in localbook_config.values() {
        let id = localbook_config.id;
        let series = localbook_config.series;
        let series_inside = localbook_config.series_inside;

        let icon = item_config
            .get(&id.to_string())
            .or_else(|| item_config_book.get(&id.to_string()))
            .map(|ic| {
                ic.icon_path
                    .strip_prefix("SpriteOutput/ItemIcon/")
                    .unwrap()
                    .strip_suffix(".png")
                    .unwrap()
                    .parse()
                    .unwrap()
            });

        let db_book = database::DbBook {
            id,
            series,
            series_name: String::new(),
            series_world: 0,
            series_world_name: String::new(),
            series_inside,
            icon,
            name: String::new(),
            comment: None,
            image1: None,
            image2: None,
            percent: 0.0,
        };

        database::set_book(&db_book, &pool).await?;
    }

    info!("Starting light cones");

    for item_config_equipment in equipment_config.values() {
        let id = item_config_equipment.id;

        let rarity = match item_config_equipment.rarity.as_str() {
            "CombatPowerLightconeRarity3" => 3,
            "CombatPowerLightconeRarity4" => 4,
            "CombatPowerLightconeRarity5" => 5,
            _ => unreachable!(),
        };

        let db_light_cone = database::DbLightCone {
            id,
            rarity,
            name: String::new(),
            path: String::new(),
            path_id: String::new(),
        };

        database::set_light_cone(&db_light_cone, &pool).await?;
    }

    info!("Starting texts");

    for language in Language::iter() {
        info!("Starting {}", language);

        let mut text_map: HashMap<String, String> =
            serde_json::from_reader(BufReader::new(File::open(format!(
                "StarRailData/TextMap/TextMap{}.json",
                language.to_string().to_uppercase()
            ))?))?;

        // -1976918066 = Dan Heng (Imbibitor Lunae)
        *text_map.get_mut("-1976918066").unwrap() = match language {
            Language::Chs => "丹恒 (饮月)",
            Language::Cht => "丹恆 (飲月)",
            Language::Jp => "丹恒 (飲月)",
            Language::Kr => "단항 (음월)",
            Language::Pt => "Dan Heng (Embebidor Lunae)",
            Language::Ru => "Дань Хэн (Пожиратель Луны)",
            Language::Th => "Dan Heng (จ้าวยลจันทรา)",
            Language::Vi => "Dan Heng (Ẩm Nguyệt)",
            _ => "Dan Heng (Imbibitor Lunae)",
        }
        .to_string();

        info!("Starting {} achievement series", language);

        for achievement_series in achievement_series.values() {
            let id = achievement_series.id;

            let name = gender_re
                .replace_all(
                    &html_re.replace_all(
                        &text_map[&achievement_series.title.hash.to_string()],
                        |_: &Captures| "",
                    ),
                    |c: &Captures| {
                        c.get(2).unwrap().as_str().to_string() + "/" + c.get(4).unwrap().as_str()
                    },
                )
                .to_string();

            let db_series_text = database::DbAchievementSeriesText { id, language, name };

            database::set_achievement_series_text(&db_series_text, &pool).await?;
        }

        info!("Starting {} achievements", language);

        for achievement_data in achievement_data.values() {
            let id = achievement_data.id;

            let name = gender_re
                .replace_all(
                    &html_re.replace_all(
                        &text_map[&achievement_data.title.hash.to_string()],
                        |_: &Captures| "",
                    ),
                    |c: &Captures| {
                        c.get(2).unwrap().as_str().to_string() + "/" + c.get(4).unwrap().as_str()
                    },
                )
                .to_string();

            let description = layout_re
                .replace_all(
                    &gender_re.replace_all(
                        &html_re.replace_all(
                            &text_map[&achievement_data.description.hash.to_string()],
                            |_: &Captures| "",
                        ),
                        |c: &Captures| {
                            c.get(2).unwrap().as_str().to_string()
                                + "/"
                                + c.get(4).unwrap().as_str()
                        },
                    ),
                    |c: &Captures| {
                        c.get(1).unwrap().as_str().to_string()
                            + "/"
                            + c.get(2).unwrap().as_str()
                            + "/"
                            + c.get(3).unwrap().as_str()
                    },
                )
                .to_string();

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

            let db_achievement_text = database::DbAchievementText {
                id,
                language,
                name,
                description,
            };

            database::set_achievement_text(&db_achievement_text, &pool).await?;
        }

        info!("Starting {} avatars", language);

        for avatar_config in avatar_config.values() {
            let element =
                text_map[&damage_type[&avatar_config.element].name.hash.to_string()].clone();

            let name = match avatar_config.id {
                8001..=8004 => {
                    // -2090701432 = Trailblazer
                    let trail_blazer = text_map["-2090701432"].clone();

                    format!("{trail_blazer} ({element})")
                }
                _ => text_map[&avatar_config.name.hash.to_string()].clone(),
            };

            let name = ruby_re
                .replace_all(
                    &gender_re.replace_all(&name, |c: &Captures| {
                        c.get(2).unwrap().as_str().to_string() + "/" + c.get(4).unwrap().as_str()
                    }),
                    |_: &Captures| "",
                )
                .to_string();

            let id = avatar_config.id;
            let path = ruby_re
                .replace_all(
                    &text_map[&avatar_base_type[&avatar_config.base_type]
                        .text
                        .hash
                        .to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let db_character_text = database::DbCharacterText {
                id,
                language,
                name,
                path,
                element,
            };

            database::set_character_text(&db_character_text, &pool).await?;

            for skill in avatar_config.skills.iter() {
                let skill = &avatar_skill_config[&skill.to_string()].one;

                let id = skill.id;

                let name = ruby_re
                    .replace_all(&text_map[&skill.name.hash.to_string()], |_: &Captures| "")
                    .to_string();

                let db_skill_text = database::DbSkillText { id, language, name };

                database::set_skill_text(&db_skill_text, &pool).await?;
            }
        }

        info!("Starting {} book series worlds", language);

        for book_series_world in book_series_world.values() {
            let id = book_series_world.id;

            let name = html_re
                .replace_all(
                    &text_map[&book_series_world.name.hash.to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let db_book_series_world_text = database::DbBookSeriesWorldText { id, language, name };

            database::set_book_series_world_text(&db_book_series_world_text, &pool).await?;
        }

        info!("Starting {} book series", language);

        for book_series_config in book_series_config.values() {
            let id = book_series_config.id;

            let name = html_re
                .replace_all(
                    &text_map[&book_series_config.name.hash.to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let db_book_series_text = database::DbBookSeriesText { id, language, name };

            database::set_book_series_text(&db_book_series_text, &pool).await?;
        }

        info!("Starting {} books", language);

        for localbook_config in localbook_config.values() {
            let id = localbook_config.id;

            let name = html_re
                .replace_all(
                    &text_map[&localbook_config.name.hash.to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let db_book_text = database::DbBookText { id, language, name };

            database::set_book_text(&db_book_text, &pool).await?;
        }

        info!("Starting {} light cones", language);

        for equipment_config in equipment_config.values() {
            let id = equipment_config.id;
            let name = ruby_re
                .replace_all(
                    &text_map[&equipment_config.name.hash.to_string()],
                    |_: &Captures| "",
                )
                .to_string();
            let path = ruby_re
                .replace_all(
                    &text_map[&avatar_base_type[&equipment_config.base_type]
                        .text
                        .hash
                        .to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let db_light_cone_text = database::DbLightConeText {
                id,
                language,
                name,
                path,
            };

            database::set_light_cone_text(&db_light_cone_text, &pool).await?;
        }
    }

    *up_to_date = true;

    Ok(())
}
