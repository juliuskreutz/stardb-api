use std::{
    collections::HashMap,
    env,
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

mod achievement_series;
mod achievements;
mod avatars;
mod buddys;
mod texts;
mod w_engines;

use actix_web::rt::{self, Runtime};
use async_process::Command;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
struct AchieveSecondClass {
    #[serde(rename = "BCHKIIKKBIA")]
    id: i32,
    #[serde(rename = "LEGJGOAMFMB")]
    name: String,
    #[serde(rename = "NOCALBFGANA")]
    priority: i32,
}

#[derive(serde::Deserialize)]
struct Achievement {
    #[serde(rename = "KBAPGDDLPLE")]
    id: i32,
    #[serde(rename = "KHOEDBBNFHJ")]
    series: i32,
    #[serde(rename = "EANFDCLPFPG")]
    name: String,
    #[serde(rename = "NHBKNJIGMEB")]
    description: String,
    #[serde(rename = "AFHCBPBBDEO")]
    reward: i32,
    #[serde(rename = "NDFEPHHEKDD")]
    hidden: i32,
    #[serde(rename = "BNJMODNCHPA")]
    priority: i32,
}

#[derive(serde::Deserialize)]
struct ArcadeAchievementGroup {
    #[serde(rename = "GKNMDKNIMHP")]
    id: i32,
    #[serde(rename = "OKLHNAJBFPM")]
    name: String,
}

#[derive(serde::Deserialize)]
struct ArcadeAchievement {
    #[serde(rename = "HCIEFCAMBMI")]
    id: i32,
    #[serde(rename = "MBNMIHLGMIO")]
    series: i32,
    #[serde(rename = "EANFDCLPFPG")]
    name: String,
    #[serde(rename = "EJKCPODPCLF")]
    description: String,
    #[serde(rename = "NHPGJJBILJF")]
    reward: i32,
}

#[derive(serde::Deserialize)]
struct Rewards {
    #[serde(rename = "NHPGJJBILJF")]
    id: i32,
    #[serde(rename = "FCLLNECAAPB")]
    rewards: Vec<Reward>,
}

#[derive(serde::Deserialize)]
struct Reward {
    #[serde(rename = "NKFGFANNKDH")]
    id: i32,
    #[serde(rename = "PPNNKNDFGMI")]
    amount: i32,
}

#[derive(serde::Deserialize)]
struct Item {
    #[serde(rename = "GKNMDKNIMHP")]
    id: i32,
    #[serde(rename = "FJECNNMMDGH")]
    name: String,
    #[serde(rename = "ELGACLKAKMD")]
    rarity: i32,
}

#[derive(serde::Deserialize)]
struct Avatar {
    #[serde(rename = "GKNMDKNIMHP")]
    id: i32,
    #[serde(rename = "FJECNNMMDGH")]
    name: String,
}

#[derive(serde::Deserialize)]
struct Weapon {
    #[serde(rename = "NKFGFANNKDH")]
    id: i32,
}

#[derive(serde::Deserialize)]
struct Buddy {
    #[serde(rename = "GKNMDKNIMHP")]
    id: i32,
}

struct Configs {
    achievement_second_class: HashMap<String, Vec<AchieveSecondClass>>,
    achievement: HashMap<String, Vec<Achievement>>,
    arcade_achievement_group: HashMap<String, Vec<ArcadeAchievementGroup>>,
    arcade_achievement: HashMap<String, Vec<ArcadeAchievement>>,
    once_reward: HashMap<String, Vec<Rewards>>,
    item: HashMap<String, Vec<Item>>,
    avatar: HashMap<String, Vec<Avatar>>,
    weapon: HashMap<String, Vec<Weapon>>,
    buddy: HashMap<String, Vec<Buddy>>,
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
                        "Dimbreath zzz update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Dimbreath zzz update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });

        rt.block_on(handle).unwrap();
    });
}

async fn update(up_to_date: &mut bool, pool: PgPool) -> anyhow::Result<()> {
    if !Path::new("dimbreath").join("ZenlessData").exists() {
        Command::new("git")
            .args(["clone", "--depth", "1", &env::var("ZENLESS_REPO")?])
            .current_dir("dimbreath")
            .output()
            .await?;

        *up_to_date = false;
    }

    let output = String::from_utf8(
        Command::new("git")
            .arg("pull")
            .current_dir(Path::new("dimbreath").join("ZenlessData"))
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

    let achievement_second_class: HashMap<String, Vec<AchieveSecondClass>> =
        serde_json::from_reader(BufReader::new(File::open(
            "dimbreath/ZenlessData/FileCfg/AchieveSecondClassConfigTemplateTb.json",
        )?))?;

    let achievement: HashMap<String, Vec<Achievement>> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/ZenlessData/FileCfg/AchievementTemplateTb.json")?,
    ))?;

    let arcade_achievement_group: HashMap<String, Vec<ArcadeAchievementGroup>> =
        serde_json::from_reader(BufReader::new(File::open(
            "dimbreath/ZenlessData/FileCfg/ArcadeAchievementGroupTemplateTb.json",
        )?))?;

    let arcade_achievement: HashMap<String, Vec<ArcadeAchievement>> =
        serde_json::from_reader(BufReader::new(File::open(
            "dimbreath/ZenlessData/FileCfg/ArcadeAchievementConfigTemplateTb.json",
        )?))?;

    let once_reward: HashMap<String, Vec<Rewards>> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/ZenlessData/FileCfg/OnceRewardTemplateTb.json")?,
    ))?;

    let item: HashMap<String, Vec<Item>> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/ZenlessData/FileCfg/ItemTemplateTb.json",
    )?))?;

    let avatar: HashMap<String, Vec<Avatar>> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/ZenlessData/FileCfg/AvatarBaseTemplateTb.json")?,
    ))?;

    let weapon: HashMap<String, Vec<Weapon>> = serde_json::from_reader(BufReader::new(
        File::open("dimbreath/ZenlessData/FileCfg/WeaponTemplateTb.json")?,
    ))?;

    let buddy: HashMap<String, Vec<Buddy>> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/ZenlessData/FileCfg/BuddyBaseTemplateTb.json",
    )?))?;

    let configs = Configs {
        achievement_second_class,
        achievement,
        arcade_achievement_group,
        arcade_achievement,
        once_reward,
        item,
        avatar,
        weapon,
        buddy,
    };

    info!("Starting achievement series");
    achievement_series::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting achievements");
    achievements::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting avatars");
    avatars::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting w-engines");
    w_engines::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting bangboos");
    buddys::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting texts");
    texts::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    *up_to_date = true;

    Ok(())
}
