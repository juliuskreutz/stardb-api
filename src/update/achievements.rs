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
struct TextHash {
    #[serde(rename = "Hash")]
    hash: i64,
}

pub async fn achievements(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 5));

        loop {
            interval.tick().await;

            let _ = update(&pool).await;
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

    for achievement_data in achievement_data.values() {
        let hrml_re = Regex::new(r"<[^>]*>")?;

        let id = achievement_data.id;

        let series = text_map[&achievement_series[&achievement_data.series.to_string()]
            .title
            .hash
            .to_string()]
            .clone();

        let title = hrml_re
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
        let description = hrml_re
            .replace_all(&description, |_: &Captures| "")
            .replace("\\n", "");

        let db_achievement = DbAchievement {
            id,
            series,
            title,
            description,
            ..Default::default()
        };

        database::set_achievement(&db_achievement, pool).await?;
    }

    Ok(())
}
