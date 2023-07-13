use std::{collections::HashMap, time::Duration};

use actix_web::rt::{self, time};
use regex::{Captures, Regex};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    database::{self, DbAchievement},
    Result,
};

#[derive(Deserialize)]
struct AchievementData {
    #[serde(rename = "AchievementID")]
    id: i64,
    #[serde(rename = "SeriesID")]
    series: usize,
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
    #[serde(rename = "SeriesTitle")]
    title: TextHash,
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

    for achievement_data in achievement_data.values() {
        let html_re = Regex::new(r"<[^>]*>")?;

        let id = achievement_data.id;

        let series = text_map[&achievement_series[&achievement_data.series.to_string()]
            .title
            .hash
            .to_string()]
            .clone();

        let title = html_re
            .replace_all(
                &text_map[&achievement_data.title.hash.to_string()],
                |_: &Captures| "",
            )
            .to_string();

        let re = Regex::new(r"#(\d+)\[i\]")?;
        let description = re
            .replace_all(
                &text_map[&achievement_data.description.hash.to_string()],
                |c: &Captures| {
                    let m = c.get(1).unwrap();
                    let i: usize = m.as_str().parse().unwrap();

                    achievement_data.param_list[i - 1].value.to_string()
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
            series,
            title,
            description,
            jades,
            hidden,
            ..Default::default()
        };

        database::set_achievement(&db_achievement, pool).await?;
    }

    Ok(())
}
