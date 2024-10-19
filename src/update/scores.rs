use std::time::{Duration, Instant};

use actix_web::rt::{self, Runtime};
use anyhow::Result;
use chrono::Utc;
use regex::{Captures, Regex};
use sqlx::PgPool;

use crate::database;

pub async fn spawn(pool: PgPool) {
    {
        let pool = pool.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            let handle = rt.spawn(async move {
                let mut interval = rt::time::interval(Duration::from_secs(60 * 20));

                loop {
                    interval.tick().await;

                    let start = Instant::now();

                    if let Err(e) = update_top_100(pool.clone()).await {
                        error!(
                            "Scores top 100 update failed with {e} in {}s",
                            start.elapsed().as_secs_f64()
                        );
                    } else {
                        info!(
                            "Scores top 100 update succeeded in {}s",
                            start.elapsed().as_secs_f64()
                        );
                    }
                }
            });

            rt.block_on(handle).unwrap();
        });
    }

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let handle = rt.spawn(async move {
            loop {
                let start = Instant::now();

                if let Err(e) = update_lower_100(pool.clone()).await {
                    error!(
                        "Scores lower 100 update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Scores lower 100 update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });

        rt.block_on(handle).unwrap();
    });
}

async fn update_top_100(pool: PgPool) -> Result<()> {
    let uids = database::achievement_scores::get(None, None, Some(100), None, &pool)
        .await?
        .into_iter()
        .map(|s| s.uid)
        .collect();

    update_scores(uids, &pool).await?;

    Ok(())
}

async fn update_lower_100(pool: PgPool) -> Result<()> {
    for i in 0.. {
        let start = Instant::now();

        let offset = (i + 1) * 100;

        let uids: Vec<_> =
            database::achievement_scores::get(None, None, Some(100), Some(offset), &pool)
                .await?
                .into_iter()
                .map(|s| s.uid)
                .collect();

        if uids.is_empty() {
            break;
        }

        update_scores(uids, &pool).await?;

        info!(
            "Scores lower 100 offset {offset} update succeeded in {}s",
            start.elapsed().as_secs_f64()
        );
    }

    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Enka {
    detail_info: DetailInfo,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailInfo {
    nickname: String,
    level: i32,
    signature: String,
    head_icon: i32,
    record_info: RecordInfo,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordInfo {
    achievement_count: i32,
}

async fn update_scores(uids: Vec<i32>, pool: &PgPool) -> Result<()> {
    for uid in uids {
        loop {
            rt::time::sleep(std::time::Duration::from_secs(1)).await;

            if update_score(uid, pool).await.is_ok() {
                break;
            }
        }
    }

    Ok(())
}

async fn update_score(uid: i32, pool: &PgPool) -> Result<()> {
    let now = Utc::now();

    let enka: Enka = reqwest::get(format!("https://enka.network/api/hsr/uid/{uid}?info",))
        .await?
        .json()
        .await?;

    let re = Regex::new(r"<[^>]*>")?;

    let name = re
        .replace_all(&enka.detail_info.nickname, |_: &Captures| "")
        .to_string();
    let region = match uid.to_string().chars().next() {
        Some('6') => "na",
        Some('7') => "eu",
        Some('8') | Some('9') => "asia",
        _ => "cn",
    }
    .to_string();
    let level = enka.detail_info.level;
    let avatar_icon = format!("icon/avatar/{}.png", enka.detail_info.head_icon);
    let signature = re
        .replace_all(&enka.detail_info.signature, |_: &Captures| "")
        .to_string();
    let achievement_count = enka.detail_info.record_info.achievement_count;
    let updated_at = now;
    let timestamp = database::achievement_scores::get_timestamp_by_uid(uid, pool)
        .await
        .ok()
        .and_then(|sd| {
            if sd.achievement_count == achievement_count {
                Some(sd.timestamp)
            } else {
                None
            }
        })
        .unwrap_or(
            now + match region.as_str() {
                "na" => chrono::Duration::try_hours(-5).unwrap(),
                "eu" => chrono::Duration::try_hours(1).unwrap(),
                _ => chrono::Duration::try_hours(8).unwrap(),
            },
        );

    let db_mihomo = database::mihomo::DbMihomo {
        uid,
        region,
        name,
        level,
        signature,
        avatar_icon,
        achievement_count,
        updated_at,
    };

    database::mihomo::set(&db_mihomo, pool).await?;

    let db_score_achievement = database::achievement_scores::DbScoreAchievement {
        uid,
        timestamp,
        ..Default::default()
    };

    database::achievement_scores::set(&db_score_achievement, pool).await?;

    Ok(())
}
