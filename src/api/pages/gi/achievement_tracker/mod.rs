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
    static ref CACHE: Mutex<Option<web::Data<GiAchievementTrackerCache>>> = Mutex::new(None);
}

#[derive(OpenApi)]
#[openapi(paths(get_gi_achievement_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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

    cfg.service(get_gi_achievement_tracker).app_data(data);
}

#[derive(Default)]
pub struct GiAchievementTrackerCache {
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
}

impl From<database::gi::achievements::DbAchievement> for Achievement {
    fn from(db_achievement: database::gi::achievements::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            series_index: 0,
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            currency: db_achievement.primogems,
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
        }
    }
}

impl core::Item for Achievement {
    fn id(&self) -> i32 {
        self.id
    }
    fn currency(&self) -> i32 {
        self.currency
    }
    fn set_series_index(&mut self, index: usize) {
        self.series_index = index;
    }
}

pub fn cache(
    pool: PgPool,
    app_config: web::Data<Arc<AppConfig>>,
) -> web::Data<GiAchievementTrackerCache> {
    let achievement_tracker_map = RwLock::new(core::load("cache/gi_achievement_tracker_map.json"));

    let achievement_tracker_cache = web::Data::new(GiAchievementTrackerCache {
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

async fn update_achievement_tracker(
    achievement_tracker_cache: web::Data<GiAchievementTrackerCache>,
    pool: PgPool,
) -> anyhow::Result<()> {
    let extra = Extra {};
    // HSR intentionally retains hidden impossible entries; GI/ZZZ filter them.
    core::refresh(
        &achievement_tracker_cache.achievement_tracker_map,
        pool,
        "cache/gi_achievement_tracker_map.json",
        extra,
        true,
        |language, pool| async move { database::gi::achievements::get_all(language, &pool).await },
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
    tag = "pages/gi",
    get,
    path = "/api/pages/gi/achievement-tracker",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "AchievementTracker"),
    )
)]
#[get("/api/pages/gi/achievement-tracker", guard = "private")]
async fn get_gi_achievement_tracker(
    session: Session,
    language_params: web::Query<LanguageParams>,
    achievement_tracker_cache: web::Data<GiAchievementTrackerCache>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let mut achievement_tracker = achievement_tracker_cache
        .achievement_tracker_map
        .read()
        .await[&language_params.lang]
        .clone();

    if let Ok(Some(username)) = session.get::<String>("username") {
        let completed =
            database::gi::users_achievements_completed::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|c| c.id)
                .collect::<HashSet<_>>();
        let favorites =
            database::gi::users_achievements_favorites::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|c| c.id)
                .collect::<HashSet<_>>();

        achievement_tracker.annotate(&completed, &favorites);
    }

    Ok(HttpResponse::Ok().json(achievement_tracker))
}
