//! Mihomo profile fetching, localized disk caching and database profile/score refreshes.

use std::{fs::File, path::PathBuf, sync::LazyLock};

use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use utoipa::ToSchema;

use anyhow::{anyhow, Context, Result};

use crate::{backoff::Backoff, database, Language};

/// Cooldown shared by every Mihomo request after Cloudflare rate limits us.
static BACKOFF: Backoff = Backoff::new("mihomo");

/// Cached profiles older than this are refreshed when read.
const MAX_CACHE_AGE_HOURS: i64 = 24;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Mihomo {
    pub player: Player,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Player {
    pub nickname: String,
    pub level: i32,
    pub avatar: Avatar,
    pub signature: String,
    pub space_info: SpaceInfo,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Avatar {
    pub icon: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SpaceInfo {
    pub achievement_count: i32,
}

/// Identifies StarDB to upstream APIs; Mihomo asks high-frequency clients to put app info here.
pub(crate) const USER_AGENT: &str = "stardb";

/// Shared client so Mihomo requests carry the User-Agent and reuse connections.
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("mihomo http client should build")
});

/// Returns the Brotli cache path for a UID and mihomo language code.
fn cache_path(language: &Language, uid: i32) -> String {
    format!("mihomo/{}_{uid}.br", language.mihomo())
}

/// Decodes a cached profile, returning None for an absent file and errors for unreadable data.
fn load_cached(language: &Language, uid: i32) -> Result<Option<Mihomo>> {
    let path = cache_path(language, uid);
    if PathBuf::from(&path).exists() {
        let decompressor = brotli::Decompressor::new(File::open(&path)?, 4096);
        Ok(Some(serde_json::from_reader(decompressor)?))
    } else {
        Ok(None)
    }
}

/// Reads a cached profile's raw JSON without contacting Mihomo; None when absent.
pub(crate) fn cached_json(language: &Language, uid: i32) -> Result<Option<Value>> {
    let path = cache_path(language, uid);
    if PathBuf::from(&path).exists() {
        let decompressor = brotli::Decompressor::new(File::open(&path)?, 4096);
        Ok(Some(serde_json::from_reader(decompressor)?))
    } else {
        Ok(None)
    }
}

/// Whether Mihomo calls are currently paused after a rate limit.
pub(crate) fn rate_limited() -> Option<std::time::Duration> {
    BACKOFF.remaining()
}

/// Fetches upstream JSON, treating non-success HTTP status as an absent profile.
/// While a rate-limit cooldown is active no request is sent and None is returned.
/// Transport, body-read and JSON-decoding failures propagate with request context.
async fn fetch_json(url: &str, uid: i32, language: Language, label: &str) -> Result<Option<Value>> {
    if let Some(remaining) = BACKOFF.remaining() {
        debug!(uid, language = %language, label, remaining_secs = remaining.as_secs(), "mihomo paused, skipping request");
        return Ok(None);
    }

    let response = CLIENT
        .get(url)
        .send()
        .await
        .with_context(|| format!("{label} request failed for uid {uid} language {language}"))?;

    let status = response.status();
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        BACKOFF.rate_limited(crate::backoff::retry_after(response.headers()));
    } else if status.is_success() {
        BACKOFF.succeeded();
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            uid,
            language = %language,
            url = %url,
            status = %status,
            body = %body,
            label,
            "response was not successful"
        );
        return Ok(None);
    }

    let text = response.text().await.with_context(|| {
        format!("{label} response text retrieval failed for uid {uid} language {language}")
    })?;

    match serde_json::from_str(&text) {
        Ok(json) => Ok(Some(json)),
        Err(err) => {
            warn!(
                uid,
                language = %language,
                url = %url,
                status = %status,
                body = %text,
                label,
                error = %err,
                "response decode failed"
            );
            Err(anyhow!("mihomo response body parsing failed")).with_context(|| {
                format!("{label} json decode failed for uid {uid} language {language}")
            })
        }
    }
}

/// Returns a cached profile or refreshes a missing or stale cache.
/// A cache is stale once older than [`MAX_CACHE_AGE_HOURS`]; a localized cache is also
/// stale when the English cache is newer. If a stale refresh yields nothing (rate limit,
/// upstream error status), the stale cache is served instead.
pub async fn get(uid: i32, language: Language, pool: &PgPool) -> Result<Option<Value>> {
    let Some(cached_json) = cached_json(&language, uid)? else {
        return update_and_get(uid, language, pool).await;
    };
    let cached: Mihomo = serde_json::from_value(cached_json.clone())?;

    let max_age = chrono::Duration::try_hours(MAX_CACHE_AGE_HOURS).unwrap();
    let mut should_update = cached.updated_at < Utc::now() - max_age;
    if !should_update && language != Language::En {
        should_update = match load_cached(&Language::En, uid)? {
            Some(en) => cached.updated_at < en.updated_at,
            None => true,
        };
    }

    if !should_update {
        return Ok(Some(cached_json));
    }

    Ok(Some(
        update_and_get(uid, language, pool)
            .await?
            .unwrap_or(cached_json),
    ))
}

