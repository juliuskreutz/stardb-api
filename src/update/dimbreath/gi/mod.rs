use std::{
    fs::File,
    io::BufReader,
    path::Path,
    time::{Duration, Instant},
};

mod avatars;
mod texts;
mod weapons;

use actix_web::rt::{self, Runtime};
use async_process::Command;
use sqlx::PgPool;

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
                        "Dimbreath gi update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Dimbreath gi update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });

        rt.block_on(handle).unwrap();
    });
}

#[derive(serde::Deserialize)]
struct AvatarData {
    id: i32,
    #[serde(rename = "nameTextMapHash")]
    name: i64,
    #[serde(rename = "qualityType")]
    quality: String,
}

#[derive(serde::Deserialize)]
struct WeaponData {
    id: i32,
    #[serde(rename = "nameTextMapHash")]
    name: i64,
    #[serde(rename = "rankLevel")]
    rank: i32,
}

struct Configs {
    avatar_data: Vec<AvatarData>,
    weapon_data: Vec<WeaponData>,
}

async fn update(up_to_date: &mut bool, pool: PgPool) -> anyhow::Result<()> {
    if !Path::new("dimbreath").join("AnimeGameData").exists() {
        Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://gitlab.com/Dimbreath/AnimeGameData",
            ])
            .current_dir("dimbreath")
            .output()
            .await?;

        *up_to_date = false;
    }

    let output = String::from_utf8(
        Command::new("git")
            .arg("pull")
            .current_dir(Path::new("dimbreath").join("AnimeGameData"))
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

    let avatar_data: Vec<AvatarData> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/AnimeGameData/ExcelBinOutput/AvatarExcelConfigData.json",
    )?))?;

    let weapon_data: Vec<WeaponData> = serde_json::from_reader(BufReader::new(File::open(
        "dimbreath/AnimeGameData/ExcelBinOutput/WeaponExcelConfigData.json",
    )?))?;

    let configs = Configs {
        avatar_data,
        weapon_data,
    };

    info!("Starting avatars");
    avatars::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting weapons");
    weapons::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    info!("Starting texts");
    texts::update(&configs, &pool).await?;
    actix_web::rt::task::yield_now().await;

    Ok(())
}
