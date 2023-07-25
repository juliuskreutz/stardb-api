use std::{collections::HashMap, time::Duration};

use actix_web::rt::{self, time};
use regex::{Captures, Regex};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    database::{self, DbAchievement, DbCharacter, DbSeries},
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
    #[serde(rename = "AvatarName")]
    name: TextHash,
    #[serde(rename = "AvatarVOTag")]
    tag: String,
}

#[derive(Deserialize)]
struct TextHash {
    #[serde(rename = "Hash")]
    hash: i64,
}

pub async fn achievements(pool: PgPool) {
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

    let text_map: HashMap<String, String> = reqwest::get(&format!("{url}TextMap/TextMapEN.json"))
        .await?
        .json()
        .await?;

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

    for series in achievement_series.values() {
        let id = series.id;
        let name = text_map[&series.title.hash.to_string()].clone();
        let priority = series.priority;

        let db_series = DbSeries { id, name, priority };
        database::set_series(&db_series, pool).await?;
    }

    for achievement_data in achievement_data.values() {
        let html_re = Regex::new(r"<[^>]*>")?;

        let id = achievement_data.id;

        let series_id = achievement_data.series;

        let title = html_re
            .replace_all(
                &text_map[&achievement_data.title.hash.to_string()],
                |_: &Captures| "",
            )
            .to_string();

        let re = Regex::new(r"#(\d+)\[i\](%?)")?;
        let description = re
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
        let description = html_re
            .replace_all(&description, |_: &Captures| "")
            .replace("\\n", "");

        let jades = reward_data[&quest_data[&id.to_string()].reward_id.to_string()]
            .jades
            .unwrap_or_default();

        let hidden = achievement_data.show_type.as_deref() == Some("ShowAfterFinish");

        let db_achievement = DbAchievement {
            id,
            series_id,
            title,
            description,
            jades,
            hidden,
            ..Default::default()
        };

        database::set_achievement(&db_achievement, pool).await?;
    }

    let avatar_config: HashMap<String, AvatarConfig> =
        reqwest::get(&format!("{url}ExcelOutput/AvatarConfig.json"))
            .await?
            .json()
            .await?;

    for avatar_config in avatar_config.values() {
        let name = text_map[&avatar_config.name.hash.to_string()].clone();

        if name == "{NICKNAME}" {
            continue;
        }

        let tag = avatar_config.tag.clone();

        let db_character = DbCharacter { tag, name };

        database::set_character(&db_character, pool).await?;
    }

    let trailblazer_phys = DbCharacter {
        tag: "trailblazerphys".to_string(),
        name: "Trailblazer\u{00A0}•\u{00A0}Physical".to_string(),
    };

    let trailblazer_fire = DbCharacter {
        tag: "trailblazerfire".to_string(),
        name: "Trailblazer\u{00A0}•\u{00A0}Fire".to_string(),
    };

    database::set_character(&trailblazer_phys, pool).await?;
    database::set_character(&trailblazer_fire, pool).await?;

    Ok(())
}
