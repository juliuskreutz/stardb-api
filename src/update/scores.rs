//! Refresh leaderboard profiles with independent population passes and bounded per-UID retries.
//!
//! Enka is the primary source: the leaderboard only needs name, level, signature, icon and
//! achievement count, so full Mihomo profiles are fetched here only when Enka has nothing.
//! Mihomo profile caches are otherwise refreshed when users view them.

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use chrono::Utc;
use regex::{Captures, Regex};
use serde_json::Value;
use sqlx::PgPool;

use crate::{backoff::Backoff, database, mihomo, Language};

/// Cooldown shared by Enka requests after rate-limit responses.
static ENKA_BACKOFF: Backoff = Backoff::new("enka");

static ENKA_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(mihomo::USER_AGENT)
        .build()
        .expect("enka http client should build")
});

/// Player avatar id -> icon path, the same index Mihomo uses to build `player.avatar.icon`.
/// Icon file names are not derivable from the id (201302 -> `1302.png`,
/// 202006 -> `IconHead_202006.png`), so Enka's head icon id must be looked up here.
const AVATARS_URL: &str =
    "https://raw.githubusercontent.com/Mar-7th/StarRailRes/master/index_new/en/avatars.json";
const AVATARS_REFRESH: Duration = Duration::from_secs(6 * 60 * 60);
const AVATARS_RETRY: Duration = Duration::from_secs(10 * 60);

struct AvatarIcons {
    next_refresh: Option<Instant>,
    icons: HashMap<i32, String>,
}

static AVATAR_ICONS: LazyLock<Mutex<AvatarIcons>> = LazyLock::new(|| {
    Mutex::new(AvatarIcons {
        next_refresh: None,
        icons: HashMap::new(),
    })
});

#[derive(serde::Deserialize)]
struct AvatarEntry {
    icon: String,
}

fn avatar_icons() -> std::sync::MutexGuard<'static, AvatarIcons> {
    AVATAR_ICONS.lock().unwrap_or_else(|e| e.into_inner())
}

async fn fetch_avatar_icons() -> Result<HashMap<i32, String>> {
    let entries: HashMap<String, AvatarEntry> = ENKA_CLIENT
        .get(AVATARS_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(entries
        .into_iter()
        .filter_map(|(id, entry)| Some((id.parse().ok()?, entry.icon)))
        .collect())
}

/// Icon path for an Enka head icon id. The index is refetched every few hours; on
/// failure the previous index is kept and the fetch is retried after a short delay.
async fn avatar_icon_for(head_icon: i32) -> Option<String> {
    let due = avatar_icons()
        .next_refresh
        .is_none_or(|next| Instant::now() >= next);
    if due {
        let result = fetch_avatar_icons().await;
        let mut state = avatar_icons();
        match result {
            Ok(icons) => {
                state.icons = icons;
                state.next_refresh = Some(Instant::now() + AVATARS_REFRESH);
            }
            Err(e) => {
                warn!("Avatar icon index fetch failed: {e}");
                state.next_refresh = Some(Instant::now() + AVATARS_RETRY);
            }
        }
    }
    avatar_icons().icons.get(&head_icon).cloned()
}

/// Start independent top-100 and remaining-population refresh loops with delayed retries.
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

/// Fetch the current top 100 UIDs and attempt each profile/score refresh.
async fn update_top_100(pool: PgPool) -> Result<()> {
    let uids = database::achievement_scores::get_refresh_uids(100, 0, &pool).await?;

    update_scores(uids, &pool).await?;

    Ok(())
}

/// Walk score pages below the top 100 until a page is empty, refreshing each UID.
async fn update_lower_100(pool: PgPool) -> Result<()> {
    for i in 0.. {
        let start = Instant::now();

        let offset = (i + 1) * 100;

        let uids = database::achievement_scores::get_refresh_uids(100, offset, &pool).await?;

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

/// Pace each UID through the finite retry schedule and continue after persistent failures.
/// Exhausted attempts are logged; failures do not permanently block later UIDs.
async fn update_scores(uids: Vec<i32>, pool: &PgPool) -> Result<()> {
    for uid in uids {
        wait_while_upstreams_paused().await;
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

/// Sleeps while both Enka and Mihomo are in a rate-limit cooldown, so UIDs reached
/// during the cooldown are not silently skipped.
async fn wait_while_upstreams_paused() {
    while let (Some(enka), Some(mihomo)) = (ENKA_BACKOFF.remaining(), mihomo::rate_limited()) {
        rt::time::sleep(enka.min(mihomo)).await;
    }
}

/// Fetches Enka's player summary. Rate limits, error statuses, transport and decode
/// failures are logged and return None; no request is sent during a cooldown.
async fn fetch_enka(uid: i32) -> Option<Enka> {
    if ENKA_BACKOFF.remaining().is_some() {
        return None;
    }

    let response = match ENKA_CLIENT
        .get(format!("https://enka.network/api/hsr/uid/{uid}?info"))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Enka request for uid {uid} failed: {e}");
            return None;
        }
    };

    let status = response.status();
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        ENKA_BACKOFF.rate_limited(crate::backoff::retry_after(response.headers()));
    }
    if !status.is_success() {
        warn!("Enka request for uid {uid} failed with status {status}");
        return None;
    }
    ENKA_BACKOFF.succeeded();

    match response.json().await {
        Ok(enka) => Some(enka),
        Err(e) => {
            warn!("Enka response for uid {uid} could not be decoded: {e}");
            None
        }
    }
}

/// Returns the cached Mihomo avatar icon path, used when the avatar index has no entry.
/// It may be outdated if the player changed avatar, but a valid path beats a guessed one.
fn cached_avatar_icon(cached: &Value) -> Option<String> {
    Some(
        cached
            .get("player")?
            .get("avatar")?
            .get("icon")?
            .as_str()?
            .to_string(),
    )
}

/// Refresh from Enka first; if Enka has nothing, fall back to a full Mihomo refresh.
/// Retain the old tie-breaking timestamp when the achievement count is unchanged.
/// Database and Mihomo transport errors propagate; Enka failures fall through to Mihomo.
async fn update_score(uid: i32, pool: &PgPool) -> Result<()> {
    let now = Utc::now();

    let Some(enka) = fetch_enka(uid).await else {
        let refreshed = mihomo::update_and_get(uid, Language::En, pool)
            .await?
            .is_some();
        debug!(uid, refreshed, "score refresh fell back to mihomo");
        return Ok(());
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
    // Prefer the index lookup (current avatar); fall back to the cached Mihomo icon
    // (cache-only: mihomo::get could trigger a Mihomo request per UID), then a guess.
    let head_icon = enka.detail_info.head_icon;
    let avatar_icon = match avatar_icon_for(head_icon).await {
        Some(icon) => icon,
        None => mihomo::cached_json(&Language::En, uid)
            .ok()
            .flatten()
            .and_then(|cached| cached_avatar_icon(&cached))
            .unwrap_or_else(|| format!("icon/avatar/{head_icon}.png")),
    };
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
    debug!(uid, achievement_count, "score refreshed from enka");

    Ok(())
}

#[cfg(test)]
mod avatar_icon_tests {
    use serde_json::json;

    use super::cached_avatar_icon;

    #[test]
    fn reads_cached_icon_path() {
        let cached = json!({"player": {"avatar": {"id": "202006", "icon": "icon/avatar/IconHead_202006.png"}}});
        assert_eq!(
            cached_avatar_icon(&cached).as_deref(),
            Some("icon/avatar/IconHead_202006.png")
        );
        assert_eq!(cached_avatar_icon(&json!({"player": {}})), None);
    }
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
