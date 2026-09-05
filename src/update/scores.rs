use std::time::Instant;

use actix_web::rt;
use anyhow::Result;
use chrono::Utc;
use regex::{Captures, Regex};
use sqlx::PgPool;

use crate::{database, mihomo, Language};

pub async fn spawn(pool: PgPool) {
    let top_pool = pool.clone();
    super::spawn_periodic(
        "Scores top 100",
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(30),
        move || update_top_100(top_pool.clone()),
    );
    super::spawn_periodic(
        "Scores lower 100",
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(30),
        move || update_lower_100(pool.clone()),
    );
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
    signature: Option<String>,
    head_icon: i32,
    record_info: RecordInfo,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordInfo {
    achievement_count: i32,
}

/// A persistent failure exhausts this finite schedule, allowing the next UID to run.
/// Failed UIDs remain eligible on the next outer pass; one corrupt cache cannot block
/// the rest of the leaderboard indefinitely. The first delay also paces successful UIDs.
fn retry_delays() -> impl Iterator<Item = std::time::Duration> {
    [5, 10, 20].into_iter().map(std::time::Duration::from_secs)
}

async fn update_scores(uids: Vec<i32>, pool: &PgPool) -> Result<()> {
    for uid in uids {
        for (attempt, delay) in retry_delays().enumerate() {
            rt::time::sleep(delay).await;
            match update_score(uid, pool).await {
                Ok(()) => break,
                Err(e) => warn!("Score uid {uid} attempt {} failed: {e}", attempt + 1),
            }
        }
    }

    Ok(())
}

async fn update_score(uid: i32, pool: &PgPool) -> Result<()> {
    let now = Utc::now();

    if mihomo::update_and_get(uid, Language::En, pool)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let client = reqwest::Client::new();

    let enka: Enka = match client
        .get(format!("https://enka.network/api/hsr/uid/{uid}?info"))
        .header(reqwest::header::USER_AGENT, "stardb")
        .send()
        .await
    {
        Ok(r) => {
            let status = r.status();
            if !status.is_success() {
                warn!("Enka request for uid {uid} failed with status {status}");
                return Ok(());
            }

            match r.json().await {
                Ok(enka) => enka,
                Err(e) => {
                    warn!("{e}");
                    return Ok(());
                }
            }
        }
        Err(e) => {
            error!("{e}");
            return Ok(());
        }
    };

    let re = Regex::new(r"<[^>]*>")?;

    let name = re
        .replace_all(&enka.detail_info.nickname, |_: &Captures| "")
        .to_string();
    let region = mihomo::region_for_uid(uid).to_string();
    let level = enka.detail_info.level;
    let signature = re
        .replace_all(
            &enka.detail_info.signature.unwrap_or_default(),
            |_: &Captures| "",
        )
        .to_string();
    let avatar_icon = mihomo::get(uid, Language::En, pool)
        .await?
        .and_then(|v| serde_json::from_value::<mihomo::Mihomo>(v).ok())
        .map(|m| m.player.avatar.icon)
        .unwrap_or(format!("icon/avatar/{}.png", enka.detail_info.head_icon));
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

    let db_score_achievement =
        database::achievement_scores::DbScoreAchievementWrite { uid, timestamp };

    database::achievement_scores::set(&db_score_achievement, pool).await?;

    Ok(())
}

#[cfg(test)]
mod retry_tests {
    #[test]
    fn retry_policy_is_finite_with_increasing_backoff() {
        let mut schedule = super::retry_delays();
        assert_eq!(
            schedule
                .by_ref()
                .map(|delay| delay.as_secs())
                .collect::<Vec<_>>(),
            vec![5, 10, 20]
        );
        assert!(schedule.next().is_none());
    }
}
