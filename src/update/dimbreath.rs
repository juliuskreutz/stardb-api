use std::{collections::HashMap, time::Duration};

use actix_web::rt::{self, time};
use regex::{Captures, Regex};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    database::{self, DbAchievement, DbAchievementText, DbCharacter, DbSeries, DbSeriesText},
    Result,
};

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
    #[serde(rename = "AvatarName")]
    name: TextHash,
    #[serde(rename = "DamageType")]
    element: String,
    #[serde(rename = "AvatarBaseType")]
    base_type: String,
}

#[derive(Deserialize)]
struct AvatarBaseType {
    #[serde(rename = "FirstWordText")]
    path: String,
}

#[derive(Deserialize)]
struct TextHash {
    #[serde(rename = "Hash")]
    hash: i64,
}

pub async fn dimbreath(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 5));

        loop {
            interval.tick().await;

            if let Err(e) = update(&pool).await {
                println!("{e}");
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let url = "https://raw.githubusercontent.com/Dimbreath/StarRailData/master/";

    let languages = [
        "CHS", "CHT", "DE", "EN", "ES", "FR", "ID", "JP", "KR", "PT", "RU", "TH", "VI",
    ];

    let mut text_maps = HashMap::new();

    for language in languages {
        let text_map: HashMap<String, String> =
            reqwest::get(&format!("{url}TextMap/TextMap{language}.json"))
                .await?
                .json()
                .await?;

        text_maps.insert(language, text_map);
    }

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

    for language in languages {
        for series in achievement_series.values() {
            let html_re = Regex::new(r"<[^>]*>")?;
            let gender_re = Regex::new(r"\{M#([^}]*)\}\{F#([^}]*)\}")?;

            let id = series.id;

            let name = gender_re
                .replace_all(
                    &html_re.replace_all(
                        &text_maps[language][&series.title.hash.to_string()],
                        |_: &Captures| "",
                    ),
                    |c: &Captures| {
                        c.get(1).unwrap().as_str().to_string() + "/" + c.get(2).unwrap().as_str()
                    },
                )
                .to_string();

            let priority = series.priority;

            let db_series = DbSeries {
                id,
                priority,
                name: String::new(),
            };
            database::set_series(&db_series, pool).await?;

            let db_series_text = DbSeriesText {
                id,
                language: language.to_lowercase(),
                name,
            };

            database::set_series_text(&db_series_text, pool).await?;
        }
    }

    for language in languages {
        for achievement_data in achievement_data.values() {
            let html_re = Regex::new(r"<[^>]*>")?;

            let id = achievement_data.id;

            let series = achievement_data.series;

            let name = html_re
                .replace_all(
                    &text_maps[language][&achievement_data.title.hash.to_string()],
                    |_: &Captures| "",
                )
                .to_string();

            let re = Regex::new(r"#(\d+)\[i\](%?)")?;
            let description = re
                .replace_all(
                    &text_maps[language][&achievement_data.description.hash.to_string()],
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
            let description = html_re
                .replace_all(&description, |_: &Captures| "")
                .replace("\\n", "");

            let jades = reward_data[&quest_data[&id.to_string()].reward_id.to_string()]
                .jades
                .unwrap_or_default();

            let hidden = achievement_data.show_type.as_deref() == Some("ShowAfterFinish");

            let priority = achievement_data.priority;

            let db_achievement = DbAchievement {
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
                gacha: false,
                set: None,
                percent: None,
            };

            database::set_achievement(&db_achievement, pool).await?;

            let db_achievement_text = DbAchievementText {
                id,
                language: language.to_lowercase(),
                name,
                description,
            };

            database::set_achievement_text(&db_achievement_text, pool).await?;
        }
    }

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

    for avatar_config in avatar_config.values() {
        // /api/characters always uses EN text
        let mut name = text_maps["EN"][&avatar_config.name.hash.to_string()].clone();

        if name == "{NICKNAME}" {
            if avatar_config.id == 8001 {
                name = "Trailblazer\u{00A0}•\u{00A0}Physical".to_string();
            } else if avatar_config.id == 8003 {
                name = "Trailblazer\u{00A0}•\u{00A0}Fire".to_string();
            } else {
                continue;
            }
        }

        let id = avatar_config.id;
        let element = avatar_config.element.clone();
        let path = avatar_base_type[&avatar_config.base_type].path.clone();

        let db_character = DbCharacter {
            id,
            name,
            element,
            path,
        };

        database::set_character(&db_character, pool).await?;
    }

    Ok(())
}
