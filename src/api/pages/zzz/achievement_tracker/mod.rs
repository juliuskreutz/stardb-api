use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
    time::Duration,
};

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use async_rwlock::RwLock;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::api::pages::achievement_tracker_core as core;
use crate::{
    api::{private, ApiResult, Language, LanguageParams},
    app_config::AppConfig,
    database, Difficulty,
};
use std::sync::Arc;

lazy_static::lazy_static! {
    static ref CACHE: Mutex<Option<web::Data<ZzzAchievementTrackerCache>>> = Mutex::new(None);
}

#[derive(OpenApi)]
#[openapi(paths(get_zzz_achievement_tracker))]
struct ApiDoc;

/// Return the OpenAPI description for these routes, including registered child routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Register this game's tracker routes and reuse its process-wide, type-distinct cache.
pub fn configure(
    cfg: &mut web::ServiceConfig,
    pool: PgPool,
    app_config: web::Data<Arc<AppConfig>>,
) {
    let data = CACHE
        .lock()
        .unwrap()
        .get_or_insert_with(|| cache(pool, app_config))
        .clone();

    cfg.service(get_zzz_achievement_tracker).app_data(data);
}

#[derive(Default)]
pub struct ZzzAchievementTrackerCache {
    achievement_tracker_map: RwLock<HashMap<Language, AchievementTracker>>,
}

type AchievementTracker = core::Tracker<Achievement, Extra>;
#[derive(Clone, Serialize, Deserialize)]
struct Extra {}

#[derive(Clone, Serialize, Deserialize)]
struct Achievement {
    id: i32,
    series: i32,
    series_name: String,
    series_index: usize,
    name: String,
    description: String,
    currency: i32,
    hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<Difficulty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<String>,
    gacha: bool,
    impossible: bool,
    percent: f64,
    arcade: bool,
}

impl From<database::zzz::achievements::DbAchievement> for Achievement {
    /// Convert localized catalog metadata to this game's tracker payload; impossible entries report zero completion percent.
    fn from(db_achievement: database::zzz::achievements::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            series_index: 0,
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            currency: db_achievement.polychromes,
            hidden: db_achievement.hidden,
            version: db_achievement.version.clone(),
            difficulty: db_achievement
                .difficulty
                .as_ref()
                .map(|d| d.parse().unwrap()),
            video: db_achievement.video.clone(),
            gacha: db_achievement.gacha,
            impossible: db_achievement.impossible,
            percent: if !db_achievement.impossible {
                db_achievement.percent.unwrap_or_default()
            } else {
                0.0
            },
            arcade: db_achievement.arcade,
        }
    }
}

impl core::Item for Achievement {
    /// Return the ID used by shared completion/favorite annotation.
    fn id(&self) -> i32 {
        self.id
    }
    /// Expose this game's reward field to the shared tracker totals.
    fn currency(&self) -> i32 {
        self.currency
    }
    /// Store the rendered series index in this game's achievement payload.
    fn set_series_index(&mut self, index: usize) {
        self.series_index = index;
    }
}

/// Load this game's persisted cache and start refreshes only when tracker updates are enabled.
pub fn cache(
    pool: PgPool,
    app_config: web::Data<Arc<AppConfig>>,
) -> web::Data<ZzzAchievementTrackerCache> {
    let achievement_tracker_map = RwLock::new(core::load("cache/zzz_achievement_tracker_map.json"));

    let achievement_tracker_cache = web::Data::new(ZzzAchievementTrackerCache {
        achievement_tracker_map,
    });

    {
        let achievement_tracker_cache = achievement_tracker_cache.clone();

        if app_config.enable_update_achievement_trackers {
            crate::update::spawn_periodic(
                "Achievement tracker",
                Duration::from_secs(60),
                Duration::from_secs(10),
                move || update_achievement_tracker(achievement_tracker_cache.clone(), pool.clone()),
            );
        }
    }

    achievement_tracker_cache
}

/// Refresh every language before publishing this game's cache, preserving its explicit visibility policy.
async fn update_achievement_tracker(
    achievement_tracker_cache: web::Data<ZzzAchievementTrackerCache>,
    pool: PgPool,
) -> anyhow::Result<()> {
    let extra = Extra {};
    // HSR intentionally retains hidden impossible entries; GI/ZZZ filter them.
    core::refresh(
        &achievement_tracker_cache.achievement_tracker_map,
        pool,
        "cache/zzz_achievement_tracker_map.json",
        extra,
        true,
        |language, pool| async move { database::zzz::achievements::get_all(language, &pool).await },
        |a| core::Entry {
            set: a.set,
            series_name: a.series_name.clone(),
            version: a.version.clone(),
            hidden: a.hidden,
            impossible: a.impossible,
            achievement: Achievement::from(a),
        },
    )
    .await
}

#[utoipa::path(
    tag = "pages/zzz",
    get,
    path = "/api/pages/zzz/achievement-tracker",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "AchievementTracker"),
    )
)]
#[get("/api/pages/zzz/achievement-tracker", guard = "private")]
/// Clone the requested language cache and overlay the signed-in user's stored completion/favorite IDs.
/// Requires a populated language entry from disk or a completed background refresh.
async fn get_zzz_achievement_tracker(
    session: Session,
    language_params: web::Query<LanguageParams>,
    achievement_tracker_cache: web::Data<ZzzAchievementTrackerCache>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let mut achievement_tracker = achievement_tracker_cache
        .achievement_tracker_map
        .read()
        .await[&language_params.lang]
        .clone();

    if let Ok(Some(username)) = session.get::<String>("username") {
        let completed =
            database::zzz::users_achievements_completed::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|c| c.id)
                .collect::<HashSet<_>>();
        let favorites =
            database::zzz::users_achievements_favorites::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|c| c.id)
                .collect::<HashSet<_>>();

        achievement_tracker.annotate(&completed, &favorites);
    }

    Ok(HttpResponse::Ok().json(achievement_tracker))
}