/// Fetches localized and English profile data and updates the cache, mihomo row and score timestamp.
/// An upstream non-success status returns None; decoding, cache and database errors propagate.
pub async fn update_and_get(uid: i32, language: Language, pool: &PgPool) -> Result<Option<Value>> {
    let now = Utc::now();
    debug!(uid, language = %language, "mihomo update_and_get start");

    let url = format!(
        "https://api.mihomo.me/sr_info_parsed/{uid}?lang={}&version=v2",
        language.mihomo()
    );

    let mut json = match fetch_json(&url, uid, language, "localized").await? {
        Some(json) => json,
        None => return Ok(None),
    };

    debug!(uid, language = %language, "fetched mihomo payload");
    if let Some(o) = json.as_object_mut() {
        o.insert("updated_at".to_string(), serde_json::to_value(now)?);
    }

    let (en_json, is_english) = if language == Language::En {
        debug!(uid, language = %language, "using fetched payload as english payload");
        (json.clone(), true)
    } else {
        let en_url = format!("https://api.mihomo.me/sr_info_parsed/{uid}?lang=en&version=v2",);
        debug!(uid, language = %language, url = %en_url, "fetching english mihomo payload");

        let mut en_json = match fetch_json(&en_url, uid, language, "english").await? {
            Some(json) => json,
            None => return Ok(None),
        };

        debug!(uid, language = %language, "fetched english mihomo payload");
        if let Some(o) = en_json.as_object_mut() {
            o.insert("updated_at".to_string(), serde_json::to_value(now)?);
        }
        (en_json, false)
    };

    if serde_json::from_value::<Mihomo>(json.clone()).is_ok() {
        let file = File::create(cache_path(&language, uid))?;

        let writer = brotli::CompressorWriter::new(file, 4096, 4, 22);

        serde_json::to_writer(writer, &json)?;
        debug!(uid, language = %language, "cached mihomo payload");
    } else {
        debug!(uid, language = %language, "skipped caching localized mihomo payload");
    }

    if !is_english && serde_json::from_value::<Mihomo>(en_json.clone()).is_ok() {
        let file = File::create(cache_path(&Language::En, uid))?;

        let writer = brotli::CompressorWriter::new(file, 4096, 4, 22);

        serde_json::to_writer(writer, &en_json)?;
        debug!(uid, language = %language, "cached english mihomo payload");
    } else if !is_english {
        debug!(uid, language = %language, "skipped caching english mihomo payload");
    }

    let mihomo: Mihomo = serde_json::from_value(en_json)?;

    let re = Regex::new(r"<[^>]*>")?;

    let name = re
        .replace_all(&mihomo.player.nickname, |_: &Captures| "")
        .to_string();
    let region = region_for_uid(uid).to_string();
    let level = mihomo.player.level;
    let avatar_icon = mihomo.player.avatar.icon.clone();
    let signature = re
        .replace_all(&mihomo.player.signature, |_: &Captures| "")
        .to_string();
    let achievement_count = mihomo.player.space_info.achievement_count;
    let updated_at = mihomo.updated_at;
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
    debug!(uid, language = %language, timestamp = %timestamp, "mihomo update_and_get complete");

    Ok(Some(json))
}

/// Maps UID prefixes 6/7/8-or-9 to NA/EU/Asia and all others to CN without validating ranges.
pub(crate) fn region_for_uid(uid: i32) -> crate::api::Region {
    use crate::api::Region;
    match uid.to_string().chars().next() {
        Some('6') => Region::Na,
        Some('7') => Region::Eu,
        Some('8' | '9') => Region::Asia,
        _ => Region::Cn,
    }
}
/// For an absent database row, consults the English profile and seeds an epoch stub on None.
/// Cache, transport and database errors propagate. A cached Some result is reused
/// as-is, so this path does not recreate a missing row from cached JSON.
pub(crate) async fn ensure_row(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    if !database::mihomo::exists(uid, pool).await? && get(uid, Language::En, pool).await?.is_none()
    {
        database::mihomo::set(
            &database::mihomo::DbMihomo {
                uid,
                region: region_for_uid(uid).to_string(),
                ..Default::default()
            },
            pool,
        )
        .await?;
    }
    Ok(())
}
#[cfg(test)]
mod region_tests {
    #[test]
    fn uid_regions() {
        for (uid, region) in [
            (600000001, "na"),
            (700000001, "eu"),
            (800000001, "asia"),
            (900000001, "asia"),
            (100000001, "cn"),
            (0, "cn"),
        ] {
            assert_eq!(super::region_for_uid(uid).to_string(), region);
        }
    }
}
