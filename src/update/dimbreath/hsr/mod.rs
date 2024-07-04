use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

mod achievement_series;
mod achievements;
mod avatars;
mod book_series;
mod book_series_worlds;
mod books;
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

struct Configs {
    achievement_data: HashMap<String, AchievementData>,
    achievement_series: HashMap<String, AchievementSeries>,
    quest_data: HashMap<String, QuestData>,
    reward_data: HashMap<String, RewardData>,
    avatar_config: HashMap<String, AvatarConfig>,
    avatar_skill_config: HashMap<String, AvatarSkillConfigWrapper>,
    avatar_base_type: HashMap<String, AvatarBaseType>,
    damage_type: HashMap<String, DamageType>,
    localbook_config: HashMap<String, LocalbookConfig>,
    book_series_config: HashMap<String, BookSeriesConfig>,
    book_series_world: HashMap<String, BookSeriesWorld>,
    item_config: HashMap<String, ItemConfig>,
    item_config_book: HashMap<String, ItemConfig>,
    equipment_config: HashMap<String, EquipmentConfig>,
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

    let configs = Configs {
        achievement_data,
        achievement_series,
        quest_data,
        reward_data,
        avatar_config,
        avatar_skill_config,
        avatar_base_type,
        damage_type,
        localbook_config,
        book_series_config,
        book_series_world,
        item_config,
        item_config_book,
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

    info!("Starting book series worlds");
    book_series_worlds::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting book series");
    book_series::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting books");
    books::update(&configs, &pool).await?;
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
