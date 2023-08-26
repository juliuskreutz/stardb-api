use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use regex::{Captures, Regex};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::time;

use crate::database;

#[derive(Deserialize)]
struct AchievementData {
    #[serde(rename = "AchievementID")]
    id: i64,
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
    #[serde(rename = "AvatarBaseType")]
    base_type: String,
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
    id: i64,
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
    #[serde(rename = "BookSeries")]
    name: TextHash,
}

#[derive(Deserialize)]
struct TextHash {
    #[serde(rename = "Hash")]
    hash: i64,
}

pub async fn dimbreath(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 10));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
                log::error!(
                    "Dimbreath update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                log::info!(
                    "Dimbreath update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let html_re = Regex::new(r"<[^>]*>")?;
    let gender_re = Regex::new(r"\{M#([^}]*)\}\{F#([^}]*)\}")?;
    let rarity_re = Regex::new(r"CombatPowerAvatarRarityType(\d+)")?;

    let url = "https://raw.githubusercontent.com/Dimbreath/StarRailData/master/";

    let languages = [
        "CHS", "CHT", "DE", "EN", "ES", "FR", "ID", "JP", "KR", "PT", "RU", "TH", "VI",
    ];

    let achievement_data: HashMap<String, AchievementData> =
        reqwest::get(&format!("{url}ExcelOutput/AchievementData.json"))
            .await?
            .json()
            .await?;

    let achievement_series: HashMap<String, AchievementSeries> =
        reqwest::get(&format!("{url}ExcelOutput/AchievementSeries.json"))
            .await?
            .json()
            .await?;

    let quest_data: HashMap<String, QuestData> =
        reqwest::get(&format!("{url}ExcelOutput/QuestData.json"))
            .await?
            .json()
            .await?;

    let reward_data: HashMap<String, RewardData> =
        reqwest::get(&format!("{url}ExcelOutput/RewardData.json"))
            .await?
            .json()
            .await?;

    let avatar_config: HashMap<String, AvatarConfig> =
        reqwest::get(&format!("{url}ExcelOutput/AvatarConfig.json"))
            .await?
            .json()
            .await?;

    let avatar_base_type: HashMap<String, AvatarBaseType> =
        reqwest::get(&format!("{url}ExcelOutput/AvatarBaseType.json"))
            .await?
            .json()
            .await?;

    let damage_type: HashMap<String, DamageType> =
        reqwest::get(&format!("{url}ExcelOutput/DamageType.json"))
            .await?
            .json()
            .await?;

    let localbook_config: HashMap<String, LocalbookConfig> =
        reqwest::get(&format!("{url}ExcelOutput/LocalbookConfig.json"))
            .await?
            .json()
            .await?;

    let book_series_config: HashMap<String, BookSeriesConfig> =
        reqwest::get(&format!("{url}ExcelOutput/BookSeriesConfig.json"))
            .await?
            .json()
            .await?;

    for achievement_series in achievement_series.values() {
        let id = achievement_series.id;

        let priority = achievement_series.priority;

        let db_series = database::DbAchievementSeries {
            id,
            priority,
            name: String::new(),
        };
        database::set_achievement_series(&db_series, pool).await?;
    }

    for achievement_data in achievement_data.values() {
        let id = achievement_data.id;

        let series = achievement_data.series;

        let jades = reward_data[&quest_data[&id.to_string()].reward_id.to_string()]
            .jades
            .unwrap_or_default();

        let hidden = achievement_data.show_type.as_deref() == Some("ShowAfterFinish");

        let priority = achievement_data.priority;

        let db_achievement = database::DbAchievement {
            id,
            series,
            series_name: String::new(),
            name: String::new(),
            description: String::new(),
            jades,
            hidden,
            priority,
            version: None,
            comment: None,
            reference: None,
            difficulty: None,
            video: None,
            gacha: false,
            set: None,
            percent: 0.0,
        };

        database::set_achievement(&db_achievement, pool).await?;
    }

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
            element: String::new(),
            path: String::new(),
        };

        database::set_character(&db_character, pool).await?;
    }

    for book_series_config in book_series_config.values() {
        let id = book_series_config.id;

        let db_series = database::DbBookSeries {
            id,
            name: String::new(),
        };

        database::set_book_series(&db_series, pool).await?;
    }

    for localbook_config in localbook_config.values() {
        let id = localbook_config.id;
        let series = localbook_config.series;
        let series_inside = localbook_config.series_inside;

        let db_book = database::DbBook {
            id,
            series,
            series_name: String::new(),
            series_inside,
            name: String::new(),
            percent: 0.0,
        };

        database::set_book(&db_book, pool).await?;
    }

    for language in languages {
        let mut text_map: HashMap<String, String> =
            reqwest::get(&format!("{url}TextMap/TextMap{language}.json"))
                .await?
                .json()
                .await?;

        // -1976918066 = Dan Heng (Imbibitor Lunae)
        *text_map.get_mut("-1976918066").unwrap() = match language {
            "CHS" => "丹恒 (饮月)",
            "CHT" => "丹恆 (飲月)",
            "JP" => "丹恒 (飲月)",
            "KR" => "단항 (음월)",
            "PT" => "Dan Heng (Embebidor Lunae)",
            "RU" => "Дань Хэн (Пожиратель Луны)",
            "TH" => "Dan Heng (จ้าวยลจันทรา)",
            "VI" => "Dan Heng (Ẩm Nguyệt)",
            _ => "Dan Heng (Imbibitor Lunae)",
        }
        .to_string();

        for achievement_series in achievement_series.values() {
            let id = achievement_series.id;

            let name = gender_re
                .replace_all(
                    &html_re.replace_all(
                        &text_map[&achievement_series.title.hash.to_string()],
                        |_: &Captures| "",
                    ),
                    |c: &Captures| {
                        c.get(1).unwrap().as_str().to_string() + "/" + c.get(2).unwrap().as_str()
                    },
                )
                .to_string();

            let db_series_text = database::DbAchievementSeriesText {
                id,
                language: language.to_lowercase(),
                name,
            };

            database::set_achievement_series_text(&db_series_text, pool).await?;
        }

        for achievement_data in achievement_data.values() {
            let id = achievement_data.id;

            let name = gender_re
                .replace_all(
                    &html_re.replace_all(
                        &text_map[&achievement_data.title.hash.to_string()],
                        |_: &Captures| "",
                    ),
                    |c: &Captures| {
                        c.get(1).unwrap().as_str().to_string() + "/" + c.get(2).unwrap().as_str()
                    },
                )
                .to_string();

            let param_re = Regex::new(r"#(\d+)\[i\](%?)")?;
            let description = param_re
                .replace_all(
                    &text_map[&achievement_data.description.hash.to_string()],
                    |c: &Captures| {
                        let m = c.get(1).unwrap();
                        let i: usize = m.as_str().parse().unwrap();

                        if c.get(2).map_or(false, |m| !m.is_empty()) {
                            ((achievement_data.param_list[i - 1].value * 100.0) as i32).to_string()
                                + "%"
                        } else {
                            achievement_data.param_list[i - 1].value.to_string()
                        }
                    },
                )
                .to_string();
            let description = gender_re
                .replace_all(
                    &html_re.replace_all(&description, |_: &Captures| ""),
                    |c: &Captures| {
                        c.get(1).unwrap().as_str().to_string() + "/" + c.get(2).unwrap().as_str()
                    },
                )
                .to_string();

            let db_achievement_text = database::DbAchievementText {
                id,
                language: language.to_lowercase(),
                name,
                description,
            };

            database::set_achievement_text(&db_achievement_text, pool).await?;
        }

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

            let name = gender_re
                .replace_all(&name, |c: &Captures| {
                    c.get(1).unwrap().as_str().to_string() + "/" + c.get(2).unwrap().as_str()
                })
                .to_string();

            let id = avatar_config.id;
            let path = text_map[&avatar_base_type[&avatar_config.base_type]
                .text
                .hash
                .to_string()]
                .clone();

            let db_character_text = database::DbCharacterText {
                id,
                language: language.to_lowercase(),
                name,
                element,
                path,
            };

            database::set_character_text(&db_character_text, pool).await?;
        }

        for book_series_config in book_series_config.values() {
            let id = book_series_config.id;

            let name = html_re
                .replace_all(
                    &text_map[&book_series_config.name.hash.to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let db_book_series_text = database::DbBookSeriesText {
                id,
                language: language.to_lowercase(),
                name,
            };

            database::set_book_series_text(&db_book_series_text, pool).await?;
        }

        for localbook_config in localbook_config.values() {
            let id = localbook_config.id;

            let name = html_re
                .replace_all(
                    &text_map[&localbook_config.name.hash.to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let db_book_text = database::DbBookText {
                id,
                language: language.to_lowercase(),
                name,
            };

            database::set_book_text(&db_book_text, pool).await?;
        }
    }

    Ok(())
}
