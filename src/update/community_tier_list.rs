use std::time::Duration;

use actix_web::rt::{self, time};
use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;

use crate::database::{self, DbCommunityTierListEntry};

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

            let _ = update(&pool).await;
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let spreadsheet: Spreadsheet = reqwest::get(format!("https://sheets.googleapis.com/v4/spreadsheets/1Ghi-Ryxr0AaKo2CA4xdCOkTh7gOE5dzheNSGEK2n2ZM?key={}&includeGridData=true&ranges=Stats!B2:F31&prettyPrint=true", dotenv::var("GOOGLE_API_KEY")?)).await?.json().await?;

    for row_data in &spreadsheet.sheets[0].data[0].row_data {
        let character = row_data.values[0].effective_value.number_value as i32;
        let eidolon = row_data.values[1].effective_value.number_value as i32;
        let average = row_data.values[2].effective_value.number_value;
        let variance = row_data.values[3].effective_value.number_value;
        let votes = row_data.values[4].effective_value.number_value as i32;

        let db_community_tier_list_entry = DbCommunityTierListEntry {
            character,
            eidolon,
            average,
            variance,
            votes,
            character_name: "".to_string(),
            character_path: "".to_string(),
            character_element: "".to_string(),
        };

        database::set_community_tier_list_entry(&db_community_tier_list_entry, pool).await?;
    }

    Ok(())
}