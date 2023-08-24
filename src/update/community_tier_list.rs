use std::{
    env,
    time::{Duration, Instant},
};

use actix_web::rt::{self, time};
use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;

use crate::database;

#[derive(Deserialize)]
struct Spreadsheet {
    sheets: Vec<Sheet>,
}

#[derive(Deserialize)]
struct Sheet {
    data: Vec<Data>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    row_data: Vec<RowData>,
}

#[derive(Deserialize)]
struct RowData {
    values: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Value {
    effective_value: EffectiveValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EffectiveValue {
    number_value: f64,
}

pub async fn community_tier_list(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 5));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
                log::error!(
                    "Community Tier List update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                log::info!(
                    "Community Tier List update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let key = env::var("GOOGLE_API_KEY")?;

    let spreadsheet: Spreadsheet = reqwest::get(format!("https://sheets.googleapis.com/v4/spreadsheets/1Ghi-Ryxr0AaKo2CA4xdCOkTh7gOE5dzheNSGEK2n2ZM?key={key}&includeGridData=true&ranges=Stats!J2")).await?.json().await?;

    let total_votes = spreadsheet.sheets[0].data[0].row_data[0].values[0]
        .effective_value
        .number_value as i32;

    let spreadsheet: Spreadsheet = reqwest::get(format!("https://sheets.googleapis.com/v4/spreadsheets/1Ghi-Ryxr0AaKo2CA4xdCOkTh7gOE5dzheNSGEK2n2ZM?key={key}&includeGridData=true&ranges=Stats!B2:I31")).await?.json().await?;

    for row_data in &spreadsheet.sheets[0].data[0].row_data {
        let character = row_data.values[0].effective_value.number_value as i32;
        let eidolon = row_data.values[1].effective_value.number_value as i32;
        let average = row_data.values[2].effective_value.number_value;
        let variance = row_data.values[3].effective_value.number_value;
        let quartile_1 = row_data.values[4].effective_value.number_value;
        let quartile_3 = row_data.values[5].effective_value.number_value;
        let confidence_interval_95 = row_data.values[6].effective_value.number_value;
        let votes = row_data.values[7].effective_value.number_value as i32;

        let db_community_tier_list_entry = database::DbCommunityTierListEntry {
            character,
            eidolon,
            average,
            variance,
            quartile_1,
            quartile_3,
            confidence_interval_95,
            votes,
            total_votes,
            character_rarity: 0,
            character_name: "".to_string(),
            character_path: "".to_string(),
            character_element: "".to_string(),
        };

        database::set_community_tier_list_entry(&db_community_tier_list_entry, pool).await?;
    }

    Ok(())
}
