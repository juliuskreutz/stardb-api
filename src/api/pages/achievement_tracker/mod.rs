use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_session::Session;
use actix_web::{get, rt, web, HttpResponse, Responder};
use async_rwlock::RwLock;
use serde::Serialize;
use sqlx::PgPool;
use strum::{EnumString, IntoEnumIterator};
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, Language, LanguageParams},
    database,
};

lazy_static::lazy_static! {
    static ref CACHE: Mutex<Option<web::Data<AchievementTrackerCache>>> = Mutex::new(None);
}

#[derive(OpenApi)]
#[openapi(paths(get_achievement_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    let data = CACHE
        .lock()
        .unwrap()
        .get_or_insert_with(|| cache(pool))
        .clone();

    cfg.service(get_achievement_tracker).app_data(data);
}

#[derive(Default)]
pub struct AchievementTrackerCache {
    achievement_tracker_map: RwLock<HashMap<Language, AchievementTracker>>,
}

#[derive(Clone, Serialize)]
struct AchievementTracker {
    achievement_count: usize,
    jade_count: i32,
    jade_count_current: i32,
    user_count: i64,
    language: Language,
    versions: Vec<String>,
    series: Vec<Series>,
}

#[derive(Clone, Serialize)]
struct Series {
    series: String,
    achievement_count: usize,
    jade_count: i32,
    jade_count_current: i32,
    achievement_groups: Vec<AchievementGroup>,
}

#[derive(Clone, Serialize)]
struct AchievementGroup {
    complete: Option<i64>,
    favorite: Option<i64>,
    achievements: Vec<Achievement>,
}

#[derive(Clone, Serialize)]
struct Achievement {
    id: i64,
    series: i32,
    series_name: String,
    series_index: usize,
    name: String,
    description: String,
    jades: i32,
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

#[derive(Clone, EnumString, Serialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl From<database::DbAchievement> for Achievement {
    fn from(db_achievement: database::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            series_index: 0,
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            jades: db_achievement.jades,
            hidden: db_achievement.hidden,
            version: db_achievement.version.clone(),
            difficulty: db_achievement
                .difficulty
                .as_ref()
                .map(|d| d.parse().unwrap()),
            video: db_achievement.video.clone(),
            gacha: db_achievement.gacha,
            impossible: db_achievement.impossible,
            percent: db_achievement.percent,
        }
    }
}

pub fn cache(pool: PgPool) -> web::Data<AchievementTrackerCache> {
    let achievement_tracker_cache = web::Data::new(AchievementTrackerCache::default());

    {
        let achievement_tracker_cache = achievement_tracker_cache.clone();

        rt::spawn(async move {
            let mut interval = rt::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update(&achievement_tracker_cache, &pool).await {
                    error!(
                        "Achievement Tracker update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Achievement Tracker update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });
    }

    achievement_tracker_cache
}

async fn update(
    achievement_tracker_cache: &web::Data<AchievementTrackerCache>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut achievement_tracker_map = HashMap::new();

    for language in Language::iter() {
        let achievements = database::get_achievements(&language.to_string(), pool).await?;

        let mut versions = HashSet::new();
        let mut series = Vec::new();

        let mut current_series = None;
        let mut current_set = None;

        for achievement in achievements
            .into_iter()
            .filter(|a| !(a.hidden && a.impossible))
        {
            versions.insert(achievement.version.clone().unwrap_or_default());

            if current_series != Some(achievement.series_name.clone()) {
                current_series = Some(achievement.series_name.clone());

                series.push(Series {
                    series: achievement.series_name.clone(),
                    achievement_count: 0,
                    jade_count: 0,
                    jade_count_current: 0,
                    achievement_groups: Vec::new(),
                });
            }

            if achievement
                .set
                .map(|set| current_set == Some(set))
                .unwrap_or(false)
            {
                let mut achievement: Achievement = achievement.into();
                achievement.series_index = series.len() - 1;

                series
                    .last_mut()
                    .unwrap()
                    .achievement_groups
                    .last_mut()
                    .unwrap()
                    .achievements
                    .push(achievement);
            } else {
                current_set = achievement.set;

                let mut achievement: Achievement = achievement.into();
                achievement.series_index = series.len() - 1;

                series
                    .last_mut()
                    .unwrap()
                    .achievement_groups
                    .push(AchievementGroup {
                        complete: None,
                        favorite: None,
                        achievements: vec![achievement],
                    });
            }
        }

        for series in series.iter_mut() {
            series.achievement_count = series.achievement_groups.len();
            series.jade_count = series
                .achievement_groups
                .iter()
                .map(|group| group.achievements[0].jades)
                .sum();
        }

        let achievement_count = series.iter().map(|s| s.achievement_count).sum();
        let jade_count = series.iter().map(|s| s.jade_count).sum();
        let user_count = database::get_users_achievements_completed_user_count(pool).await?;

        let mut versions = versions.into_iter().collect::<Vec<_>>();
        versions.sort_unstable();

        let achievement_tracker = AchievementTracker {
            achievement_count,
            jade_count,
            jade_count_current: 0,
            user_count,
            language,
            versions,
            series,
        };

        achievement_tracker_map.insert(language, achievement_tracker);
    }

    *achievement_tracker_cache
        .achievement_tracker_map
        .write()
        .await = achievement_tracker_map;

    Ok(())
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/achievement-tracker",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "AchievementTracker"),
    )
)]
#[get("/api/pages/achievement-tracker", guard = "private")]
async fn get_achievement_tracker(
    session: Session,
    language_params: web::Query<LanguageParams>,
    achievement_tracker_cache: web::Data<AchievementTrackerCache>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let mut achievement_tracker = achievement_tracker_cache
        .achievement_tracker_map
        .read()
        .await[&language_params.lang]
        .clone();

    if let Ok(Some(username)) = session.get::<String>("username") {
        let completed = database::get_user_achievements_completed_by_username(&username, &pool)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect::<HashSet<_>>();
        let favorites = database::get_user_achievements_favorites_by_username(&username, &pool)
            .await?
            .into_iter()
            .map(|c| c.id)
            .collect::<HashSet<_>>();

        for series in achievement_tracker.series.iter_mut() {
            let mut jade_count_current = 0;

            for group in series.achievement_groups.iter_mut() {
                let complete = group
                    .achievements
                    .iter()
                    .map(|c| c.id)
                    .find(|id| completed.contains(id));

                let favorite = group
                    .achievements
                    .iter()
                    .map(|c| c.id)
                    .find(|id| favorites.contains(id));

                group.complete = complete;
                group.favorite = favorite;

                if let Some(complete) = complete {
                    jade_count_current += group
                        .achievements
                        .iter()
                        .find(|a| a.id == complete)
                        .unwrap()
                        .jades;
                }
            }

            series.jade_count_current = jade_count_current;
        }

        achievement_tracker.jade_count_current = achievement_tracker
            .series
            .iter()
            .map(|s| s.jade_count_current)
            .sum();
    }

    Ok(HttpResponse::Ok().json(achievement_tracker))
}
