use std::{
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

mod achievement_series;
mod achievements;
mod avatars;
mod light_cones;
mod texts;

use actix_web::rt::{self, Runtime};
use anyhow::Result;
use async_process::Command;
use serde::Deserialize;
use sqlx::PgPool;

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
    #[serde(rename = "QuestID")]
    quest_id: i32,
    #[serde(rename = "RewardID")]
    reward_id: i32,
}

#[derive(Deserialize)]
struct RewardData {
    #[serde(rename = "RewardID")]
    reward_id: i32,
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
    #[serde(rename = "ID")]
    id: Option<String>,
    #[serde(rename = "BaseTypeText")]
    text: TextHash,
}

#[derive(Deserialize)]
struct DamageType {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "DamageTypeName")]
    name: TextHash,
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
    hash: u64,
}

struct Configs {
    achievement_data: Vec<AchievementData>,
    achievement_series: Vec<AchievementSeries>,
    quest_data: Vec<QuestData>,
    reward_data: Vec<RewardData>,
    avatar_config: Vec<AvatarConfig>,
    avatar_base_type: Vec<AvatarBaseType>,
    damage_type: Vec<DamageType>,
    equipment_config: Vec<EquipmentConfig>,
}

pub async fn spawn(pool: PgPool) {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let handle = rt.spawn(async move {
            let mut interval = rt::time::interval(Duration::from_secs(60 * 10));

            let mut up_to_date = false;

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update(&mut up_to_date, pool.clone()).await {
                    error!(
                        "Dimbreath hsr update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Dimbreath hsr update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });

        rt.block_on(handle).unwrap();
    });
}

async fn update(up_to_date: &mut bool, pool: PgPool) -> Result<()> {
    if !Path::new("dimbreath").join("TurnBasedGameData").exists() {
        Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://gitlab.com/Dimbreath/TurnBasedGameData",
            ])
            .current_dir("dimbreath")
            .output()
            .await?;

        *up_to_date = false;
    }

    let output = String::from_utf8(
        Command::new("git")
            .arg("pull")
            .current_dir(Path::new("dimbreath").join("TurnBasedGameData"))
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

    let achievement_data: Vec<AchievementData> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/TurnBasedGameData/ExcelOutput/AchievementData.json")?,
    ))?;

    let achievement_series: Vec<AchievementSeries> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/TurnBasedGameData/ExcelOutput/AchievementSeries.json")?,
    ))?;

    let quest_data: Vec<QuestData> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/TurnBasedGameData/ExcelOutput/QuestData.json",
    )?))?;

    let reward_data: Vec<RewardData> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/TurnBasedGameData/ExcelOutput/RewardData.json",
    )?))?;

    let mut avatar_config: Vec<AvatarConfig> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/TurnBasedGameData/ExcelOutput/AvatarConfig.json")?,
    ))?;

    // collab characters avatar data (fate collab)
    let avatar_config_ld: Vec<AvatarConfig> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/TurnBasedGameData/ExcelOutput/AvatarConfigLD.json",
    )?))?;
    avatar_config.extend(avatar_config_ld);

    let avatar_base_type: Vec<AvatarBaseType> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/TurnBasedGameData/ExcelOutput/AvatarBaseType.json")?,
    ))?;

    let damage_type: Vec<DamageType> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/TurnBasedGameData/ExcelOutput/DamageType.json",
    )?))?;

    let equipment_config: Vec<EquipmentConfig> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/TurnBasedGameData/ExcelOutput/EquipmentConfig.json")?,
    ))?;

    let configs = Configs {
        achievement_data,
        achievement_series,
        quest_data,
        reward_data,
        avatar_config,
        avatar_base_type,
        damage_type,
        equipment_config,
    };

    info!("Parsed all json");

    info!("Starting achievement series");
    achievement_series::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting achievements");
    achievements::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting avatars");
    avatars::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting light cones");
    light_cones::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting texts");
    texts::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    *up_to_date = true;

    Ok(())
}
